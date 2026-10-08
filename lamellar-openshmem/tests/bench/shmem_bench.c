/* OpenSHMEM 1.4 microbenchmark: one-sided latency/bandwidth/message rate, AMOs (uncontended and
 * contended), barrier and legacy collectives. Uses only the public API so the same source builds
 * against any implementation (ours, Open MPI's OSHMEM, ...).
 *
 *   shmem_bench [section ...]      sections: pt2pt amo contend barrier coll ops multi   (default: all)
 *     ops    other single-pair operations: element p/g, int32, strided iput/iget, inc/set/bitwise AMOs, nbi fetch-add
 *     multi  many PEs active at once: concurrent pairs, random targets, all-to-all put
 *
 * Env: BENCH_QUICK=1 cuts iteration counts (smoke runs); BENCH_NO_GLOBAL=1 skips targets in
 * static/global memory (backends that cannot register the data segment).
 *
 * PE 0 prints "CSV,bench,size_bytes,np,mem,p50_us,p99_us,mean_us,min_us,bw_MBps,rate_Mops".
 * Point-to-point tests: PE 0 is the initiator and PE 1 the target; other PEs only synchronize.
 */
#define _POSIX_C_SOURCE 200809L
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define MAXSZ (4u << 20)
#define GLOBSZ (4u << 20)

static int me, np, quick, no_global;
static char glob_buf[GLOBSZ];      /* .bss: target in global memory */
static long glob_var;              /* .bss: AMO target in global memory */

static double now_us(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec * 1e6 + t.tv_nsec * 1e-3;
}
static int cmp_d(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    return (x > y) - (x < y);
}

static void csv(const char *bench, size_t size, const char *mem, double p50, double p99, double mean,
                double mn, double bw, double rate) {
    if (me != 0) return;
    printf("CSV,%s,%zu,%d,%s,%.3f,%.3f,%.3f,%.3f,%.1f,%.4f\n", bench, size, np, mem, p50, p99, mean, mn, bw, rate);
    fflush(stdout);
}

/* stats over per-operation times (us) */
static void stats_csv(const char *bench, size_t size, const char *mem, double *t, int n) {
    qsort(t, n, sizeof(double), cmp_d);
    double sum = 0;
    for (int i = 0; i < n; i++) sum += t[i];
    csv(bench, size, mem, t[n / 2], t[(int)(n * 0.99) < n ? (int)(n * 0.99) : n - 1], sum / n, t[0], 0, 0);
}

static int lat_iters(size_t s) {
    int n = s <= 4096 ? 5000 : s <= 65536 ? 2000 : s <= (1u << 20) ? 300 : 60;
    return quick ? (n / 10 > 20 ? n / 10 : 20) : n;
}

static char *src, *dst; /* malloc'd: NOT registered with the fabric */

/* ---- point-to-point latency: blocking put (+quiet) and get, PE 0 -> PE 1 ---------------------- */
static void lat_test(int is_put, char *remote, const char *mem, size_t size) {
    int iters = lat_iters(size), warm = iters / 10 > 10 ? iters / 10 : 10;
    shmem_barrier_all();
    if (me == 0) {
        double *t = malloc(iters * sizeof(double));
        for (int i = 0; i < warm + iters; i++) {
            double t0 = now_us();
            if (is_put) { shmem_putmem(remote, src, size, 1); shmem_quiet(); }
            else shmem_getmem(dst, remote, size, 1);
            double dt = now_us() - t0;
            if (i >= warm) t[i - warm] = dt;
        }
        stats_csv(is_put ? "put_lat" : "get_lat", size, mem, t, iters);
        free(t);
    }
    shmem_barrier_all();
}

/* ---- put bandwidth / message rate: a window of non-blocking puts, then quiet ------------------ */
static void bw_test(char *remote, const char *mem, size_t size) {
    int window = size <= 4096 ? 64 : size <= 65536 ? 32 : size <= (1u << 20) ? 8 : 4;
    int reps = size <= 4096 ? 100 : size <= 65536 ? 50 : size <= (1u << 20) ? 30 : 10;
    int trials = quick ? 2 : 5;
    if (quick) reps = reps / 5 > 2 ? reps / 5 : 2;
    double rates[16], best = 0;
    shmem_barrier_all();
    for (int tr = 0; tr < trials + 1; tr++) { /* trial 0 is warm-up */
        shmem_barrier_all();
        if (me == 0) {
            double t0 = now_us();
            for (int r = 0; r < reps; r++) {
                for (int w = 0; w < window; w++) shmem_putmem_nbi(remote, src, size, 1);
                shmem_quiet();
            }
            double us = now_us() - t0;
            if (tr > 0) { rates[tr - 1] = (double)window * reps / us; /* msgs per us == Mmsg/s */ if (rates[tr - 1] > best) best = rates[tr - 1]; }
        }
    }
    if (me == 0) {
        qsort(rates, trials, sizeof(double), cmp_d);
        double med = rates[trials / 2];
        csv("put_bw", size, mem, 1.0 / med, 0, 1.0 / med, 1.0 / best, med * size /* MB/s */, med);
    }
    shmem_barrier_all();
}

