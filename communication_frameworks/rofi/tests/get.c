#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <rofi.h>
#include "utils.h"

#define N 1024ULL*1024ULL

int main(void) {
    unsigned int me, np, i;
    int ret = 0;
    unsigned long *source, *target;
    char test_name[128];

    #ifdef ROFI_IGET
        strcpy(test_name, "ROFI iGet Test");
    #elif ROFI_GET
        strcpy(test_name, "ROFI Get Test");
    #endif
    rofi_init("verbs", "mlx5_0");
    np = rofi_get_size();
    me = rofi_get_id();

    rofi_banner(test_name);

    ret = rofi_alloc(2 * N * sizeof(unsigned long), 0x0, ((void **)&source));
    
    target = source + N;
    for (i = 0; i < N; i++) {
        source[i] = i;
        target[i] = 0;
    }

    rofi_barrier();

    printf("[%u] Reading %d elements\n", me, N);


    #ifdef ROFI_IGET
        if (rofi_iget(target, source, sizeof(unsigned long) * N, (me + 1)%np, 0x0)) { //get from neighbor
            printf("[%u] Error writing to remote node. Aborting...\n", me);
        }
    #elif ROFI_GET
        if (rofi_get(target, source, sizeof(unsigned long) * N, (me + 1)%np, 0x0)) { //get from neighbor
            printf("[%u] Error writing to remote node. Aborting...\n", me);
        }
        rofi_wait();
    #endif

    printf("done with get\n");
    rofi_barrier();

	for (i = 0; i < N; i++) {
		if (source[i] != target[i]) {
			printf("ERROR %d: %lu != %lu\n", i, source[i], target[i]);
			ret = 1;
		}
	}
	if (ret)
		goto out;

out:
    rofi_verify(ret);
    rofi_finit();

    return 0;
}
