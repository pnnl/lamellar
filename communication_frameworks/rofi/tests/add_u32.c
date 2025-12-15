#include <unistd.h>
#include <stdio.h>

#include "utils.h"
#include <rofi.h>

int main(void) {
    rofi_init("verbs", NULL);
    uint32_t id = rofi_get_id();
    uint32_t size = rofi_get_size();
    uint32_t value = 1;

    if(id == 0) {
        rofi_banner("Atomic Add U32 Test");
    }
    rofi_barrier();

    uint32_t *ptr = NULL;
    int ret = rofi_alloc(sizeof(uint32_t), 0, (void **)&ptr);
    if (ret) {
        printf("Error allocating memory\n");
        return -1;
    }

    if (id == 0) {
        *ptr = 0;
        printf("ID: %u/%u, ptr: %p, value: %u\n", id, size, ptr, *ptr);
    }
    rofi_barrier();


    ssize_t res = rofi_atomic_add_u32(ptr, 1UL, 0);
    printf("ID: %lu/%lu added 1: %ld\n", id, size, res);
    if (res) {
        printf("Error in atomic add u32 (%ld)\n", res);
    }

    rofi_barrier();
    
    if(id == 0) {
        printf("ID: %lu/%lu Results: ptr: %p, value: %lu (assert %lu)\n", id, size, ptr, *ptr, size);
        rofi_verify(!(*ptr == (size)));
    }

    rofi_release(ptr);
    rofi_finit();

    return 0;
}