// rev 1: initial parallel Lamellar implementation of exact triangle counting.
//
// ============================================================================
// skills.md §1 procedure (Inventory / Classify / Plan) — approved before coding
// ============================================================================
// INVENTORY (values crossing a PE boundary):
//   * Inputs sent to every PE (inside the AM): either a graph-file path
//     (Option<String>) OR generation params (n_vertices, avg_degree, seed), plus
//     num_pes. All scalars/strings.
//   * Result returned per PE -> PE 0: partial_count: u64 (this PE's subtotal).
//   * The adjacency graph itself does NOT cross the boundary — each PE rebuilds
//     the identical graph locally (deterministic splitmix64) or loads the same
//     file, so no large/derived structure is serialized.
// CLASSIFY:
//   * graph_path: Option<String>, n_vertices/avg_degree: usize, seed: u64,
//     num_pes: usize  -> SERIALIZABLE (plain data).
//   * partial_count: u64 -> SERIALIZABLE.
//   * The graph Vec<Vec<usize>> would be EXPENSIVE to ship -> intentionally NOT
//     sent; rebuilt/loaded per-PE. No NON-SERIALIZABLE / UNKNOWN values.
// PLAN data flow:
//   * AM struct TriangleCountAM { graph_path, n_vertices, avg_degree, seed,
//     num_pes } with #[AmData(Debug, Clone)].
//   * Inside exec: (1) build/load the full graph locally; (2) this PE owns a
//     contiguous block of the outer-loop vertex range 0..n via
//     block_bounds(pe, num_pes, n); (3) run the ordered intersection count
//     (u < v < w) only for u in this PE's block — each triangle is counted once,
//     by the PE owning its smallest vertex u; (4) return partial_count: u64.
// PLAN launch + gather (skills §7):
//   * PE 0 calls world.exec_am_all(am).block() -> Vec<u64> gathered ON PE 0,
//     indexed by PE id. Sum -> global total. barrier(), then PE 0 prints
//     RESULT:<total> AFTER the barrier (fine-tuning.md requirement).
// Launch API: exec_am_all -> Vec<u64> (NOT typed_am_group!), per skills §6.
//
// Graph source (same two modes as the serial baseline):
//   * `--graph <path>` : whitespace edge list (`u v` per line, `#` comments;
//     self-loops and duplicate edges dropped).
//   * (no flag)        : deterministic splitmix64 generator; medium scale below.
//   NOTE: Lamellar consumes its own `-- --nodes/--pes/--lamellae` flags after a
//   `--`. Pass the graph file BEFORE that separator, e.g.
//     cargo run --release --example triangle_count_agent_generated -- --graph g.el
//   (a single `--` hands `--graph g.el` to the program).
// ============================================================================

use lamellar::active_messaging::prelude::*;

// ---- data-scale knobs (medium) — must match the serial baseline ----
const N_VERTICES: usize = 2_000;
const AVG_DEGREE: usize = 10;
const SEED: u64 = 0x1234_5678_9abc_def0;

/// Deterministic splitmix64 PRNG (identical to the serial baseline).
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
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
        Ok(_) => return false,
        Err(pos) => adj[lo].insert(pos, hi),
    }
    match adj[hi].binary_search(&lo) {
        Ok(_) => {}
        Err(pos) => adj[hi].insert(pos, lo),
    }
    true
}

/// Load an undirected graph from a whitespace edge-list file (identical parsing
/// to the serial baseline). Vertex count = max id + 1.
fn load_graph_from_file(path: &str) -> Vec<Vec<usize>> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read graph file '{}': {}", path, e));

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
            _ => continue,
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

