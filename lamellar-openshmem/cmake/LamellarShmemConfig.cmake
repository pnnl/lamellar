# find_package(LamellarShmem) -> Lamellar::shmem (shared) and Lamellar::shmem_static
# Installed by `cargo cinstall` to <prefix>/share/cmake/LamellarShmem/ (three levels below the prefix).
get_filename_component(_shmem_prefix "${CMAKE_CURRENT_LIST_DIR}/../../.." ABSOLUTE)
if(EXISTS "${_shmem_prefix}/lib64/libshmem.so" AND NOT EXISTS "${_shmem_prefix}/lib/libshmem.so")
  set(_shmem_libdir "${_shmem_prefix}/lib64")
else()
  set(_shmem_libdir "${_shmem_prefix}/lib")
endif()
# Libraries a static link of libshmem.a needs (see Libs.private in shmem.pc); override before find_package if a backend
# needs more (for example fabric;rdmacm;ibverbs for a libfabric build).
if(NOT DEFINED LamellarShmem_STATIC_LIBS)
  set(LamellarShmem_STATIC_LIBS "util;rt;pthread;m;dl")
endif()

if(NOT TARGET Lamellar::shmem)
  add_library(Lamellar::shmem SHARED IMPORTED)
  set_target_properties(Lamellar::shmem PROPERTIES
    IMPORTED_LOCATION "${_shmem_libdir}/libshmem.so"
    INTERFACE_INCLUDE_DIRECTORIES "${_shmem_prefix}/include")
endif()

if(NOT TARGET Lamellar::shmem_static AND EXISTS "${_shmem_libdir}/libshmem.a")
  add_library(Lamellar::shmem_static INTERFACE IMPORTED)
  # whole-archive keeps the runtime's static initializers (active-message registration)
  set_target_properties(Lamellar::shmem_static PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${_shmem_prefix}/include"
    INTERFACE_LINK_LIBRARIES
      "-Wl,--whole-archive;${_shmem_libdir}/libshmem.a;-Wl,--no-whole-archive;-L${_shmem_libdir};-Wl,-rpath,${_shmem_libdir};${LamellarShmem_STATIC_LIBS}")
endif()

set(LamellarShmem_FOUND TRUE)
set(LamellarShmem_PREFIX "${_shmem_prefix}")
# Launcher for tests: add_test(NAME x COMMAND ${LamellarShmem_RUN} -np 4 $<TARGET_FILE:x>)
set(LamellarShmem_RUN "${_shmem_prefix}/bin/oshrun")
