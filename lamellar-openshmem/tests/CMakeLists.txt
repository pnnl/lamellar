# Tests for the Lamellar OpenSHMEM library, built against an installed copy of it (this is how users build too):
#
#   tests/stage.sh <backend>                       install <backend> into target/stage/<backend> and build these tests
#   ctest --test-dir target/tests-build/<backend> -L single-node --output-on-failure      (inside a Slurm allocation)
#
# or, by hand:   cmake -S tests -B build -DCMAKE_PREFIX_PATH=<prefix> -DSHMEM_BACKEND=<backend> && cmake --build build
#
# Labels: single-node (any backend), multinode (needs an allocation of >= 2 nodes; not for the shmem backends), aslr
# (PIE / RELRO layouts of the data segment, across 2 nodes).
cmake_minimum_required(VERSION 3.20)
project(lamellar_openshmem_tests C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS OFF)
find_package(LamellarShmem REQUIRED)
find_package(Threads REQUIRED)

set(SHMEM_BACKEND "" CACHE STRING "run-time backend the tests run on (sets LAMELLAR_BACKEND)")
option(SHMEM_TEST_STATIC "also test linking a static libshmem.a (the one in CMAKE_PREFIX_PATH, or SHMEM_STATIC_PREFIX)" ON)
set(SHMEM_STATIC_PREFIX "" CACHE PATH "prefix made by tools/build-static.sh whose self-contained libshmem.a ring_static links (needed for UCX and libfabric, whose libraries are not in an ordinary install)")
option(SHMEM_TEST_GLOBALS "also test globals/statics as remote targets (not available on the original shmem backend)" ON)
set(SHMEM_PRRTE_DIR "" CACHE PATH "directory containing prterun, put on PATH for the multi-node tests")
set(SHMEM_TEST_TIMEOUT 240 CACHE STRING "seconds a test may run")
if(SHMEM_BACKEND MATCHES "^(shmem|shmem-opt|local)$")
  set(_single_node_backend TRUE)
else()
  set(_single_node_backend FALSE)
endif()

# ---- programs ----------------------------------------------------------------------------------------------------------
foreach(t ring full bulk globals threads atomicity nbi progress)
  add_executable(${t} c/${t}.c)
  target_compile_definitions(${t} PRIVATE _POSIX_C_SOURCE=200809L)
  target_link_libraries(${t} PRIVATE Lamellar::shmem)
endforeach()
target_link_libraries(threads PRIVATE Threads::Threads)

add_executable(hello cxx/hello.cpp)
target_link_libraries(hello PRIVATE Lamellar::shmem)

foreach(t shmem_bench reduce_check)
  add_executable(${t} bench/${t}.c)
  target_compile_definitions(${t} PRIVATE _POSIX_C_SOURCE=200809L)
  target_link_libraries(${t} PRIVATE Lamellar::shmem)
endforeach()

# the same program linked statically (whole-archive) against the installed libshmem.a
if(SHMEM_TEST_STATIC)
  add_executable(ring_static c/ring.c)
  target_compile_definitions(ring_static PRIVATE _POSIX_C_SOURCE=200809L)
  if(SHMEM_STATIC_PREFIX)
    file(READ "${SHMEM_STATIC_PREFIX}/lib/shmem.libs" _static_libs)
    string(STRIP "${_static_libs}" _static_libs)
    separate_arguments(_static_libs UNIX_COMMAND "${_static_libs}")
    target_include_directories(ring_static PRIVATE "${SHMEM_STATIC_PREFIX}/include")
    # whole-archive keeps the static initializers (active-message registration, UCX transports)
    target_link_libraries(ring_static PRIVATE -Wl,--whole-archive "${SHMEM_STATIC_PREFIX}/lib/libshmem.a" -Wl,--no-whole-archive ${_static_libs})
  else()
    target_link_libraries(ring_static PRIVATE Lamellar::shmem_static)
  endif()
endif()

# data-segment layouts: the globals test built as non-PIE, PIE, PIE with full RELRO, and non-PIE without RELRO
function(shmem_globals_layout name)
  add_executable(${name} c/globals.c)
  target_compile_definitions(${name} PRIVATE _POSIX_C_SOURCE=200809L)
  target_link_libraries(${name} PRIVATE Lamellar::shmem)
  set_target_properties(${name} PROPERTIES POSITION_INDEPENDENT_CODE OFF)
  target_compile_options(${name} PRIVATE ${ARGN})
  target_link_options(${name} PRIVATE ${ARGN})
endfunction()
shmem_globals_layout(globals_nopie -no-pie)
shmem_globals_layout(globals_pie -fPIE -pie)
shmem_globals_layout(globals_pie_fullrelro -fPIE -pie -Wl,-z,relro,-z,now)
shmem_globals_layout(globals_norelro -no-pie -Wl,-z,norelro)

# ---- tests -------------------------------------------------------------------------------------------------------------
enable_testing()

set(_env "SHMEM_SYMMETRIC_SIZE=67108864" "LAMELLAR_HEAP_SIZE=1073741824" "LAMELLAR_THREADS=2")
if(SHMEM_BACKEND)
  list(APPEND _env "LAMELLAR_BACKEND=${SHMEM_BACKEND}")
endif()
if(SHMEM_TEST_GLOBALS)
  list(APPEND _env "SHMEM_WARN_DATA=1")   # report data-segment registration failures
else()
  list(APPEND _env "NBI_NO_GLOBAL=1")
