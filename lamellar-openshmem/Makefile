# Build and stage the Lamellar OpenSHMEM library for development and testing.
#
#   make                   build libshmem (default features: shmem backend) and include/shmem.h
#   make header            regenerate include/shmem.h (python3)
#   make wrappers          build the compiler wrappers and launcher into wrappers/target/release
#   make stage-<backend>   build one backend into its own target directory and stage its test programs under
#                          target/tests (stage-fab, stage-fab-opt, stage-ucx, stage-ucx-opt, stage-shmem, stage-shmem-opt, ...)
#   make stage             a libfabric-shared tree in $(STAGE) (libs + test programs + prterun) for tests/multinode.sh
#                          and tests/aslr.sh
#   make check             header check + toolchain test (installs into a temporary prefix; needs cargo-c)
#
# Installing is done with cargo, not make:
#   cargo cinstall --release --prefix <prefix> [--features <backend>]      libshmem.{so,a}, shmem.h, shmem.pc, CMake package
#   cargo install --path wrappers --root <prefix>                          shmemcc, oshcc, shmemcxx, oshcxx, oshrun, shmemrun
# (cargo-c: `cargo install cargo-c`). The backends that need libfabric.so next to libshmem.so also need
# tools/copy-libfabric.sh <target-dir> <prefix>/lib afterwards.
CARGO    ?= cargo
FEATURES ?=
CARGOFLAGS ?=
TARGET   ?= target
STAGE    ?= target/fab
TESTBIN  ?= target/tests
FEAT     := $(if $(FEATURES),--features $(FEATURES),)

# Optional local targets (site-specific helpers); not part of the repository.
-include local.mk

.PHONY: all build header wrappers stage stage-fab stage-fab-opt stage-tests stage-ucx stage-ucx-opt stage-ucx-shared stage-shmem stage-shmem-opt stage-probe stage-prrte check clean
all: build header

build:
	$(CARGO) build --release --target-dir $(TARGET) $(FEAT) $(CARGOFLAGS)

header:
	python3 include/gen_shmem_h.py include/shmem.h

wrappers:
	$(CARGO) build --release --manifest-path wrappers/Cargo.toml

# ---- staging for the multi-node tests (libfabric, shared) --------------------------------------
stage: stage-fab stage-tests stage-prrte

stage-fab:
	$(MAKE) build header FEATURES=libfabric-shared
	install -d $(STAGE)/lib $(STAGE)/include
	install -m755 $(TARGET)/release/libshmem.so $(STAGE)/lib/
	install -m644 include/shmem.h $(STAGE)/include/
	tools/copy-libfabric.sh $(TARGET) $(STAGE)/lib

stage-tests:
	tests/build_tests.sh $(STAGE) $(TESTBIN) _fab

# UCX backend: a self-contained libshmem.so (UCX is linked in statically; only system IB libraries are
# needed at run time), built into its own target dir so it never clobbers the libfabric build.
STAGE_UCX ?= target/ucx-stage
stage-ucx:
	$(MAKE) build header FEATURES=ucx TARGET=target/ucx
	install -d $(STAGE_UCX)/lib $(STAGE_UCX)/include
	install -m755 target/ucx/release/libshmem.so $(STAGE_UCX)/lib/
	install -m644 include/shmem.h $(STAGE_UCX)/include/
	tests/build_tests.sh $(STAGE_UCX) $(TESTBIN) _ucx

# The runtime's optimized UCX lamellae (feature ucx-opt; run time LAMELLAR_BACKEND=ucx-opt), statically linked like stage-ucx.
STAGE_UCX_OPT ?= target/ucx-opt-stage
stage-ucx-opt:
	$(MAKE) build header FEATURES=ucx-opt TARGET=target/ucx-opt
	install -d $(STAGE_UCX_OPT)/lib $(STAGE_UCX_OPT)/include
	install -m755 target/ucx-opt/release/libshmem.so $(STAGE_UCX_OPT)/lib/
	install -m644 include/shmem.h $(STAGE_UCX_OPT)/include/
	tests/build_tests.sh $(STAGE_UCX_OPT) $(TESTBIN) _ucxo

