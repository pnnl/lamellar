#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "utils.h"
#include <rofi.h>

int main(int argc, char **argv) {
    int repetitions = 1;
    struct timespec tstart = {0}, tend = {0};
    double elapsed_us = 0.0;

    if (argc > 1) {
        repetitions = atoi(argv[1]);
        if (repetitions <= 0) {
            fprintf(stderr, "Invalid number of repetitions. Must be a positive integer.\n");
            return -1;
        }
    }

    rofi_init("verbs", NULL);
    uint32_t id = rofi_get_id();
    uint32_t size = rofi_get_size();

    if (id == 0) {
        rofi_banner("Atomic Add U32 Test");
        fprintf(stderr, "Atomic add test with %u processes, repeated %d times\n", size, repetitions);
    }

    rofi_barrier();

    uint32_t *ptr = NULL;
    int ret = rofi_alloc(2*sizeof(uint32_t), 0, (void **)&ptr);
    if (ret) {
        printf("Error allocating memory for global counter\n");
        return -1;
    }

    uint32_t *done = ptr + 1;

    if(id == 0) {
        *done = 0;
    }

#ifdef DEBUG_MODE    
    if (id == 0) {
        *ptr = 0;
        printf("ID: %u/%u, ptr: %p, value: %u\n", id, size, ptr, *ptr);
    }
#endif
    rofi_barrier();
    clock_gettime(CLOCK_MONOTONIC, &tstart);

    int err = 0;
    for (int i = 0; i < repetitions; ++i) {
        uint32_t value = 1;
        ssize_t res = rofi_atomic_op(ptr, &value, 1, ROFI_DATATYPE_UINT32, ROFI_ATOMIC_OP_SUM, 0);
        if (res) {
            fprintf(stderr, "ID: %u/%u Error in atomic add u32 (%ld)\n", id, size, res);
            err = 1;
            break;
        }
    }

    uint32_t done_value = 1;
    ssize_t res = rofi_atomic_op(done, &done_value, 1, ROFI_DATATYPE_UINT32, ROFI_ATOMIC_OP_SUM, 0);
    if (res) {
        fprintf(stderr, "ID: %u/%u Error in atomic add u32 (%ld)\n", id, size, res);
        goto out;
    }

    if(id == 0)
        while (*done < size)
            rofi_wait();
    
    clock_gettime(CLOCK_MONOTONIC, &tend);

    elapsed_us = (tend.tv_sec - tstart.tv_sec) * 1e6 + (tend.tv_nsec - tstart.tv_nsec) / 1e3;
    unsigned long expected = (unsigned long)size * (unsigned long)repetitions;
    
    if(id == 0) {
#ifdef DEBUG_MODE
        printf("ID: %u/%u Results: ptr: %p, value: %u (assert %lu)\n", id, size, ptr, *ptr, expected);
        fprintf(stderr, "Atomic add test completed %s (%u) in %10.2fus\n",
                err ? "with errors" : "successfully", *ptr, elapsed_us);
#endif
        fprintf(stdout, "%u %d %10.2f\n", size, repetitions, elapsed_us);
#ifdef DEBUG_MODE
        rofi_verify(!(*ptr == expected));
#endif
    }

    fprintf(stderr, "ID: %u/%u Finalized ROFI runtime.\n", id, size);
out:    
    rofi_release(ptr);
    rofi_finit();

 
    return 0;
}
