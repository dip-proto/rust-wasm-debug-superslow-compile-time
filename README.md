# wasm reg-stackify debug-info compile-time repro

This is a minimized Rust repro for a very slow `wasm32-wasip1` optimized build
with full debuginfo. It is intentionally generic: there is no dependency on
`ed25519-compact`, no crypto crate, and no curve-specific names or constants.

The manifest plus Rust source is 310 lines:

```text
  12 Cargo.toml
   6 src/lib.rs
 169 src/aggregate.rs
 123 src/limb.rs
 310 total
```

## Reproduce

```sh
rustup target add wasm32-wasip1
cargo clean
timeout 30s cargo build --release --target=wasm32-wasip1
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

On my machine, the full-debuginfo wasm release build does not finish within 30
seconds:

```text
timeout 30s cargo build --release --target=wasm32-wasip1
# final310-debug2-wasm elapsed=0:30.02 maxrss=41408KB status=124
```

Controls:

```text
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-wasip1
# final310-debug1-wasm elapsed=0:00.88 maxrss=118576KB

CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-wasip1
# final310-nodebug-wasm elapsed=0:00.51 maxrss=112332KB

cargo build --release
# final310-debug2-host elapsed=0:00.75 maxrss=135712KB

cargo rustc --release --target=wasm32-wasip1 -- --emit=llvm-ir -C no-prepopulate-passes
# final310-no-prepopulate-ir elapsed=0:00.12 maxrss=101280KB
```

With:

```sh
RUSTFLAGS='-Cllvm-args=--debug-pass=Executions' \
  timeout 12s cargo build --release --target=wasm32-wasip1
```

the compile is killed after entering:

```text
Executing Pass 'WebAssembly Register Stackify' on Function '_ZN29wasm_reg_stackify_debug_repro9aggregate2P23run17hf53ef9ee49cd11bcE'...
final310-pass-trace elapsed=0:12.01 maxrss=41356KB status=124
```

## Reduced pattern

The remaining trigger is:

- `wasm32-wasip1`
- optimized release codegen
- full Rust debuginfo, `debug = 2`
- a public `repro(u8)` that reaches one private aggregate routine
- a two-iteration reverse loop
- a two-entry fixed sign array derived from the input byte
- a two-entry precomputed aggregate array
- private inline aggregate-returning methods over `P2`, `P3`, `P1P1`, and
  `Cached`
- a synthetic five-limb `Limb([u64; 5])`
- a full 5x5 `wide_mul` using 25 `u128` products
- subtraction normalization through a five-limb carry chain
- one cached `z` multiplication in the signed aggregate operation

Minimization controls that make the build fast:

- one loop iteration instead of two: about 0.29s
- computing the sign inside the loop instead of using the fixed sign array:
  about 0.70s
- reducing `wide_mul` to 24 products: about 1.00s
- reducing `wide_mul` to 20 products, even with three loop iterations: about
  0.90s
- removing the subtraction carry normalization: about 1.00s
- removing the cached `z` multiplication from the aggregate operation: about
  1.00s
- replacing the signed add/sub branch with positive-only operation: fast
- using one precomputed aggregate instead of two: fast
- using four limbs instead of five: fast
- marking aggregate or limb helpers `#[inline(never)]`: fast
- `debug = 1`, no debuginfo, or a non-wasm host target: fast

This suggests the issue is not the source algorithm. It is a target/backend
compile-time cliff exposed by a compact combination of full debug values,
inlining, wasm stackification, 25-product `u128` arithmetic, and aggregate
value movement.

## Likely LLVM hot spot

The likely LLVM hot spot is the interaction between:

- `llvm/lib/Target/WebAssembly/WebAssemblyRegStackify.cpp`
- `llvm/lib/Target/WebAssembly/WebAssemblyDebugValueManager.cpp`

The pass trace points to `WebAssembly Register Stackify`, before
`WebAssembly Debug Fixup` runs:

```text
... WebAssembly Memory Intrinsic Results
Executing Pass 'WebAssembly Register Stackify' on Function '...aggregate...P2...run...'
```

Relevant LLVM code paths from inspection:

- `WebAssemblyRegStackify::runOnMachineFunction` walks each machine basic block
  bottom-up and recursively stackifies operands by pushing operands from newly
  stackified instructions back onto its worklist.
- The stackifier calls helpers such as `moveForSingleUse`,
  `rematerializeCheapDef`, and `moveAndTeeForMultiUse`.
- Those helpers instantiate `WebAssemblyDebugValueManager` and call `sink`,
  `cloneSink`, `updateReg`, and `removeDef` while moving or cloning machine
  instructions.
- `WebAssemblyDebugValueManager::WebAssemblyDebugValueManager(MachineInstr *Def)`
  scans forward from `Def` through the machine basic block until the next def of
  the same register, collecting matching `DBG_VALUE`s.
- `WebAssemblyDebugValueManager::getSinkableDebugValues(MachineInstr *Insert)`
  scans the region between `Def` and `Insert` again to collect intervening
  `DBG_VALUE`s and reject debug-variable reorderings.
- `sink` and `cloneSink` call `getSinkableDebugValues`, splice or clone the
  definition, clone sinkable debug values, and undef original debug values.

The suspected bug is pathological compile time, not incorrect output. For a
large inlined machine block with Rust `debug = 2`, the wasm stackifier appears
to repeat forward debug-value scans for many virtual-register definitions while
building stackified expression trees. The minimized repro keeps just enough
inlined aggregate movement and `u128` arithmetic to produce that
debug-info-dependent multiplier.
