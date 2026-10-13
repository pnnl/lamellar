#include <math.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "utils.h"
#include <rofi.h>

#define N (1UL << 30)
// #define VERBOSE

typedef struct {
    unsigned long size;
    struct timespec start, end;
    double time;
    double tput;
    unsigned long errs; // Number of errors
} results_t;

static inline int verify_data(char *in, char *out, unsigned long size) {
    unsigned long i;

    for (i = 0; i < size; i++) {
#ifdef VERBOSE
        printf("%d: %c\n", i, out[i]);
#endif
        if (in[i] != out[i]) {
            printf("ERROR %lu: %c != %c\n", i, in[i], out[i]);
            return -1;
        }
    }

    return 0;
}


// puts various sizes of data from pe 0 to pe 1
int main(void) {
    unsigned int i, j;
    int ret = 0, err = 0;
    char *src;
    char *target;
    struct timespec start, end;
    char test_name[128];
    unsigned long size = 2;
    unsigned long ntests;
    results_t *data;
    unsigned int me, np;
    

#ifdef ROFI_IPUT
    strcpy(test_name, "ROFI iPut Bw Test");
#elif ROFI_PUT
    strcpy(test_name, "ROFI Put Bw Test");
#endif

    ntests = (unsigned long)log2(N);
    data = (results_t *)malloc(ntests * sizeof(results_t));
    if (!data) {
        printf("Error allocating memory to store results. Aborting.\n");
        exit(EXIT_FAILURE);
    }

    rofi_init("verbs", NULL);
    np = rofi_get_size();
    if (np != 2) {
        printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
        ret = -1;
        goto out;
    }

    me = rofi_get_id();

    if (me == 1){
        rofi_banner(test_name);
    }

    ret = rofi_alloc(2 * N, 0x0, (void **)&src);
    if (ret) {
        printf("Error allocating ROFI heap");
        goto out;
    }

    for (i = 0; i < N; i++){
        src[i] = 'a';
    }

    target = src + N;

    for (i = 0; i < 27; i++) {
        memset((void *)target, 0, N);
        int num_bytes = (int)pow(2, i);
        int exp = 20;
        if (num_bytes <= 2048) {
            exp = 18 + i;
        }
        else {
            exp = 30;
        }
        rofi_barrier();
        clock_gettime(CLOCK_MONOTONIC, &(data[i].start));
        if (me == 0) {
            for (j = 0; j < (int)pow(2, exp); j += num_bytes) {
#ifdef ROFI_IPUT
                if (rofi_iput(target + j, src, num_bytes, me+1, 0x0)) {
                    printf("[%u] Error writing to remote node. Aborting...\n", me);
                }
#elif ROFI_PUT
                if (rofi_put(target + j, src, num_bytes, me+1, 0x0)) {
                    printf("[%u] Error reading from remote node. Aborting...\n", me);
                }
#endif
            }
#ifdef ROFI_PUT 
            rofi_wait();
#endif
        }
        rofi_barrier();
        clock_gettime(CLOCK_MONOTONIC, &(data[i].end));

        if (me == 1) {
            unsigned long err_cnt = 0;
            
            int total_size = (int)pow(2, exp);
            
            // Optimized error checking using word-sized comparisons
            const uint64_t expected_pattern = 0x6161616161616161ULL; // 'aaaaaaaa'
            uint64_t *target_64 = (uint64_t*)target;
            int word_count = total_size / 8;
            int remainder = total_size % 8;
            
            // Check 8 bytes at a time
            for (j = 0; j < word_count; j++) {
                if (target_64[j] != expected_pattern) {
                    // Count individual byte errors in this word
                    char *byte_ptr = (char*)&target_64[j];
                    for (int k = 0; k < 8; k++) {
                        if (byte_ptr[k] != 'a') {
                            err_cnt++;
                        }
                    }
                }
            }
            
            // Check remaining bytes
            char *remainder_ptr = target + (word_count * 8);
            for (j = 0; j < remainder; j++) {
                if (remainder_ptr[j] != 'a') {
                    err_cnt++;
                }
            }
            
            data[i].size = total_size;
            data[i].time = ((double)tdiff(data[i].end, data[i].start)) / BILLION;
            data[i].tput = (((double)data[i].size) / MILLION) / data[i].time;
            data[i].errs = err_cnt;
        }
    }

    rofi_barrier();

    if (me == 1) {
        fprintf(stderr,"\n");
        printf("\t %-10s \t %-11s \t %-19s \t %-11s\n", "Size (MBs)", "Time (sec)", "Throughput (MB/sec)","# Errors");
    
        for (i = 0; i < ntests; i++){
                fprintf(stderr,"\t %10lu \t %06.4f \t %16.2f \t %11lu\n",
                        data[i].size, data[i].time, data[i].tput, data[i].errs);
        }
    }
    rofi_barrier();

    if (me == 1)
        rofi_verify(err);

    rofi_release(src);
out:
    rofi_finit();
    free(data);
    return 0;
}
