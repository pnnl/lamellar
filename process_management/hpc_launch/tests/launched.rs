//! Compile (and, under a launcher, run) coverage for `#[hpc_launch::test]`.

#[hpc_launch::test]
fn launched_test_runs_inside_launcher() {
    assert!(
        std::env::var("PRTE_LAUNCHED").is_ok() || std::env::var("SLURM_LOCALID").is_ok(),
        "the launched copy of this test must run inside the launcher"
    );
}
