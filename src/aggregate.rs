use super::limb::*;

#[derive(Clone, Copy)]
struct P2 {
    x: Limb,
    y: Limb,
    z: Limb,
}

#[derive(Clone, Copy)]
struct P3 {
    x: Limb,
    y: Limb,
    z: Limb,
    t: Limb,
}

#[derive(Clone, Copy)]
struct P1P1 {
    x: Limb,
    y: Limb,
    z: Limb,
    t: Limb,
}

#[derive(Clone, Copy)]
struct Cached {
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

    fn run(input: u8, a_point: P3) -> P2 {
        let signs = [
            (((input >> 0) & 1) as i8) * 2 - 1,
            (((input >> 1) & 1) as i8) * 2 - 1,
        ];

        let mut ai = [Cached {
            y_plus_x: ZERO,
            y_minus_x: ZERO,
            z: ZERO,
            t2d: ZERO,
        }; 2];
        ai[0] = a_point.to_cached();
        let a2 = P2 {
            x: a_point.x,
            y: a_point.y,
            z: a_point.z,
        }
        .dbl()
        .to_p3();
        ai[1] = a2.apply(ai[0], false).to_p3().to_cached();

        let mut r = P2 {
            x: ZERO,
            y: ONE,
            z: ONE,
        };

        for i in (0..2).rev() {
            let mut t = r.dbl();
            if signs[i] > 0 {
                t = t.to_p3().apply(ai[(signs[i] / 2) as usize], false);
            } else {
                t = t.to_p3().apply(ai[(-signs[i] / 2) as usize], true);
            }

            r = t.to_p2();
        }
        r
    }
}

impl P3 {
    #[inline(always)]
    fn to_cached(&self) -> Cached {
        Cached {
            y_plus_x: self.y + self.x,
            y_minus_x: self.y - self.x,
            z: self.z,
            t2d: self.t,
        }
    }
    #[inline(always)]
    fn apply(self, rhs: Cached, neg: bool) -> P1P1 {
        let y1_plus_x1 = self.y + self.x;
        let y1_minus_x1 = self.y - self.x;
        let a = if neg {
            y1_plus_x1 * rhs.y_minus_x
        } else {
            y1_plus_x1 * rhs.y_plus_x
        };
        let b = if neg {
            y1_minus_x1 * rhs.y_plus_x
        } else {
            y1_minus_x1 * rhs.y_minus_x
        };
        let c = rhs.t2d * self.t;
        let zz = self.z * rhs.z;
        let d = zz + zz;
        let x3 = a - b;
        let y3 = a + b;
        let z3 = if neg { d - c } else { d + c };
        let t3 = if neg { d + c } else { d - c };

        P1P1 {
            x: x3,
            y: y3,
            z: z3,
            t: t3,
        }
    }
}

pub fn exercise(input: u8) -> u8 {
    let out = P2::run(
        input,
        P3 {
            x: ZERO,
            y: ONE,
            z: ONE,
            t: ZERO,
        },
    );
    out.x.0[0] as u8
}
