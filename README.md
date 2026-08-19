# repro: generating wasm debugging symbols is very slow (Rust)

The Rust compiler can be extremely slow to generate debugging information for the WebAssembly target.

Nearly all of the time goes into LLVM's `WebAssembly Register Stackify` pass, and the slowdown only appears when full debug info is requested.

The generated code is not the problem: the same function compiles in a couple of seconds once the `DBG_VALUE` records are gone.

This repository is a ~40 lines reproducer.

## Reproduce

```sh
rustup target add wasm32-unknown-unknown
cargo clean
cargo build --release --target=wasm32-unknown-unknown
```

The manifest sets:

```toml
[profile.release]
codegen-units = 1
incremental = false
panic = "abort"
debug = 2
```

Observed with:

```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
host: aarch64-apple-darwin
LLVM version: 22.1.6
cargo 1.97.1 (c980f4866 2026-06-30)
```

The full-debuginfo wasm release build does not finish within 30 seconds on my machine, and it is quite sensitive to thermal throttling, so repeated runs range from about 30 to 50 seconds.

## Controls

+----------------------------------------+--------------+
| Build                                  | Time         |
+----------------------------------------+--------------+
| wasm32-unknown-unknown, debug = 2      | 30 s to 50 s |
| wasm32-unknown-unknown, debug = 1      | 2.9 s        |
| wasm32-unknown-unknown, no debug info  | 1.3 s        |
| host target, debug = 2                 | 1.1 s        |
+----------------------------------------+--------------+

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-unknown-unknown
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-unknown-unknown
cargo build --release
```

This is not merely the normal cost of debug information or a large function. It requires both the WebAssembly backend and full debug information to trigger an LLVM performance bug.

Until a compiler containing the fix is available, `debug = 1` is a usable workaround. The LLVM patch included in this repository fixes the pathological part of the cost.

## Where the time goes

```sh
RUSTFLAGS='-Cllvm-args=--debug-pass=Executions' \
  cargo build --release --target=wasm32-unknown-unknown
```

Every traced pass is timestamped, so the stall is easy to attribute: measure the delay between each pass entry and the next one.

The crate generates a single function, `repro`, and summing per pass gives this.

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

Instruction selection costs exactly the same in both columns, which is the expected result since the input machine code is the same.

Stackify is roughly eleven times slower, and it is holding 86% of the whole codegen time.

## It scales with the number of debug records, not with the amount of code

The reproducer multiplies through `wrapping_add` and `wrapping_mul` rather than through `+` and `*`.

Overflow checks are already off in a release profile, so the two spellings are the same operation, and they do produce exactly the same code: built with `debug = false`, the emitted LLVM IR is identical, 3809 lines either way.

Only the debug information differs, because each `wrapping_*` is a separate inlined function whose parameters get records of their own.

+-------------------------------+------------+---------+
| Written as                    | #dbg_value | Build   |
+-------------------------------+------------+---------+
| `wrapping_add`/`wrapping_mul` |     30 612 |    55 s |
| `+` and `*`                   |     13 101 |     9 s |
+-------------------------------+------------+---------+

Same machine code, 2.3 times the debug records, and roughly six times the build time.

This looks like the most direct evidence that the cost is in handling the debug values, and not in the stackification itself.

## Confirmed root cause

The issue is in the interaction between:

- `llvm/lib/Target/WebAssembly/WebAssemblyRegStackify.cpp`
- `llvm/lib/Target/WebAssembly/WebAssemblyDebugValueManager.cpp`

`WebAssemblyRegStackify` processes stackifiable definitions and creates a
`WebAssemblyDebugValueManager` for many of them. The manager's constructor,
`getSinkableDebugValues`, and `isInsertSamePlace` repeatedly walk the raw
`MachineBasicBlock` instruction list to find or compare `DBG_VALUE`s.

The problem is amplified during stackification:

- `sink()` leaves the old `DBG_VALUE` instructions in place as undef
  tombstones.
- `cloneSink()` adds new `DBG_VALUE`s without removing old ones.
- Consequently, a basic block's debug-instruction population grows while the
  pass is running, and each repeated list walk crosses an ever larger set of
  live records and tombstones.

This produces O(n²)-like behavior in the number of `DBG_VALUE`s. It explains
both why `WebAssembly Register Stackify` consumes about 86% of code-generation
time and why changing the spelling from `+`/`*` to inlined `wrapping_*` calls
(which increases debug records without changing the generated code) makes the
build dramatically slower.

## Fix

[`llvm-wasm-debug-fix.patch`](llvm-wasm-debug-fix.patch) is a tested patch for
the LLVM source vendored by the affected Rust compiler. It keeps the debug-value
ordering semantics but avoids the expensive repeated work:

- bounds the `WebAssemblyDebugValueManager` constructor scan using the
  register's debug uses, while accounting for stale uses above the definition;
- uses a bidirectional same-block walk when deciding whether an insertion is a
  sink, filters intervening records to the relevant variables, and avoids their
  unnecessary `DebugVariable` construction and hashing;
- makes same-basic-block dominance checks use `SlotIndexes`, and skips
  unnumbered debug instructions in `SlotIndexes` without a map lookup.

On the original benchmark, using a Rust compiler rebuilt with this patch changed
the `debug = 2` build from 50.56 s to 2.72 s (18.6x faster). At the LLVM level,
`WebAssembly Register Stackify` fell from 45.9 s to 1.20 s and
`WebAssembly Explicit Locals` from 6.5 s to 0.013 s. The remaining debug-info
cost is ordinary processing and emission of the large number of debug records,
not this quadratic behavior.

The patched LLVM produced byte-identical WebAssembly assembly on this repro at
`-O0`, `-O2`, and `-O3`, and passed all 329 WebAssembly CodeGen/DebugInfo LLVM
tests in an assertions-enabled build.

### Apply the patch to a Rust source checkout

The patch changes LLVM source, so it cannot modify an already-installed rustup
toolchain. Apply it to the `src/llvm-project` submodule of a compatible
`rust-lang/rust` checkout and rebuild Rust (for example, a stage-2 compiler):

```sh
git clone https://github.com/rust-lang/rust.git
cd rust/src/llvm-project
git apply --check /absolute/path/to/llvm-wasm-debug-fix.patch
git apply /absolute/path/to/llvm-wasm-debug-fix.patch
cd ../..
./x build --stage 2 compiler/rustc library/std
```

Run `git apply` from `src/llvm-project`, because the patch paths start at that
submodule's root. The patch was developed against Rust's LLVM 22-era vendored
tree and may need rebasing for newer LLVM revisions. Rebuilding a dirty
`src/llvm-project` checkout causes Rust's bootstrap to build and use the local
LLVM rather than an unchanged downloaded CI LLVM.
