/* AMO atomicity between the target PE's own operations and remote PEs' operations.
 * Every PE (including PE 0) does N increments on counters that live on PE 0; a control run leaves PE 0
 * out. Lost updates in the "mixed" case but not in the control mean local and remote atomics are not in
 * one atomicity domain (e.g. CPU atomics on the owner vs NIC atomics from peers on an ATOMIC_HCA device).
 * N=<n> (default 20000). PE 0 prints "atomicity OK" or "atomicity LOST ...". */
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes();
    long n = getenv("N") ? atol(getenv("N")) : 20000;
    long *c = shmem_calloc(4, sizeof(long)); /* [0] fadd mixed, [1] fadd control, [2] cswap mixed, [3] cswap control */
    shmem_barrier_all();

    for (long i = 0; i < n; i++) shmem_long_atomic_fetch_add(&c[0], 1, 0);              /* everyone */
    shmem_barrier_all();
    if (me != 0) for (long i = 0; i < n; i++) shmem_long_atomic_fetch_add(&c[1], 1, 0); /* control: not the owner */
    shmem_barrier_all();
    for (long i = 0; i < n; i++) {                                                       /* CAS-loop increment, everyone */
        long old = shmem_long_atomic_fetch(&c[2], 0);
        while (1) { long seen = shmem_long_atomic_compare_swap(&c[2], old, old + 1, 0); if (seen == old) break; old = seen; }
    }
    shmem_barrier_all();
    if (me != 0) for (long i = 0; i < n; i++) {                                          /* CAS control */
        long old = shmem_long_atomic_fetch(&c[3], 0);
        while (1) { long seen = shmem_long_atomic_compare_swap(&c[3], old, old + 1, 0); if (seen == old) break; old = seen; }
    }
    shmem_barrier_all();

    int bad = 0;
    if (me == 0) {
        long l0 = (long)np * n - c[0], l1 = (long)(np - 1) * n - c[1], l2 = (long)np * n - c[2], l3 = (long)(np - 1) * n - c[3];
        bad = (l0 || l1 || l2 || l3);
        if (!bad) printf("PE 0: atomicity OK (np=%d, %ld ops/PE)\n", np, n);
        else printf("PE 0: atomicity LOST updates: fadd mixed=%ld fadd remote-only=%ld cswap mixed=%ld cswap remote-only=%ld (np=%d, %ld ops/PE)\n",
                    l0, l1, l2, l3, np, n);
    }
    shmem_barrier_all();
    shmem_finalize();
    return bad;
}
