use crate::{
    aabb::Aabb,
    data_bus::DataReader,
    error::TracerError,
    geometry::Hittable,
    scene::{SceneObject, SceneObjectEvent},
    util::random_int_range,
};

pub enum Node {
    Leaf {
        obj: SceneObject,
    },
    Inner {
        left: Box<Node>,
        right: Box<Node>,
        aabb: Aabb,
    },
}

// Currently clones everything which works but is bad. Would need an
// implementation that is a bit more robust that supports taking out
// an element and let it trickle down through the structure again.
// Every time an object moves this will have to be re-computed which
// is the bad part apart from the cloning. Will leave this for later
// since the goal is to produce an image(s) and not a real time
// experience first hand.
//
// Note to self: axis aligned could be an idea.
impl Node {
    fn build(mut objects: Vec<&SceneObject>, time_a: f64, time_b: f64) -> Node {
        let axis = random_int_range(0, 2);
        let comparator = if axis == 0 {
            Node::box_x_compare
        } else if axis == 1 {
            Node::box_y_compare
        } else {
            Node::box_z_compare
        };

        let object_span = objects.len();
        if object_span == 1 {
            Node::Leaf {
                obj: objects[0].clone(),
            }
        } else if object_span == 2 {
            let (left_index, right_index) = if comparator(objects[0], objects[1]) {
                (0, 1)
            } else {
                (1, 0)
            };

            Node::Inner {
                left: Box::new(Node::Leaf {
                    obj: objects[left_index].clone(),
                }),
                right: Box::new(Node::Leaf {
                    obj: objects[right_index].clone(),
                }),
                aabb: (
                    objects[left_index].bounding_box(time_a, time_b),
                    objects[right_index].bounding_box(time_a, time_b),
                )
                    .into(),
            }
        } else {
            objects.sort_by(|a, b| match comparator(a, b) {
                true => std::cmp::Ordering::Less,
                false => std::cmp::Ordering::Greater,
            });
            let mid = object_span / 2;
            let left = Node::build(objects[0..mid].to_vec(), time_a, time_b);
            let right = Node::build(objects[mid..].to_vec(), time_a, time_b);
            let aabb: Aabb = (&Aabb::from(&left), &Aabb::from(&right)).into();

            Node::Inner {
                left: Box::new(left),
                right: Box::new(right),
                aabb,
            }
        }
    }

    fn box_compare(a: &SceneObject, b: &SceneObject, axis: usize) -> bool {
        let box_a = a.bounding_box(0.0, 0.0);
        let box_b = b.bounding_box(0.0, 0.0);
        box_a.min()[axis] < box_b.min()[axis]
    }

    fn box_x_compare(a: &SceneObject, b: &SceneObject) -> bool {
        Self::box_compare(a, b, 0)
    }

    fn box_y_compare(a: &SceneObject, b: &SceneObject) -> bool {
        Self::box_compare(a, b, 1)
    }

    fn box_z_compare(a: &SceneObject, b: &SceneObject) -> bool {
        Self::box_compare(a, b, 2)
    }
}

impl From<&Node> for Aabb {
    fn from(n: &Node) -> Self {
        match n {
            Node::Leaf { obj } => obj.bounding_box(0.0, 1.0).clone(),
            Node::Inner { aabb, .. } => aabb.clone(),
        }
    }
}

impl Hittable for Node {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<crate::geometry::HitRecord> {
        if !self.bounding_box(t_min, t_max).hit(ray, t_min, t_max) {
            return None;
        }
        match self {
            Node::Leaf { obj } => obj.hit(ray, t_min, t_max),
            Node::Inner { left, right, .. } => match left.hit(ray, t_min, t_max) {
                Some(r) => match right.hit(ray, t_min, r.t) {
                    Some(r2) => Some(r2),
                    None => Some(r),
                },
                None => right.hit(ray, t_min, t_max),
            },
        }
    }

    fn bounding_box(&self, time_a: f64, time_b: f64) -> &Aabb {
        match self {
            Node::Leaf { obj } => obj.bounding_box(time_a, time_b),
            Node::Inner { aabb, .. } => aabb,
        }
    }
}

pub struct BoundingVolumeHirearchy {
    reader: DataReader<SceneObjectEvent>,
    objects: Vec<SceneObject>,
    node: Node,
    time_a: f64,
    time_b: f64,
    changed: bool,
}

impl BoundingVolumeHirearchy {
    pub fn new(
        objects: Vec<SceneObject>,
        reader: DataReader<SceneObjectEvent>,
        time_a: f64,
        time_b: f64,
    ) -> Self {
        Self {
            reader,
            node: Node::build(
                objects.iter().collect::<Vec<&SceneObject>>(),
                time_a,
                time_b,
            ),
            objects,
            time_a,
            time_b,
            changed: true,
        }
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        self.changed = false;
        let res = self.reader.get_messages().and_then(|messages| {
            messages.into_iter().try_for_each(|action| {
                self.changed = true;
                match action {
                    SceneObjectEvent::Remove { id } => {
                        self.objects.remove(id.id);
                        Ok(())
                    }
                    SceneObjectEvent::Pos { id, pos } => {
                        if let Some(obj) = self.objects.get_mut(id.id) {
                            obj.set_pos(pos);
                        }
                        Ok(())
                    }
                }
            })
        });

        if self.changed {
            self.node = Node::build(
                self.objects.iter().collect::<Vec<&SceneObject>>(),
                self.time_a,
                self.time_b,
            );
        }

        res
    }
}

impl Hittable for BoundingVolumeHirearchy {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<crate::geometry::HitRecord> {
        self.node.hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, time_a: f64, time_b: f64) -> &Aabb {
        self.node.bounding_box(time_a, time_b)
    }
}
