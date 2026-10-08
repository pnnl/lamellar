/* Globals and statics as remote targets: .bss, .data, large static arrays, atomics, a global
 * lock, wait_until on a global flag, and collectives whose source/dest are global arrays. */
#include <shmem.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static long counter;                 /* .bss */
static long cas_target;              /* .bss */
static int  table[64];               /* .bss */
long        flag_data = 7;           /* .data */
static int  wait_flag;               /* .bss, spun on locally */
static long lock;                    /* .bss */
static long guarded;                 /* .bss, protected by `lock` */
static int  red_src[4], red_dst[4];  /* .bss, reduction operands */
static int  fc_src[2], fc_dst[64];   /* .bss, fcollect operands */
static char big[1 << 20];            /* 1 MiB .bss */
static long sync_arr[SHMEM_REDUCE_SYNC_SIZE];  /* legacy pSync as a global */

static int me, np, fails;
#define CHECK(c, msg) do { if (!(c)) { printf("PE %d FAIL: %s (line %d)\n", me, msg, __LINE__); fails++; } } while (0)

int main(void) {
    shmem_init();
    me = shmem_my_pe(); np = shmem_n_pes();
    int right = (me + 1) % np, left = (me + np - 1) % np;
    /* GLOBALS_PRINT_ADDR=1: show where each PE's globals live (proves ASLR/PIE moves them) */
    if (getenv("GLOBALS_PRINT_ADDR"))
        printf("PE %d: &counter=%p &table=%p &flag_data=%p\n", me, (void *)&counter, (void *)table, (void *)&flag_data);
    for (int i = 0; i < SHMEM_REDUCE_SYNC_SIZE; i++) sync_arr[i] = SHMEM_SYNC_VALUE;
    shmem_barrier_all();

    /* put / get on .bss and .data */
    int src[4] = { me, me + 1, me + 2, me + 3 };
    shmem_int_put(table, src, 4, right);
    shmem_barrier_all();
    CHECK(table[0] == left && table[3] == left + 3, "put to remote .bss array");
    CHECK(shmem_long_g(&flag_data, right) == 7, "get from remote .data");
    shmem_long_p(&flag_data, 100 + me, right);
    shmem_barrier_all();
    CHECK(flag_data == 100 + left, "p to remote .data");

    /* atomics on a global */
    for (int i = 0; i < 10; i++) shmem_long_atomic_inc(&counter, 0);
    shmem_barrier_all();
    if (me == 0) CHECK(counter == 10L * np, "atomic_inc on global");
    shmem_barrier_all();
    long prev = shmem_long_atomic_compare_swap(&cas_target, 0, me + 1, 0);
    shmem_barrier_all();
    long won = shmem_long_atomic_fetch(&cas_target, 0);
    CHECK(won >= 1 && won <= np, "cswap on global produced a valid winner");
    (void)prev;

    /* wait_until on a global */
    shmem_barrier_all();
    if (me == 0) { for (int p = 1; p < np; p++) shmem_int_atomic_set(&wait_flag, 42, p); }
    else { shmem_int_wait_until(&wait_flag, SHMEM_CMP_EQ, 42); CHECK(wait_flag == 42, "wait_until on global"); }
    shmem_barrier_all();

    /* lock variable and guarded data are globals */
    for (int i = 0; i < 5; i++) {
        shmem_set_lock(&lock);
        long v = shmem_long_g(&guarded, 0);
        shmem_long_p(&guarded, v + 1, 0);
        shmem_clear_lock(&lock);
    }
    shmem_barrier_all();
    if (me == 0) CHECK(guarded == 5L * np, "global lock protects global counter");

    /* collectives on global operands */
    for (int i = 0; i < 4; i++) red_src[i] = me + i;
    shmem_barrier_all();
    shmem_int_sum_to_all(red_dst, red_src, 4, 0, 0, np, NULL, sync_arr);
    for (int i = 0; i < 4; i++) CHECK(red_dst[i] == np * (np - 1) / 2 + np * i, "sum_to_all on globals");
    fc_src[0] = me; fc_src[1] = me * 10;
    shmem_barrier_all();
    shmem_fcollect32(fc_dst, fc_src, 2, 0, 0, np, sync_arr);
    for (int p = 0; p < np; p++) CHECK(fc_dst[2 * p] == p && fc_dst[2 * p + 1] == p * 10, "fcollect on globals");

    /* bulk transfer into a large static array from a stack/malloc buffer */
    static char pat_src[1 << 20];
    for (size_t i = 0; i < sizeof pat_src; i++) pat_src[i] = (char)((i * 31 + me) & 0x7f);
    shmem_barrier_all();
    shmem_putmem(big, pat_src, sizeof big, right);
    shmem_barrier_all();
    int ok = 1;
    for (size_t i = 0; i < sizeof big; i++) if (big[i] != (char)((i * 31 + left) & 0x7f)) { ok = 0; break; }
    CHECK(ok, "1 MiB putmem into static array");

    shmem_barrier_all();
    printf("PE %d/%d: %s\n", me, np, fails ? "FAILED" : "all checks passed");
    shmem_finalize();
    return fails ? 1 : 0;
}