# The runtime's optimized libfabric lamellae (feature libfabric-sys-opt-shared; run time LAMELLAR_BACKEND=libfabric-sys-opt):
# libfabric.so is staged beside libshmem.so as for stage-fab.
STAGE_FAB_OPT ?= target/fab-opt-stage
stage-fab-opt:
	$(MAKE) build header FEATURES=libfabric-sys-opt-shared TARGET=target/fab-opt
	install -d $(STAGE_FAB_OPT)/lib $(STAGE_FAB_OPT)/include
	install -m755 target/fab-opt/release/libshmem.so $(STAGE_FAB_OPT)/lib/
	install -m644 include/shmem.h $(STAGE_FAB_OPT)/include/
	tools/copy-libfabric.sh target/fab-opt $(STAGE_FAB_OPT)/lib
	tests/build_tests.sh $(STAGE_FAB_OPT) $(TESTBIN) _fabo

# UCX backend with UCX built as shared libraries (feature ucx-shared): the libuc*.so family and the lib/ucx plugins
# are staged beside libshmem.so, so the same UCX build can also be put in front of another UCX client's
# LD_LIBRARY_PATH (for example Open MPI's OSHMEM, to compare both on one UCX version).
STAGE_UCX_SHARED ?= target/ucx-shared-stage
stage-ucx-shared:
	$(MAKE) build header FEATURES=ucx-shared TARGET=target/ucx-shared
	install -d $(STAGE_UCX_SHARED)/lib $(STAGE_UCX_SHARED)/include
	install -m755 target/ucx-shared/release/libshmem.so $(STAGE_UCX_SHARED)/lib/
	install -m644 include/shmem.h $(STAGE_UCX_SHARED)/include/
	tools/copy-ucx.sh target/ucx-shared $(STAGE_UCX_SHARED)/lib
	tests/build_tests.sh $(STAGE_UCX_SHARED) $(TESTBIN) _ucxs

# Runtime-collectives probe (examples/coll_probe.rs) on the ucx backend
stage-probe:
	$(CARGO) build --release --target-dir target/ucx --features ucx --example coll_probe
	install -d $(TESTBIN) && install -m755 target/ucx/release/examples/coll_probe $(TESTBIN)/coll_probe_ucx

# shmem backend (single node, POSIX shared memory): default features, its own staged tree
STAGE_SHM ?= target/shmem-stage
stage-shmem:
	$(MAKE) build header FEATURES= TARGET=target
	install -d $(STAGE_SHM)/lib $(STAGE_SHM)/include
	install -m755 target/release/libshmem.so $(STAGE_SHM)/lib/
	install -m644 include/shmem.h $(STAGE_SHM)/include/
	tests/build_tests.sh $(STAGE_SHM) $(TESTBIN) _shm

# The runtime's optimized shared-memory lamellae (feature shmem-opt; run time LAMELLAR_BACKEND=shmem-opt), single node.
STAGE_SHM_OPT ?= target/shmem-opt-stage
stage-shmem-opt:
	$(MAKE) build header FEATURES=shmem-opt TARGET=target/shmem-opt
	install -d $(STAGE_SHM_OPT)/lib $(STAGE_SHM_OPT)/include
	install -m755 target/shmem-opt/release/libshmem.so $(STAGE_SHM_OPT)/lib/
	install -m644 include/shmem.h $(STAGE_SHM_OPT)/include/
	tests/build_tests.sh $(STAGE_SHM_OPT) $(TESTBIN) _shmo

stage-prrte:
	tools/stage-prrte.sh $(STAGE)

check: all
	bash tests/check_header.sh $(TARGET)/release/libshmem.so
	FEATURES="$(FEATURES)" TARGET=$(TARGET) bash tests/toolchain.sh

clean:
	$(CARGO) clean
