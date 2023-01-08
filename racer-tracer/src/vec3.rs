use std::{fmt, ops};

#[derive(Default, Clone, Copy)]
pub struct Vec3 {
    data: [f64; 3],
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { data: [x, y, z] }
    }

    pub fn x(&self) -> &f64 {
        &self.data[0]
    }

    pub fn y(&self) -> &f64 {
        &self.data[1]
    }

    pub fn z(&self) -> &f64 {
        &self.data[2]
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    pub fn length_squared(&self) -> f64 {
        self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2]
    }

    pub fn dot(&self, v: &Vec3) -> f64 {
        dot(self, v)
    }

    pub fn cross(&self, v: &Vec3) -> Vec3 {
        cross(self, v)
    }

    pub fn unit_vector(&self) -> Vec3 {
        unit_vector(self)
    }

    pub fn as_color(&self) -> u32 {
        let red: u32 = (self.data[0] * 255.0) as u32;
        let green: u32 = (self.data[1] * 255.0) as u32;
        let blue: u32 = (self.data[2] * 255.0) as u32;
        ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
    }
}

pub fn dot(v1: &Vec3, v2: &Vec3) -> f64 {
    v1.data[0] * v2.data[0] + v1.data[1] * v2.data[1] + v1.data[2] * v2.data[2]
}

pub fn cross(v1: &Vec3, v2: &Vec3) -> Vec3 {
    Vec3::new(
        v1.data[1] * v2.data[2] - v1.data[2] * v2.data[1],
        v1.data[2] * v2.data[0] - v1.data[0] * v2.data[2],
        v1.data[0] * v2.data[1] - v1.data[1] * v2.data[0],
    )
}

pub fn unit_vector(v: &Vec3) -> Vec3 {
    v / v.length()
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3::new(
            self.data[0] + rhs.data[0],
            self.data[1] + rhs.data[1],
            self.data[2] + rhs.data[2],
        )
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.data[0] += rhs.data[0];
        self.data[1] += rhs.data[1];
        self.data[2] += rhs.data[2];
    }
}

fn vec_sub(v1: &[f64; 3], v2: &[f64; 3]) -> Vec3 {
    Vec3::new(v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2])
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        vec_sub(&self.data, &rhs.data)
    }
}

impl ops::Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        vec_sub(&self.data, &rhs.data)
    }
}

impl ops::Sub<Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        vec_sub(&self.data, &rhs.data)
    }
}

impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.data[0] -= rhs.data[0];
        self.data[1] -= rhs.data[1];
        self.data[2] -= rhs.data[2];
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
        self.data[0] *= t;
        self.data[1] *= t;
        self.data[2] *= t;
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self.data[0] * rhs, self.data[1] * rhs, self.data[2] * rhs)
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
            self.data[0] * rhs.data[0],
            self.data[1] * rhs.data[1],
            self.data[2] * rhs.data[2],
        )
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(rhs.data[0] * self, rhs.data[1] * self, rhs.data[2] * self)
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.data[0] *= rhs;
        self.data[1] *= rhs;
        self.data[2] *= rhs;
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.data[0], -self.data[1], -self.data[2])
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.data[0] == other.data[0]
            && self.data[1] == other.data[1]
            && self.data[2] == other.data[2]
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            format!(
                "x: {}, y: {}, z: {}",
                self.data[0], self.data[1], self.data[2]
            )
            .as_str(),
        )
    }
}

impl std::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vec3").field("data", &self.data).finish()
    }
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
