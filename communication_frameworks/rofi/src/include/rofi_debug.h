#ifndef ROFI_DEBUG_H
#define ROFI_DEBUG_H

#ifdef _DEBUG
#define DEBUG_MSG(fmt, args...) fprintf(stderr,"[ROFI DEBUG][%s %d] " fmt "\n", __FILE__, __LINE__, ##args)
#else
#define DEBUG_MSG(fmt, args...)
#endif

#define ERR_MSG(fmt, args...) fprintf(stderr,"[ROFI ERR][%s %d] " fmt "\n", __FILE__, __LINE__, ##args)

#endif
