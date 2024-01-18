#ifndef ROFI_DEBUG_H
#define ROFI_DEBUG_H

#include <sys/types.h>
#include <unistd.h>
#include <sys/syscall.h>

#ifdef _DEBUG
#define DEBUG_MSG(fmt, ...) \
	do { fprintf(stderr, "[%d][%d][DEBUG][%s][%s:%d] " fmt "\n", rt_get_rank(),syscall(__NR_gettid),__func__, __FILE__, \
		     __LINE__, ##__VA_ARGS__); } while (0)
#else
#define DEBUG_MSG(fmt, args...)
#endif

#define ERR_MSG(fmt, args...) fprintf(stderr,"[PE %d, TID: %d][ROFI ERR][%s][%s:%d] " fmt "\n", rt_get_rank(),syscall(__NR_gettid),__func__, __FILE__, __LINE__, ##args)

#endif