static void pt2pt(void) {
    static const size_t sizes[] = {1, 8, 64, 512, 4096, 65536, 1048576, 4194304};
    char *heap = shmem_malloc(MAXSZ);
    if (np < 2) { if (me == 0) fprintf(stderr, "pt2pt needs >= 2 PEs\n"); return; }
    for (size_t k = 0; k < sizeof(sizes) / sizeof(*sizes); k++) {
        lat_test(1, heap, "heap", sizes[k]);
        lat_test(0, heap, "heap", sizes[k]);
        bw_test(heap, "heap", sizes[k]);
    }
    if (!no_global) {
        static const size_t gs[] = {8, 4096, 65536, 1048576};
        for (size_t k = 0; k < sizeof(gs) / sizeof(*gs); k++) {
            lat_test(1, glob_buf, "global", gs[k]);
            lat_test(0, glob_buf, "global", gs[k]);
            bw_test(glob_buf, "global", gs[k]);
        }
    }
    shmem_free(heap);
}

/* ---- AMO latency, uncontended: PE 0 -> PE 1 -------------------------------------------------- */
static void amo_one(const char *name, long *var, const char *mem, int op) {
    int iters = quick ? 300 : 5000, warm = 200;
    shmem_barrier_all();
    if (me == 0) {
        double *t = malloc(iters * sizeof(double));
        volatile long sink = 0;
        for (int i = 0; i < warm + iters; i++) {
            double t0 = now_us();
            switch (op) {
            case 0: sink += shmem_long_atomic_fetch_add(var, 1, 1); break;
            case 1: sink += shmem_long_atomic_compare_swap(var, 0, 1, 1); break;
            case 2: sink += shmem_long_atomic_swap(var, i, 1); break;
            case 3: sink += shmem_long_atomic_fetch(var, 1); break;
            case 4: shmem_long_atomic_add(var, 1, 1); shmem_quiet(); break; /* non-fetching: time to completion */
            }
            double dt = now_us() - t0;
            if (i >= warm) t[i - warm] = dt;
        }
        (void)sink;
        stats_csv(name, 8, mem, t, iters);
        free(t);
    }
    shmem_barrier_all();
}

static void amo(void) {
    if (np < 2) return;
    long *hv = shmem_calloc(1, sizeof(long));
    static const char *names[] = {"amo_fadd", "amo_cswap", "amo_swap", "amo_fetch", "amo_add"};
    for (int op = 0; op < 5; op++) amo_one(names[op], hv, "heap", op);
    if (!no_global) for (int op = 0; op < 5; op++) amo_one(names[op], &glob_var, "global", op);
    shmem_free(hv);
}

/* ---- contended fetch-add: every PE hammers one counter on PE 0 ------------------------------- */
static void contend(void) {
    if (np < 2) return;
    long *ctr = shmem_calloc(1, sizeof(long));
    double *tb = shmem_malloc(sizeof(double));
    int iters = quick ? 200 : 2000;
    for (int i = 0; i < 100; i++) shmem_long_atomic_fetch_add(ctr, 1, 0);
    shmem_barrier_all();
    double t0 = now_us();
    for (int i = 0; i < iters; i++) shmem_long_atomic_fetch_add(ctr, 1, 0);
    double el = now_us() - t0;
    *tb = el;
    shmem_barrier_all();
    if (me == 0) {
        double mx = 0, sum = 0;
        for (int p = 0; p < np; p++) { double e = shmem_double_g(tb, p); if (e > mx) mx = e; sum += e; }
        double per_op = sum / np / iters;
        csv("amo_contended_fadd", 8, "heap", per_op, 0, per_op, 0, 0, (double)np * iters / mx);
    }
    shmem_barrier_all();
    shmem_free(tb);
    shmem_free(ctr);
}

