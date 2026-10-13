#ifndef ROFI_ATOMICS_H
#define ROFI_ATOMICS_H

/*
 * These operations return the values that was previously held in memory
 */
#define cas(addr, oldval, newval)  __sync_val_compare_and_swap(addr, oldval, newval)
#define bcas(addr, oldval, newval) __sync_bool_compare_and_swap(addr, oldval, newval)
#define faa(addr, val)             __sync_fetch_and_add(addr, val)
#define fas(addr, val)             __sync_fetch_and_sub(addr, val)
#define ainc(addr)                 faa(addr, 1)
#define adec(addr)                 fas(addr, 1)
#endif
