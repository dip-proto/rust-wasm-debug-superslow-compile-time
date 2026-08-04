#![allow(unused_parens)]
#![allow(non_camel_case_types)]

use core::ops::{Add, Mul, Sub};


pub type bit = u8;
#[inline]
pub fn wide_mul(out1: &mut [u64; 5], arg1: &[u64; 5], arg2: &[u64; 5]) {
    let x1: u128 = (((arg1[4]) as u128).wrapping_mul(((arg2[4]).wrapping_mul(0x13)) as u128));
    let x2: u128 = (((arg1[4]) as u128).wrapping_mul(((arg2[3]).wrapping_mul(0x13)) as u128));
    let x3: u128 = (((arg1[4]) as u128).wrapping_mul(((arg2[2]).wrapping_mul(0x13)) as u128));
    let x4: u128 = (((arg1[4]) as u128).wrapping_mul(((arg2[1]).wrapping_mul(0x13)) as u128));
    let x5: u128 = (((arg1[3]) as u128).wrapping_mul(((arg2[4]).wrapping_mul(0x13)) as u128));
    let x6: u128 = (((arg1[3]) as u128).wrapping_mul(((arg2[3]).wrapping_mul(0x13)) as u128));
    let x7: u128 = (((arg1[3]) as u128).wrapping_mul(((arg2[2]).wrapping_mul(0x13)) as u128));
    let x8: u128 = (((arg1[2]) as u128).wrapping_mul(((arg2[4]).wrapping_mul(0x13)) as u128));
    let x9: u128 = (((arg1[2]) as u128).wrapping_mul(((arg2[3]).wrapping_mul(0x13)) as u128));
    let x10: u128 = (((arg1[1]) as u128).wrapping_mul(((arg2[4]).wrapping_mul(0x13)) as u128));
    let x11: u128 = (((arg1[4]) as u128).wrapping_mul((arg2[0]) as u128));
    let x12: u128 = (((arg1[3]) as u128).wrapping_mul((arg2[1]) as u128));
    let x13: u128 = (((arg1[3]) as u128).wrapping_mul((arg2[0]) as u128));
    let x14: u128 = (((arg1[2]) as u128).wrapping_mul((arg2[2]) as u128));
    let x15: u128 = (((arg1[2]) as u128).wrapping_mul((arg2[1]) as u128));
    let x16: u128 = (((arg1[2]) as u128).wrapping_mul((arg2[0]) as u128));
    let x17: u128 = (((arg1[1]) as u128).wrapping_mul((arg2[3]) as u128));
    let x18: u128 = (((arg1[1]) as u128).wrapping_mul((arg2[2]) as u128));
    let x19: u128 = (((arg1[1]) as u128).wrapping_mul((arg2[1]) as u128));
    let x20: u128 = (((arg1[1]) as u128).wrapping_mul((arg2[0]) as u128));
    let x21: u128 = (((arg1[0]) as u128).wrapping_mul((arg2[4]) as u128));
    let x22: u128 = (((arg1[0]) as u128).wrapping_mul((arg2[3]) as u128));
    let x23: u128 = (((arg1[0]) as u128).wrapping_mul((arg2[2]) as u128));
    let x24: u128 = (((arg1[0]) as u128).wrapping_mul((arg2[1]) as u128));
    let x25: u128 = (((arg1[0]) as u128).wrapping_mul((arg2[0]) as u128));
    let x26: u128 = (x25.wrapping_add(x10.wrapping_add(x9.wrapping_add(x7.wrapping_add(x4)))));
    let x27: u64 = ((x26 >> 51) as u64);
    let x28: u64 = ((x26 & 0x7ffffffffffff_u128) as u64);
    let x29: u128 = (x21.wrapping_add(x17.wrapping_add(x14.wrapping_add(x12.wrapping_add(x11)))));
    let x30: u128 = (x22.wrapping_add(x18.wrapping_add(x15.wrapping_add(x13.wrapping_add(x1)))));
    let x31: u128 = (x23.wrapping_add(x19.wrapping_add(x16.wrapping_add(x5.wrapping_add(x2)))));
    let x32: u128 = (x24.wrapping_add(x20.wrapping_add(x8.wrapping_add(x6.wrapping_add(x3)))));
    let x33: u128 = ((x27 as u128).wrapping_add(x32));
    let x34: u64 = ((x33 >> 51) as u64);
    let x35: u64 = ((x33 & 0x7ffffffffffff_u128) as u64);
    let x36: u128 = ((x34 as u128).wrapping_add(x31));
    let x37: u64 = ((x36 >> 51) as u64);
    let x38: u64 = ((x36 & 0x7ffffffffffff_u128) as u64);
    let x39: u128 = ((x37 as u128).wrapping_add(x30));
    let x40: u64 = ((x39 >> 51) as u64);
    let x41: u64 = ((x39 & 0x7ffffffffffff_u128) as u64);
    let x42: u128 = ((x40 as u128).wrapping_add(x29));
    let x43: u64 = ((x42 >> 51) as u64);
    let x44: u64 = ((x42 & 0x7ffffffffffff_u128) as u64);
    let x45: u64 = (x43.wrapping_mul(0x13));
    let x46: u64 = (x28.wrapping_add(x45));
    let x47: u64 = (x46 >> 51);
    let x48: u64 = (x46 & 0x7ffffffffffff);
    let x49: u64 = (x47.wrapping_add(x35));
    let x50: bit = ((x49 >> 51) as bit);
    let x51: u64 = (x49 & 0x7ffffffffffff);
    let x52: u64 = ((x50 as u64).wrapping_add(x38));
    out1[0] = x48;
    out1[1] = x51;
    out1[2] = x52;
    out1[3] = x41;
    out1[4] = x44;
}

