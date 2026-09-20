# Lox

A learning project following Robert Nystrom's
[Crafting Interpreters](https://craftinginterpreters.com/), in two stages:

1. **Rust:** the tree-walk interpreter from Part II, in place of Java.
2. **Zig:** the bytecode compiler and virtual machine from Part III, in place of C.

The aim is to understand the implementation and learn to reason about language
design. [AGENTS.md](AGENTS.md) asks LLM assistants to teach through questions,
progressive hints, and small experiments, with direct explanations or code when
requested.

## Development environment

Install Nix with flakes and `nix-command` enabled, then enter the development
shell from this repository:

```sh
nix develop
```

The default shell provides both toolchains:

- Rust: `rustc`, Cargo, rustfmt, Clippy, rust-analyzer, and standard-library sources.
- Zig: `zig` and the ZLS language server.
- Nix: `nixfmt`, also available through `nix fmt -- flake.nix`.

For just one stage, use `nix develop .#rust` or `nix develop .#zig`. Shells are
defined for Apple Silicon macOS, and ARM64 and x86-64 Linux. The checked-in
`flake.lock` pins the package set, so toolchain updates are deliberate. Run
`nix flake update` when you want to update it, then recheck the project.

If you already use direnv with Nix support, the included `.envrc` can load the
default shell automatically after `direnv allow`. Otherwise, `nix develop` is
sufficient.

## Starting the book

This repository starts with tooling and learning guidelines. Create the language
projects as you reach each stage; no interpreter or compiler is implemented yet.
Suggested directories are `rust/` for the tree-walk interpreter and `zig/` for the
bytecode VM.

When ready to begin the Rust implementation:

```sh
nix develop .#rust
cargo new --bin --vcs none --name rlox rust
cd rust
cargo run
```

When you reach Part III, start the Zig project from the repository root:

```sh
nix develop .#zig
mkdir zig
cd zig
zig init
zig build
```

Keep the generated application lockfiles in Git when they are introduced.

## Environment checks

```sh
nix flake check
nix develop --command rustc --version
nix develop --command cargo --version
nix develop --command zig version
nix develop --command zls --version
```

The [Nix manual](https://nix.dev/manual/nix/stable/command-ref/new-cli/nix3-develop.html)
describes development shells and running individual commands with `--command`.
