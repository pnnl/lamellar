#!/usr/bin/env python3
"""Copy what the binaries need at run time out of cargo's build directory (node-local disk, invisible to compute nodes) onto shared disk.

    python3 tools/stage_runtime.py [BIN_DIR]        default: <workspace>/builds/bin     (tools/build-binaries.sh runs it)

Result: BIN_DIR/runtime/prrte/{bin,etc,share}: prterun, prted, ... with the data files of PRRTE and PMIx (both are linked statically
into the launcher, so nothing else is needed). hpc_driver.sh points LAMELLAR_PRTERUN at BIN_DIR/runtime/prrte/bin/prterun; the binaries
then set PRTE_PREFIX / PMIX_PREFIX to that tree themselves (hpc_launch does it).

Where the launcher comes from: every binary of this workspace has the path of the prterun it was built with embedded (build-time OUT_DIR),
so the build directory's location is never assumed; the newest existing one among the binaries in BIN_DIR wins. Open MPI binaries are
skipped when looking for the launcher (their embedded path is a build-directory one, too).

Open MPI (randperf-shmem-ompi-ucx): the vendored install tree (lib, share, etc) and the UCX it was built against are copied to
BIN_DIR/runtime/ompi, the build's config.log to runtime/ompi/build/ and the PRRTE it was configured with (--with-prrte in that log) to runtime/prrte-ompi, and the RPATHs
are rewritten with patchelf: $ORIGIN for the libraries, $ORIGIN/.. for the plugin directories (lib/openmpi, lib/ucx), and
$ORIGIN/runtime/ompi/lib for the binary (it sits in BIN_DIR, next to runtime/). hpc_driver.sh sets OPAL_PREFIX=BIN_DIR/runtime/ompi.
A system Open MPI (built with OMPI_DIR) is left where it is. Needs patchelf.
Running it again is harmless (a binary whose libraries already resolve into runtime/ is not touched). Python 3.6 compatible.
"""
import os
import re
import shutil
import subprocess
import sys

SYSTEM = ("/lib/", "/lib64/", "/usr/", "/share/apps/")


def merge(src, dst):
    os.makedirs(dst, exist_ok=True)
    subprocess.check_call(["cp", "-a", src.rstrip("/") + "/.", dst])


def find_prterun(bin_dir):
    found = {}
    for name in sorted(os.listdir(bin_dir)):
        b = os.path.join(bin_dir, name)
        if not os.path.isfile(b) or "ompi" in name or name.endswith((".sha256", ".provenance")):
            continue
        with open(b, "rb") as fh:
            for m in re.finditer(rb"(/[^\0\s]*/prrte-sys-[0-9a-f]+/out/bin/prterun)", fh.read()):
                p = m.group(1).decode()
                if os.path.exists(p):
                    found[p] = os.path.getmtime(p)
    return max(found, key=found.get) if found else None


def stage_prrte(prterun, dest):
    """Copy the PRRTE tree of `prterun` (bin, etc, share) and the data files of the PMIx its RUNPATH names to dest."""
    out = os.path.dirname(os.path.dirname(prterun))                       # .../prrte-sys-<hash>/out
    for sub in ("bin", "etc", "share"):
        if os.path.isdir(os.path.join(out, sub)):
            merge(os.path.join(out, sub), os.path.join(dest, sub))
    pmix = None                                        # PMIx's own files: the pmix-sys directory the launcher's RUNPATH names
    for line in subprocess.check_output(["readelf", "-d", prterun], universal_newlines=True).splitlines():
        m = re.search(r"(/\S*/pmix-sys-[0-9a-f]+/out)/lib", line)
        if m:
            pmix = m.group(1)
            break
    if pmix:
        for sub in ("etc", "share"):
            if os.path.isdir(os.path.join(pmix, sub)):
                merge(os.path.join(pmix, sub), os.path.join(dest, sub))
    print("staged PRRTE%s from %s -> %s" % (" + PMIx" if pmix else "", os.path.basename(os.path.dirname(out)), dest))


def sh(*cmd):
    return subprocess.check_output(cmd, universal_newlines=True, stderr=subprocess.STDOUT)


