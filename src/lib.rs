const MASK: u64 = 0x0007_ffff_ffff_ffff;

type F = [u64; 5];

#[inline(always)]
fn mul(a: F, b: F) -> F {
    let mut t = [0u128; 9];
    for i in 0..5 {
        for j in 0..5 {
            t[i + j] = t[i + j].wrapping_add((a[i] as u128).wrapping_mul(b[j] as u128));
        }
    }
    core::array::from_fn(|i| t[i].wrapping_add(t[i + 4] >> 51) as u64 & MASK)
}

#[inline(always)]
fn s1(a: F) -> F {
    mul(mul(a, a), a)
}
#[inline(always)]
fn s2(a: F) -> F {
    s1(s1(a))
}
#[inline(always)]
fn s4(a: F) -> F {
    s2(s2(a))
}
#[inline(always)]
fn s8(a: F) -> F {
    s4(s4(a))
}
#[inline(always)]
fn s16(a: F) -> F {
    s8(s8(a))
}
#[inline(always)]
fn s32(a: F) -> F {
    s16(s16(a))
}

pub fn repro(input: u8) -> u64 {
    s8(s32([input as u64, 1, 2, 3, 4]))[0]
}
