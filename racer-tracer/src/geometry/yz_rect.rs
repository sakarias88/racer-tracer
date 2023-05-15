use crate::{aabb::Aabb, scene::HittableSceneObject, vec3::Vec3};

use super::HitRecord;

#[derive(Clone)]
pub struct YzRect {
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
}

impl YzRect {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64) -> Self {
        Self { y0, y1, z0, z1, k }
    }
}

impl HittableSceneObject for YzRect {
    fn obj_hit(
        &self,
        obj: &crate::scene::SceneObject,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<HitRecord> {
        let origin = ray.origin();
        let direction = ray.direction();
        let t = (self.k - origin.x()) / direction.x();
        if t < t_min || t > t_max {
            return None;
        }

        let y = origin.y() + t * direction.y();
        let z = origin.z() + t * direction.z();

        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let u = (y - self.y0) / (self.y1 - self.y0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        let mut hit_record = HitRecord::new(ray.at(t), t, obj.material(), u, v);
        hit_record.set_face_normal(ray, Vec3::new(1.0, 0.0, 0.0));

        Some(hit_record)
    }

    fn create_bounding_box(&self, _pos: &Vec3, _time_a: f64, _time_b: f64) -> Aabb {
        Aabb::new(
            Vec3::new(self.k - 0.0001, self.y0, self.z0),
            Vec3::new(self.k + 0.0001, self.y1, self.z1),
        )
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.y0 += pos_delta.y();
        self.y1 += pos_delta.y();
        self.z0 += pos_delta.z();
        self.z1 += pos_delta.z();
    }
}
