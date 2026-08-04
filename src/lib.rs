#![allow(dead_code, unused_imports)]

mod aggregate;
mod limb;

#[unsafe(no_mangle)]
pub extern "C" fn repro(input: u8) -> u8 {
    aggregate::exercise(input)
}
