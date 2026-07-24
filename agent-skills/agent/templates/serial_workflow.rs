/// A serial loop to build oner workflow per matrix and runing several computations within
/// each iteration
/// 
/// This is without using any async, threads etc so that to test `skills.md` on a serial loop
/// 
/// 


#[derive(Debug, Clone)]
struct Matrix{
    name: String,
    rows: usize,
    cols:usize,
    data: Vec<Vec<f64>>
}

impl Matrix {
    fn new(name:&str, data:Vec<Vec<f64>>) -> Self {
        let rows = data.len();
        let cols = if rows ==0{0} else {data[0].len()};
        Matrix{
            name:name.to_string(),
            rows,
            cols,
            data
        }

    }
}

#[derive(Debug)]
struct Workflow{
    matrix:String,
    transpose: Matrix,
    trace: f64,
    row_sums: Vec<f64>,
    col_sums: Vec<f64>,
    frobenius_norm: f64,
    gram:Matrix, 
}



/// Computations
/// diagonal sum  s
fn trace(matrix: &Matrix) -> f64{
    let n = matrix.rows.min(matrix.cols);
    (0..n).map(|i| matrix.data[i][i]).sum()
}

fn transpose(matrix: &Matrix) -> Matrix{
    let mut out = vec![vec![0.0;matrix.rows]; matrix.cols];
    for i in 0..matrix.rows{
        for j in 0..matrix.cols{
            out[j][i]= matrix.data[i][j];
        }
    }
    Matrix::new(&format!("{}^T", matrix.name), out)
}

fn row_sums(matrix: &Matrix) -> Vec<f64>{
    matrix.data.iter().map(|row| row.iter().sum()). collect()
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
    let sum_of_squares: f64 = matrix.data.iter().flat_map(|row| row.iter()) .map(|x| x * x).sum();
    sum_of_squares.sqrt()
}

//matrix multiolication (x.rows * x.cols )*(y.rows * y.cols )
fn gram(matrix1: &Matrix,matrix2: &Matrix, name:&str) -> Matrix{
    let mut out = vec![vec![0.0;matrix2.cols]; matrix1.rows];
    for i in 0..matrix1.rows{
        for j in 0..matrix2.cols{
            let mut comb = 0.0;
            for k in 0..matrix1.cols{
                comb +=matrix1.data[i][k]*matrix2.data[k][j];
            }
            out[i][j] = comb;
        }
    }
    Matrix::new(name,out)
}

fn build_workflow(matrix: &Matrix)-> Workflow{
    let transp = transpose(matrix);
    let tr = trace(matrix);
    let rs = row_sums(matrix);
    let cs = col_sums(matrix);
    let norm = frobenius_norm(matrix);
    let mm = gram(matrix, &transp, &format!("{}*{}^T", matrix.name, matrix.name));

    Workflow{
        matrix:matrix.name.clone(),
        transpose: transp,
        trace: tr,
        row_sums: rs,
        col_sums: cs,
        frobenius_norm: norm,
        gram:mm, 
    }

}

fn main(){
    let matrices = vec![Matrix::new("A", vec![vec![1.0,2.0,3.0], vec![4.0,5.0,6.0]]),
                                        Matrix::new("B", vec![vec![1.0,2.0,3.0], vec![7.0,8.0,9.0]]),
                                         Matrix::new("C", vec![vec![1.0,11.0,12.0], vec![13.0,14.0,15.0]])
                                        ];

    let mut results : Vec<Workflow> = Vec::new();

    // bulding one workflow built and flly computed per matric

    for matrix in &matrices{
        println!("Processing matrix:{} ({} by {})", matrix.name, matrix.rows, matrix.cols);
        let workflow = build_workflow(matrix);
        results.push(workflow);

    }

    for workflow in &results{
        println!("Matrix{}:", workflow.matrix);
        println!("transpose={:?} by {:?}", workflow.transpose.rows, workflow.transpose.cols);
        println!("trace={:?}", workflow.trace);
        println!("row_sums={:?}", workflow.row_sums);
        println!("col_sums={:?}", workflow.col_sums);
        println!("frobenius_norm={:?}", workflow.frobenius_norm);
        println!("gram:{} ({} by {})", workflow.gram.name, workflow.gram.rows, workflow.gram.cols);

    }
    
}