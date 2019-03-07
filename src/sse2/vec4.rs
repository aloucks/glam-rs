#![allow(dead_code)]

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::sse2::Vec3;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::{f32, fmt, mem, ops::*};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vec4(__m128);

impl fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Vec4 {{ x: {}, y: {}, z: {}, w: {} }}",
            self.get_x(),
            self.get_y(),
            self.get_z(),
            self.get_w(),
        )
    }
}

#[inline]
pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4::new(x, y, z, w)
}

impl Vec4 {
    #[inline]
    pub fn zero() -> Vec4 {
        unsafe { Vec4(_mm_set1_ps(0.0)) }
    }

    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        unsafe { Vec4(_mm_set_ps(w, z, y, x)) }
    }

    #[inline]
    pub fn truncate(self) -> Vec3 {
        self.0.into()
    }

    #[inline]
    pub fn splat(v: f32) -> Vec4 {
        unsafe { Vec4(_mm_set_ps1(v)) }
    }

    #[inline]
    pub fn get_x(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.0) }
    }

    #[inline]
    pub fn get_y(self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.0, self.0, 0b01_01_01_01)) }
    }

    #[inline]
    pub fn get_z(self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.0, self.0, 0b10_10_10_10)) }
    }

    #[inline]
    pub fn get_w(self) -> f32 {
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.0, self.0, 0b11_11_11_11)) }
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        unsafe {
            self.0 = _mm_move_ss(self.0, _mm_set_ss(x));
        }
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        unsafe {
            let mut t = _mm_move_ss(self.0, _mm_set_ss(y));
            t = _mm_shuffle_ps(t, t, 0b11_10_00_00);
            self.0 = _mm_move_ss(t, self.0);
        }
    }

    #[inline]
    pub fn set_z(&mut self, z: f32) {
        unsafe {
            let mut t = _mm_move_ss(self.0, _mm_set_ss(z));
            t = _mm_shuffle_ps(t, t, 0b11_00_01_00);
            self.0 = _mm_move_ss(t, self.0);
        }
    }

    #[inline]
    pub fn set_w(&mut self, w: f32) {
        unsafe {
            let mut t = _mm_move_ss(self.0, _mm_set_ss(w));
            t = _mm_shuffle_ps(t, t, 0b00_10_01_00);
            self.0 = _mm_move_ss(t, self.0);
        }
    }

    #[inline]
    pub fn dot(self, rhs: Vec4) -> f32 {
        unsafe {
            let x2_y2_z2_w2 = _mm_mul_ps(self.0, rhs.0);
            let z2_w2_0_0 = _mm_shuffle_ps(x2_y2_z2_w2, x2_y2_z2_w2, 0b00_00_11_10);
            let x2z2_y2w2_0_0 = _mm_add_ps(x2_y2_z2_w2, z2_w2_0_0);
            let y2w2_0_0_0 = _mm_shuffle_ps(x2z2_y2w2_0_0, x2z2_y2w2_0_0, 0b00_00_00_01);
            let x2y2z2w2_0_0_0 = _mm_add_ps(x2z2_y2w2_0_0, y2w2_0_0_0);
            _mm_cvtss_f32(x2y2z2w2_0_0_0)
        }
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn normalize(self) -> Vec4 {
        let inv_length = 1.0 / self.dot(self).sqrt();
        self * inv_length
    }

    #[inline]
    pub fn min(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_min_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn max(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_max_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn hmin(self) -> f32 {
        unimplemented!();
        // unsafe {
        //     let v = self.0;
        //     let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, 0b00_00_11_10));
        //     let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, 0b00_00_00_01));
        //     _mm_cvtss_f32(v)
        // }
    }

    #[inline]
    pub fn hmax(self) -> f32 {
        unimplemented!();
        // unsafe {
        //     let v = self.0;
        //     let v = _mm_max_ps(v, _mm_shuffle_ps(v, v, 0b00_00_11_10));
        //     let v = _mm_max_ps(v, _mm_shuffle_ps(v, v, 0b00_00_00_01));
        //     _mm_cvtss_f32(v)
        // }
    }

    #[inline]
    pub fn cmpeq(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmpeq_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn cmpne(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmpeq_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn cmpge(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmpge_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn cmpgt(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmpgt_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn cmple(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmple_ps(self.0, rhs.0)) }
    }

    #[inline]
    pub fn cmplt(self, rhs: Vec4) -> Vec4b {
        unsafe { Vec4b(_mm_cmplt_ps(self.0, rhs.0)) }
    }
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.get_x(),
            self.get_y(),
            self.get_z(),
            self.get_w()
        )
    }
}

impl Div<Vec4> for Vec4 {
    type Output = Vec4;
    #[inline]
    fn div(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_div_ps(self.0, rhs.0)) }
    }
}