#[inline]
pub fn wide_square(out1: &mut [u64; 5], arg1: &[u64; 5]) {
    let x1: u64 = ((arg1[4]).wrapping_mul(0x13));
    let x2: u64 = (x1.wrapping_mul(0x2));
    let x3: u64 = ((arg1[4]).wrapping_mul(0x2));
    let x4: u64 = ((arg1[3]).wrapping_mul(0x13));
    let x5: u64 = (x4.wrapping_mul(0x2));
    let x6: u64 = ((arg1[3]).wrapping_mul(0x2));
    let x7: u64 = ((arg1[2]).wrapping_mul(0x2));
    let x8: u64 = ((arg1[1]).wrapping_mul(0x2));
    let x9: u128 = (((arg1[4]) as u128).wrapping_mul(x1 as u128));
    let x10: u128 = (((arg1[3]) as u128).wrapping_mul(x2 as u128));
    let x11: u128 = (((arg1[3]) as u128).wrapping_mul(x4 as u128));
    let x12: u128 = (((arg1[2]) as u128).wrapping_mul(x2 as u128));
    let x13: u128 = (((arg1[2]) as u128).wrapping_mul(x5 as u128));
    let x14: u128 = (((arg1[2]) as u128).wrapping_mul((arg1[2]) as u128));
    let x15: u128 = (((arg1[1]) as u128).wrapping_mul(x2 as u128));
    let x16: u128 = (((arg1[1]) as u128).wrapping_mul(x6 as u128));
    let x17: u128 = (((arg1[1]) as u128).wrapping_mul(x7 as u128));
    let x18: u128 = (((arg1[1]) as u128).wrapping_mul((arg1[1]) as u128));
    let x19: u128 = (((arg1[0]) as u128).wrapping_mul(x3 as u128));
    let x20: u128 = (((arg1[0]) as u128).wrapping_mul(x6 as u128));
    let x21: u128 = (((arg1[0]) as u128).wrapping_mul(x7 as u128));
    let x22: u128 = (((arg1[0]) as u128).wrapping_mul(x8 as u128));
    let x23: u128 = (((arg1[0]) as u128).wrapping_mul((arg1[0]) as u128));
    let x24: u128 = (x23.wrapping_add(x15.wrapping_add(x13)));
    let x25: u64 = ((x24 >> 51) as u64);
    let x26: u64 = ((x24 & 0x7ffffffffffff_u128) as u64);
    let x27: u128 = (x19.wrapping_add(x16.wrapping_add(x14)));
    let x28: u128 = (x20.wrapping_add(x17.wrapping_add(x9)));
    let x29: u128 = (x21.wrapping_add(x18.wrapping_add(x10)));
    let x30: u128 = (x22.wrapping_add(x12.wrapping_add(x11)));
    let x31: u128 = ((x25 as u128).wrapping_add(x30));
    let x32: u64 = ((x31 >> 51) as u64);
    let x33: u64 = ((x31 & 0x7ffffffffffff_u128) as u64);
    let x34: u128 = ((x32 as u128).wrapping_add(x29));
    let x35: u64 = ((x34 >> 51) as u64);
    let x36: u64 = ((x34 & 0x7ffffffffffff_u128) as u64);
    let x37: u128 = ((x35 as u128).wrapping_add(x28));
    let x38: u64 = ((x37 >> 51) as u64);
    let x39: u64 = ((x37 & 0x7ffffffffffff_u128) as u64);
    let x40: u128 = ((x38 as u128).wrapping_add(x27));
    let x41: u64 = ((x40 >> 51) as u64);
    let x42: u64 = ((x40 & 0x7ffffffffffff_u128) as u64);
    let x43: u64 = (x41.wrapping_mul(0x13));
    let x44: u64 = (x26.wrapping_add(x43));
    let x45: u64 = (x44 >> 51);
    let x46: u64 = (x44 & 0x7ffffffffffff);
    let x47: u64 = (x45.wrapping_add(x33));
    let x48: bit = ((x47 >> 51) as bit);
    let x49: u64 = (x47 & 0x7ffffffffffff);
    let x50: u64 = ((x48 as u64).wrapping_add(x36));
    out1[0] = x46;
    out1[1] = x49;
    out1[2] = x50;
    out1[3] = x39;
    out1[4] = x42;
}

