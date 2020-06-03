SUBDIRS = pmi-simple

AM_CFLAGS=-I$(srcdir)/include -I$(srcdir)/pmi-simple

lib_LTLIBRARIES    = librofi.la 
librofi_la_SOURCES = api.c core.c rt.c
librofi_la_SOURCES += include/rofi_atomic.h include/rofi_debug.h include/rofi_internal.h
librofi_la_LIBADD = $(srcdir)/pmi-simple/libpmi_simple.la

include_HEADERS    = $(top_srcdir)/src/include/rofi.h
EXTRA_DIST = $(top_srcdir)/src/include/rofi_debug.h
