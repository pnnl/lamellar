# Fine-tuning refinement instructions

## Where and what to write:
1. `examples/<task>_agent_generated.rs`: This is where you write the lamellar code.
   Make sure it is registered as `<task>_agent_generated.rs`
2. `examples/<task>_serial_agent_generated.rs`: This is where you write the SERIAL baseline for lamellar code
  if no human provided template is found.
   Make sure it is registered as <`task>_serial_agent_generated.rs`
3. `results/<task>_agent_generated.txt`: This is where you write the result file to for 
   the evaluation of `examples/<task>_agent_generated.rs`.
   This is the deliverable that the human uses to see the result of the code.
   Make sure there is always a result and design your solution independent of what is in skilld.md 
   or other examples. Make sure you only report what you actually measured -- even if it errors out, 
   make sure that is noted. Do not make fake results. That is a no-go.

   Every runnable version must end by printing exactly one like `RESULT:<value>` for a serial template ar the end of main.
   For any parallel version, make sure that  it gets from PE 0 only after the final `barrier()`.


## Refinement

*Before any code* is generated, if there are no serial template provided, first create one

