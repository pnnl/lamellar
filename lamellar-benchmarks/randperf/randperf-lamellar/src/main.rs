use std::time::Instant;
use std::marker::PhantomData;
use std::mem::size_of;
use std::pin::Pin;
use std::ptr::null_mut;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use lamellar::SharedMemoryRegion;
use paste::paste;
use rand::{rngs::StdRng, Rng, SeedableRng};
use lamellar::array::prelude::*;
use lamellar::memregion::prelude::*;

use randperf_core::{OperationGet, OperationPut, OperationSwap, TestArray, TestWorld, run_optest};

use tracing_subscriber::prelude::*;


type TestType = isize;
//////////////////////////////////////////////////////////////////////////
// THE IMPLEMENTATIONS
//////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////



// Lamellar wrapper to satisfy Rust's orphan rules:
// we define a local type `LamellarWorldLocal` and implement the
// external `TestWorld` trait for it. Also provide `Deref`
// so it can be used wherever `&lamellar::LamellarWorld` is expected.

struct LamellarWorldLocal {
    inner: lamellar::LamellarWorld,
}

// impl Drop for LamellarWorldLocal {
//     fn drop(&mut self) {
//         self.inner.print_setup_op_times();
//     }
// }

impl std::ops::Deref for LamellarWorldLocal {
    type Target = lamellar::LamellarWorld;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TestWorld for LamellarWorldLocal {
    fn init() -> Self {
        LamellarWorldLocal {
            inner: lamellar::LamellarWorldBuilder::new().build(),
        }
    }

    fn mype(&self) -> usize {
        self.inner.my_pe()
    }
    fn numpes(&self) -> usize {
        self.inner.num_pes()
    }
    fn numthreads(&self) -> usize {
        self.inner.num_threads_per_pe()
    }
    fn xbarrier(&self) {
        self.inner.barrier();
    }
}

struct XUnsafeArray<T: Dist> {
    array: UnsafeArray<T>,
    context: Vec<lamellar::RdmaHandle<T>>,
}

impl<T> TestArray for XUnsafeArray<T>
where
    T: Dist + ArrayOps + Default + 'static,
{
    type Item = T;
    type World = LamellarWorldLocal;

    fn xlen(&self) -> usize {
        self.array.len()
    }

    // Create cyclic array of type Item and the given size/PE.
    fn xnew(world: &Self::World, size_per_pe: usize) -> Self {
        let num_pes = world.num_pes();
        XUnsafeArray {
            array: UnsafeArray::<T>::new(&world.inner, size_per_pe * num_pes, Distribution::Cyclic)
                .block(),
            context: Vec::new(),
        }
    }

    fn run_on_local<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut [Self::Item]),
    {
        let local = unsafe { self.array.local_as_mut_slice() };
        f(local);
        self
    }

    fn xget(&self, index: usize) -> Self::Item {
        // unsafe { self.array.get(index).block() }
        unsafe { self.array.blocking_get(index) }
    }

    fn xput(&mut self, index: usize, val: Self::Item) {
        unsafe { self.array.put_unmanaged(index, val) };
    }

    fn xswap(&self, index: usize, val: Self::Item) -> Self::Item {
        // unsafe { self.array.swap(index, val).block() }
        unsafe { self.array.blocking_swap(index, val) }
    }

    fn fence(&mut self) {
        self.array.wait_all();
    }
}

struct XAtomicArray<T>
where
    T: Dist,
{
    array: AtomicArray<T>,
    context: Vec<LamellarTask<()>>,
}

impl<T> TestArray for XAtomicArray<T>
where
    T: Dist + ArrayOps + Default,
{
    type Item = T;
    type World = LamellarWorldLocal;

    fn xlen(&self) -> usize {
        self.array.len()
    }

    // Create cyclic array of type Item and the given size/PE.
    fn xnew(world: &Self::World, size_per_pes: usize) -> Self {
        let num_pes = world.num_pes();
        XAtomicArray {
            array: AtomicArray::<T>::new(&world.inner, size_per_pes * num_pes, Distribution::Cyclic)
                .block(),
            context: Vec::new(),
        }
    }

    fn run_on_local<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut [Self::Item]),
    {
        let mut start = Instant::now();
        let tmp = self.array.into_local_lock().block();
        start = Instant::now();
        let mut local = tmp.write_local_data().block();
        f(&mut *local);
        drop(local);
        start = Instant::now();
        let array = tmp.into_atomic().block();
        eprintln!(
            "run_on_local into_atomic time: {}",
            start.elapsed().as_secs_f64()
        );
        XAtomicArray {
            array,
            context: self.context,
        }
        // self
    }

    fn xget(&self, index: usize) -> Self::Item {
        // self.array.get(index).block()
        unsafe { self.array.blocking_get(index) }
    }

    fn xput(&mut self, index: usize, val: Self::Item) {
        unsafe { self.array.put_unmanaged(index, val) };
    }

    fn xswap(&self, index: usize, val: Self::Item) -> Self::Item {
        // self.array.swap(index, val).block()
        unsafe { self.array.blocking_swap(index, val) }
    }

    fn fence(&mut self) {
        self.array.wait_all();
    }
}


////////////////////////////////////////////////////////////////////////
// MAIN
//////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Operation type (e.g., put, get, swap)
    #[arg(long, value_enum)]
    op: OpType,

    /// Array type (e.g., Shmem, ShmemAtomic, Libfabric, LibfabricAtomic)
    #[arg(long, value_enum)]
    array_type: ArrayType,

    /// Number of NODES the per-PE array size is scaled by (100000 * N elements per PE). This is not the PE count: it is the same
    /// for 1 PE per node or 32 (--pes in the launcher options is the PE count). It is part of the command line, so every rank
    /// gets the same value. Default: $SLURM_NNODES, with a warning.
    #[arg(long)]
    scale_nodes: Option<usize>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OpType {
    Put,
    Get,
    Swap,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ArrayType {
    Lamellar,
    LamellarAtomic,
}

fn launch_test<Array>(args: &Args) -> Result<()>
where
    Array: TestArray<Item = TestType>,
{
    match args.op {
        OpType::Put => run_optest::<Array, _>(OperationPut(), args.scale_nodes)?,
        OpType::Get => run_optest::<Array, _>(OperationGet(), args.scale_nodes)?,
        OpType::Swap => run_optest::<Array, _>(OperationSwap(), args.scale_nodes)?,
    };
    Ok(())
}

#[lamellar::main]
fn main() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_level(true),
        )
        .init();
    let args = Args::parse();
    if let Some(0) = lamellar::pe_id() {
        println!(
            "Selected op: {:?}, array type: {:?}",
            args.op, args.array_type
        );
    }
    match args.array_type {
        ArrayType::Lamellar => launch_test::<XUnsafeArray<TestType>>(&args)?,
        ArrayType::LamellarAtomic => launch_test::<XAtomicArray<TestType>>(&args)?,
    }

    Ok(())
}
