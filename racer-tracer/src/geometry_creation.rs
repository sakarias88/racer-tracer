use std::sync::Arc;

use crate::{
    geometry::{
        moving_sphere::MovingSphere, r#box::Boxx, rotate_y::RotateY, sphere::Sphere,
        translate::Translate, xy_rect::XyRect, xz_rect::XzRect, yz_rect::YzRect,
    },
    material::Material,
    scene::{HittableSceneObject, SceneObject},
    vec3::Vec3,
};

pub fn create_sphere(material: Arc<dyn Material>, pos: Vec3, radius: f64) -> SceneObject {
    let sphere = Sphere::new(radius);
    SceneObject::new(
        sphere.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(sphere),
    )
}

pub fn create_movable_sphere(
    material: Arc<dyn Material>,
    pos_a: Vec3,
    pos_b: Vec3,
    radius: f64,
    time_a: f64,
    time_b: f64,
) -> SceneObject {
    let moving_sphere = MovingSphere::new(pos_b, radius, time_a, time_b);
    SceneObject::new(
        moving_sphere.create_bounding_box(&pos_a, 0.0, 0.0),
        pos_a,
        material,
        Box::new(moving_sphere),
    )
}

pub fn create_xy_rect(
    material: Arc<dyn Material>,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
) -> SceneObject {
    let xy_rect = XyRect::new(x0, x1, y0, y1, k);
    let pos = Vec3::new(x0, y0, k);
    SceneObject::new(
        xy_rect.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(xy_rect),
    )
}

pub fn create_xz_rect(
    material: Arc<dyn Material>,
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    k: f64,
) -> SceneObject {
    let xz_rect = XzRect::new(x0, x1, z0, z1, k);
    let pos = Vec3::new(x0, k, z0);
    SceneObject::new(
        xz_rect.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(xz_rect),
    )
}

pub fn create_yz_rect(
    material: Arc<dyn Material>,
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    k: f64,
) -> SceneObject {
    let yz_rect = YzRect::new(y0, y1, z0, z1, k);
    let pos = Vec3::new(k, y0, z0);
    SceneObject::new(
        yz_rect.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(yz_rect),
    )
}

pub fn create_box(material: Arc<dyn Material>, min: Vec3, max: Vec3) -> SceneObject {
    let boxx = Boxx::new(min, max, Arc::clone(&material));
    SceneObject::new(
        boxx.create_bounding_box(&Vec3::default(), 0.0, 0.0),
        min,
        material,
        Box::new(boxx),
    )
}

pub fn create_translate(offset: Vec3, obj: SceneObject) -> SceneObject {
    let pos = obj.pos();
    let material = obj.material();
    let translate = Translate::new(offset, obj);
    SceneObject::new(
        translate.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(translate),
    )
}

pub fn create_rotate_y(rotate_deg: f64, obj: SceneObject) -> SceneObject {
    let pos = obj.pos();
    let material = obj.material();
    let rotate_y = RotateY::new(obj, rotate_deg);
    SceneObject::new(
        rotate_y.create_bounding_box(&pos, 0.0, 0.0),
        pos,
        material,
        Box::new(rotate_y),
    )
}
