SUBDIRS = pmi-simple

AM_CFLAGS=-I$(abs_srcdir)/include -I$(abs_srcdir)/pmi-simple -I$(abs_srcdir)/../uthash/include

lib_LTLIBRARIES    = librofi.la 
librofi_la_SOURCES = api.c core.c rt.c transport.c mr.c context.c
librofi_la_SOURCES += include/rofi_atomic.h include/rofi_debug.h include/rofi_internal.h
librofi_la_LIBADD = $(top_builddir)/src/pmi-simple/libpmi_simple.la

include_HEADERS    = $(abs_top_srcdir)/src/include/rofi.h
EXTRA_DIST = $(abs_top_srcdir)/src/include/rofi_debug.h
