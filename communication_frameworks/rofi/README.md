Rust OpenFabrics Interface Transport Layer (ROFI)
=================================================

The Rust OpenFabrics library is a transport layer for the Rust runtime based on the [OpenFabrics Interface (OFI)](https://www.openfabrics.org) standard.

SUMMARY
-------

ROFI is a transport layer based on libfabtrics that is meant to facilitate data transfers in distributed environmnets. ROFI's goal is to provide a high-level abstraction compared to OFI but not to implement sophisticated communication protocols, which are supposed to be implemented at higher levels of the software stack.

In particular, ROFI has been desiged to integrate well with [Rust](https://www.rust-lang.org) code, minimizing the reasons that may complicate Rust-C integration (see Rust [FFI](https://doc.rust-lang.org/nomicon/ffi.html) and [bindgen](https://github.com/rust-lang/rust-bindgen)).

ROFI provide support for synchronous and asynchronous RDMA-based APIs (i.e., PUT and GET) and treats messages as a sequence of bytes, without interpreting the actual contenct or semantics.

NEWS
----
Feb 2020: First alpha release

BUILD REQUIREMENTS
------------------
ROFI requires the following dependencies:

* [GNU Autotools](https://www.gnu.org/software/automake/manual/html_node/Autotools-Introduction.html): To build and install ROFI package
* [libfabric](https://github.com/ofiwg/libfabric): OFI library
* [libibverbs](https://github.com/linux-rdma/rdma-core/tree/master/libibverbs): Infiniband VERBS support
* [UTHash](https://github.com/troydhanson/uthash): Runtime hashtable
* [libatomic](https://github.com/gcc-mirror/gcc/tree/master/libatomic): Atomic operations, generally installed on Linux development clusters
* librt: Extended realtime library, generally installed on Linux development clusters
* libpthread: POSIX thread library, generally installed on Linux development clusters

At the time of release, ROFI has been tested with the following external packages:

| **GCC** | **OFI**   | **IB VERBS**  | **MPI**       | **SLURM** |
|--------:|----------:|--------------:|--------------:|----------:|
| 4.8.5   | 1.7.1     | 1.13          | mvapich2/2.3a | 17.02.7   |
|         | 1.8.0     |               |               |           |
|         | 1.9.0     |               |               |           |
| 7.1.0.  | 1.10.0    |               |               |           |
|         | 1.11.0    |               |               |           |
|         | 1.12.0    |               |               |           |

BUILDING PACKAGE
----------------
In the following, assume that ROFI source code has been downloaded in $(SRCDIR)

0. If ROFI has been cloned from the GIT repository, generate the configure script and populate uthash submodule:

`autoreconf --install`
`git submodule update --init`


1. Configure ROFI for your system:  (can be buildt in source or out of source)

`./configure --prefix=/${HOME}/workspace CPPFLAGS=-I$USER_INC CFLAGS=-O3 LDFLAGS=-L$USER_LIBS`

where `$USER_INC` and `$USER_LIBS` point to the installation directories for header files (e.g., uthash.h) and libraries (e.g., libfabrics.so) of dependencies. Passing `--enable-debug` during configuration will enable ROFI to emit debugging information during execution (NOTE: debug logs, if enabled, will be verbose). Other useful options include:

* `--prefix`: installation path
* `--enable-debug`: emit debugging information at runtime, enabling this option add `-O0` to `CFLAGS`
* `--enable-stats`: emit statistics during execution

Compiler and linking options can be expressed using the following enviromental variables:

* `CC`:       C compiler command
* `CFLAGS`:   C compiler flags
* `LDFLAGS`:  linker flags
* `LIBS`:     libraries to pass to the linker
* `CPPFLAGS`: (Objective) C/C++ preprocessor flags

Running

`./configure --help`

shows the list of available options and flags:

```
`configure' configures Rust OFI Library 0.1 to adapt to many kinds of systems.

Usage: ./configure [OPTION]... [VAR=VALUE]...

To assign environment variables (e.g., CC, CFLAGS...), specify them as
VAR=VALUE.  See below for descriptions of some of the useful variables.

Defaults for the options are specified in brackets.

Configuration:
  -h, --help              display this help and exit
      --help=short        display options specific to this package
      --help=recursive    display the short help of all the included packages
  -V, --version           display version information and exit
  -q, --quiet, --silent   do not print `checking ...' messages
      --cache-file=FILE   cache test results in FILE [disabled]
  -C, --config-cache      alias for `--cache-file=config.cache'
  -n, --no-create         do not create output files
      --srcdir=DIR        find the sources in DIR [configure dir or `..']

Installation directories:
  --prefix=PREFIX         install architecture-independent files in PREFIX
                          [/usr/local]
  --exec-prefix=EPREFIX   install architecture-dependent files in EPREFIX
                          [PREFIX]

By default, `make install' will install all the files in
`/usr/local/bin', `/usr/local/lib' etc.  You can specify
an installation prefix other than `/usr/local' using `--prefix',
for instance `--prefix=$HOME'.

For better control, use the options below.

Fine tuning of the installation directories:
  --bindir=DIR            user executables [EPREFIX/bin]
  --sbindir=DIR           system admin executables [EPREFIX/sbin]
  --libexecdir=DIR        program executables [EPREFIX/libexec]
  --sysconfdir=DIR        read-only single-machine data [PREFIX/etc]
  --sharedstatedir=DIR    modifiable architecture-independent data [PREFIX/com]
  --localstatedir=DIR     modifiable single-machine data [PREFIX/var]
  --libdir=DIR            object code libraries [EPREFIX/lib]
  --includedir=DIR        C header files [PREFIX/include]
  --oldincludedir=DIR     C header files for non-gcc [/usr/include]
  --datarootdir=DIR       read-only arch.-independent data root [PREFIX/share]
  --datadir=DIR           read-only architecture-independent data [DATAROOTDIR]
  --infodir=DIR           info documentation [DATAROOTDIR/info]
  --localedir=DIR         locale-dependent data [DATAROOTDIR/locale]
  --mandir=DIR            man documentation [DATAROOTDIR/man]
  --docdir=DIR            documentation root [DATAROOTDIR/doc/rofi]
  --htmldir=DIR           html documentation [DOCDIR]
  --dvidir=DIR            dvi documentation [DOCDIR]
  --pdfdir=DIR            pdf documentation [DOCDIR]
  --psdir=DIR             ps documentation [DOCDIR]

Program names:
  --program-prefix=PREFIX            prepend PREFIX to installed program names
  --program-suffix=SUFFIX            append SUFFIX to installed program names
  --program-transform-name=PROGRAM   run sed PROGRAM on installed program names

System types:
  --build=BUILD     configure for building on BUILD [guessed]
  --host=HOST       cross-compile to build programs to run on HOST [BUILD]

Optional Features:
  --disable-option-checking  ignore unrecognized --enable/--with options
  --disable-FEATURE       do not include FEATURE (same as --enable-FEATURE=no)
  --enable-FEATURE[=ARG]  include FEATURE [ARG=yes]
  --enable-silent-rules   less verbose build output (undo: "make V=1")
  --disable-silent-rules  verbose build output (undo: "make V=0")
  --enable-dependency-tracking
                          do not reject slow dependency extractors
  --disable-dependency-tracking
                          speeds up one-time build
  --enable-shared[=PKGS]  build shared libraries [default=yes]
  --enable-static[=PKGS]  build static libraries [default=yes]
  --enable-fast-install[=PKGS]
                          optimize for fast installation [default=yes]
  --disable-libtool-lock  avoid locking (might break parallel builds)
  --enable-debug          Enable debugging mode (default: disabled)
  --enable-stats          Enable statistics (default: disabled)

Optional Packages:
  --with-PACKAGE[=ARG]    use PACKAGE [ARG=yes]
  --without-PACKAGE       do not use PACKAGE (same as --with-PACKAGE=no)
  --with-pic[=PKGS]       try to use only PIC/non-PIC objects [default=use
                          both]
  --with-aix-soname=aix|svr4|both
                          shared library versioning (aka "SONAME") variant to
                          provide on AIX, [default=aix].
  --with-gnu-ld           assume the C compiler uses GNU ld [default=no]
  --with-sysroot[=DIR]    Search for dependent libraries within DIR (or the
                          compiler's sysroot if not specified).

Some influential environment variables:
  CC          C compiler command
  CFLAGS      C compiler flags
  LDFLAGS     linker flags, e.g. -L<lib dir> if you have libraries in a
              nonstandard directory <lib dir>
  LIBS        libraries to pass to the linker, e.g. -l<library>
  CPPFLAGS    (Objective) C/C++ preprocessor flags, e.g. -I<include dir> if
              you have headers in a nonstandard directory <include dir>
  LT_SYS_LIBRARY_PATH
              User-defined run-time library search path.
  CPP         C preprocessor

Use these variables to override the choices made by `configure' or to help
it to find libraries and programs with nonstandard names/locations.

Report bugs to <roberto.gioiosa@pnnl.gov>.
```

2. Compile ROFI

`make`

3. Compile and run unit tests (optional but recommended). the ROFI package comes with a set of unit tests that can be used to assess whether the compilation and linking process have been successful and that all external dependencies have been met.

`make check`

will produce a report for each test:

```
Applications/Xcode.app/Contents/Developer/usr/bin/make  check-TESTS
PASS: rofi_init
PASS: rofi_put
PASS: rofi_get
PASS: rofi_alloc
PASS: rofi_pp
PASS: rofi_put_bw
============================================================================
Testsuite summary for Rust OFI Library 0.1
============================================================================
# TOTAL: 6
# PASS:  6
# SKIP:  0
# XFAIL: 0
# FAIL:  0
# XPASS: 0
# ERROR: 0
============================================================================
```
These tests are only meant to check that the ROFI library has been successfull built and it is operational.  We assume users will want also to verify that ROFI works in a particular distributed environment.  Please see the TESTING section below.

4. Install ROFI

`make install`

This command will install ROFI in standard location or in the location specified with `--prefix`.

DOCUMENTATION
-------------
To build the documentation, go to `$(SRCDIR)/docs` and type:

`make`

This will produce the support files to build HTML and PDF documentation. To actually build the reference manual, move to the appropriated directory (`html` or `latex`) and type:

`make`

For example, issuing the command in the `latex` directory will generate documentation in PDF format (`refman.pdf`).

TESTING
-------
Most of the tests below require two compute nodes to be executed. Here is a simple proceedure to run the tests on a compute cluster with the [SLURM](https://slurm.schedmd.com) job manager. Please, refer to the job manager documentaiton for details on how to run command on different clusters. ROFI obtains job information (size, distribution, etc.) from the job manager and runtime launcher (e.g., MPI, please refer to the BUILING REQUIREMENTS section for a list of tested software library versions known to work with ROFI).

1. Allocates two compute nodes on the cluster:

`salloc -N 2 -p compute`

2. Run ROFI test using `mpiexec` launcher:

`mpiexec -n 2 ./tests/rofi_<test>`

where `<test>` in {`alloc, get, init, pp, put, put_bw`}. For example, to run a simple init test (check libraries, providers, etc.), run:

`mpiexec -n 2 ./tests/rofi_init`

SUPPORTED PROVIDERS
-------------------
ROFI is currently actively tested with libfabric `verbs` provider. Other providers, including `sockets` and `shm` are currently under development and not guaranteed to work.

To select a provider, add the provider name to the `rofi_init()` function. For example, to select the `shm` provider, call:

`rofi_init("shm")`

then a test on a single node can be started in the same way as for distributed systems. For example,

`mpiexec -n 2 -ppn 2 ./tests/rofi_put`

launches two ROFI processes on the same compute node, while the command:

`mpiexec -n 2 -ppn 1 ./tests/rofi_put`

launches two ROFI processes on different nodes (note that `shm` does not work on multiple compute nodes).

Note that libfabric `shm` uses Linux Cross Memory Attach (CMA) for shared memory communication. Any recent Linux kernel should support CMA, but for example OSX/Darwin does not. On old Linux kernels that do not support CMA, libfabrics will leverage the traditional POSIX shared memory Inter-Process Communication primitives (IPC). One could also use `sockets` (under development) or `verbs` to emulate `shm`.  On single-node OSX, `sockets` would be the best experimental provider option. From perfromance prospective, `shm` should perform better than `sockets`, thus we recommend Linux environments for single-node, production development.

In the case no option is specified with `rofi_init()`, ROFI selects the first provider that satisfies its requirements. This is usually the best one, though libfabric does not _guarantee_ that. If a **specific** provider is requested that is not available in the system, ROFI initialization fails, to provide expected performance.

HISTORY
-------
- version 0.1:
  - Basic init/finit functionalities
  - Basic memory management (heap and data section)
  - Initial communication primitives (sync/async PUT/GET, wait)
  - Unit tests
  
  
NOTES
-----

STATUS
------
ROFI is still under development, thus not all intended features are yet
implemented.

CONTACTS
--------
Roberto Gioiosa - roberto.gioiosa@pnnl.gov  
Ryan Friese     - ryan.friese@pnnl.gov  
Mark Raugas     - mark.raugas@pnnl.gov  

## License

This project is licensed under the BSD License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
