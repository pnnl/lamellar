/* Phase A coverage test: generic RMA, AMOs, locks, wait_until, active-set barrier,
 * broadcast, collect/fcollect and reductions. All symmetric data is heap allocated. */
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>

static int me, np, fails;
#define CHECK(cond, msg) do { if (!(cond)) { printf("PE %d FAIL: %s (line %d)\n", me, msg, __LINE__); fails++; } } while (0)

int main(void) {
    shmem_init();
    me = shmem_my_pe(); np = shmem_n_pes();

    /* generic put/get/p/g */
    int *a = shmem_malloc(16 * sizeof(int));
    for (int i = 0; i < 16; i++) a[i] = -1;
    shmem_barrier_all();
    int src[4] = { me, me + 1, me + 2, me + 3 };
    shmem_put(a, src, 4, (me + 1) % np);
    shmem_p(&a[8], 1000 + me, (me + 1) % np);
    shmem_barrier_all();
    int l = (me + np - 1) % np;
    CHECK(a[0] == l && a[3] == l + 3, "generic put");
    CHECK(a[8] == 1000 + l, "generic p");
    CHECK(shmem_g(&a[8], (me + 1) % np) == 1000 + me, "generic g");
    int back[4];
    shmem_get(back, a, 4, (me + 1) % np);
    CHECK(back[0] == me && back[3] == me + 3, "generic get");

    /* AMOs: every PE increments and adds on PE 0 */
    long *ctr = shmem_calloc(2, sizeof(long));
    shmem_barrier_all();
    for (int i = 0; i < 10; i++) shmem_atomic_inc(&ctr[0], 0);
    long old = shmem_atomic_fetch_add(&ctr[1], me + 1, 0);
    (void)old;
    shmem_barrier_all();
    if (me == 0) {
        CHECK(ctr[0] == 10L * np, "atomic_inc total");
        CHECK(ctr[1] == (long)np * (np + 1) / 2, "fetch_add total");
    }
    shmem_barrier_all(); /* PE 0 has checked the totals before anyone swaps */
    /* every PE races to swap 10*np -> 7; exactly one may observe the old value */
    long cs = shmem_atomic_compare_swap(&ctr[0], 10L * np, 7, 0);
    long *wins = shmem_calloc(1, sizeof(long));
    shmem_barrier_all();
    if (cs == 10L * np) shmem_atomic_inc(wins, 0);
    shmem_barrier_all();
    if (me == 0) CHECK(*wins == 1 && ctr[0] == 7, "cswap exactly one winner");
    shmem_barrier_all();

    /* lock protects a non-atomic read-modify-write on PE 0 */
    long *lock = shmem_calloc(1, sizeof(long));
    long *shared = shmem_calloc(1, sizeof(long));
    shmem_barrier_all();
    for (int i = 0; i < 5; i++) {
        shmem_set_lock(lock);
        long v = shmem_long_g(shared, 0);
        shmem_long_p(shared, v + 1, 0);
        shmem_clear_lock(lock);
    }
    shmem_barrier_all();
    if (me == 0) CHECK(*shared == 5L * np, "lock-protected counter");

    /* wait_until: PE 0 signals every other PE */
    int *flag = shmem_calloc(1, sizeof(int));
    shmem_barrier_all();
    if (me == 0) { for (int p = 1; p < np; p++) shmem_atomic_set(flag, 42, p); }
    else { shmem_wait_until(flag, SHMEM_CMP_EQ, 42); CHECK(*flag == 42, "wait_until"); }
    shmem_barrier_all();

    /* active-set barrier on even PEs only (start 0, stride 2) */
    if (me % 2 == 0) shmem_barrier(0, 1, (np + 1) / 2, NULL);
    shmem_barrier_all();

    /* broadcast from active-set index 1 */
    long *bsrc = shmem_malloc(4 * sizeof(long)), *bdst = shmem_malloc(4 * sizeof(long));
    for (int i = 0; i < 4; i++) { bsrc[i] = 500 + me * 10 + i; bdst[i] = -1; }
    shmem_barrier_all();
    int root = np > 1 ? 1 : 0;
    shmem_broadcast64(bdst, bsrc, 4, root, 0, 0, np, NULL);
    if (me != root) CHECK(bdst[0] == 500 + root * 10 && bdst[3] == 503 + root * 10, "broadcast");
    else CHECK(bdst[0] == -1, "broadcast root untouched");

    /* fcollect + collect (PE i contributes i+1 elements) */
    int *fsrc = shmem_malloc(2 * sizeof(int)), *fdst = shmem_malloc(2 * np * sizeof(int));
    fsrc[0] = me; fsrc[1] = me * 100;
    shmem_barrier_all();
    shmem_fcollect32(fdst, fsrc, 2, 0, 0, np, NULL);
    for (int p = 0; p < np; p++) CHECK(fdst[2 * p] == p && fdst[2 * p + 1] == p * 100, "fcollect");
    int total = np * (np + 1) / 2;
    int *csrc = shmem_malloc(np * sizeof(int)), *cdst = shmem_malloc(total * sizeof(int));
    for (int i = 0; i < me + 1; i++) csrc[i] = me;
    shmem_barrier_all();
    shmem_collect32(cdst, csrc, me + 1, 0, 0, np, NULL);
    int pos = 0, ok = 1;
    for (int p = 0; p < np; p++) for (int i = 0; i < p + 1; i++) ok &= (cdst[pos++] == p);
    CHECK(ok, "collect");

    /* reductions, including dest == source */
    int *rs = shmem_malloc(3 * sizeof(int)), *rd = shmem_malloc(3 * sizeof(int));
    for (int i = 0; i < 3; i++) rs[i] = me + i;
    shmem_barrier_all();
    shmem_int_sum_to_all(rd, rs, 3, 0, 0, np, NULL, NULL);
    for (int i = 0; i < 3; i++) CHECK(rd[i] == np * (np - 1) / 2 + np * i, "sum_to_all");
    shmem_int_max_to_all(rs, rs, 3, 0, 0, np, NULL, NULL);
    for (int i = 0; i < 3; i++) CHECK(rs[i] == np - 1 + i, "max_to_all in place");
    double *ds = shmem_malloc(sizeof(double)), *dd = shmem_malloc(sizeof(double));
    *ds = 0.5 * (me + 1);
    shmem_barrier_all();
    shmem_double_prod_to_all(dd, ds, 1, 0, 0, np, NULL, NULL);
    double want = 1; for (int p = 0; p < np; p++) want *= 0.5 * (p + 1);
    CHECK(*dd == want, "double prod");

    shmem_barrier_all();
    printf("PE %d/%d: %s\n", me, np, fails ? "FAILED" : "all checks passed");
    shmem_finalize();
    return fails ? 1 : 0;
}
