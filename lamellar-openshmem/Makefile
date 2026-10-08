# Convenience targets for working on the Lamellar OpenSHMEM library.
#
#   make              build libshmem (default features: shmem backend) and regenerate include/shmem.h
#   make header       regenerate include/shmem.h (python3)
#   make wrappers     build the compiler wrappers and launcher into wrappers/target/release
#   make prrte        one-time: build the vendored PRRTE (prterun) into target/fab/prrte for the multi-node tests
#                     on clusters whose Slurm has no PMIx support (several minutes; run on a login node)
#   make check        header check + toolchain test (installs into a temporary prefix; needs cargo-c)
#
# Installing is done with cargo, not make:
#   cargo cinstall --release --prefix <prefix> [--features <backend>]      libshmem.{so,a}, shmem.h, shmem.pc, CMake package
#   cargo install --path wrappers --root <prefix>                          shmemcc, oshcc, shmemcxx, oshcxx, oshrun, shmemrun
# (cargo-c: `cargo install cargo-c`). Staging a backend and running the tests: tests/stage.sh <backend>, then ctest
# (see tests/CMakeLists.txt).
CARGO    ?= cargo
FEATURES ?=
CARGOFLAGS ?=
TARGET   ?= target
FEAT     := $(if $(FEATURES),--features $(FEATURES),)

# Optional local targets (site-specific helpers); not part of the repository.
-include local.mk

.PHONY: all build header wrappers prrte check clean
all: build header

build:
	$(CARGO) build --release --target-dir $(TARGET) $(FEAT) $(CARGOFLAGS)

header:
	python3 include/gen_shmem_h.py include/shmem.h

wrappers:
	$(CARGO) build --release --manifest-path wrappers/Cargo.toml

prrte:
	tools/stage-prrte.sh target/fab

check: all
	bash tests/check_header.sh $(TARGET)/release/libshmem.so
	FEATURES="$(FEATURES)" TARGET=$(TARGET) bash tests/toolchain.sh

clean:
	$(CARGO) clean
