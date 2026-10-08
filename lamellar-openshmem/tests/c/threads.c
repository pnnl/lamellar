/* SHMEM_THREAD_MULTIPLE: several application threads per PE call SHMEM concurrently.
 * Checks (per thread, distinct slots): put/get round trips, bulk put/get with a data pattern,
 * and one shared counter hit by every thread of every PE (atomicity). THREADS=<n> (default 8). */
#include <shmem.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ITERS 500
#define BULK 4096

static int me, np, nthreads;
static long *slots;      /* heap: one slot per thread, written by the LEFT neighbour's same-numbered thread */
static char *bulk;       /* heap: one BULK-byte block per thread */
static long *ctr;        /* heap counter on PE 0 */
static int fails[64];

static void *worker(void *arg) {
    int tid = (int)(long)arg, right = (me + 1) % np;
    char *src = malloc(BULK), *dst = malloc(BULK);
    for (int i = 1; i <= ITERS; i++) {
        long val = ((long)me << 40) | ((long)tid << 20) | i;
        shmem_long_p(&slots[tid], val, right);          /* write to my right neighbour's slot[tid] */
        shmem_quiet();
        if (shmem_long_g(&slots[tid], right) != val) fails[tid]++;
        memset(src, (me * 31 + tid * 7 + i) & 0xff, BULK);
        shmem_putmem(bulk + (size_t)tid * BULK, src, BULK, right);
        shmem_getmem(dst, bulk + (size_t)tid * BULK, BULK, right);
        if (memcmp(src, dst, BULK)) fails[tid]++;
        shmem_long_atomic_fetch_add(ctr, 1, 0);
        if ((i & 63) == 0) shmem_fence();
    }
    free(src); free(dst);
    return NULL;
}

int main(void) {
    int provided = -1;
    shmem_init_thread(SHMEM_THREAD_MULTIPLE, &provided);
    me = shmem_my_pe(); np = shmem_n_pes();
    nthreads = getenv("THREADS") ? atoi(getenv("THREADS")) : 8;
    if (nthreads > 64) nthreads = 64;
    slots = shmem_calloc(nthreads, sizeof(long));
    bulk = shmem_malloc((size_t)nthreads * BULK);
    ctr = shmem_calloc(1, sizeof(long));
    shmem_barrier_all();

    pthread_t th[64];
    for (long t = 0; t < nthreads; t++) pthread_create(&th[t], NULL, worker, (void *)t);
    for (int t = 0; t < nthreads; t++) pthread_join(th[t], NULL);
    shmem_barrier_all();

    int bad = 0;
    for (int t = 0; t < nthreads; t++) bad += fails[t];
    if (me == 0 && *ctr != (long)np * nthreads * ITERS) {
        printf("PE 0 FAIL: shared counter %ld, expected %ld\n", *ctr, (long)np * nthreads * ITERS);
        bad++;
    }
    printf("PE %d/%d: threads=%d provided_level=%d %s\n", me, np, nthreads, provided, bad ? "FAILED" : "threads OK");
    shmem_barrier_all();
    shmem_finalize();
    return bad ? 1 : 0;
}
