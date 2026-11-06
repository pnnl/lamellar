/*********************************************************************************/
/* Wrappers for C APIs that depend on macros, along with the _end bug workaround */
/*********************************************************************************/

#define _GNU_SOURCE
#include <sched.h>

#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <unistd.h>

#include <shmem.h>

/* ------------------------------------------------------------------*/
/* Platform-specific wrangling. */
/* ------------------------------------------------------------------*/

#if (_SHMEM_MAJOR_VERSION == 1 && _SHMEM_MINOR_VERSION >= 4)

// OpenShmem 1.4+ versions of these functions

// This is only used to determine how many PEs are on the node,
// so for now just return NULL.
void *shmem_local_ptr(void *p, size_t targ_len, int pe) {
    return NULL;
}

static int init() {
    int ret;
    shmem_init_thread(SHMEM_THREAD_MULTIPLE, &ret);
    return ret;
}

// These are Cray-specific functions.
// They are not needed with sufficiently recent OpenShmem.
void shmem_thread_register(void) {}
void shmem_thread_unregister(void) {}

#if defined(OSHMEM_MAJOR_VERSION) && OSHMEM_MAJOR_VERSION == 4 && OSHMEM_MINOR_VERSION == 0

// Buggy OShmem 4.0.0 release, which incorrectly claims to support OpenShmem 1.4
// The context support is broken; also, it is missing some of the 1.4-mandated atomics.

#define HAVE_CONTEXTS 0

#else // OSHMEM VERSION 4.0

#define HAVE_CONTEXTS 1

#endif // OSHMEM VERSION 4.0

#if defined(OSHMEM_MAJOR_VERSION) && OSHMEM_MAJOR_VERSION == 4

// At least early OShmem 4.* releases put some of the int64
// atomics in shmemx rather than shmem.

#include <shmemx.h>

int64_t shmem_int64_atomic_fetch(int64_t *src, int pe) {
    shmemx_int64_fetch(src, pe);
}

void shmem_int64_atomic_set(int64_t *dst, int64_t val, int pe) {
    shmemx_int64_set(dst, val, pe);
}

void shmem_int64_atomic_or(int64_t *dst, int64_t val, int pe) {
    shmemx_int64_atomic_or((unsigned long *)dst, val, pe);
}

#endif // OSHMEM VERSION 4.x

#elif defined(CRAY_SHMEM_MAJOR_VERSION) && (CRAY_SHMEM_MAJOR_VERSION == 7 && CRAY_SHMEM_MINOR_VERSION >= 7)

// While Cray Shmem v7.7 does not fully support OpenShmem 1.4, it
// has the most important features.

#define HAVE_CONTEXTS 0

static int init() {
    return shmem_init_thread(SHMEM_THREAD_MULTIPLE);
}

#else

#error "This library requires OpenShmem 1.4 or later"

#endif

/* ------------------------------------------------------------------*/
/* Contexts  */
/* ------------------------------------------------------------------*/

#if HAVE_CONTEXTS

void *crpgas_create_context(long options) {
    shmem_ctx_t *ctx;
    ctx = malloc(sizeof(shmem_ctx_t));
    shmem_ctx_create(options, ctx);
    return ctx;
}

void crpgas_destroy_context(void *ctx) {
    shmem_ctx_destroy(*(shmem_ctx_t *)ctx);
    free(ctx);
}

void crpgas_fence(void *ctx) {
    shmem_ctx_fence(*(shmem_ctx_t *)ctx);
}

void crpgas_quiet(void *ctx) {
    shmem_ctx_quiet(*(shmem_ctx_t *)ctx);
}

#else // ...HAVE_CONTEXTS is false

// Contexts aren't supported for this Shmem version,
// so all fence and quiet operations will be global.

void *crpgas_create_context(long options) {
    return NULL;
}

void crpgas_destroy_context(void *ctx) {}

void crpgas_fence(void *ctx) {
    shmem_fence();
}

void crpgas_quiet(void *ctx) {
    shmem_quiet();
}

#endif

/* ------------------------------------------------------------------*/
/* Common code. */
/* ------------------------------------------------------------------*/

int crpgas_init() {
    return (init() == SHMEM_THREAD_MULTIPLE);
}

long sched_affinity_cores() {
    cpu_set_t cpumask;
    int depth;
    int ret = sched_getaffinity(getpid(), sizeof(cpu_set_t), &cpumask);
    if (ret) {
        return 0;
    }
    depth = CPU_COUNT(&cpumask);
    return depth;
}

// Cray Shmem uses _end, but Rust does not export it as a public symbol;
//   as a result, Shmem ends up using the wrong _end value and gets confused.
// So here we force _end to be a public symbol.
// (There must be a linker argument that would also fix this, but I haven't
// found it.)
extern char _end;

const void *crpgas_get_endptr() {
    // This is here just to force _end to be used.
    return &_end;
}

/* Finally, this is a debugging tool to help diagnose certain
   memory problems between Rust and Shmem. */

void crpgas_debug_printseg(void) {
    char buffer[1000];
    FILE *f;
    f = fopen("/proc/self/maps", "r");
    if (f == NULL) {
        printf("ERROR in crpgas_debug_printseg: Could not open /proc/self/maps.\n");
        return;
    }
    printf("=======================\nSEGMENTS:  _end at %p\n", &_end);
    while (!feof(f)) {
        if (fgets(buffer, 1000, f) == NULL) {
            break;
        }
        printf("%s", buffer);
    }
    printf("=======================\n");
    fflush(stdout);
}


