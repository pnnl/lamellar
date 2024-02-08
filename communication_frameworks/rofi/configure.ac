AC_INIT([Rust OFI Library], [0.1], [roberto.gioiosa@pnnl.gov] , [rofi])
AM_INIT_AUTOMAKE([-Wall -Werror foreign subdir-objects tar-pax])

AC_CONFIG_MACRO_DIR([m4])
AM_PROG_AR

AC_CANONICAL_HOST

AC_PROG_CC
AM_PROG_CC_C_O
AC_PROG_LIBTOOL

AC_ARG_ENABLE([debug],
    [AC_HELP_STRING([--enable-debug],
		    [Enable debugging mode (default: disabled)])])

AC_ARG_ENABLE([stats],
    [AC_HELP_STRING([--enable-stats],
		    [Enable statistics (default: disabled)])])

AC_ARG_WITH([ofi-provider],
    [AC_HELP_STRING([--with-ofi-provider],
                    [Manually select the underlying OFI provider (e.g., verbs, default: none)])],
		    [AM_CONDITIONAL(WITH_OFI_PROVIDER, true)
		     AC_DEFINE(ROFI_OFI_PROVIDER,[$with_ofi_provider],[OFI Provider])],
		    [AM_CONDITIONAL(WITH_OFI_PROVIDER, false)])


AC_CHECK_HEADERS([stdio.h],       [],[AC_MSG_ERROR[stdio.h  not found!]])
AC_CHECK_HEADERS([sys/socket.h], [],[AC_MSG_ERROR[sys/socket.h  not found!]])
AC_CHECK_HEADERS([rdma/fabric.h], [],[AC_MSG_ERROR[fabric.h not found!]])
AC_CHECK_LIB([fabric],fi_getinfo, [],[AC_MSG_ERROR[libfabric not found!]],[-libverbs -pthread -ldl -lrdmacm -lrt])

if test "$enable_debug" = "yes" ; then
  AS_VAR_APPEND(CFLAGS, " -O0 -D_DEBUG")
fi

if test "$enable_stats" = "yes" ; then
  AS_VAR_APPEND(CFLAGS, " -D_STATS=1")
fi

AC_MSG_CHECKING([for data segment pointer])
AC_TRY_RUN([
#ifdef __APPLE__
#include <mach-o/getsect.h>
#else
#pragma weak __data_start
#pragma weak _end
extern int __data_start;
extern int _end;
#endif

/* Ensure data segment is not empty.  Anything simpler gets optimized
 * away by the compiler. */
int ensure_nonempty_data(void) {
    static int ncall = 0;
    return ncall++;
}

int main(void) {
    void *base;
    long length;
#ifdef __APPLE__
    base = (void*) get_etext();
    length = get_end() - get_etext();
#else
    if (&__data_start == (int*) 0 || &_end == (int*) 0) return 1;
    base = (void*) &__data_start;
    length = (long) ((char*) &_end - (char*) &__data_start);
#endif
    if (!(base != (void*)0 && length > 0)) return 2;
    return 0;
}],
    AC_MSG_RESULT([found]),
    [ AC_MSG_RESULT([not found])
      AC_MSG_ERROR([Could not locate data segment])
    ])

dnl final output

AC_CHECK_PROGS([DOXYGEN], [doxygen])
if test -z "$DOXYGEN";
   then AC_MSG_WARN([Doxygen not found - continuing without Doxygen support])
fi

AM_CONDITIONAL([HAVE_DOXYGEN], 
[test -n "$DOXYGEN"]) AM_COND_IF([HAVE_DOXYGEN], [AC_CONFIG_FILES([docs/Doxyfile])])

LT_INIT

AC_CONFIG_HEADERS([config.h])
AC_CONFIG_FILES([Makefile src/Makefile src/pmi-simple/Makefile docs/Makefile tests/Makefile])

AC_OUTPUT

echo "====================================================="
echo "|                                                   |"
echo "|     Rust OFI Library                              |"
echo "|                                                   |"
echo "| Pacific Northwest  National Laboratory            |"
echo "| Contact: R. Gioiosa - roberto.gioiosa@pnnl.gov    |"
echo "|                                                   |"
echo "====================================================="
echo ""
echo ""
echo "Platorm:			$platform"
echo "Compiler vendor:		$xcompiler"
echo ""
echo "Options:   "
echo "		 Debug:		$enable_debug"
echo "		 Stats:		$enable_stats"
echo "		 OFI Provider:  $ROFI_OFI_PROVIDER"
echo ""
echo "Compilers: "
echo "		 C:		$CC"
echo "		 CFLAGS:	$CFLAGS"
echo "		 CPPFLAGS:	$CPPFLAGS"
echo "		 LDFLAGS:	$LDFLAGS"
echo ""
echo "Other: "
echo "		 Documentation: $DOXYGEN"
echo "====================================================="