static void barrier_bench(void) {
    int iters = quick ? 200 : 2000;
    for (int i = 0; i < 100; i++) shmem_barrier_all();
    shmem_barrier_all();
    double *t = malloc(iters * sizeof(double));
    for (int i = 0; i < iters; i++) {
        double t0 = now_us();
        shmem_barrier_all();
        t[i] = now_us() - t0;
    }
    if (me == 0) stats_csv("barrier_all", 0, "heap", t, iters);
    shmem_barrier_all();
    /* the legacy active-set sync over the whole team (the shim runs its own dissemination barrier for it
     * unless SHMEM_RUNTIME_BARRIER=1 hands it to the runtime) */
    long *ps = shmem_malloc(SHMEM_BARRIER_SYNC_SIZE * sizeof(long));
    for (int i = 0; i < SHMEM_BARRIER_SYNC_SIZE; i++) ps[i] = SHMEM_SYNC_VALUE;
    shmem_barrier_all();
    for (int i = 0; i < 100; i++) shmem_sync(0, 0, np, ps);
    shmem_barrier_all();
    for (int i = 0; i < iters; i++) {
        double t0 = now_us();
        shmem_sync(0, 0, np, ps);
        t[i] = now_us() - t0;
    }
    if (me == 0) stats_csv("barrier_sync_active_set", 0, "heap", t, iters);
    free(t);
    shmem_barrier_all();
    shmem_free(ps);
    shmem_barrier_all();
}

/* ---- legacy collectives (1.4): broadcast64, fcollect64, long sum_to_all ------------------------ */
static void coll(void) {
    static const int ns[] = {1, 128, 8192}; /* elements of 8 bytes: 8 B, 1 KiB, 64 KiB per PE */
    long *psA = shmem_malloc(SHMEM_REDUCE_SYNC_SIZE * sizeof(long));
    long *psB = shmem_malloc(SHMEM_REDUCE_SYNC_SIZE * sizeof(long));
    long *ps[2] = {psA, psB};
    for (int p = 0; p < 2; p++) for (int i = 0; i < SHMEM_REDUCE_SYNC_SIZE; i++) ps[p][i] = SHMEM_SYNC_VALUE;
    size_t maxn = 8192;
    long *bsrc = shmem_malloc(maxn * sizeof(long)), *bdst = shmem_malloc(maxn * sizeof(long));
    long *fdst = shmem_malloc(maxn * np * sizeof(long));
    long *rsrc = shmem_malloc(maxn * sizeof(long)), *rdst = shmem_malloc(maxn * sizeof(long));
    size_t wsz = maxn / 2 + 1 + SHMEM_REDUCE_MIN_WRKDATA_SIZE;
    long *wrk[2] = {shmem_malloc(wsz * sizeof(long)), shmem_malloc(wsz * sizeof(long))};
    for (size_t i = 0; i < maxn; i++) { bsrc[i] = me + i; rsrc[i] = me + i; }
    /* Per-iteration time of the SLOWEST PE. Timing PE 0 alone is wrong for broadcast: PE 0 is the root and
     * returns once its sends are issued, before the last PE has the data. A barrier before each iteration
     * starts the PEs together; every PE times its own call and the per-iteration maximum over PEs is reduced
     * to PE 0 after the timed loop (so the reduction never perturbs the measurement). */
    enum { MAXIT = 400 };
    double *tsrc = shmem_malloc(MAXIT * sizeof(double)), *tdst = shmem_malloc(MAXIT * sizeof(double));
    double *twrk = shmem_malloc((MAXIT / 2 + 1 + SHMEM_REDUCE_MIN_WRKDATA_SIZE) * sizeof(double));
    long *tps = shmem_malloc(SHMEM_REDUCE_SYNC_SIZE * sizeof(long));
    for (int i = 0; i < SHMEM_REDUCE_SYNC_SIZE; i++) tps[i] = SHMEM_SYNC_VALUE;
    shmem_barrier_all();
    for (int k = 0; k < 3; k++) {
        int n = ns[k], iters = quick ? 20 : (n >= 8192 ? 100 : MAXIT), warm = 10;
        for (int op = 0; op < 3; op++) {
            shmem_barrier_all();
            for (int i = 0; i < warm + iters; i++) {
                long *s = ps[i & 1]; long *w = wrk[i & 1];
                shmem_barrier_all();
                double t0 = now_us();
                if (op == 0) shmem_broadcast64(bdst, bsrc, n, 0, 0, 0, np, s);
                else if (op == 1) shmem_fcollect64(fdst, bsrc, n, 0, 0, np, s);
                else shmem_long_sum_to_all(rdst, rsrc, n, 0, 0, np, w, s);
                double dt = now_us() - t0;
                if (i >= warm) tsrc[i - warm] = dt;
            }
            shmem_barrier_all();
            shmem_double_max_to_all(tdst, tsrc, iters, 0, 0, np, twrk, tps);
            shmem_barrier_all();
            if (me == 0) stats_csv(op == 0 ? "broadcast64" : op == 1 ? "fcollect64" : "sum_to_all", (size_t)n * 8, "heap", tdst, iters);
        }
    }
    shmem_barrier_all();
    shmem_free(tps); shmem_free(twrk); shmem_free(tdst); shmem_free(tsrc);
    shmem_free(wrk[1]); shmem_free(wrk[0]); shmem_free(rdst); shmem_free(rsrc);
    shmem_free(fdst); shmem_free(bdst); shmem_free(bsrc); shmem_free(psB); shmem_free(psA);
}

