use core::ops::{Add, Sub};

use super::limb::*;

#[derive(Clone, Copy)]
pub struct P2 {
    x: Limb,
    y: Limb,
    z: Limb,
}

#[derive(Clone, Copy)]
pub struct P3 {
    x: Limb,
    y: Limb,
    z: Limb,
    t: Limb,
}

#[derive(Clone, Copy, Default)]
pub struct P1P1 {
    x: Limb,
    y: Limb,
    z: Limb,
    t: Limb,
}

#[derive(Clone, Copy, Default)]
pub struct Cached {
    y_plus_x: Limb,
    y_minus_x: Limb,
    z: Limb,
    t2d: Limb,
}

impl P1P1 {
    #[inline(always)]
    fn to_p2(&self) -> P2 {
        P2 {
            x: self.x * self.t,
            y: self.y * self.z,
            z: self.z * self.t,
        }
    }
    #[inline(always)]
    fn to_p3(&self) -> P3 {
        P3 {
            x: self.x * self.t,
            y: self.y * self.z,
            z: self.z * self.t,
            t: self.x * self.y,
        }
    }
}

impl P2 {
    #[inline(always)]
    fn zero() -> P2 {
        P2 {
            x: ZERO,
            y: ONE,
            z: ONE,
        }
    }
    #[inline(always)]
    fn dbl(&self) -> P1P1 {
        let xx = self.x.square();
        let yy = self.y.square();
        let b = self.z.square_and_double();
        let a = self.x + self.y;
        let aa = a.square();
        let y3 = yy + xx;
        let z3 = yy - xx;
        let x3 = aa - y3;
        let t3 = b - z3;

        P1P1 {
            x: x3,
            y: y3,
            z: z3,
            t: t3,
        }
    }

    fn schedule(a: &[u8]) -> [i8; 256] {
        let mut r = [0i8; 256];
        for i in 0..256 {
            r[i] = ((((a[i & 31] >> (i & 7)) & 1) as i8) * 2) - 1;
        }
        r
    }

    #[allow(clippy::comparison_chain)]
    pub fn run(a_scalar: &[u8], a_point: P3) -> P2 {
        let aslide = P2::schedule(a_scalar);

        let mut ai = [Cached {
            y_plus_x: ZERO,
            y_minus_x: ZERO,
            z: ZERO,
            t2d: ZERO,
        }; 2];
        ai[0] = a_point.to_cached();
        let a2 = a_point.dbl().to_p3();
        ai[1] = (a2 + ai[0]).to_p3().to_cached();

        let mut r = P2::zero();

        let mut i: usize = 255;

        loop {
            let mut t = r.dbl();
            if aslide[i] > 0 {
                t = t.to_p3() + ai[(aslide[i] / 2) as usize];
            } else if aslide[i] < 0 {
                t = t.to_p3() - ai[(-aslide[i] / 2) as usize];
            }

            r = t.to_p2();

            if i == 0 {
                return r;
            }
            i -= 1;
        }
    }
}

impl P3 {
    #[inline(always)]
    fn to_p2(&self) -> P2 {
        P2 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
    #[inline(always)]
    fn to_cached(&self) -> Cached {
        Cached {
            y_plus_x: self.y + self.x,
            y_minus_x: self.y - self.x,
            z: self.z,
            t2d: self.t * MIX,
        }
    }
    #[inline(always)]
    fn zero() -> P3 {
        P3 {
            x: ZERO,
            y: ONE,
            z: ONE,
            t: ZERO,
        }
    }
    #[inline(always)]
    fn dbl(&self) -> P1P1 {
        self.to_p2().dbl()
    }

}


impl Add<Cached> for P3 {
    type Output = P1P1;
    #[inline(always)]
    fn add(self, _rhs: Cached) -> P1P1 {
        let y1_plus_x1 = self.y + self.x;
        let y1_minus_x1 = self.y - self.x;
        let a = y1_plus_x1 * _rhs.y_plus_x;
        let b = y1_minus_x1 * _rhs.y_minus_x;
        let c = _rhs.t2d * self.t;
        let zz = self.z * _rhs.z;
        let d = zz + zz;
        let x3 = a - b;
        let y3 = a + b;
        let z3 = d + c;
        let t3 = d - c;

        P1P1 {
            x: x3,
            y: y3,
            z: z3,
            t: t3,
        }
    }
}

impl Sub<Cached> for P3 {
    type Output = P1P1;
    #[inline(always)]
    fn sub(self, _rhs: Cached) -> P1P1 {
        let y1_plus_x1 = self.y + self.x;
        let y1_minus_x1 = self.y - self.x;
        let a = y1_plus_x1 * _rhs.y_minus_x;
        let b = y1_minus_x1 * _rhs.y_plus_x;
        let c = _rhs.t2d * self.t;
        let zz = self.z * _rhs.z;
        let d = zz + zz;
        let x3 = a - b;
        let y3 = a + b;
        let z3 = d - c;
        let t3 = d + c;

        P1P1 {
            x: x3,
            y: y3,
            z: z3,
            t: t3,
        }
    }
}

pub fn exercise(input: u8) -> u8 {
    let mut a = [input; 32];
    a[0] |= 1;
    a[31] |= 64;

    let doubled = P2::run(&a, P3::zero())
        .dbl()
        .to_p3();
    doubled.x.0[0] as u8
}
