use std::sync::Arc;

use crate::{
    aabb::Aabb,
    geometry_creation::{create_xy_rect, create_xz_rect, create_yz_rect},
    material::Material,
    scene::{HittableSceneObject, SceneObject},
    vec3::Vec3,
};

use super::Hittable;

#[derive(Clone)]
pub struct Boxx {
    box_min: Vec3,
    box_max: Vec3,
    sides: Vec<SceneObject>,
}

impl Boxx {
    pub fn new(box_min: Vec3, box_max: Vec3, material: Arc<dyn Material>) -> Self {
        let sides = vec![
            create_xy_rect(
                Arc::clone(&material),
                *box_min.x(),
                *box_max.x(),
                *box_min.y(),
                *box_max.y(),
                *box_max.z(),
            ),
            create_xy_rect(
                Arc::clone(&material),
                *box_min.x(),
                *box_max.x(),
                *box_min.y(),
                *box_max.y(),
                *box_min.z(),
            ),
            create_xz_rect(
                Arc::clone(&material),
                *box_min.x(),
                *box_max.x(),
                *box_min.z(),
                *box_max.z(),
                *box_max.y(),
            ),
            create_xz_rect(
                Arc::clone(&material),
                *box_min.x(),
                *box_max.x(),
                *box_min.z(),
                *box_max.z(),
                *box_min.y(),
            ),
            create_yz_rect(
                Arc::clone(&material),
                *box_min.y(),
                *box_max.y(),
                *box_min.z(),
                *box_max.z(),
                *box_max.x(),
            ),
            create_yz_rect(
                Arc::clone(&material),
                *box_min.y(),
                *box_max.y(),
                *box_min.z(),
                *box_max.z(),
                *box_min.x(),
            ),
        ];

        Self {
            box_min,
            box_max,
            sides,
        }
    }
}

impl HittableSceneObject for Boxx {
    fn obj_hit(
        &self,
        _obj: &crate::scene::SceneObject,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<super::HitRecord> {
        let mut rec = None;
        let mut closes_so_far = t_max;

        for obj in self.sides.iter() {
            if let Some(hit_rec) = obj.hit(ray, t_min, closes_so_far) {
                closes_so_far = hit_rec.t;
                rec = Some(hit_rec);
            }
        }

        rec
    }

    fn create_bounding_box(
        &self,
        _pos: &crate::vec3::Vec3,
        _time_a: f64,
        _time_b: f64,
    ) -> crate::aabb::Aabb {
        Aabb::new(self.box_min, self.box_max)
    }

    fn update_pos(&mut self, pos_delta: &Vec3) {
        self.sides[0].update_pos(pos_delta);
        self.sides[1].update_pos(pos_delta);
        self.sides[2].update_pos(pos_delta);
        self.sides[3].update_pos(pos_delta);
        self.sides[4].update_pos(pos_delta);
        self.sides[5].update_pos(pos_delta);
        self.box_min += pos_delta;
        self.box_max += pos_delta;
    }
}
