/// A Lamellar-parallelized version of `hello_world_serial_workflow.rs`.
///
/// ORIGINAL (serial) design: a plain `for` loop built one `Workflow` per matrix,
/// running several computations (transpose, trace, row/col sums, Frobenius norm,
/// gram = M * M^T) sequentially, one matrix after another.
///
/// PARALLELIZED design (following `skills.md`):
///   - Each matrix's full workflow is packaged into an Active Message (`WorkflowAM`).
///   - The matrices are distributed round-robin across PEs (multi-processing) and
///     launched concurrently with `exec_am_pe(...).spawn()`; each PE's worker
///     threads run the AMs concurrently (multi-threading).
///   - Each AM returns a slim, serializable `WorkflowResult` (NOT the full matrices)
///     so we don't ship huge gram/transpose matrices back over the wire.
///   - Results are gathered on PE 0 by driving the spawned handles.
///
/// §1 procedure from skills.md:
///   INVENTORY : in  -> Matrix { name, rows, cols, data: Vec<Vec<f64>> }
///               out -> per-matrix computed scalars/vectors + result dims/names
///   CLASSIFY  : Matrix is SERIALIZABLE (String + usize + Vec<Vec<f64>>).
///               The full Workflow (stores whole transpose+gram matrices) is
///               SERIALIZABLE but EXPENSIVE -> return a slim WorkflowResult instead.
///   DATA FLOW : ship Matrix in, compute inside exec(), return WorkflowResult.
///   LAUNCH    : exec_am_pe(i % num_pes, am).spawn(); gather on PE 0.
///
/// Run:
///   cargo run --release --example hello_world_serial_to_parallel_using_skills
///   ./lamellar_run.sh -N=4 -T=8 ./target/release/examples/hello_world_serial_to_parallel_using_skills
use lamellar::active_messaging::prelude::*;
use serde::{Deserialize, Serialize};

/// Number of matrices (workflows) to process. Big enough that spreading the work
/// across PEs / worker threads is worthwhile.
const NUM_MATRICES: usize = 64;
/// Dimensions of each matrix. Each gram = M * M^T is DIM x DIM, so the matmul is
/// O(DIM^3) work per matrix -- heavy enough to benefit from parallelization.
const DIM: usize = 256;

//----------------------------- Data types --------------------------------//

/// A dense matrix. `AmData` makes it serializable so it can travel to a remote PE.
#[lamellar::AmData(Debug, Clone)]
struct Matrix {
    name: String,
    rows: usize,
    cols: usize,
    data: Vec<Vec<f64>>,
}

impl Matrix {
    fn new(name: &str, data: Vec<Vec<f64>>) -> Self {
        let rows = data.len();
        let cols = if rows == 0 { 0 } else { data[0].len() };
        Matrix {
            name: name.to_string(),
            rows,
            cols,
            data,
        }
    }
}

/// The slim, serializable result of a workflow. We deliberately do NOT return the
/// full transpose/gram matrices (EXPENSIVE) -- only what the original code printed.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowResult {
    matrix: String,
    transpose_rows: usize,
    transpose_cols: usize,
    trace: f64,
    row_sums: Vec<f64>,
    col_sums: Vec<f64>,
    frobenius_norm: f64,
    gram_name: String,
    gram_rows: usize,
    gram_cols: usize,
}

//----------------------------- Computations ------------------------------//

/// diagonal sum
fn trace(matrix: &Matrix) -> f64 {
    let n = matrix.rows.min(matrix.cols);
    (0..n).map(|i| matrix.data[i][i]).sum()
}

fn transpose(matrix: &Matrix) -> Matrix {
    let mut out = vec![vec![0.0; matrix.rows]; matrix.cols];
    for i in 0..matrix.rows {
        for j in 0..matrix.cols {
            out[j][i] = matrix.data[i][j];
        }
    }
    Matrix::new(&format!("{}^T", matrix.name), out)
}

fn row_sums(matrix: &Matrix) -> Vec<f64> {
    matrix.data.iter().map(|row| row.iter().sum()).collect()
}

fn col_sums(matrix: &Matrix) -> Vec<f64> {
    let mut sums = vec![0.0; matrix.cols];
    for row in &matrix.data {
        for (i, val) in row.iter().enumerate() {
            sums[i] += *val;
        }
    }
    sums
}

fn frobenius_norm(matrix: &Matrix) -> f64 {
    let sum_of_squares: f64 = matrix
        .data
        .iter()
        .flat_map(|row| row.iter())
        .map(|x| x * x)
        .sum();
    sum_of_squares.sqrt()
}

