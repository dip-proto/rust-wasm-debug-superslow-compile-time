use core::ops::{Add, Mul, Sub};

const MASK: u128 = 0x0007_ffff_ffff_ffff;

#[derive(Clone, Default, Copy)]
pub struct Limb(pub [u64; 5]);

pub static ZERO: Limb = Limb([0, 0, 0, 0, 0]);
pub static ONE: Limb = Limb([1, 0, 0, 0, 0]);

#[inline]
pub fn wide_mul(out: &mut [u64; 5], a: &[u64; 5], b: &[u64; 5]) {
    let x1: u128 = (a[0] as u128).wrapping_mul(b[0] as u128);
    let x2: u128 = (a[0] as u128).wrapping_mul(b[1] as u128);
    let x3: u128 = (a[0] as u128).wrapping_mul(b[2] as u128);
    let x4: u128 = (a[0] as u128).wrapping_mul(b[3] as u128);
    let x5: u128 = (a[0] as u128).wrapping_mul(b[4] as u128);
    let x6: u128 = (a[1] as u128).wrapping_mul(b[0] as u128);
    let x7: u128 = (a[1] as u128).wrapping_mul(b[1] as u128);
    let x8: u128 = (a[1] as u128).wrapping_mul(b[2] as u128);
    let x9: u128 = (a[1] as u128).wrapping_mul(b[3] as u128);
    let x10: u128 = (a[1] as u128).wrapping_mul(b[4] as u128);
    let x11: u128 = (a[2] as u128).wrapping_mul(b[0] as u128);
    let x12: u128 = (a[2] as u128).wrapping_mul(b[1] as u128);
    let x13: u128 = (a[2] as u128).wrapping_mul(b[2] as u128);
    let x14: u128 = (a[2] as u128).wrapping_mul(b[3] as u128);
    let x15: u128 = (a[2] as u128).wrapping_mul(b[4] as u128);
    let x16: u128 = (a[3] as u128).wrapping_mul(b[0] as u128);
    let x17: u128 = (a[3] as u128).wrapping_mul(b[1] as u128);
    let x18: u128 = (a[3] as u128).wrapping_mul(b[2] as u128);
    let x19: u128 = (a[3] as u128).wrapping_mul(b[3] as u128);
    let x20: u128 = (a[3] as u128).wrapping_mul(b[4] as u128);
    let x21: u128 = (a[4] as u128).wrapping_mul(b[0] as u128);
    let x22: u128 = (a[4] as u128).wrapping_mul(b[1] as u128);
    let x23: u128 = (a[4] as u128).wrapping_mul(b[2] as u128);
    let x24: u128 = (a[4] as u128).wrapping_mul(b[3] as u128);
    let x25: u128 = (a[4] as u128).wrapping_mul(b[4] as u128);
    let x26 = x1.wrapping_add(x10).wrapping_add(x14).wrapping_add(x18).wrapping_add(x22);
    let x27 = x2.wrapping_add(x6).wrapping_add(x15).wrapping_add(x19).wrapping_add(x23);
    let x28 = x3.wrapping_add(x7).wrapping_add(x11).wrapping_add(x20).wrapping_add(x24);
    let x29 = x4.wrapping_add(x8).wrapping_add(x12).wrapping_add(x16).wrapping_add(x25);
    let x30 = x5.wrapping_add(x9).wrapping_add(x13).wrapping_add(x17).wrapping_add(x21);
    let x31 = x26.wrapping_add(x30 >> 51);
    let x32 = x27.wrapping_add(x31 >> 51);
    let x33 = x28.wrapping_add(x32 >> 51);
    let x34 = x29.wrapping_add(x33 >> 51);
    let x35 = x30.wrapping_add(x34 >> 51);
    out[0] = (x31 & MASK) as u64;
    out[1] = (x32 & MASK) as u64;
    out[2] = (x33 & MASK) as u64;
    out[3] = (x34 & MASK) as u64;
    out[4] = (x35 & MASK) as u64;
}

impl Add for Limb {
    type Output = Limb;
    #[inline(always)]
    fn add(self, rhs: Limb) -> Limb {
        Limb([
            self.0[0].wrapping_add(rhs.0[0]),
            self.0[1].wrapping_add(rhs.0[1]),
            self.0[2].wrapping_add(rhs.0[2]),
            self.0[3].wrapping_add(rhs.0[3]),
            self.0[4].wrapping_add(rhs.0[4]),
        ])
    }
}

impl Sub for Limb {
    type Output = Limb;
    #[inline(always)]
    fn sub(self, rhs: Limb) -> Limb {
        Limb([
            self.0[0].wrapping_add(MASK as u64).wrapping_sub(rhs.0[0]),
            self.0[1].wrapping_add(MASK as u64).wrapping_sub(rhs.0[1]),
            self.0[2].wrapping_add(MASK as u64).wrapping_sub(rhs.0[2]),
            self.0[3].wrapping_add(MASK as u64).wrapping_sub(rhs.0[3]),
            self.0[4].wrapping_add(MASK as u64).wrapping_sub(rhs.0[4]),
        ])
        .carry()
    }
}

impl Mul for Limb {
    type Output = Limb;
    #[inline(always)]
    fn mul(self, rhs: Limb) -> Limb {
        let mut out = Limb::default();
        wide_mul(&mut out.0, &self.0, &rhs.0);
        out
    }
}

impl Limb {
    #[inline(always)]
    pub fn carry(&self) -> Limb {
        let x1 = self.0[0].wrapping_add(self.0[4] >> 51);
        let x2 = self.0[1].wrapping_add(x1 >> 51);
        let x3 = self.0[2].wrapping_add(x2 >> 51);
        let x4 = self.0[3].wrapping_add(x3 >> 51);
        let x5 = self.0[4].wrapping_add(x4 >> 51);
        Limb([
            x1 & MASK as u64,
            x2 & MASK as u64,
            x3 & MASK as u64,
            x4 & MASK as u64,
            x5 & MASK as u64,
        ])
    }

    #[inline(always)]
    pub fn square(&self) -> Limb {
        let mut out = Limb::default();
        wide_mul(&mut out.0, &self.0, &self.0);
        out
    }

    #[inline(always)]
    pub fn square_and_double(&self) -> Limb {
        let out = self.square();
        out + out
    }
}
