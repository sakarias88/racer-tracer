use crate::{aabb::Aabb, scene::HittableSceneObject, vec3::Vec3};

use super::HitRecord;

#[derive(Clone)]
pub struct XzRect {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
}

impl XzRect {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64) -> Self {
        Self { x0, x1, z0, z1, k }
    }
}

impl HittableSceneObject for XzRect {
    fn obj_hit(
        &self,
        obj: &crate::scene::SceneObject,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
        obj_id: usize,
    ) -> Option<HitRecord> {
        let origin = ray.origin();
        let direction = ray.direction();
        let t = (self.k - origin.y()) / direction.y();
        if t < t_min || t > t_max {
            return None;
        }

        let x = origin.x() + t * direction.x();
        let z = origin.z() + t * direction.z();

        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        let mut hit_record = HitRecord::new(ray.at(t), t, obj.material(), u, v, obj_id);
        hit_record.set_face_normal(ray, Vec3::new(0.0, 1.0, 0.0));

        Some(hit_record)
    }

    fn create_bounding_box(&self, _pos: &Vec3, _time_a: f64, _time_b: f64) -> Aabb {
        Aabb::new(
            Vec3::new(self.x0, self.k - 0.0001, self.z0),
            Vec3::new(self.x1, self.k + 0.0001, self.z1),
        )
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.x0 += pos_delta.x();
        self.x1 += pos_delta.x();
        self.z0 += pos_delta.z();
        self.z1 += pos_delta.z();
    }
}
