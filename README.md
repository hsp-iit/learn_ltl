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

If you have a sample in `.ron` format, you can run the solver on it with the following command:

```
$ cargo run --release --bin solver -- --sample <path-to-sample> --format ron
```

Similarly, if you have a sample in `.json` format, you can run the solver on it with the following command:

```
$ cargo run --release --bin solver -- --sample <path-to-sample> --format json
```

Alternatively, you can invoke directly the compiled binary, which by default is at `target/release/solver`:

```
$ <path-to-solver> --sample <path-to-sample> --format ron
```
There is also a help file:

```
$ target/release/solver --help
Search for a formula consistent with the given sample

Usage: solver --sample <SAMPLE> --format <FORMAT>

Options:
  -s, --sample <SAMPLE>  Filename of the target sample
  -f, --format <FORMAT>  File format of the target sample
  -h, --help             Print help information
```

## Generating samples

To generate samples from a log file as obtained from [](),
use the `trace_generator` tool

## Experiments

xxx
