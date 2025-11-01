use crate::{
    aabb::AABB,
    object::{Axis, Object},
};

const T_OBJ: f64 = 1.2;
const T_AABB: f64 = 1.0;

pub struct BvhNode {
    pub bbox: AABB,
    pub children: (i32, i32), //(left-id, right-id), when leaf, -1
    pub elements: Vec<usize>,
}

impl BvhNode {
    fn root(objs: &Vec<&Object>) -> Self {
        let elements = (0..objs.len()).collect();

        let bbox = AABB::entire_box(objs);
        BvhNode {
            bbox,
            children: (-1, -1),
            elements,
        }
    }

    fn new(bbox: AABB, children: (i32, i32), elements: Vec<usize>) -> Self {
        BvhNode {
            bbox,
            children,
            elements,
        }
    }
}

fn sort_objects(axis: &Axis, objs: &mut Vec<&Object>) {
    match axis {
        Axis::X => objs.sort_by(|o1, o2| o1.get_center().0.total_cmp(&o2.get_center().0)),
        Axis::Y => objs.sort_by(|o1, o2| o1.get_center().1.total_cmp(&o2.get_center().1)),
        Axis::Z => objs.sort_by(|o1, o2| o1.get_center().2.total_cmp(&o2.get_center().2)),
    }
}

pub type BvhTree = Vec<BvhNode>;

pub fn construct_bvh(scene_objs: &Vec<&Object>) -> BvhTree {
    let mut tree = vec![BvhNode::root(scene_objs)];
    let mut objects;

    for idx in 0.. {
        if idx > tree.len() - 1 {
            break;
        }

        objects = Vec::new();
        for i in tree[idx].elements.iter() {
            objects.push(scene_objs[*i]);
        }

        let size = objects.len();
        if size <= 1 {
            continue;
        }

        let mut s1_area = vec![0.; size - 1];
        let root_area_inv = 1. / tree[idx].bbox.get_area();
        let mut best_idx = -1;
        let mut best_axis = Axis::X;
        let mut best_cost = T_OBJ * size as f64;

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            sort_objects(&axis, &mut objects);
            let mut box1 = AABB::empty();
            let mut box2 = AABB::empty();

            for i in 0..size - 1 {
                box1 = box1 + objects[i].get_bbox();
                s1_area[i] = box1.get_area();
            }

            for i in (1..size).rev() {
                box2 = box2 + objects[i].get_bbox();
                let s2_area = box2.get_area();
                let cost = 2. * T_AABB
                    + (s1_area[i - 1] * i as f64 + s2_area * (size - i) as f64)
                        * T_OBJ
                        * root_area_inv;
                if cost < best_cost {
                    best_idx = i as i32;
                    best_axis = axis;
                    best_cost = cost;
                }
            }
        }
        if best_idx == -1 {
            continue;
        } else {
            sort_objects(&best_axis, &mut objects);
            let mut left = Vec::new();
            let mut right = Vec::new();
            for i in 0..best_idx as usize {
                left.push(objects[i]);
            }
            for i in best_idx as usize..size {
                right.push(objects[i]);
            }

            let left_node = BvhNode::new(
                AABB::entire_box(&left),
                (-1, -1),
                left.iter().map(|obj| obj.get_obj_id() as usize).collect(),
            );
            let right_node = BvhNode::new(
                AABB::entire_box(&right),
                (-1, -1),
                right.iter().map(|obj| obj.get_obj_id() as usize).collect(),
            );

            tree[idx].children = (tree.len() as i32, (tree.len() + 1) as i32);
            tree[idx].elements.clear();
            tree.push(left_node);
            tree.push(right_node);
        }
    }
    println!("constructed!, {}", tree.len());
    tree
}
