/* Non-blocking put/get: issue many large *_nbi operations, complete them with shmem_quiet, and verify every byte.
 * A quiet that returns early (or an nbi that writes the wrong place) shows up as corrupt data. Also prints how long
 * issuing took vs quiet, which shows whether the calls really are asynchronous (informational).
 * Targets: heap, and a static array unless NBI_NO_GLOBAL is set. */
#define _POSIX_C_SOURCE 200809L
#include <shmem.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define BLK (1u << 20)
#define NBLK 16
static char gbuf[NBLK * BLK];   /* 16 MiB .bss target */
static double now_us(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e6 + t.tv_nsec * 1e-3; }
static unsigned char pat(int pe, size_t i) { return (unsigned char)(i * 131 + pe * 29 + (i >> 11)); }

static int check(const char *what, const unsigned char *p, int from_pe, size_t n) {
    for (size_t i = 0; i < n; i++)
        if (p[i] != pat(from_pe, i)) { printf("FAIL: %s mismatch at byte %zu\n", what, i); return 1; }
    return 0;
}

int main(void) {
    shmem_init();
    int me = shmem_my_pe(), np = shmem_n_pes(), bad = 0;
    int right = (me + 1) % np, left = (me + np - 1) % np, use_global = getenv("NBI_NO_GLOBAL") == NULL;
    size_t total = (size_t)NBLK * BLK;
    unsigned char *hbuf = shmem_malloc(total);
    unsigned char *src = malloc(total), *dst = malloc(total);
    for (size_t i = 0; i < total; i++) src[i] = pat(me, i);
    memset(hbuf, 0, total); memset(gbuf, 0, sizeof gbuf);
    shmem_barrier_all();

    /* puts: heap and global targets, all issued before a single quiet */
    double t0 = now_us();
    for (int i = 0; i < NBLK; i++) {
        shmem_putmem_nbi(hbuf + (size_t)i * BLK, src + (size_t)i * BLK, BLK, right);
        if (use_global) shmem_putmem_nbi(gbuf + (size_t)i * BLK, src + (size_t)i * BLK, BLK, right);
    }
    double t1 = now_us();
    shmem_quiet();
    double t2 = now_us();
    shmem_barrier_all();
    bad += check("nbi put to heap", hbuf, left, total);
    if (use_global) bad += check("nbi put to global", (unsigned char *)gbuf, left, total);

    /* typed nbi */
    int *ib = shmem_calloc(1024, sizeof(int));
    int isrc[1024];
    for (int i = 0; i < 1024; i++) isrc[i] = me * 100000 + i;
    shmem_int_put_nbi(ib, isrc, 1024, right);
    shmem_quiet();
    shmem_barrier_all();
    for (int i = 0; i < 1024; i++) if (ib[i] != left * 100000 + i) { printf("FAIL: typed nbi put at %d\n", i); bad++; break; }

    /* gets: what my left neighbour wrote to me is on my right neighbour's heap only for *their* left (me) */
    memset(dst, 0, total);
    double t3 = now_us();
    for (int i = 0; i < NBLK; i++) shmem_getmem_nbi(dst + (size_t)i * BLK, hbuf + (size_t)i * BLK, BLK, right);
    double t4 = now_us();
    shmem_quiet();
    double t5 = now_us();
    bad += check("nbi get from heap", dst, me, total);  /* right's hbuf holds data I put there */

    /* many mid-sized nbi operations from/into ordinary heap buffers, repeated with fresh contents each round: the
     * backends may stage such buffers through recycled scratch memory, so a block reused too early, a copy-out that
     * runs before the data landed, or one that is lost shows up as corrupt data here */
    {
        static const size_t sizes[] = { 513, 4096, 70000 };
        enum { OPS = 48, ROUNDS = 20 };
        unsigned char *mh = shmem_malloc((size_t)OPS * 70000);
        for (int r = 0; r < ROUNDS && !bad; r++) {
            for (int si = 0; si < 3 && !bad; si++) {
                size_t n = sizes[si];
                unsigned char *s = malloc((size_t)OPS * n), *d = malloc((size_t)OPS * n);
                for (size_t i = 0; i < (size_t)OPS * n; i++) s[i] = (unsigned char)(pat(me, i) + r * 7 + si);
                for (int k = 0; k < OPS; k++) shmem_putmem_nbi(mh + (size_t)k * n, s + (size_t)k * n, n, right);
                shmem_quiet();
                memset(s, 0xEE, (size_t)OPS * n);   /* the source of an nbi put may be reused after the quiet */
                shmem_barrier_all();
                for (int k = 0; k < OPS; k++) shmem_getmem_nbi(d + (size_t)k * n, mh + (size_t)k * n, n, right);
                shmem_quiet();
                for (size_t i = 0; i < (size_t)OPS * n; i++)
                    if (d[i] != (unsigned char)(pat(me, i) + r * 7 + si)) {
                        printf("FAIL: mixed nbi round %d size %zu mismatch at byte %zu\n", r, n, i);
                        bad++;
                        break;
                    }
                free(s); free(d);
                shmem_barrier_all();
            }
        }
        shmem_free(mh);
    }

    if (me == 0)
        printf("PE 0: nbi: put issue %.0f us, quiet %.0f us | get issue %.0f us, quiet %.0f us (16 x 1 MiB%s)\n",
               t1 - t0, t2 - t1, t4 - t3, t5 - t4, use_global ? " x heap+global for puts" : "");
    shmem_barrier_all();
    printf("PE %d/%d: %s\n", me, np, bad ? "FAILED" : "nbi OK");
    shmem_finalize();
    return bad ? 1 : 0;
}
