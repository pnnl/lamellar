//! Compiler wrappers and launcher for the Lamellar OpenSHMEM library.
//!
//! * [`cc`]: `shmemcc` / `oshcc` (C) and `shmemcxx` / `oshcxx` (C++): run the system compiler with the include and
//!   library flags for libshmem.
//! * [`launch`]: `oshrun` / `shmemrun`: start N PEs of a program locally, under Slurm, or through PRRTE.

pub mod cc;
pub mod launch;
