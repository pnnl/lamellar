#define _GNU_SOURCE
#include "utils.h"
#include <rofi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

struct args_t {
    int local_table_size;
    int global_table_size;
    int local_num_gathers;
    int global_num_gathers;
    int seed;
};

void parse_args(int argc, char *argv[], struct args_t *args) {
    int opt;
    args->local_table_size = 100000; // default value
    args->global_table_size = 0;     // default value
    args->local_num_gathers = 10000; // default value
    args->global_num_gathers = 0;    // default value
    args->seed = 42;                 // default value
    while ((opt = getopt(argc, argv, "t:T:n:N:s:")) != -1) {
        switch (opt) {
        case 't': // local table size
            args->local_table_size = atoi(optarg);
            break;
        case 'T': // global table size
            args->global_table_size = atoi(optarg);
            break;
        case 'n':
            args->local_num_gathers = atoi(optarg);
            break;
        case 's': // random number seed
            args->seed = atoi(optarg);
            break;
        case 'N': // global num gathers
            args->global_num_gathers = atoi(optarg);
            break;
        default:
            fprintf(stderr, "Usage: %s [-t table_size] [-n num_gathers]\n", argv[0]);
            exit(1);
        }
    }
}

int main(int argc, char *argv[]) {
    struct args_t args;
    parse_args(argc, argv, &args);

    rofi_init("verbs", "mlx5_0");
    int np = rofi_get_size();
    int me = rofi_get_id();
    if (args.global_table_size == 0) {
        args.global_table_size = args.local_table_size * np;
    }
    else {
        args.local_table_size = args.global_table_size / np;
    }
    if (args.global_num_gathers == 0) {
        args.global_num_gathers = args.local_num_gathers * np;
    }
    else {
        args.local_num_gathers = args.global_num_gathers / np;
    }

    printf("Global Table size: %d\n", args.global_table_size);
    printf("Total number of elements to gather: %d\n", args.global_num_gathers);

    unsigned long long *table;

    // Table is the distributed array
    if (rofi_alloc(args.local_table_size * sizeof(unsigned long long), 0x0, (void **)&table)) {
        fprintf(stderr, "[%d] Error allocating table\n", me);
        return 1;
    }
    // initialize table
    for (int i = 0; i < args.local_table_size; i++) {
        table[i] = me * args.local_table_size + i;
    }

    // generate random indices to gather
    int *indices = malloc(args.local_num_gathers * sizeof(int));
    unsigned long long *gathered = malloc(args.local_num_gathers * sizeof(unsigned long long));
    if (!indices || !gathered) {
        fprintf(stderr, "[%d] Error allocating indices or gathered array\n", me);
        return 1;
    }
    srand(args.seed + me);
    for (int i = 0; i < args.local_num_gathers; i++) {
        indices[i] = rand() % (args.global_table_size);
    }

    struct timespec start, end;
    rofi_barrier();

    // ----------------start index gather---------------------
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int j = 0; j < args.local_num_gathers; j++) {
        int idx = indices[j];
        int target_pe = idx / args.local_table_size;
        int local_idx = idx % args.local_table_size;
        if (rofi_get(&gathered[j], &table[local_idx], sizeof(unsigned long long), target_pe, 0x0)) {
            fprintf(stderr, "[%d] Error in indexed gather from PE %d index %d\n", me, target_pe, local_idx);
            return 1;
        }
    }
    rofi_wait();
    rofi_barrier();
    clock_gettime(CLOCK_MONOTONIC, &end);
    // ----------------end index gather---------------------

    // verify results
    int errors = 0;
    for (int i = 0; i < args.local_num_gathers; i++) {
        if (gathered[i] != indices[i]) {
            if (errors < 10) {
                fprintf(stderr, "[%d] Error at gather %d: got %llu, expected %d\n", me, i, gathered[i], indices[i]);
            }
            errors++;
        }
    }

    double total_time = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    if (me == 0) {
        printf("Indexed gather completed in %f seconds\n", total_time);
    }
    if (errors != 0) {
        printf("PE %d Indexed gather completed with %d errors\n", me, errors);
    }

    return 0;
}