/* ---- other operations, uncontended, PE 0 -> PE 1 (heap): element p/g, strided, int32, the remaining AMOs,
 *      and non-blocking fetching AMOs -------------------------------------------------------------- */
enum { OP_LP, OP_LG, OP_IP, OP_IG, OP_IPUT, OP_IGET, OP_INC, OP_FINC, OP_SET, OP_AND, OP_OR, OP_XOR,
       OP_FAND, OP_FOR, OP_FXOR, OP_IFADD, OP_ICSWAP, OP_ISWAP, OP_FADD_NBI, OP_N };
static const char *op_names[OP_N] = {"op_long_p", "op_long_g", "op_int_p", "op_int_g", "op_iput", "op_iget",
    "op_inc", "op_fetch_inc", "op_set", "op_and", "op_or", "op_xor", "op_fetch_and", "op_fetch_or",
    "op_fetch_xor", "op_int_fadd", "op_int_cswap", "op_int_swap", "op_fadd_nbi"};

static void ops_one(int op, long *lv, int *iv, unsigned long *uv, long *strided, int nelem) {
    int iters = (op == OP_IPUT || op == OP_IGET) ? (quick ? 20 : nelem >= 1024 ? 50 : 500) : (quick ? 300 : 3000);
    int warm = iters / 10 > 10 ? iters / 10 : 10;
    static long larr[2048];
    volatile long sink = 0;
    shmem_barrier_all();
    if (me == 0) {
        double *t = malloc(iters * sizeof(double));
        for (int i = 0; i < warm + iters; i++) {
            double t0 = now_us();
            switch (op) {
            case OP_LP: shmem_long_p(lv, i, 1); shmem_quiet(); break;
            case OP_LG: sink += shmem_long_g(lv, 1); break;
            case OP_IP: shmem_int_p(iv, i, 1); shmem_quiet(); break;
            case OP_IG: sink += shmem_int_g(iv, 1); break;
            case OP_IPUT: shmem_long_iput(strided, larr, 2, 1, nelem, 1); shmem_quiet(); break;
            case OP_IGET: shmem_long_iget(larr, strided, 1, 2, nelem, 1); break;
            /* non-fetching AMOs return once posted: quiet so the time is to completion, like the fetching forms */
            case OP_INC: shmem_long_atomic_inc(lv, 1); shmem_quiet(); break;
            case OP_FINC: sink += shmem_long_atomic_fetch_inc(lv, 1); break;
            case OP_SET: shmem_long_atomic_set(lv, i, 1); shmem_quiet(); break;
            case OP_AND: shmem_ulong_atomic_and(uv, ~0UL, 1); shmem_quiet(); break;
            case OP_OR: shmem_ulong_atomic_or(uv, 0, 1); shmem_quiet(); break;
            case OP_XOR: shmem_ulong_atomic_xor(uv, 0, 1); shmem_quiet(); break;
            case OP_FAND: sink += shmem_ulong_atomic_fetch_and(uv, ~0UL, 1); break;
            case OP_FOR: sink += shmem_ulong_atomic_fetch_or(uv, 0, 1); break;
            case OP_FXOR: sink += shmem_ulong_atomic_fetch_xor(uv, 0, 1); break;
            case OP_IFADD: sink += shmem_int_atomic_fetch_add(iv, 1, 1); break;
            case OP_ICSWAP: sink += shmem_int_atomic_compare_swap(iv, 0, 1, 1); break;
            case OP_ISWAP: sink += shmem_int_atomic_swap(iv, i, 1); break;
            case OP_FADD_NBI: { long f; shmem_long_atomic_fetch_add_nbi(&f, lv, 1, 1); shmem_quiet(); sink += f; } break;
            }
            double dt = now_us() - t0;
            if (i >= warm) t[i - warm] = dt;
        }
        (void)sink;
        size_t bytes = 8;
        if (op == OP_IPUT || op == OP_IGET) bytes = (size_t)nelem * 8;
        else if (op == OP_IP || op == OP_IG || op == OP_IFADD || op == OP_ICSWAP || op == OP_ISWAP) bytes = 4;
        stats_csv(op_names[op], bytes, "heap", t, iters);
        free(t);
    }
    shmem_barrier_all();
}

