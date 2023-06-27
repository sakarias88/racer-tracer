use crate::{
    aabb::Aabb,
    geometry::Hittable,
    ray::Ray,
    scene::{HittableSceneObject, SceneObject},
    vec3::Vec3,
};

use super::HitRecord;

#[derive(Clone)]
pub struct Translate {
    object: SceneObject,
    offset: Vec3,
}

impl Translate {
    pub fn new(offset: Vec3, object: SceneObject) -> Self {
        Self { object, offset }
    }
}

impl HittableSceneObject for Translate {
    fn obj_hit(
        &self,
        _obj: &SceneObject,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
        _obj_id: usize,
    ) -> Option<HitRecord> {
        let moved = Ray::new(ray.origin() - self.offset, *ray.direction(), ray.time());
        match self.object.hit(&moved, t_min, t_max) {
            Some(mut record) => {
                record.point += self.offset;
                record.set_face_normal(&moved, record.normal);
                Some(record)
            }
            None => None,
        }
    }

    fn create_bounding_box(&self, _pos: &Vec3, _time_a: f64, _time_b: f64) -> crate::aabb::Aabb {
        let obj_aabb = self.object.aabb();
        Aabb::new(obj_aabb.min() + self.offset, obj_aabb.max() + self.offset)
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.offset += pos_delta;
    }
}
