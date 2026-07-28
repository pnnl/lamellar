This provides a pipeline for having an AI agent write Lamellar code from serial templates or from scratch with measured results.


# Building environment
```
cargo build --release
```
## Compile examples

```
cargo build --release --examples
```
## Run examples
```
cargo run --release --example <example-name>
```

```
salloc -N 2 --exclusive
cargo run --release --example <example-name> -- -- --nodes 2 --pes 4 --lamellae ucx

The layout:

```
/lamellar/agent-skills/
├── agent/
    ├── prompts/
        ├── generate_prompt.md
    ├── examples/
    ├── results/
    ├── skills.md
    ├── fine-tuning.md
    ├── lamellar-runtime_examples.md


```