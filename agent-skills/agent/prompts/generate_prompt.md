# Generate prompt

## Purpose 

This is the main interactive entry point for the agent that a user invokes and the agent must ask the folloiwng questions
below one at a time and wait for responses. Make sure to not generate any code before user responses are obtained

## Required context -- read this silently
- skills.md
- fune-tuning.md
- lamellar-runtime_examples.md
- Cargo.toml

## Questions that the agent must ask the user

** Tasks (REQUIRED)
1. What should the code do? Write 1-2 sentences.
2. What should the final result the code should print?
3. What should the name of the `<task>` be? 

** Input files([REQUIRED, this also includes "None")
4. Is there a serial template of the code that is to be parallelized?
* If the user response is yes, ask user for input file path and output would go to `examples/<task>_agent_generated.rs`.
* If the user response is no or none, ask a follow up question: do you want a serial baseline first?[default:yes]:
    - If yes which is recommended then, write to `examples/<task>_serial_agent_generated.rs`, run it 
      and show you the source code and RESULT and wait for user response for approval before any parallel code. 
    - If no, go straight to parallel version, build independent checks into it. Make sure the results file will sat rhar the baseline=none 
      since there were not serial code 
  In `fine-tuning.md`, it is said to create a sample first for a serial baseline to 

** Execution parameters (OPTIONAL)
5. What should be the PE count for evaluation? Set default to 2.
6. How many nodes? Set default to 1.
7. How do you want to scale the data? Set default to small

## After collecting answers, agent needs to
1. Run `skills.md` procedure
2. Generate code to the path that user agrees to 
3. DO NOT add dependencies in `Cargo.toml`, the only thing an agent is allowed to add and WITH USER APPROVAL is `[[example]]`, but if USER needs other dependencies then the agent can suggest it and print it for the USER to decide and add. You CANNOT make that decision without user approval.
3. Run `skills.md` verification cheklist one by one and result for each checklist explicitly.
   This includes but not limited to imports match use case, handles driven by exactly one of .block()/.spawn()/.await,
   make sure there is no .lock() inside async, collect result collection types, barrier before collective reads.
4. ASK for approval for the generated code.
5. Print the `[[example]]` blocks for `Cargo.toml` that the user MUST add to `Cargo.toml` -- the ones that apply to this <task>'s path:
  ```toml
  [[example]]
  name = "<task>_agent_generated"
  path = "agent/examples/<task>_agent_generated.rs"
  ```
6. Print Evaluation commands in `fine-tuning.md`.
  ```bash
  cargo build --release --example
  cargo run --release --example <task>_agent_generated.rs # for 1 PE
  
  # multi-node for when an allocation is available
  salloc -N 2 --exclusive
  cargo run --release --example <task> _agent_generated.rs -- -- --nodes 2 --pes 2 --lamellae ucx # for 2 PE
  ```

  Run these and record time in seconds.
  State clearly what run was conducted. Remember that a 1 PE run does NOT mean that it validates multi-PE correctness so do NOT make that assumption.

7. Write results to `agent/results/<task>_agent_generated.text` as stated in `fine-tuning.md` with summary line, fixed, table, completed checklist and notes section. 
Make sure to end the response by reproducing the table. ONLY include measured values -- NEVER speculate any measured values if you can not get it.


## Failure handling
* If there are conflicting answers, for example, a named serial template does not exist then STOP and ASK the user -- do NOT guess and speculate.
* If there is a task name that violates naming rules or collides with existing target, then flag it and propose an alternative --NEVER OVERWRITE WITHOUT PERMISSION.
* If there are an API that you want does not exist in this version or your memory does not agree with lamellar's then lamellar's version wins and use a closlely verified pattern
* If when running step 2 `After collecting answers, agent needs to` gives Unknown type -- ask the user for type definition -- so not assume serializability.