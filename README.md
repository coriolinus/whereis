# `whereis` that crate?

When working in large Rust codebases, one occasionally encounters repositories containing dozens
or hundreds of individual crates. Rust knows where they all are, due to the definitions in
`Cargo.toml`, but it's not always so obvious to the developer. This tool makes it easy to locate the
actual path of a crate.

## Usage Examples

Produce the path to a crate in the workspace:

```bash
substrate$ cargo whereis --relative sp-offchain
primitives/offchain
```

Produce the URL of a dependency:

```bash
substrate$ cargo whereis --url lazy_static
https://crates.io/crates/lazy_static
```

Produce either a local filesystem path or a URL if you're not sure:

```bash
substrate$ cargo whereis --force parity-scale-codec
https://crates.io/crates/parity-scale-codec
```