/* a window of non-blocking fetching AMOs completed by one quiet: shows whether the _nbi form overlaps */
static void fadd_nbi_window(long *lv) {
    int window = 64, reps = quick ? 10 : 100, trials = quick ? 2 : 5;
    long *res = shmem_malloc(window * sizeof(long));
    double rates[16];
    shmem_barrier_all();
    for (int tr = 0; tr < trials + 1; tr++) {
        shmem_barrier_all();
        if (me == 0) {
            double t0 = now_us();
            for (int r = 0; r < reps; r++) {
                for (int w = 0; w < window; w++) shmem_long_atomic_fetch_add_nbi(&res[w], lv, 1, 1);
                shmem_quiet();
            }
            if (tr > 0) rates[tr - 1] = (double)window * reps / (now_us() - t0);
        }
    }
    if (me == 0) {
        qsort(rates, trials, sizeof(double), cmp_d);
        csv("op_fadd_nbi_window", 8, "heap", 1.0 / rates[trials / 2], 0, 1.0 / rates[trials / 2], 0, 0, rates[trials / 2]);
    }
    shmem_barrier_all();
    shmem_free(res);
}

/* cost pieces of a blocking put, PE 0 -> PE 1: an empty quiet, a put with no quiet (a quiet every 256 puts, outside
 * the timing, keeps the queue bounded), and the per-op cost of a batch of 64 non-blocking puts + one quiet */
static void cost_pieces(char *heap) {
    int it = quick ? 300 : 3000, warm = 100;
    static const size_t sz[] = {8, 4096};
    double *t = malloc((it + warm) * sizeof(double));
    shmem_barrier_all();
    if (me == 0) {
        for (int i = 0; i < warm + it; i++) {
            double t0 = now_us();
            shmem_quiet();
            double dt = now_us() - t0;
            if (i >= warm) t[i - warm] = dt;
        }
        stats_csv("op_quiet_empty", 0, "heap", t, it);
    }
    shmem_barrier_all();
    for (size_t k = 0; k < sizeof sz / sizeof *sz; k++) {
        if (me == 0) {
            for (int i = 0; i < warm + it; i++) {
                double t0 = now_us();
                shmem_putmem(heap, src, sz[k], 1);
                double dt = now_us() - t0;
                if (i >= warm) t[i - warm] = dt;
                if ((i & 255) == 255) shmem_quiet();
            }
            shmem_quiet();
            stats_csv("op_put_noquiet", sz[k], "heap", t, it);
        }
        shmem_barrier_all();
        if (me == 0) {
            int reps = it / 64 > 10 ? it / 64 : 10, n = 0;
            for (int r = 0; r < reps + 5; r++) {
                double t0 = now_us();
                for (int w = 0; w < 64; w++) shmem_putmem_nbi(heap, src, sz[k], 1);
                shmem_quiet();
                double dt = (now_us() - t0) / 64;
                if (r >= 5) t[n++] = dt;
            }
            stats_csv("op_putnbi_batch64", sz[k], "heap", t, n);
        }
        shmem_barrier_all();
    }
    free(t);
}

static void ops(void) {
    if (np < 2) return;
    char *cp = shmem_malloc(4096);
    cost_pieces(cp);
    shmem_free(cp);
    long *lv = shmem_calloc(1, sizeof(long));
    int *iv = shmem_calloc(1, sizeof(int));
    unsigned long *uv = shmem_calloc(1, sizeof(unsigned long));
    long *strided = shmem_calloc(2 * 1024 + 2, sizeof(long));
    for (int op = 0; op < OP_N; op++) {
        if (op == OP_IPUT || op == OP_IGET) { ops_one(op, lv, iv, uv, strided, 64); ops_one(op, lv, iv, uv, strided, 1024); }
        else ops_one(op, lv, iv, uv, strided, 0);
    }
    fadd_nbi_window(lv);
    shmem_free(strided); shmem_free(uv); shmem_free(iv); shmem_free(lv);
}

/* ---- many PEs active at once: concurrent pairs, random targets, all-to-all put ------------------- */
static double *red; /* symmetric scalar used to gather one number per PE on PE 0 */
static void gather_stats(double v, double *mx, double *sum) {
    *red = v;
    shmem_barrier_all();
    *mx = 0; *sum = 0;
    if (me == 0) for (int p = 0; p < np; p++) { double e = shmem_double_g(red, p); if (e > *mx) *mx = e; *sum += e; }
    shmem_barrier_all();
}

/* Report one multi-PE result: p50 is the slowest PE's own median per-op time, rate the aggregate over all PEs
 * (ops per us of the slowest PE's elapsed time), bw the aggregate payload rate. */
