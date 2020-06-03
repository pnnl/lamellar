AM_CFLAGS = -I$(top_srcdir)/src/include

check_PROGRAMS = rofi_init rofi_put rofi_get rofi_alloc rofi_pp rofi_put_bw
TESTS = $(check_PROGRAMS)

rofi_init_SOURCES      = init.c utils.c utils.h
rofi_init_LDFLAGS      = $(linker_flags)
rofi_init_LDADD        = ../src/librofi.la

rofi_pp_SOURCES      = pp.c utils.c utils.h
rofi_pp_LDFLAGS      = $(linker_flags)
rofi_pp_LDADD        = ../src/librofi.la

rofi_put_SOURCES      = put.c utils.c utils.h
rofi_put_LDFLAGS      = $(linker_flags)
rofi_put_LDADD        = ../src/librofi.la

rofi_get_SOURCES      = get.c utils.c utils.h
rofi_get_LDFLAGS      = $(linker_flags)
rofi_get_LDADD        = ../src/librofi.la

rofi_alloc_SOURCES      = alloc.c utils.c utils.h
rofi_alloc_LDFLAGS      = $(linker_flags)
rofi_alloc_LDADD        = ../src/librofi.la

rofi_put_bw_SOURCES      = put_bw.c utils.c utils.h
rofi_put_bw_LDFLAGS      = $(linker_flags)
rofi_put_bw_LDADD        = ../src/librofi.la
