#include <math.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "utils.h"
#include <rofi.h>

#define MAX_SIZE 1 << 20
#define MIN_SIZE 64

struct pp_stats {
    unsigned long size;
    struct timespec start;
    struct timespec end;
    double bw;
    double time;
};

int main(void) {
    unsigned int me, np, i, j;
    int ret = 0;
    size_t size;
    char *send_buf = NULL;
    char *recv_buf = NULL;
    struct pp_stats *results;
    unsigned long reps;

    rofi_banner("Ping Pong Test");
    reps = log2l(MAX_SIZE) - log2l(MIN_SIZE) + 1;

    send_buf = (char *)malloc(sizeof(char) * MAX_SIZE);
    recv_buf = (char *)malloc(sizeof(char) * MAX_SIZE);
    results = (struct pp_stats *)malloc(sizeof(struct pp_stats) * reps);

    if (!send_buf || !recv_buf || !results)
        exit(EXIT_FAILURE);

    rofi_init("verbs", NULL);
    // rofi_init(NULL, "ib0");

    np = rofi_get_size();
    // if (np != 2) {
    //     printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
    //     ret = -1;
    //     goto out;
    // }

    me = rofi_get_id();
    if (!me)
        printf("Reps = %lu\n", reps);

    rofi_barrier();

    for (size = MIN_SIZE, i = 0; size <= MAX_SIZE; size *= 2, i++) {
        if (ret)
            goto out;

        if (!me)
            printf("Ping pong test size %lu...\n", size);
        switch (me) {
        case 0:
            memset(send_buf, '0', size);
            break;
        case 1:
            memset(recv_buf, '1', size);
            break;
        case 2:
            memset(send_buf, '2', size);
            break;
        case 3:
            memset(recv_buf, '3', size);
            break;
        default:
            memset(send_buf, 'a', size);
            break;
        }

        memset(recv_buf, 'z', size);

        rofi_barrier();

        clock_gettime(CLOCK_MONOTONIC, &(results[i].start));
        if (me) {
            ret = rofi_send(0, (void *)send_buf, size, 0x0);
            clock_gettime(CLOCK_MONOTONIC, &(results[i].end));
        }
        else {
            for (int p = 1; p < np; p++) {
                ret = rofi_recv((void *)recv_buf, size, 0x0);
                printf("pe: %d ", recv_buf[0]);
            }
            clock_gettime(CLOCK_MONOTONIC, &(results[i].end));
            printf("\n");
            // for (j = 0; j < size; j++) {
            //     if (recv_buf[j] != send_buf[j]) {
            //         printf("Error transferring element %d: sent %c != rec %c\n", i,
            //                send_buf[j], recv_buf[j]);
            //         ret = -1;
            //     }
            // }
        }

        results[i].size = size;
        results[i].time = ((double)tdiff(results[i].end, results[i].start));
        results[i].bw = 2.0 * results[i].size / results[i].time;
    }

    rofi_barrier();
    // if (!me) {
    printf("%10s %10s %10s\n", "size", "time (msec)", "BW (GB/s)");
    for (i = 0; i < reps; i++)
        printf(" %9lu %6.4f %7.2f\n", results[i].size, results[i].time / 1000000, results[i].bw);
    // }

out:
    if (!me)
        rofi_verify(ret);
    rofi_finit();

    free(results);
    free(send_buf);
    free(recv_buf);

    return 0;
}