static void multi_report(const char *name, size_t size, double p50, double total_ops, double max_elapsed) {
    double rate = total_ops / max_elapsed; /* Mops/s */
    csv(name, size, "heap", p50, 0, p50, 0, rate * size, rate);
}

/* kind: 0 put+quiet, 1 get, 2 fetch_add; target(i): 0 fixed partner across the machine, 1 pseudo-random */
static void multi_pairs(const char *name, int kind, int random, size_t size, char *buf, long *ctr) {
    if (np < 2) return;
    int iters = quick ? 200 : size <= 4096 ? 2000 : 300, warm = iters / 10 > 10 ? iters / 10 : 10;
    unsigned long long rng = 88172645463325252ULL ^ ((unsigned long long)(me + 1) * 0x9E3779B97F4A7C15ULL);
    int partner = (me + np / 2) % np;
    if (partner == me) partner = (me + 1) % np;
    double *t = malloc(iters * sizeof(double));
    shmem_barrier_all();
    double tstart = 0;
    for (int i = 0; i < warm + iters; i++) {
        int tgt = partner;
        if (random) {
            rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
            tgt = (me + 1 + (int)(rng % (unsigned long long)(np - 1))) % np;
        }
        if (i == warm) tstart = now_us();
        double t0 = now_us();
        if (kind == 0) { shmem_putmem(buf, src, size, tgt); shmem_quiet(); }
        else if (kind == 1) shmem_getmem(dst, buf, size, tgt);
        else (void)shmem_long_atomic_fetch_add(ctr, 1, tgt);
        double dt = now_us() - t0;
        if (i >= warm) t[i - warm] = dt;
    }
    double elapsed = now_us() - tstart;
    qsort(t, iters, sizeof(double), cmp_d);
    double p50 = t[iters / 2], mx50, sum50, mxel, sumel;
    gather_stats(p50, &mx50, &sum50);
    gather_stats(elapsed, &mxel, &sumel);
    if (me == 0) multi_report(name, size, mx50, (double)np * iters, mxel);
    free(t);
    shmem_barrier_all();
}

/* every PE puts `size` bytes to every other PE (non-blocking), then quiet; time of a full round */
static void multi_alltoall(size_t size, char *region) {
    if (np < 2) return;
    int iters = quick ? 10 : (size <= 512 ? 200 : 100), warm = 5;
    double *t = malloc(iters * sizeof(double));
    shmem_barrier_all();
    double tstart = 0;
    for (int i = 0; i < warm + iters; i++) {
        shmem_barrier_all();
        if (i == warm) tstart = now_us();
        double t0 = now_us();
        for (int k = 1; k < np; k++) shmem_putmem_nbi(region + (size_t)me * size, src, size, (me + k) % np);
        shmem_quiet();
        double dt = now_us() - t0;
        if (i >= warm) t[i - warm] = dt;
    }
    double elapsed = now_us() - tstart;
    qsort(t, iters, sizeof(double), cmp_d);
    double mx50, s50, mxel, sel;
    gather_stats(t[iters / 2], &mx50, &s50);
    gather_stats(elapsed, &mxel, &sel);
    if (me == 0) {
        double msgs = (double)np * (np - 1); /* messages per round */
        csv("multi_alltoall_put", size, "heap", mx50, 0, mx50, 0, msgs * size / mx50, msgs / mx50);
    }
    (void)mxel; (void)sel;
    free(t);
    shmem_barrier_all();
}

static void multi(void) {
    if (np < 2) return;
    red = shmem_malloc(sizeof(double));
    char *buf = shmem_malloc(65536);
    char *a2a = shmem_malloc((size_t)np * 4096);
    long *ctr = shmem_calloc(1, sizeof(long));
    static const size_t sz[] = {8, 4096, 65536};
    for (int k = 0; k < 3; k++) {
        multi_pairs("multi_pairs_put", 0, 0, sz[k], buf, ctr);
        multi_pairs("multi_pairs_get", 1, 0, sz[k], buf, ctr);
    }
    multi_pairs("multi_rand_put", 0, 1, 8, buf, ctr);
    multi_pairs("multi_rand_get", 1, 1, 8, buf, ctr);
    multi_pairs("multi_rand_fadd", 2, 1, 8, buf, ctr);
    multi_alltoall(64, a2a);
    multi_alltoall(4096, a2a);
    shmem_free(ctr); shmem_free(a2a); shmem_free(buf); shmem_free(red);
}

