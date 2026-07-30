# Fine-tuning refinement instructions

## Where and what to write:
* `examples/<task>_agent_generated.rs`: This is where you write the lamellar code.
   Make sure it is registered as `<task>_agent_generated.rs`
* `examples/<task>_serial_agent_generated.rs`: This is where you write the SERIAL baseline for lamellar code
  if no human provided template is found.
   Make sure it is registered as <`task>_serial_agent_generated.rs`
* `examples/revisions/<task>_agent_generated_rev_<n>.rs`: These are archived copies of prior revisions of AI versions, one per revision number.
   These are not registered in Cargo.toml -- only `<task>_agent_generated.rs` is the build target.
   To re-run an old revision version, copy it over `<task>_agent_generated.rs` or temporarily register it.
* `results/<task>_agent_generated.md`: This is where you write the result file to for 
   the evaluation of `examples/<task>_agent_generated.rs`.
   This is the deliverable that the human uses to see the result of the code.
   Make sure there is always exactly one result file per task. 
   Write and update this after every evaluation run as stated in Evaluation section. 
   It's top section always reflects the current/latest version -- that is what the human user compares.
   Design your solution independent of what is in `skills.md` 
   or other examples. Make sure you only report what you actually measured -- even if it errors out, 
   make sure that is noted. Do not make fake results. That is a no-go.

   Every runnable version must end by printing exactly one like `RESULT:<value>` for a serial template ar the end of main.
   For any parallel version, make sure that  it gets from PE 0 only after the final `barrier()`.


## Refinement

*Before any code* is generated, if there are no serial template provided,  ask the user whether they want a seial baseline [Default: yes].
If yes, then create, run, and get approval on the serial baseline. If they decline, go to "User declined serial baseline" section
Then run skills.md


## Evaluation
This is what passing would mean. This is the evaluation test steps that needs to be done (Note that it scales gradually).

A PE 1 run proves that the logic compiles and runs but it doesn't say anything about distribution.
Multi-PE bugs appear at N>=2 with global index errors, collective deadlocks, darc-locality errors, etc. 
So follow the steps:
|Step | Command ( inside `salloc -N 1 --exclusive` for steps 1-4)| What it proves|
|-----|---------|---------------|
|1|`cargo build --release --examples`  | It compiles | 
|2|time `cargo build --release --example <task>_serial_agent_generated -- -- --nodes 1 --pes 1 --lamellae shmem`| Logic is okay at 1 PE | 
|3|time `cargo build --release --example <task>_serial_agent_generated -- -- --nodes 1 --pes 2 --lamellae shmem`| Distribution is okay at 2 PE | 
|4|time `cargo build --release --example <task>_serial_agent_generated -- -- --nodes 1 --pes 4 --lamellae shmem`| Scaling behavior at 4 PE | 
|5| `salloc -N 2 --exclusive` then `cargo build --release --example <task>_serial_agent_generated -- -- --nodes 2 --pes 2 --lamellae ucx`| This is the multi node behavior / adjust --nodes and --pes to allocation  | 


Consider the code generated passes when the following holds:




## Mandatory report required: write to `results/<task>_agent_generated.md`
After every evaluation run, you MUST write (or overwrite) `results/<task>_agent_generated.md` and reproduce its table at the end of your respoonses.
A user (human) will look at this file against expert measurements outside this repo, so the format is fixed -- no prose tables, no ad hoc layouts, no extra or renames columns:

```markdown
# Results: <task>

SUMMARY: task=<task>, rev=<n>, baseline = <provided|generataed|none>, results=<value>, status_1pe=<ok|..>, status_npe=<okay|untested|..>, time_1pe_in_sec=<sec>, time_npe_in_sec=<sec>, n_pes=<N>, threads=<T>, speedup_vs_serial=<x|N/A>, verdit=<pass|pass-no-baseline|fail-reason>

Generated: <datetime>, lamellar <version from Cargo.toml>, Backend <local/shmem/ucx>

| Task | Version | Rev | 1 PE | N PEs | Time 1 PE | Time N PEs | Speedup vs serial | RESULT | Verdict |
|------|---------|-----|------|-------|-----------|------------|-------------------|--------|---------|
| <task> | <template (serial) OR serial_agent_generated (agent-generated) OR none (user-declined)> | — | ok | n/a | <t>s | n/a | 1.00x | `<v>` | baseline |
| <task> | agent_generated | <n> | <status> | <status> | <t>s | <t>s | <x>x | `<v>` | <verdict> | 


## Checklist
Use `skill.md` verification checklist and check it item nby item. Then list it here with pass/fail for each. 

## Notes
<revision history one-liners; any caveat, e.g. "data too small to amortize
AM overhead" or a proposed float tolerance>

## Summary
<one SUMMARY: line per prior revision, oldest first — appended, never
edited or deleted>

```

Rules:
- The `SUMMARY:` line is one line, key=value, exactly those keys. 
It exists so the human user can grep/join results across tasks and revisions
against their external expert data. NEVER omit it or reformat it.
- `baseline=` records provenance and is never misreported: 
   - `provided` if and only if the baseline is a human file in `templates/`
   - `generated` if and only if it is your `examples/<task>_serial_agent_generated.rs`
   - `none` if and only if the human user explicitly declined a baseline
- **Report only measured values.** NEVER fabricate, estimate, or speculate
about expert numbers — or any numbers — anywhere in the file.
- **Verdict**: `pass` only if RESULT matches the serial baseline at both PE
counts AND is PE-invariant. Otherwise the specific failure are:
`wrong-vs-template`, `fails-at-N-pes`, `not-pe-invariant`, `build-fail`,
`timeout`. With `baseline=none` the ceiling is `pass-no-baseline`
(PE-invariant + internal checks pass) — NEVER report a plain `pass`
without a baseline; the two must stay distinguishable.
- **Rev** is the revision number from the top-of-file comment.
Convergence is fully trackable and auditable: each revision's WHY lives
as a one-liner in Notes, its measured WHAT (the old `SUMMARY:` line)
lives in History, and its CODE lives at
`examples/revisions/<task>_agent_generated_rev<n>.rs` — never delete any of the three.
- One file per task, written by YOU from your own measured runs of the steps
steps —- run the baseline for the baseline row, run your version for the
ai row, compute the speedup from the measured times.


## User declined serial baseline
This is user choice, so make sure to follow their choice. Do compensate as much as possible honestly (do not make things up):
1. Go directly to `skills.md` and create a parallel version as expected in `examples/<task>_agent_generated.rs`-- no serial file is created.
2. Make sure to build every possible independent checks into the parallel code: with `assert!` and make sure that is none are possible, it says it in the notes of the code and get approval from user.
3. Run the Evaluation section.
4. In `results/<task>_agent_generated.md`, make sure there base `baseline=none` where the baseline table row reads `none (user declined)` with all cells comtaing N/A.
   And verdict at best being `pass-no-baseline`. There must state that user declined the serial baseline created and list of which internal checks were passed.
5. If user changes their mind later and wants to create a baseline and re evaluate then the earlier pass does not carry over with the re-run and re-report.

