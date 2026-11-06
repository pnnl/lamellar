use anyhow::Result;
use num_traits::{FromPrimitive, PrimInt, ToPrimitive};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::marker::PhantomData;
use std::time::Instant;

//////////////////////////////////////////////////////////////////////////
// THE TESTS
//////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////
// The interface for our tests

pub trait TestWorld {
    // Initialize the PGAS library and return a world object.
    fn init() -> Self;

    fn mype(&self) -> usize;
    fn numpes(&self) -> usize;
    fn numthreads(&self) -> usize {
        //per pe
        1
    }
    fn xbarrier(&self);
}

pub trait TestArray {
    type Item;
    type World: TestWorld;

    /// Create cyclic array of type Item and the given size/PE.
    fn xnew(world: &Self::World, per_pe: usize) -> Self;

    fn xlen(&self) -> usize;

    /// Collective function: on each PE, run f on the local slice.
    /// (Depending on array type, this may need to convert array types and
    /// then convert them back.)
    fn run_on_local<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut [Self::Item]);

    fn xget(&self, index: usize) -> Self::Item;
    fn xput(&mut self, index: usize, val: Self::Item);
    fn xswap(&self, index: usize, val: Self::Item) -> Self::Item;

    /// Wait on all outstanding operations.
    /// Currently, this only affects xput (since get and swap need to wait for
    /// their return values, anyhow).
    fn fence(&mut self);
}

////////////////////////////////////////////////////////////////////////
// The benchmarks themselves

struct GenData<Item> {
    rng: StdRng,
    range: usize,
    _phantom: PhantomData<Item>,
}

impl<Item> GenData<Item>
where
    Item: PrimInt + FromPrimitive + ToPrimitive + Copy,
{
    fn new(seed: u64, range: usize) -> Self {
        GenData {
            rng: StdRng::seed_from_u64(seed),
            range,
            _phantom: PhantomData,
        }
    }

    fn next(&mut self) -> Item {
        let v = self.rng.gen_range(0..self.range);
        Item::from_u64(v as u64).expect("conversion from usize failed")
    }
}

//////////////////////////////////////////////////////////////////////////
// Individual operation tests
//////////////////////////////////////////////////////////////////////////

pub trait OperationTest<Array>
where
    Array: TestArray,
{
    fn run(
        array: &mut Array,
        index: usize,
        val: <Array as TestArray>::Item,
    ) -> <Array as TestArray>::Item;

    fn finish(array: &mut Array);
}

/// Put operation
pub struct OperationPut();

impl<Array> OperationTest<Array> for OperationPut
where
    Array: TestArray,
    <Array as TestArray>::Item: Default,
{
    fn run(
        array: &mut Array,
        index: usize,
        val: <Array as TestArray>::Item,
    ) -> <Array as TestArray>::Item {
        array.xput(index, val);
        <Array as TestArray>::Item::default()
    }

    fn finish(array: &mut Array) {
        array.fence();
    }
}

/// Get operation
pub struct OperationGet();

impl<Array> OperationTest<Array> for OperationGet
where
    Array: TestArray,
{
    fn run(
        array: &mut Array,
        index: usize,
        _val: <Array as TestArray>::Item,
    ) -> <Array as TestArray>::Item {
        array.xget(index)
    }

    fn finish(_array: &mut Array) {}
}

/// Swap operation.
/// For this operation, the accumulate value will not be deterministic.
pub struct OperationSwap();

impl<Array> OperationTest<Array> for OperationSwap
where
    Array: TestArray,
{
    fn run(
        array: &mut Array,
        index: usize,
        val: <Array as TestArray>::Item,
    ) -> <Array as TestArray>::Item {
        array.xswap(index, val)
    }

    fn finish(_array: &mut Array) {}
}

fn test_operation<Array, OpTest>(
    array: &mut Array,
    my_pe: usize,
    count: usize,
    seed: u64,
    start: &Instant,
) -> Result<<Array as TestArray>::Item>
where
    Array: TestArray,
    OpTest: OperationTest<Array>,
    <Array as TestArray>::Item: PrimInt + FromPrimitive + ToPrimitive + Copy,
{
    let n = array.xlen();
    let mut gendata: GenData<<Array as TestArray>::Item> = GenData::new(seed, array.xlen());

    let mut accum: <Array as TestArray>::Item =
        <Array as TestArray>::Item::from_u64(0).expect("from_u64 failed");

    for i in 0..count {
        let val = gendata.next();

        // if i % 1000 == 0 && my_pe == 0 {
        //     eprintln!("..{}", i);
        // }

        let v_usize = val.to_usize().expect("to_usize failed");
        let index = (3usize * v_usize) % n;
        let val = OpTest::run(array, index, val);
        accum = accum + val;
    }
    if my_pe == 0 {
        eprintln!("launch time: {:?}", start.elapsed().as_secs_f64());
    }
    

    OpTest::finish(array);

    Ok(accum)
}

