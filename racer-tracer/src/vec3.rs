use std::{
    fmt,
    ops::{self, Index},
};

use serde::Deserialize;

use crate::util::{random_double, random_double_range};

//https://doc.rust-lang.org/core/arch/x86_64/struct.__m128.html
//https://doc.rust-lang.org/core/arch/x86_64/fn._mm_mul_ps.html
#[derive(Default, Clone, Copy, Deserialize)]
pub struct Vec3 {
    #[serde(alias = "color")]
    pos: [f64; 3],
}

pub type Color = Vec3;

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { pos: [x, y, z] }
    }

    pub fn x(&self) -> &f64 {
        &self.pos[0]
    }

    pub fn y(&self) -> &f64 {
        &self.pos[1]
    }

    pub fn z(&self) -> &f64 {
        &self.pos[2]
    }

    pub fn add(&mut self, v: Vec3) {
        self.pos[0] += v.pos[0];
        self.pos[1] += v.pos[1];
        self.pos[2] += v.pos[2];
    }

    pub fn sub(&mut self, v: Vec3) {
        self.pos[0] -= v.pos[0];
        self.pos[1] -= v.pos[1];
        self.pos[2] -= v.pos[2];
    }

    pub fn div(&mut self, v: f64) {
        self.pos[0] /= v;
        self.pos[1] /= v;
        self.pos[2] /= v;
    }

    pub fn mul(&mut self, v: f64) {
        self.pos[0] *= v;
        self.pos[1] *= v;
        self.pos[2] *= v;
    }

    pub fn reflect(&mut self, mut v: Vec3) {
        let double_dot = 2.0 * self.dot(&v);
        v.mul(double_dot);
        self.sub(v);
    }

    pub fn min(&mut self, v: &Vec3) {
        self.pos[0] = self.pos[0].min(v.pos[0]);
        self.pos[1] = self.pos[1].min(v.pos[1]);
        self.pos[2] = self.pos[2].min(v.pos[2]);
    }

    pub fn max(&mut self, v: &Vec3) {
        self.pos[0] = self.pos[0].max(v.pos[0]);
        self.pos[1] = self.pos[1].max(v.pos[1]);
        self.pos[2] = self.pos[2].max(v.pos[2]);
    }

    pub fn unit_vector(mut self) -> Vec3 {
        let len = self.length();
        self.pos[0] /= len;
        self.pos[1] /= len;
        self.pos[2] /= len;
        self
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    pub fn length_squared(&self) -> f64 {
        self.pos[0] * self.pos[0] + self.pos[1] * self.pos[1] + self.pos[2] * self.pos[2]
    }

    pub fn dot(&self, v: &Vec3) -> f64 {
        dot(self, v)
    }

    pub fn cross(&self, v: &Vec3) -> Vec3 {
        cross(self, v)
    }

    pub fn as_color(&self) -> u32 {
        let red: u32 = (self.pos[0] * 255.0) as u32;
        let green: u32 = (self.pos[1] * 255.0) as u32;
        let blue: u32 = (self.pos[2] * 255.0) as u32;
        // XRGB
        (255 << 24) | (red << 16) | green << 8 | blue
    }

    pub fn random() -> Self {
        Vec3 {
            pos: [random_double(), random_double(), random_double()],
        }
    }

    pub fn random_range(min: f64, max: f64) -> Self {
        Vec3 {
            pos: [
                random_double_range(min, max),
                random_double_range(min, max),
                random_double_range(min, max),
            ],
        }
    }

    pub fn scale_sqrt(mut self, samples: usize) -> Vec3 {
        let scale = 1.0 / samples as f64;
        self.pos[0] = (scale * self.pos[0]).sqrt();
        self.pos[1] = (scale * self.pos[1]).sqrt();
        self.pos[2] = (scale * self.pos[2]).sqrt();
        self
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.pos[0].abs() < s && self.pos[1].abs() < s && self.pos[2].abs() < s
    }

    fn hamilton_product(a: [f64; 4], e: [f64; 4]) -> [f64; 4] {
        [
            a[0] * e[0] - a[1] * e[1] - a[2] * e[2] - a[3] * e[3],
            a[0] * e[1] + a[1] * e[0] + a[2] * e[3] - a[3] * e[2],
            a[0] * e[2] - a[1] * e[3] + a[2] * e[0] + a[3] * e[1],
            a[0] * e[3] + a[1] * e[2] - a[2] * e[1] + a[3] * e[0],
        ]
    }

    fn get_rotation(degrees: f64, axis: &Vec3) -> ([f64; 4], [f64; 4]) {
        let hd = degrees * 0.5;
        let rot = [
            hd.cos(),
            hd.sin() * *axis.x(),
            hd.sin() * *axis.y(),
            hd.sin() * *axis.z(),
        ];
        (rot, [rot[0], -rot[1], -rot[2], -rot[3]])
    }

    pub fn rotate(&mut self, degrees: f64, axis: &Vec3) {
        let p = [0.0, self.pos[0], self.pos[1], self.pos[2]];
        let (r, r_neg) = Vec3::get_rotation(degrees, axis);
        let rpr_neg = Vec3::hamilton_product(Vec3::hamilton_product(r, p), r_neg);
        self.pos[0] = rpr_neg[1];
        self.pos[1] = rpr_neg[2];
        self.pos[2] = rpr_neg[3];
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pos[index]
    }
}

