#ifndef ROFI_DEBUG_H
#define ROFI_DEBUG_H

#ifdef _DEBUG
#define DEBUG_MSG(fmt, ...) \
	do { fprintf(stderr, "[%d][DEBUG][%s:%d] " fmt "\n", rt_get_rank(), __FILE__, \
		     __LINE__, ##__VA_ARGS__); } while (0)
#else
#define DEBUG_MSG(fmt, args...)
#endif

#define ERR_MSG(fmt, args...) fprintf(stderr,"[PE %d][ROFI ERR][%s:%d] " fmt "\n", rt_get_rank(), __FILE__, __LINE__, ##args)

#endif
