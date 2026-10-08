// C++ smoke test: typed API only (the C11 _Generic macros are C-only).
#include <shmem.h>
#include <cstdio>

int main() {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes();
    long *x = static_cast<long *>(shmem_calloc(1, sizeof(long)));
    shmem_barrier_all();
    shmem_long_atomic_add(x, me + 1, 0);
    shmem_barrier_all();
    bool ok = true;
    if (me == 0) ok = (*x == static_cast<long>(np) * (np + 1) / 2);
    std::printf("PE %d/%d (C++): %s\n", me, np, ok ? "OK" : "FAIL");
    shmem_barrier_all();
    shmem_free(x);
    shmem_finalize();
    return ok ? 0 : 1;
}
