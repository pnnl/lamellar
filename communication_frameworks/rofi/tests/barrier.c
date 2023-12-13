#include <math.h>
#include <sched.h>
#include <stdio.h>
#include <unistd.h>

#include "utils.h"
#include <rofi.h>

#define N 1024 * 1024
// #define VERBOSE

// unsigned long source[N];
// unsigned long target[N];

int main(void) {
    unsigned int me, np, i;
    int ret = 0;

    rofi_banner("PUT Test");
    rofi_init("verbs");

    np = rofi_get_size();
    // if(np != 2){
    // 	printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
    // 	ret = -1;
    // 	goto out;
    // }

    me = rofi_get_id();

    int n = 2;
    int num_rounds = ceil(log2((double)np) / log2((double)n));

    uint64_t *src;
    uint64_t *target[n];
    ret = rofi_alloc((n + 1) * num_rounds * sizeof(uint64_t), 0x0, (void **)&src);
    if (ret) {
        printf("Error allocating ROFI heap");
        goto out;
    }

    printf("Allocated %p\n", src);

    for (i = 0; i < n; i++) {
        printf("%d: %p\n", i, src + i * num_rounds);
        target[i] = (uint64_t *)src + i * num_rounds;
        for (int j = 0; j < num_rounds; j++) {
            target[i][j] = 0;
        }
    }
    src = src + n * num_rounds;
    for (i = 0; i < num_rounds; i++) {
        src[i] = 0;
    }

    rofi_barrier();

    for (int bc = 1; bc <= 5; bc++) {
        printf("BC: %d\n", bc);
        for (int round = 0; round < num_rounds; round++) {
            printf("Round: %d\n", round);
            src[round] = bc;
            for (int i = 1; i <= n; i++) {

                int send_pe = (int)(me + i * pow(n + 1, round)) % np;
                printf("%d Sending %d to %d %p\n", me, bc, send_pe, &target[i - 1][round]);
                rofi_iput(&target[i - 1][round], src + round, sizeof(uint64_t), send_pe, 0);
            }
            for (int i = 1; i <= n; i++) {
                int recv_pe = (int)(me - i * pow(n + 1, round)) % np;
                printf("%d Receiving %d from %d\n", me, bc, recv_pe);
                while (target[i - 1][round] < bc) {
                    rofi_flush();
                    sched_yield();
                }
            }
            printf("\n");
        }
    }
    rofi_barrier();

out:
    rofi_verify(ret);
    rofi_finit();

    return 0;
}
