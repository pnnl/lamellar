/* Smoke test: heap put/get ring + malloc/free. */
#include <shmem.h>
#include <stdio.h>

int main(void) {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes();
    int *buf = shmem_malloc(1024 * sizeof(int));
    buf[0] = -1;
    shmem_barrier_all();

    int val = 100 + me;
    shmem_int_put(buf, &val, 1, (me + 1) % np);
    shmem_quiet();
    shmem_barrier_all();

    int expect = 100 + (me + np - 1) % np;
    int got = shmem_int_g(buf, (me + 1) % np); /* what my right neighbor received: from me */
    int ok = (buf[0] == expect) && (got == 100 + me);
    printf("PE %d/%d: local=%d (expect %d) remote_g=%d -> %s\n", me, np, buf[0], expect, got,
           ok ? "OK" : "FAIL");
    shmem_barrier_all();
    shmem_free(buf);
    shmem_finalize();
    return ok ? 0 : 1;
}
