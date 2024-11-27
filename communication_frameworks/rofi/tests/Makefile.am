AM_CFLAGS = -I$(top_srcdir)/src/include

#check_PROGRAMS = rofi_init rofi_put rofi_get rofi_alloc rofi_pp rofi_put_bw
check_PROGRAMS = rofi_init rofi_pp rofi_alloc rofi_sub_alloc rofi_put rofi_get rofi_multimr rofi_sharray rofi_barrier 
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


rofi_put_SOURCES      = putget.c utils.c utils.h
rofi_put_CFLAGS       = $(AM_CFLAGS) -DROFI_IPUT
rofi_put_LDFLAGS      = $(linker_flags) -lm
rofi_put_LDADD        = ../src/librofi.la

rofi_get_SOURCES      = putget.c utils.c utils.h
rofi_get_CFLAGS       = $(AM_CFLAGS) -DROFI_IGET
rofi_get_LDFLAGS      = $(linker_flags) -lm
rofi_get_LDADD        = ../src/librofi.la

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