/* ---- sweep behind the 4 KiB concurrent-pairs gap: message size x senders per node x direction -----
 * Half-machine partners as in multi_pairs. k = how many PEs per node send (the rest only take part in the
 * barriers); uni = only the first half of the PEs send (node 0 -> node 1), otherwise both halves send.
 * Needs BENCH_PPN (PEs per node; the runner sets it). Result name: sweep_<put|get>_<bi|uni>_k<k>. */
static void sweep_case(int kind, int uni, int k, int ppn, size_t size, char *buf) {
    int half = np / 2, partner = (me + half) % np;
    int active = (me % ppn) < k && (!uni || me < half);
    int iters = quick ? 100 : 1000, warm = 50;
    double *t = malloc(iters * sizeof(double)), *tp = malloc(iters * sizeof(double)), *tq = malloc(iters * sizeof(double));
    int split = kind == 0 && getenv("BENCH_SWEEP_SPLIT") != NULL;  /* also report the putmem call and the quiet separately */
    shmem_barrier_all();
    double tstart = 0;
    /* BENCH_SWEEP_JITTER_US=N: before each operation (outside the timed region) wait a random 0..N us, so the PEs cannot
     * stay in lock-step (tests whether a result depends on all PEs posting at the same instant) */
    double jit = getenv("BENCH_SWEEP_JITTER_US") ? atof(getenv("BENCH_SWEEP_JITTER_US")) : 0;
    unsigned long long jr = 0x9E3779B97F4A7C15ULL * (unsigned long long)(me + 1);
    for (int i = 0; i < warm + iters && active; i++) {
        if (jit > 0) {
            jr ^= jr << 13; jr ^= jr >> 7; jr ^= jr << 17;
            double until = now_us() + jit * (double)(jr % 1000003ULL) / 1000003.0;
            while (now_us() < until) { }
        }
        if (i == warm) tstart = now_us();
        double t0 = now_us(), t1 = t0;
        if (kind == 0) { shmem_putmem(buf, src, size, partner); t1 = now_us(); shmem_quiet(); }
        else shmem_getmem(dst, buf, size, partner);
        double dt = now_us() - t0;
        if (i >= warm) { t[i - warm] = dt; tp[i - warm] = t1 - t0; tq[i - warm] = dt - (t1 - t0); }
    }
    double elapsed = active ? now_us() - tstart : 0, p50 = 0, pp50 = 0, pq50 = 0;
    if (active) {
        qsort(t, iters, sizeof(double), cmp_d); p50 = t[iters / 2];
        qsort(tp, iters, sizeof(double), cmp_d); pp50 = tp[iters / 2];
        qsort(tq, iters, sizeof(double), cmp_d); pq50 = tq[iters / 2];
    }
    double mx50, s50, mxel, sel, mxa, nact, mxp, mxq;
    gather_stats(p50, &mx50, &s50);
    gather_stats(elapsed, &mxel, &sel);
    gather_stats(active ? 1.0 : 0.0, &mxa, &nact);
    if (split) {
        gather_stats(pp50, &mxp, &s50);
        gather_stats(pq50, &mxq, &s50);
        if (me == 0 && nact > 0) {
            char nm[72];
            snprintf(nm, sizeof nm, "sweep_put_%s_k%d_putcall", uni ? "uni" : "bi", k);
            csv(nm, size, "heap", mxp, 0, mxp, 0, 0, 0);
            snprintf(nm, sizeof nm, "sweep_put_%s_k%d_quiet", uni ? "uni" : "bi", k);
            csv(nm, size, "heap", mxq, 0, mxq, 0, 0, 0);
        }
    }
    if (me == 0 && nact > 0) {
        char name[64];
        snprintf(name, sizeof name, "sweep_%s_%s_k%d", kind ? "get" : "put", uni ? "uni" : "bi", k);
        multi_report(name, size, mx50, nact * iters, mxel);
    }
    free(t); free(tp); free(tq);
    shmem_barrier_all();
}