#[inline]
pub fn normalize(out1: &mut [u64; 5], arg1: &[u64; 5]) {
    let x1: u64 = (arg1[0]);
    let x2: u64 = ((x1 >> 51).wrapping_add(arg1[1]));
    let x3: u64 = ((x2 >> 51).wrapping_add(arg1[2]));
    let x4: u64 = ((x3 >> 51).wrapping_add(arg1[3]));
    let x5: u64 = ((x4 >> 51).wrapping_add(arg1[4]));
    let x6: u64 = ((x1 & 0x7ffffffffffff).wrapping_add((x5 >> 51).wrapping_mul(0x13)));
    let x7: u64 = ((((x6 >> 51) as bit) as u64).wrapping_add(x2 & 0x7ffffffffffff));
    let x8: u64 = (x6 & 0x7ffffffffffff);
    let x9: u64 = (x7 & 0x7ffffffffffff);
    let x10: u64 = ((((x7 >> 51) as bit) as u64).wrapping_add(x3 & 0x7ffffffffffff));
    let x11: u64 = (x4 & 0x7ffffffffffff);
    let x12: u64 = (x5 & 0x7ffffffffffff);
    out1[0] = x8;
    out1[1] = x9;
    out1[2] = x10;
    out1[3] = x11;
    out1[4] = x12;
}

#[inline]
pub fn wide_add(out1: &mut [u64; 5], arg1: &[u64; 5], arg2: &[u64; 5]) {
    let x1: u64 = ((arg1[0]).wrapping_add(arg2[0]));
    let x2: u64 = ((arg1[1]).wrapping_add(arg2[1]));
    let x3: u64 = ((arg1[2]).wrapping_add(arg2[2]));
    let x4: u64 = ((arg1[3]).wrapping_add(arg2[3]));
    let x5: u64 = ((arg1[4]).wrapping_add(arg2[4]));
    out1[0] = x1;
    out1[1] = x2;
    out1[2] = x3;
    out1[3] = x4;
    out1[4] = x5;
}

#[inline]
pub fn wide_sub(out1: &mut [u64; 5], arg1: &[u64; 5], arg2: &[u64; 5]) {
    let x1: u64 = ((0xfffffffffffdau64.wrapping_add(arg1[0])).wrapping_sub(arg2[0]));
    let x2: u64 = ((0xffffffffffffeu64.wrapping_add(arg1[1])).wrapping_sub(arg2[1]));
    let x3: u64 = ((0xffffffffffffeu64.wrapping_add(arg1[2])).wrapping_sub(arg2[2]));
    let x4: u64 = ((0xffffffffffffeu64.wrapping_add(arg1[3])).wrapping_sub(arg2[3]));
    let x5: u64 = ((0xffffffffffffeu64.wrapping_add(arg1[4])).wrapping_sub(arg2[4]));
    out1[0] = x1;
    out1[1] = x2;
    out1[2] = x3;
    out1[3] = x4;
    out1[4] = x5;
}



#[derive(Clone, Default, Copy)]
pub struct Limb(pub [u64; 5]);

pub static ZERO: Limb = Limb([0, 0, 0, 0, 0]);
pub static ONE: Limb = Limb([1, 0, 0, 0, 0]);
pub(crate) static MIX: Limb = Limb([
    1859910466990425,
    932731440258426,
    1072319116312658,
    1815898335770999,
    633789495995903,
]);


impl Add for Limb {
    type Output = Limb;
    #[inline(always)]
    fn add(self, _rhs: Limb) -> Limb {
        let Limb(f) = self;
        let Limb(g) = _rhs;
        let mut h = Limb::default();
        wide_add(&mut h.0, &f, &g);
        h
    }
}

impl Sub for Limb {
    type Output = Limb;
    #[inline(always)]
    fn sub(self, _rhs: Limb) -> Limb {
        let Limb(f) = self;
        let Limb(g) = _rhs;
        let mut h = Limb::default();
        wide_sub(&mut h.0, &f, &g);
        h.carry()
    }
}

impl Mul for Limb {
    type Output = Limb;
    #[inline(always)]
    fn mul(self, _rhs: Limb) -> Limb {
        let Limb(f) = self;
        let Limb(g) = _rhs;
        let mut h = Limb::default();
        wide_mul(&mut h.0, &f, &g);
        h
    }
}

impl Limb {
    #[inline(always)]
    pub fn carry(&self) -> Limb {
        let mut h = Limb::default();
        normalize(&mut h.0, &self.0);
        h
    }
    #[inline(always)]
    pub fn square(&self) -> Limb {
        let &Limb(f) = &self;
        let mut h = Limb::default();
        wide_square(&mut h.0, &f);
        h
    }
    #[inline(always)]
    pub fn square_and_double(&self) -> Limb {
        let h = self.square();
        (h + h)
    }
}

