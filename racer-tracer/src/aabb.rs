use crate::{ray::Ray, vec3::Vec3};

#[derive(Clone, Default)]
pub struct Aabb {
    minimum: Vec3,
    maximum: Vec3,
}

impl Aabb {
    pub fn new(vec_a: Vec3, vec_b: Vec3) -> Self {
        // People can do weird things (speaking of myself) such as
        // spheres with negative radius etc. Just doing this to be
        // really nice.
        let minimum = Vec3::new(
            vec_a.x().min(*vec_b.x()),
            vec_a.y().min(*vec_b.y()),
            vec_a.z().min(*vec_b.z()),
        );
        let maximum = Vec3::new(
            vec_a.x().max(*vec_b.x()),
            vec_a.y().max(*vec_b.y()),
            vec_a.z().max(*vec_b.z()),
        );

        Self { minimum, maximum }
    }

    pub fn min(&self) -> &Vec3 {
        &self.minimum
    }

    pub fn max(&self) -> &Vec3 {
        &self.maximum
    }

    // Fastest
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction()[a];
            let mut t0 = (self.minimum[a] - ray.origin()[a]) * inv_d;
            let mut t1 = (self.maximum[a] - ray.origin()[a]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            let min = if t0 > t_min { t0 } else { t_min };
            let max = if t1 < t_max { t1 } else { t_max };

            if max <= min {
                return false;
            }
        }
        true
    }

    // Faster
    pub fn hit_b(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction()[a];
            let mut t0 = (self.minimum[a] - ray.origin()[a]) * inv_d;
            let mut t1 = (self.maximum[a] - ray.origin()[a]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            if t1.min(t_max) <= t0.max(t_min) {
                return false;
            }
        }
        true
    }

    // Slow
    pub fn hit_c(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        for a in 0..3 {
            let va = (self.minimum[a] - ray.origin()[a]) / ray.direction()[a];
            let vb = (self.maximum[a] - ray.origin()[a]) / ray.direction()[a];
            let t0: f64 = va.min(vb);
            let t1: f64 = va.max(vb);
            let min = t0.min(t_min);
            let max = t1.max(t_max);

            if max <= min {
                return false;
            }
        }
        true
    }
}

impl From<(&Aabb, &Aabb)> for Aabb {
    fn from((box_a, box_b): (&Aabb, &Aabb)) -> Self {
        let mina = box_a.min();
        let maxa = box_a.max();
        let minb = box_b.min();
        let maxb = box_b.max();
        Aabb {
            minimum: Vec3::new(
                mina[0].min(minb[0]),
                mina[1].min(minb[1]),
                mina[2].min(minb[2]),
            ),
            maximum: Vec3::new(
                maxa[0].max(maxb[0]),
                maxa[1].max(maxb[1]),
                maxa[2].max(maxb[2]),
            ),
        }
    }
}
