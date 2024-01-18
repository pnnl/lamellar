#include <stdio.h>
#include <unistd.h>

#include "utils.h"
#include <rofi.h>

#define N 1
#define SIZE 128
// #define VERBOSE

int main(void) {
    unsigned int me, np;
    unsigned int i;
    int ret = 0;
    char *addr1;

    rofi_init("verbs");

    np = rofi_get_size();
    me = rofi_get_id();

    if (!me)
        rofi_banner("Sub Alloc Test");

    printf("Process %d/%d allocating memory region of size %d\n",
           me, np, SIZE);

    uint64_t *first_half_pes = malloc(np / 2 * sizeof(uint64_t));
    for (int pe = 0; pe < np / 2; pe++) {
        first_half_pes[pe] = pe;
    }

    if (me < np / 2) {
        ret = rofi_sub_alloc(SIZE, 0x0, (void **)&addr1, first_half_pes, np / 2);
        if (ret) {
            printf("Error allocating memory region! Aborting.");
            goto out;
        }
    }
    rofi_barrier();
    printf("Added memory region at %p size %d\n", addr1, SIZE);

    if (me < np / 2) {
        printf("Address Mappig on all nodes:\n");
        for (i = 0; i < np; i++)
            printf("\t Node %u: %d\n", i, rofi_get_remote_addr((void *)addr1, i));
    }

    if (me < np / 2) {
        ret = rofi_sub_release((void *)addr1, first_half_pes, np / 2);
        if (ret) {
            printf("Error removing region %p!", addr1);
            goto out;
        }
    }
    rofi_barrier();

out:
    if (!me)
        rofi_verify(ret);
    rofi_finit();

    return 0;
}
