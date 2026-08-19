# repro: generating wasm debugging symbols is very slow (Rust)

The Rust compiler can be extremely slow when generating debugging information
for the WebAssembly target.

Nearly all of the time goes into LLVM's `WebAssembly Register Stackify` pass.
The slowdown appears only when full debug information is requested.

See [Why compiling Rust to WebAssembly is slow](https://00f.net/2026/08/19/why-compiling-rust-to-webassembly-is-slow/) for more information.

## Reproduce

```sh
rustup target add wasm32-unknown-unknown
cargo clean
cargo build --release --target=wasm32-unknown-unknown
```

In `cargo.toml`:

```toml
[profile.release]
codegen-units = 1
incremental = false
panic = "abort"
debug = 2
```

## Controls

On my machine:

```text
+----------------------------------------+--------------+
| Build                                  | Time         |
+----------------------------------------+--------------+
| wasm32-unknown-unknown, debug = 2      | 30 s to 50 s |
| wasm32-unknown-unknown, debug = 1      | 2.9 s        |
| wasm32-unknown-unknown, no debug info  | 1.3 s        |
| host target, debug = 2                 | 1.1 s        |
+----------------------------------------+--------------+
```

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-unknown-unknown
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-unknown-unknown
cargo build --release
```

This is not merely the normal cost of debug information or a large function.
Both the WebAssembly backend and full debug information trigger an LLVM
performance bug.

Until a compiler containing the fix is available, `debug = 1` is a usable
workaround. The LLVM patch in this repository fixes the pathological cost.

## Where the time goes

```sh
RUSTFLAGS='-Cllvm-args=--debug-pass=Executions' \
  cargo build --release --target=wasm32-unknown-unknown
```

Each traced pass is timestamped. Measure the delay between one pass entry and
the next to attribute the stall.

The crate generates a single function, `repro`. Summing time for each pass
produces the following results.

```text
+---------------------------------+-----------+-----------+
| Pass                            | debug = 2 | debug = 1 |
+---------------------------------+-----------+-----------+
| WebAssembly Register Stackify   |  26.19 s  |   2.30 s  |
| WebAssembly Explicit Locals     |   3.58 s  |   0.25 s  |
| WebAssembly Instruction Sel.    |   0.17 s  |   0.19 s  |
| everything else                 |   0.37 s  |   0.04 s  |
+---------------------------------+-----------+-----------+
| total                           |  30.31 s  |   2.78 s  |
+---------------------------------+-----------+-----------+
```

Instruction selection costs the same in both columns. This result is expected,
because the input machine code is the same.

Stackify is roughly eleven times slower. It accounts for 86% of total codegen
time.

## It scales with the number of debug records, not with the amount of code

The reproducer uses `wrapping_add` and `wrapping_mul` instead of `+` and `*`.

Overflow checks are off in a release profile. Therefore, the two spellings have
the same operation and generate the same code. With `debug = false`, the
emitted LLVM IR is identical at 3,809 lines for either spelling.

Only the debug information differs. Each `wrapping_*` call is a separate
inlined function. Its parameters receive their own records.

```text
+-------------------------------+------------+---------+
| Written as                    | #dbg_value | Build   |
+-------------------------------+------------+---------+
| `wrapping_add`/`wrapping_mul` |     30 612 |    55 s |
| `+` and `*`                   |     13 101 |     9 s |
+-------------------------------+------------+---------+
```

The machine code is the same. There are 2.3 times as many debug records and
roughly six times the build time.

### Fix

[The patch](llvm-wasm-debug-fix.patch) is for the LLVM fork included in the current Rust version.

Apply it to the `src/llvm-project` submodule of a `rust-lang/rust` checkout.

Then rebuild Rust.