static void sweep(void) {
    if (np < 4) return;
    int ppn = getenv("BENCH_PPN") ? atoi(getenv("BENCH_PPN")) : np / 2;
    /* 1536, 2049, 2560, 3072 sit around UCX's short/copy-in boundary at 2 KiB (ucx backend) */
    static const size_t sizes[] = {1024, 1536, 2048, 2049, 2560, 3072, 4096, 8192, 16384};
    static const int ks[] = {4, 16, 32};
    red = shmem_malloc(sizeof(double));
    char *buf = shmem_malloc(16384);
    /* BENCH_SWEEP_ONLY=put_bi_k32 runs just that configuration; BENCH_SWEEP_SIZES=2048,4096 just those sizes */
    const char *only = getenv("BENCH_SWEEP_ONLY"), *szs = getenv("BENCH_SWEEP_SIZES");
    char szlist[128] = "";
    if (szs) snprintf(szlist, sizeof szlist, ",%s,", szs);
    for (size_t ki = 0; ki < sizeof ks / sizeof *ks; ki++) {
        if (ks[ki] > ppn) continue;
        for (int uni = 0; uni < 2; uni++)
            for (int kind = 0; kind < 2; kind++) {
                char cfg[48];
                snprintf(cfg, sizeof cfg, "%s_%s_k%d", kind ? "get" : "put", uni ? "uni" : "bi", ks[ki]);
                if (only && strcmp(only, cfg)) continue;
                for (size_t si = 0; si < sizeof sizes / sizeof *sizes; si++) {
                    char tok[24];
                    snprintf(tok, sizeof tok, ",%zu,", sizes[si]);
                    if (szs && !strstr(szlist, tok)) continue;
                    sweep_case(kind, uni, ks[ki], ppn, sizes[si], buf);
                }
            }
    }
    shmem_free(buf); shmem_free(red);
}

/* BENCH_INFO=1: every PE reports where it runs and how many threads the runtime created. */
static void info(void) {
    char host[64] = "?", line[256], cpus[128] = "?";
    int threads = -1;
    gethostname(host, sizeof host);
    FILE *f = fopen("/proc/self/status", "r");
    while (f && fgets(line, sizeof line, f)) {
        if (!strncmp(line, "Threads:", 8)) threads = atoi(line + 8);
        if (!strncmp(line, "Cpus_allowed_list:", 18)) { sscanf(line + 18, " %127s", cpus); }
    }
    if (f) fclose(f);
    /* which UCX is actually mapped into this process (none when UCX is linked in statically) */
    char ucp[320] = "none";
    f = fopen("/proc/self/maps", "r");
    while (f && fgets(line, sizeof line, f)) {
        char *p = strstr(line, "libucp.so");
        if (p) {
            char *s = strchr(line, '/');
            if (s) { snprintf(ucp, sizeof ucp, "%s", s); ucp[strcspn(ucp, "\n")] = 0; }
            break;
        }
    }
    if (f) fclose(f);
    /* every UCX_* variable in this process after shmem_init (an implementation may setenv them itself) */
    extern char **environ;
    char ucxenv[512] = "";
    for (char **e = environ; e && *e; e++)
        if (!strncmp(*e, "UCX_", 4) && strlen(ucxenv) + strlen(*e) + 2 < sizeof ucxenv) { strcat(ucxenv, *e); strcat(ucxenv, ";"); }
    printf("INFO,pe=%d,host=%s,threads=%d,cpus=%s,ucp=%s,ucxenv=%s\n", me, host, threads, cpus, ucp, ucxenv[0] ? ucxenv : "none");
    fflush(stdout);
}

int main(int argc, char **argv) {
    if (getenv("BENCH_THREAD_MULTIPLE")) { /* request SHMEM_THREAD_MULTIPLE (implementations may configure their transport for it) */
        int provided = -1;
        shmem_init_thread(SHMEM_THREAD_MULTIPLE, &provided);
        if (getenv("BENCH_INFO")) printf("INFO,thread_level_provided=%d\n", provided);
    } else {
        shmem_init();
    }
    me = shmem_my_pe(); np = shmem_n_pes();
    quick = getenv("BENCH_QUICK") != NULL;
    no_global = getenv("BENCH_NO_GLOBAL") != NULL;
    src = malloc(MAXSZ); dst = malloc(MAXSZ);
    memset(src, 0x5a, MAXSZ); memset(dst, 0, MAXSZ); /* fault the pages in before timing */
    memset(glob_buf, 0, sizeof glob_buf);
    if (getenv("BENCH_INFO")) info();
    if (me == 0) printf("CSV_HEADER,bench,size_bytes,np,mem,p50_us,p99_us,mean_us,min_us,bw_MBps,rate_Mops\n");

    int all = argc < 2;
    for (int i = 1; i < argc || all; i++) {
        const char *sec = all ? "all" : argv[i];
        int a = !strcmp(sec, "all");
        if (a || !strcmp(sec, "pt2pt")) pt2pt();
        if (a || !strcmp(sec, "amo")) amo();
        if (a || !strcmp(sec, "contend")) contend();
        if (a || !strcmp(sec, "barrier")) barrier_bench();
        if (a || !strcmp(sec, "coll")) coll();
        if (a || !strcmp(sec, "ops")) ops();
        if (a || !strcmp(sec, "multi")) multi();
        if (!a && !strcmp(sec, "sweep")) sweep(); /* explicit only: not part of the default set */
        if (all) break;
    }
    shmem_barrier_all();
    free(src); free(dst);
    shmem_finalize();
    return 0;
}
