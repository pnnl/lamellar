#ifndef ROFI_H
#define ROFI_H
#include <stddef.h>

#define ROFI_ERR_ALLOC 0x01

int          rofi_init(void);
int          rofi_finit(void);
unsigned int rofi_get_size(void);
unsigned int rofi_get_id(void);
int          rofi_put(void*, const void*, size_t, unsigned int, unsigned long);
int          rofi_iput(void*, const void*, size_t, unsigned int, unsigned long);
int          rofi_get(void*, const void*, size_t, unsigned int, unsigned long);
int          rofi_iget(void*, const void*, size_t, unsigned int, unsigned long);
int          rofi_alloc(size_t, unsigned long, void**);
int          rofi_release(void);
void         rofi_barrier(void);
int          rofi_wait(void);
#endif