fn init_array<Array>(my_pe: usize, num_pes: usize, array: Array) -> Array
where
    Array: TestArray,
    <Array as TestArray>::Item: PrimInt + FromPrimitive + ToPrimitive + Copy,
{
    let my_pe_item = <Array as TestArray>::Item::from_u64(my_pe as u64).expect("from_usize failed");
    let num_pes_item =
        <Array as TestArray>::Item::from_u64(num_pes as u64).expect("from_usize failed");
    let array = array.run_on_local(|data| {
        for (i, x) in data.iter_mut().enumerate() {
            let i_item = <Array as TestArray>::Item::from_u64(i as u64).expect("from_usize failed");
            *x = my_pe_item + i_item * num_pes_item;
        }
    });
    array
}

fn check_array<Array>(
    _my_pe: usize,
    _num_pes: usize,
    array: Array,
) -> (Array, <Array as TestArray>::Item)
where
    Array: TestArray,
    <Array as TestArray>::Item: PrimInt + FromPrimitive + ToPrimitive + Copy,
{
    let mut check: <Array as TestArray>::Item =
        <Array as TestArray>::Item::from_u64(0).expect("from_u64 failed");
    let array = array.run_on_local(|data| {
        for x in data.iter() {
            check = check + *x;
        }
    });
    (array, check)
}

/// Benchmark the given operation.
///
/// `nodes` scales the array: every PE holds `100000 * nodes` elements. This is the number of nodes, not the number of PEs: a run
/// with 32 PEs per node on 2 nodes uses `nodes = 2`, and so does one with 1 PE per node. It must be the same on every PE, so it is passed in (the
/// binaries take it from `--scale-nodes`, which the launcher hands to all ranks identically). The fallback, `SLURM_NNODES`, is
/// not safe: a launcher run from inside an sbatch script gives the rank on its own node the batch job's value and every other
/// rank its srun step's, so the PEs disagree about the array size and remote accesses run past the end of the smaller one.
pub fn run_optest<Array, OpTest>(_op: OpTest, nodes: Option<usize>) -> Result<()>
where
    Array: TestArray,
    OpTest: OperationTest<Array>,
    <Array as TestArray>::Item: PrimInt + FromPrimitive + ToPrimitive + Copy,
{
    let world = Array::World::init();
    let my_pe = world.mype();
    let num_pes = world.numpes();

    world.xbarrier();

    if my_pe == 0 {
        eprintln!("{}/{}: Starting optest ", my_pe, num_pes);
    }

    let num_nodes = match nodes {
        Some(n) if n > 0 => n,
        _ => {
            let from_env = std::env::var("SLURM_NNODES").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            if my_pe == 0 {
                eprintln!(
                    "warning: no --scale-nodes given; scaling the array by SLURM_NNODES={} as seen by PE 0. Other PEs may see a \
                     different value (see run_optest); pass --scale-nodes N.",
                    from_env
                );
            }
            from_env
        }
    };
    let mut array = Array::xnew(&world, 100000 * num_nodes);

    // Timed iterations per run: 25 for real measurements; set RANDPERF_ITERS lower for smoke tests.
    let iterations: u64 = std::env::var("RANDPERF_ITERS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(25);

    for i in 0..iterations {
        if my_pe == 0 {
            println!("{}/{}: Optest iteration {}", my_pe, num_pes, i);
        }
        
        world.xbarrier();

        array = init_array(my_pe, num_pes, array);
        // let (t_array, check) = check_array(my_pe, num_pes, array);

        // array = t_array;

        // eprintln!("{}/{}: Optest initialized check {check}", my_pe, num_pes);

        world.xbarrier();
        let start_time = Instant::now();
        let accum = test_operation::<Array, OpTest>(
            &mut array,
            my_pe,
            20000,
            123456789 + i + (my_pe as u64) * 523423,
            &start_time,
        )?;
        let runtime = start_time.elapsed().as_secs_f64();

        world.xbarrier();

        let full_runtime = start_time.elapsed().as_secs_f64();

        if my_pe == 0 {
            eprintln!(
                "{}/{}: Optest iteration {} complete; checking results.",
                my_pe, num_pes, i
            );
        }

        let (t_array, check) = check_array(my_pe, num_pes, array);
        array = t_array;

        if my_pe == 0 {
            eprintln!(
                "{}/{}: Completed(Accum:{}; Check:{}) in {} seconds ({} seconds)",
                my_pe,
                num_pes,
                accum.to_usize().unwrap_or(0),
                check.to_usize().unwrap_or(0),
                runtime,
                full_runtime
            );
        }

        world.xbarrier();

        if my_pe == 0 {
            println!(
                "FINAL TIME: {} seconds   (PE 0 Accum:{}  Check:{})",
                full_runtime,
                accum.to_usize().unwrap_or(0),
                check.to_usize().unwrap_or(0)
            );
        }
    }

    Ok(())
}
