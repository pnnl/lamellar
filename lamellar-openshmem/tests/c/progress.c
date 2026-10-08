/* Target progress by operation class: for each class, PE 1 computes (no SHMEM calls) for STALL_MS while PE 0
 * issues OPS operations of that class to it, timing each one. A class is "async" if even the slowest operation
 * finishes long before PE 1 returns to SHMEM, i.e. the fabric does it without the target's CPU. The batch total
 * is printed too. Informational: shows which operations need target progress with the single-threaded runtime
 * (and whether SHMEM_PROGRESS_THREAD=1 removes that need). */
#define _POSIX_C_SOURCE 200809L
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define STALL_MS 1000
#define OPS 200
static int cmp_d(const void *a, const void *b) { double x = *(const double *)a, y = *(const double *)b; return (x > y) - (x < y); }
static double now_ms(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e3 + t.tv_nsec * 1e-6; }

int main(void) {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes(), bad = 0;
    long *v = shmem_calloc(4, sizeof(long));
    static const char *cls[] = {"put", "get", "fadd", "cswap"};
    double med[4] = {0}, mx[4] = {0}, tot[4] = {0}, t[OPS];
    for (int c = 0; c < 4 && np >= 2; c++) {
        shmem_barrier_all();
        if (me == 1) {
            double t0 = now_ms();
            volatile double x = 1.0;
            while (now_ms() - t0 < STALL_MS) x = x * 1.0000001 + 1e-9;   /* compute, no SHMEM calls */
        } else if (me == 0) {
            double tc = now_ms();
            for (int i = 1; i <= OPS; i++) {
                double t0 = now_ms();
                switch (c) {
                case 0: shmem_long_p(&v[0], i, 1); shmem_quiet(); break;
                case 1: if (shmem_long_g(&v[0], 1) < 0) bad++; break;
                case 2: shmem_long_atomic_fetch_add(&v[1], 1, 1); break;
                case 3: shmem_long_atomic_compare_swap(&v[2], i - 1, i, 1); break;
                }
                t[i - 1] = (now_ms() - t0) * 1e3;   /* per-operation time, microseconds */
            }
            tot[c] = now_ms() - tc;
            qsort(t, OPS, sizeof(double), cmp_d);
            med[c] = t[OPS / 2]; mx[c] = t[OPS - 1];
        }
    }
    shmem_barrier_all();
    if (me == 0 && np >= 2) {
        printf("PE 0: progress: %d ops to a PE computing %d ms (per-op median/max us, batch ms):", OPS, STALL_MS);
        for (int c = 0; c < 4; c++)
            printf("  %s %.1f/%.1f us, %.1f ms [%s]", cls[c], med[c], mx[c], tot[c],
                   mx[c] < STALL_MS * 1e3 / 2 ? "async" : "an op waited for the target");
        printf("\n");
    }
    if (me == 1 && v[1] != OPS) { printf("PE 1 FAIL: fadd count %ld\n", v[1]); bad++; }
    shmem_barrier_all();
    shmem_finalize();
    return bad ? 1 : 0;
}