def stage_ompi_prrte(runtime, cfg):
    """Stage the PRRTE named by --with-prrte in Open MPI's config.log (a no-op if the log or that build directory is gone)."""
    if not os.path.isfile(cfg):
        return
    m = re.search(r"--with-prrte=(\S+)", open(cfg, errors="replace").read())
    pr = os.path.join(m.group(1).strip("'"), "bin", "prterun") if m else ""
    if pr and os.path.isfile(pr):
        stage_prrte(pr, os.path.join(runtime, "prrte-ompi"))


def stage_ompi(bin_dir, runtime):
    binary = os.path.join(bin_dir, "randperf-shmem-ompi-ucx")
    if not os.path.isfile(binary):
        return
    libs = [os.path.realpath(l) for l in re.findall(r"=> (/\S+)", sh("ldd", binary))]
    osh = next((l for l in libs if "/liboshmem" in l), None)
    ucp = next((l for l in libs if "/libucp.so" in l), None)
    real_runtime = os.path.realpath(runtime)
    # nothing to do for a system Open MPI, when the binary already resolves its libraries from the staged tree (second run),
    # or for a binary that has no Open MPI / UCX libraries at all
    if osh is None or ucp is None or osh.startswith(real_runtime + os.sep) or osh.startswith(SYSTEM):
        staged = bool(osh and osh.startswith(real_runtime + os.sep))
        print("Open MPI: nothing to stage (%s)" % ("already staged" if staged else "system or no Open MPI libraries"))
        if staged:
            stage_ompi_prrte(runtime, os.path.join(runtime, "ompi", "build", "config.log"))
        return
    prefix = os.path.dirname(os.path.dirname(osh))
    dest = os.path.join(runtime, "ompi")
    for sub in ("lib", "share", "etc"):
        if os.path.isdir(os.path.join(prefix, sub)):
            merge(os.path.join(prefix, sub), os.path.join(dest, sub))
    merge(os.path.dirname(ucp), os.path.join(dest, "lib"))                    # libucp / libuct / libucs / libucm and the ucx/ modules
    cfg = os.path.join(prefix, "build", "config.log")                         # names the PRRTE this Open MPI was configured with
    if os.path.isfile(cfg):
        os.makedirs(os.path.join(dest, "build"), exist_ok=True)
        shutil.copy2(cfg, os.path.join(dest, "build", "config.log"))
    lib = os.path.join(dest, "lib")
    for root, _dirs, files in os.walk(lib):
        for f in files:
            path = os.path.join(root, f)
            if ".so" in f and not os.path.islink(path):
                subprocess.call(["patchelf", "--set-rpath", "$ORIGIN" if root == lib else "$ORIGIN/..", path])
    # The PRRTE this Open MPI was configured with (--with-prrte=<dir> in config.log): its PMIx has to match the one in Open MPI, so it is staged
    # on its own, next to the generic launcher, as runtime/prrte-ompi (hpc_driver.sh prefers it for --osh ompi).
    stage_ompi_prrte(runtime, cfg)
    subprocess.check_call(["patchelf", "--set-rpath", "$ORIGIN/runtime/ompi/lib", binary])
    print("staged Open MPI (%s) + UCX -> %s" % (prefix, dest))


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    bin_dir = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.join(here, "..", "builds", "bin")
    bin_dir = os.path.abspath(bin_dir)
    prterun = find_prterun(bin_dir)
    if prterun is None:
        if os.path.exists(os.path.join(bin_dir, "runtime", "prrte", "bin", "prterun")):
            print("PRRTE: build directory gone, keeping the staged copy")      # second run on a machine without the build directory
            stage_ompi(bin_dir, os.path.join(bin_dir, "runtime"))
            return
        sys.exit("stage_runtime: no prterun path found in the binaries of %s (build them first: tools/build-binaries.sh)" % bin_dir)
    stage_prrte(prterun, os.path.join(bin_dir, "runtime", "prrte"))
    stage_ompi(bin_dir, os.path.join(bin_dir, "runtime"))


if __name__ == "__main__":
    main()
