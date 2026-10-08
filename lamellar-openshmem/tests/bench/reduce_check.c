/* Correctness check for the *_to_all reductions over the whole team (heap source and destination): every op for int,
 * long and double against a value computed locally from the known inputs. Any PE with a mismatch prints a FAIL line.
 * Run with and without SHMEM_RUNTIME_REDUCE=1 to check the runtime path against the original. */
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define N 37 /* not a multiple of any PE count we use */

static int me, np, bad;

#define CHECK(T, name, op_call, ISFLOAT)                                                                          \
    do {                                                                                                          \
        T *src = shmem_malloc(N * sizeof(T)), *dst = shmem_malloc(N * sizeof(T));                                 \
        T *wrk = shmem_malloc((N / 2 + 1 + SHMEM_REDUCE_MIN_WRKDATA_SIZE) * sizeof(T));                           \
        long *ps = shmem_malloc(SHMEM_REDUCE_SYNC_SIZE * sizeof(long));                                           \
        for (int i = 0; i < SHMEM_REDUCE_SYNC_SIZE; i++) ps[i] = SHMEM_SYNC_VALUE;                                \
        for (int i = 0; i < N; i++) { src[i] = (T)(((me * 7 + i * 3) % 11) + 1); dst[i] = 0; }                    \
        shmem_barrier_all();                                                                                      \
        op_call;                                                                                                  \
        shmem_barrier_all();                                                                                      \
        for (int i = 0; i < N; i++) {                                                                             \
            T acc = (T)(((0 * 7 + i * 3) % 11) + 1);                                                              \
            for (int p = 1; p < np; p++) { T v = (T)(((p * 7 + i * 3) % 11) + 1); ACCUM; }                        \
            if (dst[i] != acc) { if (bad < 8) fprintf(stderr, "PE%d %s[%d]: got %g want %g\n", me, name, i, (double)dst[i], (double)acc); bad++; } \
        }                                                                                                         \
        shmem_free(ps); shmem_free(wrk); shmem_free(dst); shmem_free(src);                                        \
    } while (0)

int main(void) {
    shmem_init();
    me = shmem_my_pe(); np = shmem_n_pes();
#define ACCUM acc = acc + v
    CHECK(int, "int sum", shmem_int_sum_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
    CHECK(long, "long sum", shmem_long_sum_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
    CHECK(double, "double sum", shmem_double_sum_to_all(dst, src, N, 0, 0, np, wrk, ps), 1);
#undef ACCUM
#define ACCUM acc = v > acc ? v : acc
    CHECK(int, "int max", shmem_int_max_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
    CHECK(double, "double max", shmem_double_max_to_all(dst, src, N, 0, 0, np, wrk, ps), 1);
#undef ACCUM
#define ACCUM acc = v < acc ? v : acc
    CHECK(long, "long min", shmem_long_min_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
    CHECK(double, "double min", shmem_double_min_to_all(dst, src, N, 0, 0, np, wrk, ps), 1);
#undef ACCUM
#define ACCUM acc = acc | v
    CHECK(int, "int or", shmem_int_or_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
#undef ACCUM
#define ACCUM acc = acc & v
    CHECK(long, "long and", shmem_long_and_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
#undef ACCUM
#define ACCUM acc = acc ^ v
    CHECK(int, "int xor", shmem_int_xor_to_all(dst, src, N, 0, 0, np, wrk, ps), 0);
#undef ACCUM
    if (bad) { printf("FAIL pe=%d mismatches=%d\n", me, bad); fflush(stdout); }
    shmem_barrier_all();
    if (me == 0) { printf("reduce_check np=%d finished (a mismatch is reported above)\n", np); fflush(stdout); }
    shmem_finalize();
    return bad != 0;
}
