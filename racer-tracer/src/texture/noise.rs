use crate::{
    util::random_int_range,
    vec3::{Color, Vec3},
};

use super::Texture;

pub struct Noise {
    perlin: Perlin,
    depth: i32,
    color: Color,
    scale: f64,
}

impl Noise {
    pub fn new(scale: f64, depth: Option<i32>, color: Color) -> Self {
        Self {
            scale,
            depth: depth.unwrap_or(7),
            perlin: Perlin::new(),
            color,
        }
    }
}

impl Texture for Noise {
    fn value(&self, _u: f64, _v: f64, point: &Vec3) -> Color {
        self.color
            * 0.5
            * (1.0
                + (self.scale * point.z() + 10.0 * self.perlin.turbulence(point, self.depth)).sin())
    }
}

const POINT_COUNT: usize = 256;
pub struct Perlin {
    ran_vec: [Vec3; POINT_COUNT],
    perm_x: [i32; POINT_COUNT],
    perm_y: [i32; POINT_COUNT],
    perm_z: [i32; POINT_COUNT],
}

impl Perlin {
    pub fn new() -> Self {
        let mut ran = [Vec3::default(); POINT_COUNT];
        ran.iter_mut()
            .for_each(|v| *v = Vec3::random_range(-1.0, 1.0).unit_vector());

        Self {
            ran_vec: ran,
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, point: &Vec3) -> f64 {
        let u = point.x() - point.x().floor();
        let v = point.y() - point.y().floor();
        let w = point.z() - point.z().floor();
        let i = point.x().floor() as i32;
        let j = point.y().floor() as i32;
        let k = point.z().floor() as i32;
        let mut c = [[[Vec3::default(); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let index = self.perm_x[((i + di) & 255) as usize]
                        ^ self.perm_y[((j + dj) & 255) as usize]
                        ^ self.perm_z[((k + dk) & 255) as usize];

                    c[di as usize][dj as usize][dk as usize] = self.ran_vec[index as usize];
                }
            }
        }
        Self::perlin_interp(c, u, v, w)
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        c.iter().enumerate().for_each(|(i, ci)| {
            ci.iter().enumerate().for_each(|(j, cij)| {
                cij.iter().enumerate().for_each(|(k, cijk)| {
                    let weight = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1.0 - i as f64) * (1.0 - uu))
                        * (j as f64 * vv + (1.0 - j as f64) * (1.0 - vv))
                        * (k as f64 * ww + (1.0 - k as f64) * (1.0 - ww))
                        * cijk.dot(&weight);
                });
            });
        });
        accum
    }

    fn turbulence(&self, point: &Vec3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *point;
        let mut weight = 1.0;

        (0..depth).for_each(|_| {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        });
        accum.abs()
    }

    pub fn perlin_generate_perm() -> [i32; POINT_COUNT] {
        let mut p = [0i32; POINT_COUNT];
        p.iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as i32;
        });

        Self::permute(p, POINT_COUNT)
    }

    #[allow(clippy::manual_swap)] // Cannot borrow the same array twice. Don't feel like doing split_at_mut either.
    pub fn permute(mut perlins: [i32; POINT_COUNT], count: usize) -> [i32; POINT_COUNT] {
        for i in (count - 1)..0 {
            let target = random_int_range(0, i as i32);

            let tmp = perlins[i];
            perlins[i] = perlins[target as usize];
            perlins[target as usize] = tmp;
        }
        perlins
    }
}
