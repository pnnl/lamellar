AM_CFLAGS = -I$(top_srcdir)/src/include

check_PROGRAMS = rofi_init rofi_iput rofi_put rofi_iget rofi_get rofi_pp rofi_alloc rofi_sub_alloc rofi_iput_bw rofi_put_bw rofi_iget_bw rofi_get_bw rofi_multimr rofi_sharray rofi_barrier 
TESTS = $(check_PROGRAMS)

rofi_init_SOURCES      = init.c utils.c utils.h
rofi_init_LDFLAGS      = $(linker_flags)
rofi_init_LDADD        = ../src/librofi.la

rofi_pp_SOURCES      = pp.c utils.c utils.h
rofi_pp_LDFLAGS      = $(linker_flags)
rofi_pp_LDADD        = ../src/librofi.la

rofi_alloc_SOURCES      = alloc.c utils.c utils.h
rofi_alloc_LDFLAGS      = $(linker_flags)
rofi_alloc_LDADD        = ../src/librofi.la

rofi_sub_alloc_SOURCES      = sub_alloc.c utils.c utils.h
rofi_sub_alloc_LDFLAGS      = $(linker_flags)
rofi_sub_alloc_LDADD        = ../src/librofi.la

rofi_iput_SOURCES      = put.c utils.c utils.h
rofi_iput_CFLAGS       = $(AM_CFLAGS) -DROFI_IPUT
rofi_iput_LDFLAGS      = $(linker_flags) -lm
rofi_iput_LDADD        = ../src/librofi.la

rofi_put_SOURCES      = put.c utils.c utils.h
rofi_put_CFLAGS       = $(AM_CFLAGS) -DROFI_PUT
rofi_put_LDFLAGS      = $(linker_flags) -lm
rofi_put_LDADD        = ../src/librofi.la

rofi_iget_SOURCES      = get.c utils.c utils.h
rofi_iget_CFLAGS       = $(AM_CFLAGS) -DROFI_IGET
rofi_iget_LDFLAGS      = $(linker_flags) -lm
rofi_iget_LDADD        = ../src/librofi.la

rofi_get_SOURCES      = get.c utils.c utils.h
rofi_get_CFLAGS       = $(AM_CFLAGS) -DROFI_GET
rofi_get_LDFLAGS      = $(linker_flags) -lm
rofi_get_LDADD        = ../src/librofi.la

rofi_iput_bw_SOURCES      = put_bw.c utils.c utils.h
rofi_iput_bw_CFLAGS       = $(AM_CFLAGS) -DROFI_IPUT
rofi_iput_bw_LDFLAGS      = $(linker_flags) -lm
rofi_iput_bw_LDADD        = ../src/librofi.la

rofi_put_bw_SOURCES      = put_bw.c utils.c utils.h
rofi_put_bw_CFLAGS       = $(AM_CFLAGS) -DROFI_PUT
rofi_put_bw_LDFLAGS      = $(linker_flags) -lm
rofi_put_bw_LDADD        = ../src/librofi.la

rofi_iget_bw_SOURCES      = get_bw.c utils.c utils.h
rofi_iget_bw_CFLAGS       = $(AM_CFLAGS) -DROFI_IGET
rofi_iget_bw_LDFLAGS      = $(linker_flags) -lm
rofi_iget_bw_LDADD        = ../src/librofi.la

rofi_get_bw_SOURCES      = get_bw.c utils.c utils.h
rofi_get_bw_CFLAGS       = $(AM_CFLAGS) -DROFI_GET
rofi_get_bw_LDFLAGS      = $(linker_flags) -lm
rofi_get_bw_LDADD        = ../src/librofi.la

rofi_multimr_SOURCES      = multi_mr.c utils.c utils.h
rofi_multimr_LDFLAGS      = $(linker_flags)
rofi_multimr_LDADD        = ../src/librofi.la

rofi_sharray_SOURCES      = sharray.c utils.c utils.h
rofi_sharray_LDFLAGS      = $(linker_flags)
rofi_sharray_LDADD        = ../src/librofi.la

rofi_barrier_SOURCES      = barrier.c utils.c utils.h
rofi_barrier_LDFLAGS      = $(linker_flags)
rofi_barrier_LDADD        = ../src/librofi.la -lm

AM_LDFLAGS = $(LDFLAGS) 