endif()
if(SHMEM_PRRTE_DIR)
  list(APPEND _env "PATH=${SHMEM_PRRTE_DIR}:$ENV{PATH}")
endif()

# shmem_test(<name> NP <n> | NODES <m> PPN <p>  EXPECT <count> PATTERN <regex>  [PROG <target>] [ENV <VAR=val>...] [LABELS ...])
# Passes when the launcher exits 0, PATTERN appears EXPECT times, and no failure text appears (see cmake/expect.cmake).
function(shmem_test name)
  cmake_parse_arguments(T "" "PROG;NP;NODES;PPN;EXPECT;PATTERN;TIMEOUT" "ENV;LABELS" ${ARGN})
  if(NOT T_PROG)
    set(T_PROG ${name})
  endif()
  if(NOT T_TIMEOUT)
    set(T_TIMEOUT ${SHMEM_TEST_TIMEOUT})
  endif()
  if(T_NODES)
    set(launch "-N ${T_NODES} --pes-per-node ${T_PPN}")
  else()
    set(launch "-np ${T_NP}")
  endif()
  add_test(NAME ${name}
    COMMAND ${CMAKE_COMMAND} "-DRUN=${LamellarShmem_RUN}" "-DLAUNCH=${launch}" "-DPROG=$<TARGET_FILE:${T_PROG}>"
            "-DEXPECT=${T_EXPECT}" "-DPATTERN=${T_PATTERN}" "-DTIMEOUT=${T_TIMEOUT}"
            -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/expect.cmake)
  set_tests_properties(${name} PROPERTIES LABELS "${T_LABELS}" ENVIRONMENT "${_env};${T_ENV}"
                       TIMEOUT "${T_TIMEOUT}")
endfunction()

# single node, any backend
shmem_test(ring          NP 2 EXPECT 2 PATTERN "-> OK"             LABELS single-node)
shmem_test(full          NP 4 EXPECT 4 PATTERN "all checks passed" LABELS single-node)
if(SHMEM_TEST_GLOBALS)
  shmem_test(globals_np2 PROG globals NP 2 EXPECT 2 PATTERN "all checks passed" LABELS single-node)
  shmem_test(globals_np4 PROG globals NP 4 EXPECT 4 PATTERN "all checks passed" LABELS single-node)
endif()
shmem_test(bulk          NP 2 EXPECT 2 PATTERN "all checks passed" LABELS single-node)
shmem_test(threads       NP 2 EXPECT 2 PATTERN "threads OK"        LABELS single-node)
shmem_test(threads_16    PROG threads NP 2 EXPECT 2 PATTERN "threads OK" ENV THREADS=16 LABELS single-node)
shmem_test(nbi           NP 4 EXPECT 4 PATTERN "nbi OK"            LABELS single-node)
shmem_test(atomicity     NP 4 EXPECT 1 PATTERN "atomicity OK"      ENV N=20000 LABELS single-node)
shmem_test(progress      NP 2 EXPECT 1 PATTERN "progress:"         LABELS single-node)
shmem_test(hello_cxx     PROG hello NP 3 EXPECT 3 PATTERN "\\(C\\+\\+\\): OK" LABELS single-node)
if(SHMEM_TEST_STATIC)
  shmem_test(ring_static_np2 PROG ring_static NP 2 EXPECT 2 PATTERN "-> OK" LABELS single-node)
endif()
shmem_test(reduce_check  NP 4 EXPECT 1 PATTERN "reduce_check np=4 finished" LABELS single-node)

# several nodes: network backends only
if(NOT _single_node_backend)
  add_test(NAME rank_placement COMMAND ${CMAKE_COMMAND} "-DRUN=${LamellarShmem_RUN}" -DNODES=2 -DPPN=2
           -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/hosts.cmake)
  set_tests_properties(rank_placement PROPERTIES LABELS multinode ENVIRONMENT "${_env}" TIMEOUT 150)
  shmem_test(mn_ring     PROG ring  NODES 2 PPN 2 EXPECT 4 PATTERN "-> OK"             LABELS multinode)
  shmem_test(mn_hello    PROG hello NODES 2 PPN 2 EXPECT 4 PATTERN "\\(C\\+\\+\\): OK" LABELS multinode)
  shmem_test(mn_full     PROG full  NODES 2 PPN 2 EXPECT 4 PATTERN "all checks passed" LABELS multinode)
  shmem_test(mn_full_8pe PROG full  NODES 2 PPN 4 EXPECT 8 PATTERN "all checks passed" LABELS multinode)
  if(SHMEM_TEST_GLOBALS)
    shmem_test(mn_globals PROG globals NODES 2 PPN 2 EXPECT 4 PATTERN "all checks passed" LABELS multinode)
  endif()
  shmem_test(mn_bulk     PROG bulk  NODES 2 PPN 2 EXPECT 4 PATTERN "all checks passed" TIMEOUT 280 LABELS multinode)
  foreach(i RANGE 1 5)
    shmem_test(mn_full_repeat_${i} PROG full NODES 2 PPN 2 EXPECT 4 PATTERN "all checks passed" LABELS multinode)
  endforeach()
  if(SHMEM_TEST_GLOBALS)
    foreach(v nopie pie pie_fullrelro norelro)
      shmem_test(aslr_${v} PROG globals_${v} NODES 2 PPN 2 EXPECT 4 PATTERN "all checks passed"
                 ENV GLOBALS_PRINT_ADDR=1 LABELS "multinode;aslr")
    endforeach()
  endif()
endif()
