///////////////////////////////////////////////////////////////////////
// Shmem version of the code : ShmemWorld

use anyhow::Result;
use clap::{Parser, ValueEnum};
use paste::paste;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::null_mut;

use randperf_core::{OperationGet, OperationPut, OperationSwap, TestArray, TestWorld, run_optest};

use shmem_shim as shmem;
// Local alias so changing test element type is a single edit here.
type TestType = isize;
struct ShmemWorld {}

impl TestWorld for ShmemWorld {
    // Initialize the PGAS library and return a world object.
    fn init() -> Self {
        unsafe { shmem::shmem_init() };
        Self {}
    }

    fn mype(&self) -> usize {
        unsafe { shmem::shmem_my_pe().try_into().unwrap() }
    }

    fn numpes(&self) -> usize {
        unsafe { shmem::shmem_n_pes().try_into().unwrap() }
    }

    fn xbarrier(&self) {
        unsafe { shmem::shmem_barrier_all() };
    }
}

impl Drop for ShmemWorld {
    fn drop(&mut self) {
        unsafe { shmem::shmem_finalize() };
    }
}

////////////////////////////////////////////////////////////////////////
// Shmem version of the code: arrays

trait ShmemAtomic {
    unsafe fn put(dest: *mut Self, val: Self, pe: usize);
    unsafe fn get(src: *mut Self, pe: usize) -> Self;

    unsafe fn atomic_put(dest: *mut Self, val: Self, pe: usize);
    unsafe fn atomic_get(src: *mut Self, pe: usize) -> Self;
    unsafe fn atomic_swap(dest: *mut Self, val: Self, pe: usize) -> Self;
}

macro_rules! shmematomic {
    ( $t:ty, $ct:ty, $name:ident ) => {
        impl ShmemAtomic for $t {
            unsafe fn put(dest: *mut Self, val: Self, pe: usize) {
                unsafe {
                    paste! { shmem::[< shmem_ $name _p >](dest as *mut _,val as $ct,pe as i32) }
                };
            }

            unsafe fn get(src: *mut Self, pe: usize) -> Self {
                unsafe {
                    paste! { shmem::[< shmem_ $name _g >](src as *mut _,pe as i32) as $t }
                }
            }

            unsafe fn atomic_put(dest: *mut Self, val: Self, pe: usize) {
                unsafe {
                    paste! { shmem::[< shmem_ $name _atomic_set >](dest as *mut _,
                    val as $ct,pe as i32) }
                };
            }

            unsafe fn atomic_get(src: *mut Self, pe: usize) -> Self {
                unsafe {
                    paste! { shmem::[< shmem_ $name _atomic_fetch >](src as *mut _,
                    pe as i32) as $t }
                }
            }

            unsafe fn atomic_swap(dest: *mut Self, val: Self, pe: usize) -> Self {
                unsafe {
                    paste! { shmem::[< shmem_ $name _atomic_swap >](dest as *mut _,
                    val as $ct,pe as i32) as $t }
                }
            }
        }
    };
}

shmematomic!(i32, i32, int);
shmematomic!(isize, i64, longlong);

struct ShmemArray<T> {
    data: *mut T,
    size: usize,
    mype: usize,
    numpes: usize,
    _xdata: PhantomData<Box<[T]>>,
}

impl<T> ShmemArray<T> {
    /// Check bounds and return (pe, offset-adjusted pointer)
    fn check(&self, index: usize) -> (usize, *mut T) {
        assert!(index < self.size);
        let pe = index % self.numpes;
        let ofs = (index / self.numpes).try_into().unwrap();
        (pe, unsafe { self.data.offset(ofs) as *mut T })
    }
}

impl<T> TestArray for ShmemArray<T>
where
    T: Default + ShmemAtomic,
{
    type Item = T;
    type World = ShmemWorld;

    // Create cyclic array of type Item and the given size/PE.
    fn xnew(world: &Self::World, per_pe: usize) -> Self {
        let numpes = world.numpes();
        let mype = world.mype();
        let alloc_size = per_pe * size_of::<T>();
        let data = unsafe { shmem::shmem_malloc(alloc_size) as *mut T };
        // assert!(!data.is_null());
        Self {
            data,
            size: per_pe * numpes,
            mype,
            numpes,
            _xdata: PhantomData,
        }
    }

    fn xlen(&self) -> usize {
        self.size
    }

    fn run_on_local<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut [Self::Item]),
    {
        let local_size = (self.size + self.numpes - self.mype - 1) / self.numpes;
        let slice = unsafe { std::slice::from_raw_parts_mut(self.data, local_size) };
        f(slice);
        self
    }

    fn xget(&self, index: usize) -> Self::Item {
        let (pe, shared) = self.check(index);
        unsafe { Self::Item::get(shared, pe) }
    }

    fn xput(&mut self, index: usize, val: Self::Item) {
        let (pe, shared) = self.check(index);
        unsafe { Self::Item::put(shared, val, pe) }
    }

    fn xswap(&self, index: usize, val: Self::Item) -> Self::Item {
        let (pe, shared) = self.check(index);
        unsafe { Self::Item::atomic_swap(shared, val, pe) }
    }

    fn fence(&mut self) {
        unsafe {
            shmem::shmem_quiet();
        }
    }
}

impl<T> Drop for ShmemArray<T> {
    fn drop(&mut self) {
        assert!(!self.data.is_null());
        unsafe { shmem::shmem_free(self.data as *mut _) };
        self.data = null_mut();
    }
}

struct ShmemAtomicArray<T>(ShmemArray<T>);

impl<T> TestArray for ShmemAtomicArray<T>
where
    T: Default + ShmemAtomic,
{
    type Item = T;
    type World = ShmemWorld;

    fn xnew(world: &Self::World, per_pe: usize) -> Self {
        Self(ShmemArray::xnew(world, per_pe))
    }

    fn run_on_local<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut [Self::Item]),
    {
        Self(self.0.run_on_local(f))
    }

    fn xlen(&self) -> usize {
        self.0.size
    }

    fn xget(&self, index: usize) -> Self::Item {
        let (pe, shared) = self.0.check(index);
        unsafe { Self::Item::atomic_get(shared, pe) }
    }

    fn xput(&mut self, index: usize, val: Self::Item) {
        let (pe, shared) = self.0.check(index);
        unsafe { Self::Item::atomic_put(shared, val, pe) }
    }

    fn xswap(&self, index: usize, val: Self::Item) -> Self::Item {
        // Already atomic
        self.0.xswap(index, val)
    }

    fn fence(&mut self) {
        unsafe {
            shmem::shmem_quiet();
        }
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
    Shmem,
    ShmemAtomic,
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

#[hpc_launch::main]
fn main() -> Result<()> {
    // tracing removed
    let args = Args::parse();
    if let Some(0) = hpc_launch::pe_id() {
        println!(
            "Selected op: {:?}, array type: {:?}",
            args.op, args.array_type
        );
    }

    match args.array_type {
        ArrayType::Shmem => launch_test::<ShmemArray<TestType>>(&args)?,
        ArrayType::ShmemAtomic => launch_test::<ShmemAtomicArray<TestType>>(&args)?,
    }

    Ok(())
}
