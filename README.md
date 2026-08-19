# repro: generating wasm debugging symbols is very slow (Rust)

The Rust compiler can be extremely slow when generating debugging information
for the WebAssembly target.

Nearly all of the time goes into LLVM's `WebAssembly Register Stackify` pass.
The slowdown appears only when full debug information is requested.

The generated code is not the problem. The same function compiles in a couple
of seconds once the `DBG_VALUE` records are gone.

This repository is a reproducer with about 40 lines.

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

The full-debuginfo wasm release build does not finish within 30 seconds on my
machine. It is sensitive to thermal throttling. Repeated runs take about 30 to
50 seconds.

## Controls

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

This is direct evidence that the cost is in handling debug values. The cost is
not in stackification itself.

## Confirmed root cause

The issue comes from the interaction between:

- `llvm/lib/Target/WebAssembly/WebAssemblyRegStackify.cpp`
- `llvm/lib/Target/WebAssembly/WebAssemblyDebugValueManager.cpp`

`WebAssemblyRegStackify` processes stackifiable definitions. It creates a
`WebAssemblyDebugValueManager` for many definitions. The manager's constructor,
`getSinkableDebugValues`, and `isInsertSamePlace` repeatedly walk the raw
`MachineBasicBlock` instruction list. They use the list to find or compare
`DBG_VALUE` records.

The problem grows during stackification:

- `sink()` leaves old `DBG_VALUE` instructions as undef tombstones.
- `cloneSink()` adds new `DBG_VALUE` instructions without removing old ones.
- A basic block's debug-instruction population grows during the pass. Each list
  walk crosses an increasing set of live records and tombstones.

This produces O(n^2)-like behavior in the number of `DBG_VALUE` records. It
explains why `WebAssembly Register Stackify` consumes about 86% of codegen
time.

It also explains why inlined `wrapping_*` calls slow the build. They increase
debug records without changing the generated code.

## Fix

[`llvm-wasm-debug-fix.patch`](llvm-wasm-debug-fix.patch) is a tested patch for
the LLVM source vendored by the affected Rust compiler. It preserves debug-value
ordering semantics while avoiding repeated expensive work:

- It bounds the `WebAssemblyDebugValueManager` constructor scan with the
  register's debug uses. It accounts for stale uses above the definition.
- It uses a bidirectional same-block walk to determine whether an insertion is
  a sink. It filters intervening records to relevant variables. It avoids
  unnecessary `DebugVariable` construction and hashing.
- It uses `SlotIndexes` for same-basic-block dominance checks. It skips
  unnumbered debug instructions without a `SlotIndexes` map lookup.

On the original benchmark, a Rust compiler rebuilt with this patch reduced the
`debug = 2` build from 50.56 seconds to 2.72 seconds. This is 18.6 times
faster.

At the LLVM level, `WebAssembly Register Stackify` fell from 45.9 seconds to
1.20 seconds. `WebAssembly Explicit Locals` fell from 6.5 seconds to 0.013
seconds.

### Apply the patch to a Rust source checkout

The patch changes LLVM source. It cannot modify an installed rustup toolchain.
Apply it to the `src/llvm-project` submodule of a compatible `rust-lang/rust`
checkout. Then rebuild Rust, such as a stage-2 compiler:

```sh
git clone https://github.com/rust-lang/rust.git
cd rust/src/llvm-project
git apply --check /absolute/path/to/llvm-wasm-debug-fix.patch
git apply /absolute/path/to/llvm-wasm-debug-fix.patch
cd ../..
./x build --stage 2 compiler/rustc library/std
```

Run `git apply` from `src/llvm-project`. The patch paths start at that
submodule's root.