// matrix multiplication (matrix1.rows x matrix1.cols) * (matrix2.rows x matrix2.cols)
fn gram(matrix1: &Matrix, matrix2: &Matrix, name: &str) -> Matrix {
    let mut out = vec![vec![0.0; matrix2.cols]; matrix1.rows];
    for i in 0..matrix1.rows {
        for j in 0..matrix2.cols {
            let mut comb = 0.0;
            for k in 0..matrix1.cols {
                comb += matrix1.data[i][k] * matrix2.data[k][j];
            }
            out[i][j] = comb;
        }
    }
    Matrix::new(name, out)
}

/// Build the full workflow for one matrix and package the slim result.
fn build_workflow(matrix: &Matrix) -> WorkflowResult {
    let transp = transpose(matrix);
    let tr = trace(matrix);
    let rs = row_sums(matrix);
    let cs = col_sums(matrix);
    let norm = frobenius_norm(matrix);
    let mm = gram(matrix, &transp, &format!("{}*{}^T", matrix.name, matrix.name));

    WorkflowResult {
        matrix: matrix.name.clone(),
        transpose_rows: transp.rows,
        transpose_cols: transp.cols,
        trace: tr,
        row_sums: rs,
        col_sums: cs,
        frobenius_norm: norm,
        gram_name: mm.name,
        gram_rows: mm.rows,
        gram_cols: mm.cols,
    }
}

//----------------------------- Active Message ----------------------------//

/// The AM that runs one full workflow on a (possibly remote) PE and returns the
/// slim serializable result.
#[lamellar::AmData(Debug, Clone)]
struct WorkflowAM {
    matrix: Matrix,
}

#[lamellar::am]
impl LamellarAM for WorkflowAM {
    async fn exec(self) -> WorkflowResult {
        println!(
            "\t[PE {}] processing matrix {} ({} by {})",
            lamellar::current_pe, self.matrix.name, self.matrix.rows, self.matrix.cols
        );
        build_workflow(&self.matrix)
    }
}

//----------------------------- Input generation --------------------------//

/// Deterministically build a DIM x DIM matrix so runs are reproducible.
fn make_matrix(idx: usize) -> Matrix {
    let mut data = vec![vec![0.0; DIM]; DIM];
    for i in 0..DIM {
        for j in 0..DIM {
            // simple deterministic fill that depends on the matrix index
            data[i][j] = ((i * 31 + j * 7 + idx * 13) % 97) as f64 * 0.5 + 1.0;
        }
    }
    Matrix::new(&format!("M{idx}"), data)
}

//----------------------------- Main --------------------------------------//

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    world.barrier();

    // Only PE 0 builds inputs, launches, and gathers.
    if my_pe == 0 {
        println!(
            "Launching {} workflows ({}x{} matrices) across {} PE(s)",
            NUM_MATRICES, DIM, DIM, num_pes
        );

        let matrices: Vec<Matrix> = (0..NUM_MATRICES).map(make_matrix).collect();

        // MULTI-PROCESSING: distribute matrices round-robin across PEs.
        // MULTI-THREADING: each PE's worker threads run its AMs concurrently.
        // spawn() launches lazily and returns a handle we drive later.
        let mut handles = Vec::with_capacity(NUM_MATRICES);
        for (i, matrix) in matrices.into_iter().enumerate() {
            let am = WorkflowAM { matrix };
            handles.push(world.exec_am_pe(i % num_pes, am).spawn());
        }

        // Gather: drive each handle to completion, collecting results on PE 0.
        let results: Vec<WorkflowResult> = handles.into_iter().map(|h| h.block()).collect();

        for workflow in &results {
            println!("Matrix {}:", workflow.matrix);
            println!(
                "  transpose = {} by {}",
                workflow.transpose_rows, workflow.transpose_cols
            );
            println!("  trace = {:?}", workflow.trace);
            println!(
                "  row_sums[0..4] = {:?} ...",
                &workflow.row_sums[..workflow.row_sums.len().min(4)]
            );
            println!(
                "  col_sums[0..4] = {:?} ...",
                &workflow.col_sums[..workflow.col_sums.len().min(4)]
            );
            println!("  frobenius_norm = {:?}", workflow.frobenius_norm);
            println!(
                "  gram: {} ({} by {})",
                workflow.gram_name, workflow.gram_rows, workflow.gram_cols
            );
        }

        println!("Completed {} workflows.", results.len());
    }

    world.barrier();
}