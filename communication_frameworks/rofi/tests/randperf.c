#define _GNU_SOURCE
#include "utils.h"
#include <rofi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

enum OpType {
    OP_GET,
    OP_PUT
};

struct args_t {
    int local_array_size;
    int global_array_size;
    int local_num_ops;
    int global_num_ops;
    enum OpType op_type;
    int num_runs;
    int seed;
};

void parse_args(int argc, char *argv[], struct args_t *args) {
    int opt;
    args->local_array_size = 100000; // default value
    args->global_array_size = 0;     // default value
    args->local_num_ops = 20000;     // default value
    args->global_num_ops = 0;        // default value
    args->op_type = OP_PUT;          // default value
    args->num_runs = 1;              // default value
    args->seed = 42;                 // default value
    while ((opt = getopt(argc, argv, "t:T:n:N:s:r:o:")) != -1) {
        switch (opt) {
        case 't': // local array size
            args->local_array_size = atoi(optarg);
            break;
        case 'T': // global array size
            args->global_array_size = atoi(optarg);
            break;
        case 'n':
            args->local_num_ops = atoi(optarg);
            break;
        case 'o': // operation type
            fprintf(stderr, "Operation type: %s\n", optarg);
            if (strcmp(optarg, "get") == 0) {
                args->op_type = OP_GET;
            }
            else if (strcmp(optarg, "put") == 0) {
                args->op_type = OP_PUT;
            }
            else {
                fprintf(stderr, "Invalid operation type: %s. Use 'get' or 'put'.\n", optarg);
                exit(1);
            }
            break;
        case 's': // random number seed
            args->seed = atoi(optarg);
            break;
        case 'N': // global num ops
            args->global_num_ops = atoi(optarg);
            break;
        case 'r': // number of runs
            args->num_runs = atoi(optarg);
            break;
        default:
            fprintf(stderr, "Usage: %s [-t array_size] [-n num_ops] -o <get|put>\n", argv[0]);
            exit(1);
        }
    }
}

void calc_pe_and_offset(int idx, int local_size, int *pe, int *offset) {
    *pe = idx / local_size;
    *offset = idx % local_size;
}

// puts do not need to complete on the remote side before returning from this function
int put_op(unsigned long long *array, int pe, int offset, unsigned long long value) {
    if (rofi_put(&array[offset], &value, sizeof(unsigned long long), pe, 0x0)) {
        fprintf(stderr, "Error in put to PE %d index %d\n", pe, offset);
        exit(1);
    }
    return 0;
}

// gets need to complete on the remote side before returning from this function
int get_op(unsigned long long *array, int pe, int offset, unsigned long long *value) {
    if (rofi_iget(value, &array[offset], sizeof(unsigned long long), pe, 0x0)) {
        fprintf(stderr, "Error in get from PE %d index %d\n", pe, offset);
        exit(1);
    }
    return *value;
}

int main(int argc, char *argv[]) {
    struct args_t args;
    parse_args(argc, argv, &args);

    rofi_init("verbs", "mlx5_0");
    int np = rofi_get_size();
    int me = rofi_get_id();
    if (args.global_array_size == 0) {
        args.global_array_size = args.local_array_size * np;
    }
    else {
        args.local_array_size = args.global_array_size / np;
    }
    if (args.global_num_ops == 0) {
        args.global_num_ops = args.local_num_ops * np;
    }
    else {
        args.local_num_ops = args.global_num_ops / np;
    }

    int (*op_ptr)(unsigned long long *, int, int, unsigned long long *);
    if (args.op_type == OP_GET) {
        op_ptr = get_op;
    }
    else {
        op_ptr = put_op;
    }

    printf("Global Array size: %d\n", args.global_array_size);
    printf("Total number of operations to perform: %d\n", args.global_num_ops);

    unsigned long long *array;

    // Array is the distributed array
    if (rofi_alloc(args.local_array_size * sizeof(unsigned long long), 0x0, (void **)&array)) {
        fprintf(stderr, "[%d] Error allocating array\n", me);
        return 1;
    }

    struct timespec start, end;
    for (int run = 0; run < args.num_runs; run++) {
        //
        // initialize array
        for (int i = 0; i < args.local_array_size; i++) {
            array[i] = me * args.local_array_size + i;
        }

        srand(args.seed + run + me * 523423);
        int pe, offset;
        unsigned long long accum = 0;
        rofi_barrier();
        clock_gettime(CLOCK_MONOTONIC, &start);
        // ----------------start randperf ---------------------//
        for (int j = 0; j < args.local_num_ops; j++) {
            unsigned long long val = (unsigned long long)rand();
            int idx = (3 * val) % (args.global_array_size);
            calc_pe_and_offset(idx, args.local_array_size, &pe, &offset);
            accum += op_ptr(array, pe, offset, &val);
        }
        rofi_wait(); // needed to ensure all puts complete, shouldnt impact timing for gets much
        rofi_barrier();
        // ----------------end randperf ---------------------
        clock_gettime(CLOCK_MONOTONIC, &end);

        unsigned long long local_sum = 0;
        for (int i = 0; i < args.local_array_size; i++) {
            local_sum += array[i];
        }

        printf("Run %d: Local sum is %llu Accum is %llu\n", run, local_sum, accum);

        double total_time = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
        if (me == 0) {
            printf("FINAL TIME: %f seconds\n", total_time);
        }
    }

    return 0;
}