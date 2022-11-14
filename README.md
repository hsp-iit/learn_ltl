# Learn Linear Temporal Logic formulae

This repository provides a set of tools to passively learn Linear Temporal Logic formulae from positive and negative examples.

## Dependencies

Some of the tools are written in [Rust](https://www.rust-lang.org/),
which can be easily installed by using [rustup](https://rustup.rs/).
The code has been tested on Linux with Rust 1.65, release toolchain,
but it should work on any OS supported by Rust and with any Rust version supporting the 2021 edition (versions >=1.56).

## Building

Inside the main folder, run:

```
$ cargo build --release --all
```

You can safely ignore any compilation warning.

## Solver

The `solver` tool runs the learning algoritm on a sample to learn a formula consistent with it.

If you have a sample in `.ron` or `.json` format, you can run the solver on it with the following command:

```
$ cargo run --release --bin solver -- <SAMPLE>
```

Alternatively, you can invoke directly the compiled binary, which by default is at `target/release/solver`:

```
$ target/release/solver <SAMPLE>
```

There is also a help file:

```
$ target/release/solver --help
Search for a formula consistent with the given sample. Supported file types: ron, json

Usage: solver <SAMPLE>

Arguments:
  <SAMPLE>  

Options:
  -h, --help  Print help information
```

To discard a variable from a sample, open the sample with a text editor,
find the name of the variable you want to ignore and add `~` at the beginning of the name.
For example, to discard `X_d1dd`, rename it to `~X_d1dd`.

## Experiments

The sample used for the experiments was obtained by:

- Running the simulator from <https://github.com/SCOPE-ROBMOSYS/RAL2022-experiments> multiple times.
- Turn the logs thus obtained into a sample using <https://github.com/piquet8/TraceGenerator_Script>.
- (Optional) name the variables suitably.

The logs and the sample can be found into the `R1experiments` folder.

Then, a solution to the passive learning problem on this sample can be found with:

```
cargo run --release --bin solver -- R1experiments/sample.json
```

For the second experiment, discard the variables `X_1d1dd` and `Y_1ddd` as explained above,
i.e., by prefixing a `~` to their name.
