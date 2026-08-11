# repro: generating wasm debugging symbols is very slow (Rust)

The Rust compiler can be extremely slow to generate debugging information for the WebAssembly target.

Nearly all of the time goes into a single LLVM pass, `WebAssembly Register Stackify`, and the slowdown only appears when full debug info is requested.

The machine code itself is not the problem: the very same function compiles in a couple of seconds once the `DBG_VALUE` records are gone.

This repository is a ~50 lines reproducer in a single file.

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
| wasm32-unknown-unknown, debug = 1      | 4.6 s        |
| wasm32-unknown-unknown, no debug info  | 1.8 s        |
| host target, debug = 2                 | 1.5 s        |
+----------------------------------------+--------------+

```sh
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-unknown-unknown
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-unknown-unknown
cargo build --release
```

So this is neither "debug info is expensive" nor "this code is large". It takes both the WebAssembly backend and full debug info to trigger it.

`debug = 1` is a usable workaround, but the root cause should be fixed.

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
| WebAssembly Register Stackify   |  30.01 s  |   2.06 s  |
| WebAssembly Explicit Locals     |   4.26 s  |   0.23 s  |
| WebAssembly Instruction Sel.    |   0.18 s  |   0.18 s  |
| everything else                 |   0.39 s  |   0.04 s  |
+---------------------------------+-----------+-----------+
| total                           |  34.85 s  |   2.50 s  |
+---------------------------------+-----------+-----------+

Instruction selection costs exactly the same in both columns, which is the expected result since the input machine code is the same.

Stackify is roughly fifteen times slower, and it is holding 86% of the whole codegen time.

## Likely LLVM hot spot

The likely hot spot is the interaction between:

- `llvm/lib/Target/WebAssembly/WebAssemblyRegStackify.cpp`
- `llvm/lib/Target/WebAssembly/WebAssemblyDebugValueManager.cpp`

Relevant code paths, from inspection:

- `WebAssemblyRegStackify::runOnMachineFunction` walks each machine basic block   bottom-up and recursively stackifies operands, pushing the operands of newly stackified instructions back onto its worklist.
- The stackifier calls helpers such as `moveForSingleUse`, `rematerializeCheapDef` and `moveAndTeeForMultiUse`.
- Those helpers instantiate a `WebAssemblyDebugValueManager` and call `sink`, `cloneSink`, `updateReg` and `removeDef` while moving or cloning machine instructions.
- `WebAssemblyDebugValueManager::WebAssemblyDebugValueManager(MachineInstr *Def)` scans forward from `Def` through the machine basic block until the next def of the same register, collecting matching `DBG_VALUE`s.
- `WebAssemblyDebugValueManager::getSinkableDebugValues(MachineInstr *Insert)` scans the region between `Def` and `Insert` again, to collect intervening `DBG_VALUE`s and reject debug variable reorderings.
- `sink` and `cloneSink` call `getSinkableDebugValues`, splice or clone the definition, clone the sinkable debug values, and undef the original ones.

For a large inlined machine block built with `debug = 2`, the stackifier appears to repeat these forward debug value scans for many virtual register definitions while it builds stackified expression trees.

The cost is therefore superlinear in the size of the block, which matches what the reproducer does: adding one more level to the ladder in `src/lib.rs` roughly doubles the amount of code and multiplies the build time by an order of magnitude.

I've no idea how to fix that, so if you can help, please do.
