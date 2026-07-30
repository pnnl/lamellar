//! triangle_count — SERIAL baseline (plain Rust, no lamellar, no threads/async).
//!
//! Task: count all triangles in an undirected graph, each triangle exactly once.
//! Final line prints `RESULT:<count>`.
//!
//! Graph source (two modes):
//!   * `--graph <path>` : load an undirected edge list (whitespace `u v` per
//!     line, `#` comments ignored, self-loops and duplicate edges dropped).
//!   * (no flag)        : generate deterministically from a fixed seed
//!     (splitmix64, no extra deps) so the count is reproducible and identical to
//!     the parallel version. "medium" scale is set below via N_VERTICES/AVG_DEGREE.
//!
//! Counting method (exact, once-per-triangle): build sorted adjacency lists with
//! no self-loops and no duplicate edges. For each vertex `u`, for each neighbor
//! `v > u`, count common neighbors `w` with `w > v` via a sorted-list
//! intersection. The strict ordering u < v < w guarantees each triangle is
//! counted exactly once.

// ---- data-scale knobs (medium) ----
const N_VERTICES: usize = 2_000;
const AVG_DEGREE: usize = 10;
const SEED: u64 = 0x1234_5678_9abc_def0;

/// Deterministic splitmix64 PRNG. Process-local, no deps. Good bit distribution
/// (all bits well-mixed), so `% n` gives a well-spread vertex id.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        // splitmix64
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn gen_range(&mut self, upper: usize) -> usize {
        (self.next_u64() % (upper as u64)) as usize
    }
}

/// Insert an undirected edge into sorted, deduped adjacency lists.
/// Ignores self-loops and duplicate edges. Returns true if a NEW edge was added.
fn insert_edge(adj: &mut [Vec<usize>], a: usize, b: usize) -> bool {
    if a == b {
        return false;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    match adj[lo].binary_search(&hi) {
        Ok(_) => return false, // edge already present
        Err(pos) => adj[lo].insert(pos, hi),
    }
    match adj[hi].binary_search(&lo) {
        Ok(_) => {}
        Err(pos) => adj[hi].insert(pos, lo),
    }
    true
}

/// Load an undirected graph from a whitespace edge-list file.
/// Each non-comment line is `u v`; `#` starts a comment. Vertex ids are usize;
/// the vertex count is (max id + 1). Self-loops and duplicate edges are dropped.
fn load_graph_from_file(path: &str) -> Vec<Vec<usize>> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read graph file '{}': {}", path, e));

    // First pass: collect edges and find the max vertex id.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut max_id = 0usize;
    for line in text.lines() {
        let line = match line.find('#') {
            Some(p) => &line[..p],
            None => line,
        };
        let mut it = line.split_whitespace();
        let (a, b) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => continue, // blank / malformed line
        };
        let a: usize = a
            .parse()
            .unwrap_or_else(|_| panic!("bad vertex id '{}' in {}", a, path));
        let b: usize = b
            .parse()
            .unwrap_or_else(|_| panic!("bad vertex id '{}' in {}", b, path));
        max_id = max_id.max(a).max(b);
        edges.push((a, b));
    }

    let n = if edges.is_empty() { 0 } else { max_id + 1 };
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (a, b) in edges {
        insert_edge(&mut adj, a, b);
    }
    adj
}

/// Build an undirected graph as sorted adjacency lists.
/// Deterministic from `seed`. Produces about `n * avg_degree / 2` unique edges,
/// no self-loops, no duplicate edges.
fn build_graph(n: usize, avg_degree: usize, seed: u64) -> Vec<Vec<usize>> {
    let target_edges = n * avg_degree / 2;
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut rng = Rng::new(seed);

    let mut added = 0usize;
    // Bound the attempts so we always terminate even if the graph saturates.
    let max_attempts = target_edges * 8 + 1000;
    let mut attempts = 0usize;
    while added < target_edges && attempts < max_attempts {
        attempts += 1;
        let a = rng.gen_range(n);
        let b = rng.gen_range(n);
        if insert_edge(&mut adj, a, b) {
            added += 1;
        }
    }
    adj
}

/// Parse `--graph <path>` (or `--graph=<path>`) from argv. Returns the path if present.
fn parse_graph_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        if a == "--graph" {
            return args.get(i + 1).cloned();
        } else if let Some(rest) = a.strip_prefix("--graph=") {
            return Some(rest.to_string());
        }
        i += 1;
    }
    None
}

/// Count common neighbors of u and v that are strictly greater than `min_exclusive`.
/// Both neighbor lists are sorted ascending. We only consider w > min_exclusive.
fn count_common_greater(a: &[usize], b: &[usize], min_exclusive: usize) -> u64 {
    let mut i = 0;
    let mut j = 0;
    let mut count = 0u64;
    while i < a.len() && j < b.len() {
        let x = a[i];
        let y = b[j];
        if x == y {
            if x > min_exclusive {
                count += 1;
            }
            i += 1;
            j += 1;
        } else if x < y {
            i += 1;
        } else {
            j += 1;
        }
    }
    count
}

/// Count triangles over the vertex range [lo, hi) as the outer-loop owner.
/// Ordering u < v < w ensures each triangle counted exactly once.
fn count_triangles_range(adj: &[Vec<usize>], lo: usize, hi: usize) -> u64 {
    let mut total = 0u64;
    for u in lo..hi {
        for &v in &adj[u] {
            if v <= u {
                continue; // enforce u < v
            }
            // common neighbors w with w > v  => u < v < w
            total += count_common_greater(&adj[u], &adj[v], v);
        }
    }
    total
}

fn main() {
    let (adj, source) = match parse_graph_arg() {
        Some(path) => {
            let adj = load_graph_from_file(&path);
            (adj, format!("file:{}", path))
        }
        None => (
            build_graph(N_VERTICES, AVG_DEGREE, SEED),
            "generated".to_string(),
        ),
    };

    let n_vertices = adj.len();
    let edge_count: usize = adj.iter().map(|nbrs| nbrs.len()).sum::<usize>() / 2;
    println!(
        "graph: source={}, n_vertices={}, edges={}, avg_degree~={:.2}",
        source,
        n_vertices,
        edge_count,
        if n_vertices == 0 {
            0.0
        } else {
            2.0 * edge_count as f64 / n_vertices as f64
        }
    );

    let total = count_triangles_range(&adj, 0, n_vertices);

    // Independent sanity check: recompute with a brute-force-ish alternate order
    // (iterate all v neighbors, no >u short circuit) then divide — verifies the
    // ordered count is self-consistent. Each triangle {u,v,w} is discovered from
    // its smallest vertex only in the ordered method; the check below counts the
    // same triangles via a different traversal and must match.
    let mut check = 0u64;
    for u in 0..n_vertices {
        for &v in &adj[u] {
            if v <= u {
                continue;
            }
            for &w in &adj[v] {
                if w <= v {
                    continue;
                }
                // is (u,w) an edge?
                if adj[u].binary_search(&w).is_ok() {
                    check += 1;
                }
            }
        }
    }
    assert_eq!(
        total, check,
        "serial self-check failed: ordered count {} != alternate traversal {}",
        total, check
    );

    println!("RESULT:{}", total);
}