impl DivAssign<Vec4> for Vec4 {
    #[inline]
    fn div_assign(&mut self, rhs: Vec4) {
        unsafe {
            self.0 = _mm_div_ps(self.0, rhs.0);
        }
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;
    #[inline]
    fn div(self, rhs: f32) -> Vec4 {
        unsafe { Vec4(_mm_div_ps(self.0, _mm_set1_ps(rhs))) }
    }
}

impl DivAssign<f32> for Vec4 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        unsafe { self.0 = _mm_div_ps(self.0, _mm_set1_ps(rhs)) }
    }
}

impl Mul<Vec4> for Vec4 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_mul_ps(self.0, rhs.0)) }
    }
}

impl MulAssign<Vec4> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vec4) {
        unsafe {
            self.0 = _mm_mul_ps(self.0, rhs.0);
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: f32) -> Vec4 {
        unsafe { Vec4(_mm_mul_ps(self.0, _mm_set1_ps(rhs))) }
    }
}

impl MulAssign<f32> for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        unsafe { self.0 = _mm_mul_ps(self.0, _mm_set1_ps(rhs)) }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;
    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_mul_ps(_mm_set1_ps(self), rhs.0)) }
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    #[inline]
    fn add(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_add_ps(self.0, rhs.0)) }
    }
}

impl AddAssign for Vec4 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec4) {
        unsafe { self.0 = _mm_add_ps(self.0, rhs.0) }
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    #[inline]
    fn sub(self, rhs: Vec4) -> Vec4 {
        unsafe { Vec4(_mm_sub_ps(self.0, rhs.0)) }
    }
}

impl SubAssign for Vec4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec4) {
        unsafe { self.0 = _mm_sub_ps(self.0, rhs.0) }
    }
}

impl Neg for Vec4 {
    type Output = Vec4;
    #[inline]
    fn neg(self) -> Vec4 {
        unsafe { Vec4(_mm_sub_ps(_mm_set1_ps(0.0), self.0)) }
    }
}

impl PartialEq for Vec4 {
    #[inline]
    fn eq(&self, rhs: &Vec4) -> bool {
        self.cmpeq(*rhs).all()
    }
}

impl From<Vec4> for __m128 {
    #[inline]
    fn from(t: Vec4) -> Self {
        t.0
    }
}

impl From<__m128> for Vec4 {
    #[inline]
    fn from(t: __m128) -> Self {
        Vec4(t)
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    #[inline]
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Vec4::new(t.0, t.1, t.2, t.3)
    }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    #[inline]
    fn from(v: Vec4) -> Self {
        (v.get_x(), v.get_y(), v.get_z(), v.get_w())
    }
}

impl From<[f32; 4]> for Vec4 {
    #[inline]
    fn from(a: [f32; 4]) -> Self {
        unsafe { Vec4(_mm_loadu_ps(a.as_ptr())) }
    }
}

impl From<Vec4> for [f32; 4] {
    #[inline]
    fn from(v: Vec4) -> Self {
        unsafe {
            let mut out: [f32; 4] = mem::uninitialized();
            _mm_storeu_ps(out.as_mut_ptr(), v.0);
            out
        }
    }
}

impl Distribution<Vec4> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec4 {
        rng.gen::<[f32; 4]>().into()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vec4b(__m128);

impl Vec4b {
    #[inline]
    pub fn mask(&self) -> u32 {
        unsafe { (_mm_movemask_ps(self.0) as u32) }
    }

    #[inline]
    pub fn any(&self) -> bool {
        unsafe { _mm_movemask_ps(self.0) != 0 }
    }

    #[inline]
    pub fn all(&self) -> bool {
        unsafe { _mm_movemask_ps(self.0) == 0xf }
    }
}
