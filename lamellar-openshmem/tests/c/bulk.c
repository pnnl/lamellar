#define _POSIX_C_SOURCE 200809L
/* Bulk RMA integrity test with *unregistered* local buffers (plain malloc) across message
 * sizes spanning the inject threshold up to several MiB. Also prints rough bandwidth. */
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + t.tv_nsec * 1e-9; }
static unsigned char pat(int pe, size_t i) { return (unsigned char)((i * 131 + pe * 17 + (i >> 8)) & 0xff); }

int main(void) {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes(), fails = 0;
    size_t sizes[] = { 1, 8, 64, 256, 1024, 4096, 65536, 1u << 20, 4u << 20, 16u << 20 };
    size_t maxsz = 16u << 20;
    unsigned char *sym = shmem_malloc(maxsz);        /* registered, symmetric */
    unsigned char *src = malloc(maxsz), *dst = malloc(maxsz); /* NOT registered */
    int right = (me + 1) % np, left = (me + np - 1) % np;

    for (size_t k = 0; k < sizeof(sizes) / sizeof(*sizes); k++) {
        size_t n = sizes[k];
        memset(sym, 0, n);
        for (size_t i = 0; i < n; i++) src[i] = pat(me, i);
        shmem_barrier_all();
        double t0 = now();
        shmem_putmem(sym, src, n, right);            /* unregistered src -> remote */
        shmem_quiet();
        double t1 = now();
        shmem_barrier_all();
        int ok = 1;
        for (size_t i = 0; i < n; i++) if (sym[i] != pat(left, i)) { ok = 0; break; }
        if (!ok) { printf("PE %d FAIL put size %zu\n", me, n); fails++; }

        memset(dst, 0xee, n);
        double t2 = now();
        shmem_getmem(dst, sym, n, right);            /* remote -> unregistered dst */
        double t3 = now();
        for (size_t i = 0; i < n; i++) if (dst[i] != pat(me, i)) { ok = 0; break; }
        if (!ok) { printf("PE %d FAIL get size %zu\n", me, n); fails++; }
        if (me == 0 && n >= 4096)
            printf("size %9zu  put %8.1f MB/s  get %8.1f MB/s\n", n, n / (t1 - t0) / 1e6, n / (t3 - t2) / 1e6);
        shmem_barrier_all();
    }
    printf("PE %d/%d: %s\n", me, np, fails ? "FAILED" : "all checks passed");
    shmem_finalize();
    return fails ? 1 : 0;
}
