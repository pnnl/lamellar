#ifndef ROFI_DEBUG_H
#define ROFI_DEBUG_H

#include <sys/syscall.h>
#include <sys/types.h>
#include <unistd.h>

#ifdef _DEBUG
#define DEBUG_MSG(fmt, ...)                                                                                                            \
    do {                                                                                                                               \
        fprintf(stderr, "[ROFI DEBUG][PE: %d][TID: %d][%s][%s:%d] " fmt "\n", rt_get_rank(), syscall(__NR_gettid), __func__, __FILE__, \
                __LINE__, ##__VA_ARGS__);                                                                                              \
    } while (0)
#else
#define DEBUG_MSG(fmt, args...)
#endif

#define PRINT_MSG(fmt, ...)                                                                                                      \
    do {                                                                                                                         \
        fprintf(stdout, "[ROFI][PE: %d][TID: %d][%s][%s:%d] " fmt "\n", rt_get_rank(), syscall(__NR_gettid), __func__, __FILE__, \
                __LINE__, ##__VA_ARGS__);                                                                                        \
    } while (0)

#define WARN_MSG(fmt, args...) fprintf(stderr, "[ROFI WARNING][PE: %d][TID: %d][%s][%s:%d] " fmt "\n", rt_get_rank(), syscall(__NR_gettid), __func__, __FILE__, __LINE__, ##args)

#define ERR_MSG(fmt, args...) fprintf(stderr, "[ROFI ERROR][PE: %d][TID: %d][%s][%s:%d] " fmt "\n", rt_get_rank(), syscall(__NR_gettid), __func__, __FILE__, __LINE__, ##args)

#endif
