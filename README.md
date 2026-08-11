# repro: generating wasm debugging symbols is very slow (Rust)

The Rust compiler can be extremely slow to generate debugging information for the WebAssembly target.

This repository is a ~300 lines reproducer.

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
rustc 1.94.1 (e408947bf 2026-03-25)
host: x86_64-unknown-linux-gnu
LLVM version: 21.1.8
cargo 1.94.1 (29ea6fb6a 2026-03-24)
```

On my machine, the full-debuginfo wasm release build does not finish within 30 seconds.

Workaround is to use `debug = 1`, but the root cause should be fixed.

Controls:

```text
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-unknown-unknown
# final310-debug1-wasm elapsed=0:00.88 maxrss=118576KB

CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-unknown-unknown
# final310-nodebug-wasm elapsed=0:00.51 maxrss=112332KB

cargo build --release
# final310-debug2-host elapsed=0:00.75 maxrss=135712KB

cargo rustc --release --target=wasm32-unknown-unknown -- --emit=llvm-ir -C no-prepopulate-passes
# final310-no-prepopulate-ir elapsed=0:00.12 maxrss=101280KB
```

With:

```sh
RUSTFLAGS='-Cllvm-args=--debug-pass=Executions' \
  timeout 12s cargo build --release --target=wasm32-unknown-unknown
```

the compile is killed after entering:

```text
Executing Pass 'WebAssembly Register Stackify' on Function '_ZN29wasm_reg_stackify_debug_repro9aggregate2P23run17hf53ef9ee49cd11bcE'...
final310-pass-trace elapsed=0:12.01 maxrss=41356KB status=124
```

## Likely LLVM hot spot

The likely LLVM hot spot is the interaction between:

- `llvm/lib/Target/WebAssembly/WebAssemblyRegStackify.cpp`
- `llvm/lib/Target/WebAssembly/WebAssemblyDebugValueManager.cpp`

The pass trace points to `WebAssembly Register Stackify`, before `WebAssembly Debug Fixup` runs:

```text
... WebAssembly Memory Intrinsic Results
Executing Pass 'WebAssembly Register Stackify' on Function '...aggregate...P2...run...'
```

Relevant LLVM code paths from inspection:

- `WebAssemblyRegStackify::runOnMachineFunction` walks each machine basic block bottom-up and recursively stackifies operands by pushing operands from newly stackified instructions back onto its worklist.
- The stackifier calls helpers such as `moveForSingleUse`, `rematerializeCheapDef`, and `moveAndTeeForMultiUse`.
- Those helpers instantiate `WebAssemblyDebugValueManager` and call `sink`, `cloneSink`, `updateReg`, and `removeDef` while moving or cloning machine instructions.
- `WebAssemblyDebugValueManager::WebAssemblyDebugValueManager(MachineInstr *Def)` scans forward from `Def` through the machine basic block until the next def of the same register, collecting matching `DBG_VALUE`s.
- `WebAssemblyDebugValueManager::getSinkableDebugValues(MachineInstr *Insert)` scans the region between `Def` and `Insert` again to collect intervening `DBG_VALUE`s and reject debug-variable reorderings.
- `sink` and `cloneSink` call `getSinkableDebugValues`, splice or clone the definition, clone sinkable debug values, and undef original debug values.

The suspected bug is pathological compile time, not incorrect output.

For large inlined machine block with Rust `debug = 2`, the wasm stackifier appears to repeat forward debug-value scans for many virtual-register definitions while building stackified expression trees.

I've no idea how to fix that, so if you can help, please do.
