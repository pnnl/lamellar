#include <stdio.h>
#include <unistd.h>

#include "utils.h"
#include <rofi.h>

#define N 1024 * 1024
// #define VERBOSE

unsigned long source[N];
unsigned long target[N];

int main(void) {
    unsigned int me, np, i;
    int ret = 0;

    rofi_banner("PUT Test");
    rofi_init();

    np = rofi_get_size();
    if (np != 2) {
        printf("Invalid number of processes (%u) (Required 2)! Aborting.\n", np);
        ret = -1;
        goto out;
    }

    me = rofi_get_id();

    for (i = 0; i < N; i++) {
        source[i] = i;
        target[i] = 0;
    }

    rofi_barrier();

    printf("[%u] Writing %d elements\n", me, N);

    // if(me)
    if (rofi_iput(target, source, sizeof(unsigned long) * N, 0, 0x0)) {
        printf("[%u] Error writing to remote node. Aborting...\n", me);
    }
    printf("done with put\n");
    rofi_wait();

    rofi_barrier();

    if (!me) {
        for (i = 0; i < N; i++) {
#ifdef VERBOSE
            printf("[%lu] %d: %lu\n", me, i, target[i]);
#endif
            if (source[i] != target[i]) {
                printf("ERROR %d: %lu != %lu\n", i, source[i], target[i]);
                ret = 1;
            }
        }
        if (ret)
            goto out;
    }

out:
    rofi_verify(ret);
    rofi_finit();

    return 0;
}
