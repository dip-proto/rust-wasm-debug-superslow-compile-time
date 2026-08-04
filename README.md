# wasm reg-stackify debug-info compile-time repro

This is a minimized Rust repro for a very slow `wasm32-wasip1` release build
with full debuginfo.

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
LLVM version: 21.1.8
```

`debug = 2` for `wasm32-wasip1` does not finish within 30 seconds on my
machine:

```text
timeout 30s cargo build --release --target=wasm32-wasip1  # 30.02s, status 124
```

Smaller/faster controls:

```text
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --release --target=wasm32-wasip1  # 0.89s
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target=wasm32-wasip1  # 0.44s
cargo build --release  # host target, 0.74s
cargo rustc --release --target=wasm32-wasip1 -- --emit=llvm-ir -C no-prepopulate-passes  # 0.13s
```

With:

```sh
RUSTFLAGS='-Cllvm-args=--debug-pass=Executions' \
  timeout 12s cargo build --release --target=wasm32-wasip1
```

the compile is killed after entering:

```text
Executing Pass 'WebAssembly Register Stackify' on Function '...aggregate...P2...run...'
```

## Reduced pattern

The trigger is not Ed25519-specific. The remaining pattern is:

- `wasm32-wasip1`
- optimized release codegen
- full debuginfo, `debug = 2`
- a 256-iteration loop with a signed dynamic branch
- inlined aggregate-returning methods (`P2`, `P3`, `P1P1`, `Cached`)
- a two-entry precomputed aggregate array
- inlined generated limb arithmetic using many `u128` temporaries

Removing the second precomputed aggregate makes this fast. Replacing the signed
add/sub branch with positive-only addition also makes this fast. Reducing to
`debug = 1` or no debuginfo makes this fast.

## Likely LLVM issue

The likely LLVM hot spot is the interaction between:

- `WebAssemblyRegStackify.cpp`
- `WebAssemblyDebugValueManager.cpp`

The pass trace points to `WebAssembly Register Stackify`, before
`WebAssembly Debug Fixup` runs:

```text
... WebAssembly Memory Intrinsic Results
Executing Pass 'WebAssembly Register Stackify' on Function '...aggregate...P2...run...'
```

Relevant LLVM code paths:

- `WebAssemblyRegStackify::runOnMachineFunction` walks each machine basic block
  bottom-up and recursively stackifies operands by pushing each newly
  stackified instruction's operands back onto the worklist.
- The stackifier calls `moveForSingleUse`, `rematerializeCheapDef`, and
  `moveAndTeeForMultiUse` for many virtual register defs.
- Those helpers instantiate `WebAssemblyDebugValueManager` and call `sink`,
  `cloneSink`, and `updateReg` to preserve debug value information while moving
  or cloning machine instructions.
- `WebAssemblyDebugValueManager::WebAssemblyDebugValueManager(MachineInstr *Def)`
  scans forward from `Def` through the machine basic block until the next def of
  the same register, collecting matching `DBG_VALUE`s.
- `WebAssemblyDebugValueManager::getSinkableDebugValues(MachineInstr *Insert)`
  scans the region between `Def` and `Insert` again to collect intervening
  `DBG_VALUE`s and reject debug-variable reorderings.

The suspected bug is not incorrect output but pathological compile-time
behavior: for a large inlined machine block with full Rust `debug = 2`, the wasm
stackifier appears to repeat forward debug-value scans for many defs while it is
building stackified expression trees. That gives the pass a large
debug-info-dependent multiplier. This matches the observed controls:

- `debug = 2` for `wasm32-wasip1` times out after 30 seconds.
- `debug = 1` for the same optimized wasm build finishes in under 1 second.
- no debuginfo for the same optimized wasm build finishes in under 1 second.
- host `debug = 2` finishes in under 1 second.
- raw LLVM IR emission with `-C no-prepopulate-passes` finishes in about 0.13
  seconds, so the Rust frontend and initial IR generation are not the slow part.

The small Rust trigger that exposes this is a generic inlined aggregate loop:
a two-entry precomputed aggregate array, dynamic signed add/sub in a 256-step
loop, and inlined generated `u128` limb arithmetic. Reducing the array to one
entry or making the branch positive-only removes the slowdown, presumably
because there are fewer machine defs/debug values for `WebAssemblyRegStackify`
to rearrange.
