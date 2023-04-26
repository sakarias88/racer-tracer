use std::option::Option;
use std::sync::Arc;

use crate::aabb::Aabb;
use crate::geometry::HitRecord;
use crate::ray::Ray;
use crate::scene::HittableSceneObject;
use crate::scene::SceneObject;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Sphere {
    radius: f64,
    aabb: Aabb,
}

impl Sphere {
    pub fn new(pos: &Vec3, radius: f64) -> Self {
        Self {
            radius,
            aabb: Aabb::new(
                pos - Vec3::new(radius, radius, radius),
                pos + Vec3::new(radius, radius, radius),
            ),
        }
    }
}

impl HittableSceneObject for Sphere {
    fn obj_hit(&self, obj: &SceneObject, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin() - obj.pos;
        let a = ray.direction().length_squared();
        let half_b = oc.dot(ray.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;

            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - obj.pos) / self.radius;

        let mut hit_record = HitRecord::new(point, root, Arc::clone(&obj.material));
        hit_record.set_face_normal(ray, outward_normal);
        Some(hit_record)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> &Aabb {
        &self.aabb
    }
}