pub fn dot(v1: &Vec3, v2: &Vec3) -> f64 {
    v1.pos[0] * v2.pos[0] + v1.pos[1] * v2.pos[1] + v1.pos[2] * v2.pos[2]
}

pub fn cross(v1: &Vec3, v2: &Vec3) -> Vec3 {
    Vec3::new(
        v1.pos[1] * v2.pos[2] - v1.pos[2] * v2.pos[1],
        v1.pos[2] * v2.pos[0] - v1.pos[0] * v2.pos[2],
        v1.pos[0] * v2.pos[1] - v1.pos[1] * v2.pos[0],
    )
}

#[allow(dead_code)]
pub fn unit_vector(v: &Vec3) -> Vec3 {
    v / v.length()
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.pos[0] + rhs.pos[0],
            self.pos[1] + rhs.pos[1],
            self.pos[2] + rhs.pos[2],
        )
    }
}

impl ops::Add<Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.pos[0] + rhs.pos[0],
            self.pos[1] + rhs.pos[1],
            self.pos[2] + rhs.pos[2],
        )
    }
}

impl ops::Add<f64> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: f64) -> Self::Output {
        Vec3::new(self.pos[0] + rhs, self.pos[1] + rhs, self.pos[2] + rhs)
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.pos[0] += rhs.pos[0];
        self.pos[1] += rhs.pos[1];
        self.pos[2] += rhs.pos[2];
    }
}

impl ops::AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: &Vec3) {
        self.pos[0] += rhs.pos[0];
        self.pos[1] += rhs.pos[1];
        self.pos[2] += rhs.pos[2];
    }
}

fn vec_sub(v1: &[f64; 3], v2: &[f64; 3]) -> Vec3 {
    Vec3::new(v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2])
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        vec_sub(&self.pos, &rhs.pos)
    }
}

impl ops::Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        vec_sub(&self.pos, &rhs.pos)
    }
}

impl ops::Sub<Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        vec_sub(&self.pos, &rhs.pos)
    }
}

impl ops::Sub<f64> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: f64) -> Self::Output {
        Vec3::new(self.pos[0] - rhs, self.pos[1] - rhs, self.pos[2] - rhs)
    }
}

impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.pos[0] -= rhs.pos[0];
        self.pos[1] -= rhs.pos[1];
        self.pos[2] -= rhs.pos[2];
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        (1.0 / rhs) * self
    }
}

impl ops::Div<&Vec3> for f64 {
    type Output = Vec3;

    fn div(self, rhs: &Vec3) -> Self::Output {
        (1.0 / self) * rhs
    }
}

impl ops::Div<f64> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        (1.0 / rhs) * self
    }
}

impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        let t = 1.0 / rhs;
        self.pos[0] *= t;
        self.pos[1] *= t;
        self.pos[2] *= t;
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.pos[0] * rhs, self.pos[1] * rhs, self.pos[2] * rhs)
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.pos[0] * rhs, self.pos[1] * rhs, self.pos[2] * rhs)
    }
}

impl ops::Mul<&Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        *rhs * self
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.pos[0] * rhs.pos[0],
            self.pos[1] * rhs.pos[1],
            self.pos[2] * rhs.pos[2],
        )
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(rhs.pos[0] * self, rhs.pos[1] * self, rhs.pos[2] * self)
    }
}

impl ops::Mul<&f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: &f64) -> Self::Output {
        Vec3::new(self.pos[0] * rhs, self.pos[1] * rhs, self.pos[2] * rhs)
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.pos[0] *= rhs;
        self.pos[1] *= rhs;
        self.pos[2] *= rhs;
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.pos[0], -self.pos[1], -self.pos[2])
    }
}

impl ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.pos[0], -self.pos[1], -self.pos[2])
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.pos[0] == other.pos[0] && self.pos[1] == other.pos[1] && self.pos[2] == other.pos[2]
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("x: {}, y: {}, z: {}", self.pos[0], self.pos[1], self.pos[2]).as_str())
    }
}

impl std::ops::IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &'_ mut Self::Output {
        &mut self.pos[index]
    }
}

impl std::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vec3").field("data", &self.pos).finish()
    }
}

pub fn reflect(v1: &Vec3, v2: &Vec3) -> Vec3 {
    v1 - 2.0 * v1.dot(v2) * v2
}

pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = f64::min(dot(&-uv, n), 1.0);

    let r_out_perp = etai_over_etat * (uv + (cos_theta * n));
    let r_out_parallel = -f64::sqrt(f64::abs(1.0 - r_out_perp.length_squared())) * n;
    r_out_perp + r_out_parallel
}

pub fn random_in_unit_sphere() -> Vec3 {
    let mut v = Vec3::random_range(-1.0, 1.0);
    while v.length_squared() >= 1.0 {
        v = Vec3::random_range(-1.0, 1.0);
    }
    v
}

#[allow(dead_code)]
pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
    let unit_sphere = random_in_unit_sphere();
    if unit_sphere.dot(normal) > 0.0 {
        unit_sphere
    } else {
        -unit_sphere
    }
}

pub fn random_unit_vector() -> Vec3 {
    random_in_unit_sphere().unit_vector()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(2.0, 4.0, 6.0);
        let v3 = v1 + v2;
        let v4 = v2 + v1;

        assert_eq!(*v3.x(), 3.0);
        assert_eq!(*v3.y(), 6.0);
        assert_eq!(*v3.z(), 9.0);
        assert_eq!(v3, v4);
    }

    #[test]
    fn sub() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(2.0, 4.0, 6.0);
        let v3 = v1 - v2;
        assert_eq!(v3.x(), &-1.0);
        assert_eq!(v3.y(), &-2.0);
        assert_eq!(v3.z(), &-3.0);

        let v4 = v2 - v1;
        assert_eq!(v4.x(), &1.0);
        assert_eq!(v4.y(), &2.0);
        assert_eq!(v4.z(), &3.0);
    }

    #[test]
    fn mul() {
        let v1 = Vec3::new(1.0, -2.0, 3.0);
        let v2 = v1 * 5.0;

        assert_eq!(v2.x(), &5.0);
        assert_eq!(v2.y(), &-10.0);
        assert_eq!(v2.z(), &15.0);

        let v3 = Vec3::new(4.0, 8.0, 16.0);
        let v4 = v1 * v3;

        assert_eq!(v4.x(), &4.0);
        assert_eq!(v4.y(), &-16.0);
        assert_eq!(v4.z(), &48.0);
    }

    #[test]
    fn div() {
        let v1 = Vec3::new(1.0, -2.0, 3.0);
        let v2 = v1 / 2.0;

        assert_eq!(v2.x(), &0.5);
        assert_eq!(v2.y(), &-1.0);
        assert_eq!(v2.z(), &1.5);
    }
}
