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
    int buffer_size;
};

void parse_args(int argc, char *argv[], struct args_t *args) {
    int opt;
    args->local_table_size = 100000; // default value
    args->global_table_size = 0;     // default value
    args->local_num_gathers = 10000; // default value
    args->global_num_gathers = 0;    // default value
    args->seed = 42;                 // default value
    args->buffer_size = 1000;        // default value
    while ((opt = getopt(argc, argv, "t:T:n:N:s:b:")) != -1) {
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
        case 'N': // global num gathers
            args->global_num_gathers = atoi(optarg);
            break;
        case 's': // random number seed
            args->seed = atoi(optarg);
            break;
        case 'b': // buffer size
            args->buffer_size = atoi(optarg);
            break;
        default:
            fprintf(stderr, "Usage: %s [-t table_size] [-n num_gathers] [-b buffer_size] [-s seed]\n", argv[0]);
            exit(1);
        }
    }
}

void check_for_full(int *gather_ready, int *index_ready, unsigned long long **pe_gather_buffers, unsigned long long **pe_index_buffers, int **pe_gathered_idx, unsigned long long *table, unsigned long long *gathered, int *pe_offsets, int np, int me) {
    for (int p = 0; p < np; p++) {
        // check if another PE is ready to send indices
        if (index_ready[p] > 0) {
            int num_entries = index_ready[p];
            index_ready[p] = -1; // mark as being processed
            // buffer full, get data for the indices
            for (int buf_idx = 0; buf_idx < num_entries; buf_idx++) {
                int local_idx = pe_index_buffers[p][buf_idx];
                pe_index_buffers[p][buf_idx] = table[local_idx];
            }

            // send back gathered data
            rofi_iput(pe_gather_buffers[me], pe_index_buffers[p], num_entries * sizeof(unsigned long long), p, 0x0);
            rofi_iput(&gather_ready[me], &num_entries, sizeof(int), p, 0x0);
        }
        // check if another PE has sent gathered data
        if (gather_ready[p] > 0) {
            for (int k = 0; k < pe_offsets[p]; k++) {
                int gather_idx = pe_gathered_idx[p][k];
                gathered[gather_idx] = pe_gather_buffers[p][k];
            }
            pe_offsets[p] = 0;
            gather_ready[p] = -1;
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

    unsigned long long *table, *gather_buffer, *index_buffer;
    int *gather_ready, *index_ready;

    // Table is the distributed array
    if (rofi_alloc(args.local_table_size * sizeof(unsigned long long), 0x0, (void **)&table)) {
        fprintf(stderr, "[%d] Error allocating table\n", me);
        return 1;
    }

    // gather_buffer is the buffer we use to gather data into from other PEs
    if (rofi_alloc(args.buffer_size * sizeof(unsigned long long) * np, 0x0, (void **)&gather_buffer)) {
        fprintf(stderr, "[%d] Error allocating buffer\n", me);
        return 1;
    }

    // index_buffer is the buffer we use to send indices to other PEs
    if (rofi_alloc(args.buffer_size * sizeof(unsigned long long) * np, 0x0, (void **)&index_buffer)) {
        fprintf(stderr, "[%d] Error allocating buffer\n", me);
        return 1;
    }

    // gather_ready is an array of flags, one per PE, indicating if that PE has data ready for us
    if (rofi_alloc(np * sizeof(int), 0x0, (void **)&gather_ready)) {
        fprintf(stderr, "[%d] Error allocating buffer ready array\n", me);
        return 1;
    }

    // index_ready is an array of flags, one per PE, indicating if that PE has indices ready for us
    if (rofi_alloc(np * sizeof(int), 0x0, (void **)&index_ready)) {
        fprintf(stderr, "[%d] Error allocating buffer ready array\n", me);
        return 1;
    }

    // initialize data
    for (int i = 0; i < args.local_table_size; i++) {
        table[i] = me * args.local_table_size + i;
    }
    memset(gather_buffer, 0, args.buffer_size * sizeof(unsigned long long) * np);
    memset(index_buffer, 0, args.buffer_size * sizeof(unsigned long long) * np);
    memset(gather_ready, 0, np * sizeof(int));
    memset(index_ready, 0, np * sizeof(int));

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

    // Initialize per-PE buffers

    // pe_offsets indicates how many entries are currently in the buffer for each PE
    int pe_offsets[np];

    // Arrays of pointers to each PE's buffers
    unsigned long long *pe_gather_buffers[np];
    unsigned long long *pe_index_buffers[np];

    // local_index_buffer is used to build the index buffer to send to each PE
    unsigned long long *local_index_buffer[np];

    // pe_gathered_idx correlates gathered values back to the original gather request
    int *pe_gathered_idx[np];

    // initialize
    for (int i = 0; i < np; i++) {
        pe_offsets[i] = 0;
        pe_gather_buffers[i] = &gather_buffer[i * args.buffer_size];
        pe_index_buffers[i] = &index_buffer[i * args.buffer_size];
        local_index_buffer[i] = calloc(args.buffer_size, sizeof(unsigned long long));
        pe_gathered_idx[i] = calloc(args.buffer_size, sizeof(int));
    }
    struct timespec start, end;

    rofi_barrier();
    // ----------------start index gather---------------------
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int j = 0; j < args.local_num_gathers; j++) {

        // calculate target PE and local index
        int idx = indices[j];
        int target_pe = idx / args.local_table_size;
        int local_idx = idx % args.local_table_size;

        // if the target PE is me, just copy the value
        if (target_pe == me) {
            gathered[j] = table[local_idx];
            continue;
        }

        // if the buffer for the target PE is full, wait for it to be processed
        if (pe_offsets[target_pe] == args.buffer_size) {
            while (gather_ready[target_pe] != -1 || pe_offsets[target_pe]) {
                check_for_full(gather_ready, index_ready, pe_gather_buffers, pe_index_buffers, pe_gathered_idx, table, gathered, pe_offsets, np, me);
            }
            // check_for_full(gather_ready, index_ready, pe_gather_buffers, pe_index_buffers, pe_gathered_idx, table, gathered, pe_offsets, np, me);
        }

        // add to buffer
        if (pe_offsets[target_pe] < args.buffer_size) {
            pe_gathered_idx[target_pe][pe_offsets[target_pe]] = j;
            local_index_buffer[target_pe][pe_offsets[target_pe]] = local_idx;
            pe_offsets[target_pe]++;
        }

        // if the buffer is now full, send it
        if (pe_offsets[target_pe] == args.buffer_size) {
            rofi_iput(pe_index_buffers[me], local_index_buffer[target_pe], pe_offsets[target_pe] * sizeof(unsigned long long), target_pe, 0x0);
            rofi_iput(&index_ready[me], &pe_offsets[target_pe], sizeof(int), target_pe, 0x0);
        }
        check_for_full(gather_ready, index_ready, pe_gather_buffers, pe_index_buffers, pe_gathered_idx, table, gathered, pe_offsets, np, me);
    }

    // flush remaining buffers
    for (int p = 0; p < np; p++) {
        if (pe_offsets[p] > 0) {
            rofi_iput(pe_index_buffers[me], local_index_buffer[p], pe_offsets[p] * sizeof(unsigned long long), p, 0x0);
            rofi_iput(&index_ready[me], &pe_offsets[p], sizeof(int), p, 0x0);
        }
    }
    for (int p = 0; p < np; p++) {
        while ((gather_ready[p] != -1 || pe_offsets[p] != 0) && p != me) {
            rofi_wait();
            check_for_full(gather_ready, index_ready, pe_gather_buffers, pe_index_buffers, pe_gathered_idx, table, gathered, pe_offsets, np, me);
        }
    }

    // tell all PEs we are done
    int done = -3;
    for (int p = 0; p < np; p++) {
        rofi_iput(&index_ready[me], &done, sizeof(int), p, 0x0);
    }
    for (int p = 0; p < np; p++) {
        while (index_ready[p] != -3) {
            rofi_wait();
            check_for_full(gather_ready, index_ready, pe_gather_buffers, pe_index_buffers, pe_gathered_idx, table, gathered, pe_offsets, np, me);
        }
    }
    rofi_barrier();
    clock_gettime(CLOCK_MONOTONIC, &end);
    // ----------------end index gather---------------------

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

    // free resources
    rofi_release(table);
    rofi_release(gather_buffer);
    rofi_release(index_buffer);
    rofi_release(gather_ready);
    rofi_release(index_ready);
    free(indices);
    free(gathered);
    for (int i = 0; i < np; i++) {
        free(local_index_buffer[i]);
        free(pe_gathered_idx[i]);
    }
    rofi_finit();

    return 0;
}