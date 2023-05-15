use crate::{aabb::Aabb, scene::HittableSceneObject, vec3::Vec3};

use super::HitRecord;

#[derive(Clone)]
pub struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
}

impl XyRect {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64) -> Self {
        Self { x0, x1, y0, y1, k }
    }
}

impl HittableSceneObject for XyRect {
    fn obj_hit(
        &self,
        obj: &crate::scene::SceneObject,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<HitRecord> {
        let origin = ray.origin();
        let direction = ray.direction();
        let t = (self.k - origin.z()) / direction.z();
        if t < t_min || t > t_max {
            return None;
        }

        let x = origin.x() + t * direction.x();
        let y = origin.y() + t * direction.y();

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let mut hit_record = HitRecord::new(ray.at(t), t, obj.material(), u, v);
        hit_record.set_face_normal(ray, Vec3::new(0.0, 0.0, 1.0));

        Some(hit_record)
    }

    fn create_bounding_box(&self, _pos: &Vec3, _time_a: f64, _time_b: f64) -> Aabb {
        Aabb::new(
            Vec3::new(self.x0, self.y0, self.k - 0.0001),
            Vec3::new(self.x1, self.y1, self.k + 0.0001),
        )
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.x0 += pos_delta.x();
        self.x1 += pos_delta.x();
        self.y0 += pos_delta.y();
        self.y1 += pos_delta.y();
    }
}