/// Deterministic random graph (identical to the serial baseline).
fn build_graph(n: usize, avg_degree: usize, seed: u64) -> Vec<Vec<usize>> {
    let target_edges = n * avg_degree / 2;
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut rng = Rng::new(seed);

    let mut added = 0usize;
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

/// Contiguous block [lo, hi) of 0..n owned by `pe` out of `num_pes`.
/// Blocks partition 0..n with no gaps or overlaps.
fn block_bounds(pe: usize, num_pes: usize, n: usize) -> (usize, usize) {
    let base = n / num_pes;
    let rem = n % num_pes;
    // first `rem` PEs get one extra element
    let lo = pe * base + pe.min(rem);
    let extra = if pe < rem { 1 } else { 0 };
    let hi = lo + base + extra;
    (lo, hi)
}

/// Count common neighbors of u and v that are strictly greater than `min_exclusive`.
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

/// Count triangles whose smallest vertex u is in [lo, hi). Ordering u < v < w
/// ensures each triangle counted exactly once, by the owner of its smallest vertex.
fn count_triangles_range(adj: &[Vec<usize>], lo: usize, hi: usize) -> u64 {
    let mut total = 0u64;
    for u in lo..hi {
        for &v in &adj[u] {
            if v <= u {
                continue;
            }
            total += count_common_greater(&adj[u], &adj[v], v);
        }
    }
    total
}

// ---- Active Message: each PE computes the triangle subtotal for its vertex block ----
#[AmData(Debug, Clone)]
struct TriangleCountAM {
    graph_path: Option<String>,
    n_vertices: usize,
    avg_degree: usize,
    seed: u64,
    num_pes: usize,
}

#[lamellar::am]
impl LamellarAM for TriangleCountAM {
    async fn exec(self) -> u64 {
        let pe = lamellar::current_pe;

        // Each PE independently builds/loads the IDENTICAL graph (no graph is
        // shipped across the boundary).
        let adj = match &self.graph_path {
            Some(path) => load_graph_from_file(path),
            None => build_graph(self.n_vertices, self.avg_degree, self.seed),
        };
        let n = adj.len();

        // This PE owns a contiguous block of the outer-loop vertex range.
        let (lo, hi) = block_bounds(pe, self.num_pes, n);

        // Internal check: block is within range.
        assert!(lo <= hi && hi <= n, "block bounds out of range");

        count_triangles_range(&adj, lo, hi)
    }
}

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

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();

    let graph_path = parse_graph_arg();

    world.barrier();

    // Only PE 0 launches exec_am_all, so the Vec<u64> of partials is gathered ON PE 0.
    let am = TriangleCountAM {
        graph_path: graph_path.clone(),
        n_vertices: N_VERTICES,
        avg_degree: AVG_DEGREE,
        seed: SEED,
        num_pes,
    };
    let partials: Vec<u64> = world.exec_am_all(am).block();

    // Independent correctness checks (run on PE 0, where the full Vec landed).
    // 1. exec_am_all returns one entry per PE.
    assert_eq!(
        partials.len(),
        num_pes,
        "expected {} partials, got {}",
        num_pes,
        partials.len()
    );
    // 2. Block partition of 0..n is complete and non-overlapping. Rebuild n the
    //    same way the AM did to validate the bounds cover every vertex exactly once.
    let n_local = match &graph_path {
        Some(path) => load_graph_from_file(path).len(),
        None => N_VERTICES,
    };
    let mut covered = 0usize;
    let mut prev_hi = 0usize;
    for pe in 0..num_pes {
        let (lo, hi) = block_bounds(pe, num_pes, n_local);
        assert_eq!(lo, prev_hi, "block gap/overlap at pe {}", pe);
        covered += hi - lo;
        prev_hi = hi;
    }
    assert_eq!(prev_hi, n_local, "blocks do not end at n");
    assert_eq!(covered, n_local, "block coverage != n");

    let total: u64 = partials.iter().sum();

    // Barrier before PE 0 reads/prints the collectively-produced result.
    world.barrier();

    if my_pe == 0 {
        println!(
            "graph: source={}, n_vertices={}, num_pes={}, partials={:?}",
            match &graph_path {
                Some(p) => format!("file:{}", p),
                None => "generated".to_string(),
            },
            n_local,
            num_pes,
            partials
        );
        println!("RESULT:{}", total);
    }
    world.barrier();
}
