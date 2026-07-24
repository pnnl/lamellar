# Generate prompt

## Purpose 

This is the main interactive entry point for the agent that a user invokes and the agent must ask the folloiwng questions
below one at a time and wait for responses. Make sure to not generate any code before user responses are obtained

## Required context -- read this silently
- skills.md
- lamellar-runtime_examples.md
- Cargo.toml

## Questions that the agent must ask the user

** Tasks [REQUIRED]
1. What should the code do? Write 1-2 sentences.
2. What should the final result the code should print?
3. What should the name of the `<task>` be? 

** Input files [REQUIRED, this also includes "None"]
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
2. Generate code
3. Run `skills.md` verification cheklist one by one and result for each checklist

