use crate::bouncy::{bounce_on_collision, CollisionTime};
use crate::bouncy_bbox::AxisAlignedBoundingBox;
use crate::Rect2D;
use bevy::prelude::{Entity, MessageWriter, Query, Res, ResMut, Resource, Transform, Vec2};
use my_library::bevy_framework::Impulse;
use std::collections::{HashMap, HashSet};

pub const QUAD_TREE_DEPTH: usize = 1;

#[derive(Debug)]
pub struct StaticQuadTreeNode {
    bounds: Rect2D,
    children: Option<[usize; 4]>,
}

#[derive(Debug, Resource)]
pub struct StaticQuadTree {
    nodes: Vec<StaticQuadTreeNode>,
}

impl StaticQuadTree {
    pub fn new(screen_size: Vec2, max_depth: usize) -> Self {
        let mut nodes = Vec::new();

        let half = screen_size / 2.0;
        let top = StaticQuadTreeNode {
            bounds: Rect2D::new(
                Vec2::new(0.0 - half.x, 0.0 - half.y),
                Vec2::new(half.x, half.y),
            ),
            children: None,
        };
        nodes.push(top);
        Self::subdivide(&mut nodes, 0, 1, max_depth);
        Self { nodes }
    }

    fn subdivide(
        nodes: &mut Vec<StaticQuadTreeNode>,
        index: usize,
        depth: usize,
        max_depth: usize,
    ) {
        let mut children = nodes[index].bounds.quadrants();
        let child_index = [
            nodes.len(),
            nodes.len() + 1,
            nodes.len() + 2,
            nodes.len() + 3,
        ];
        nodes[index].children = Some(child_index);
        children.drain(0..4).for_each(|quad| {
            nodes.push(StaticQuadTreeNode {
                bounds: quad,
                children: None,
            })
        });

        if depth < max_depth {
            for index in child_index {
                Self::subdivide(nodes, index, depth + 1, max_depth);
            }
        }
    }

    fn smallest_node(&self, target: &Rect2D) -> usize {
        let mut current_index = 0;

        #[allow(clippy::while_let_loop)]
        loop {
            if let Some(children) = self.nodes[current_index].children {
                let matches: Vec<usize> = children
                    .iter()
                    .filter_map(|child| {
                        if self.nodes[*child].bounds.intersect(target) {
                            Some(*child)
                        } else {
                            None
                        }
                    })
                    .collect();

                if matches.len() == 1 {
                    current_index = matches[0];
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        current_index
    }

    fn intersecting_nodes(&self, target: &Rect2D) -> HashSet<usize> {
        let mut result = HashSet::new();
        self.intersect(0, &mut result, target);
        result
    }

    fn intersect(
        &self,
        index: usize,
        result: &mut HashSet<usize>,
        target: &Rect2D,
    ) {
        if self.nodes[index].bounds.intersect(target) {
            result.insert(index);
            if let Some(children) = &self.nodes[index].children {
                for child in children {
                    self.intersect(*child, result, target);
                }
            }
        }
    }

}

pub fn collision_bbox_quadtree(
    mut collision_time: ResMut<CollisionTime>,
    query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>,
    mut impulse: MessageWriter<Impulse>,
    quad_tree: Res<StaticQuadTree>,
) {
    // Start the clock
    let now = std::time::Instant::now();

    let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> =
        HashMap::new();// (17)

    let tree_positions: Vec<(Entity, usize, Rect2D)> = query// (18)
        .iter()
        .map(|(entity, transform, bbox)| {
            let bbox = bbox.as_rect(transform.translation.truncate());// (19)
            let node = quad_tree.smallest_node(&bbox);// (20)
            for in_node in quad_tree.intersecting_nodes(&bbox) {// (21)
                if let Some(contents) = spatial_index.get_mut(&in_node) {// (22)
                    contents.push((entity, bbox.clone()));
                } else {
                    spatial_index.insert(in_node, vec![(entity, bbox.clone())]);
                }
            }

            (entity, node, bbox)
        })
        .collect();// (23)
    let mut n = 0;

    for (entity, node, box_a) in tree_positions {
        if let Some(entities_here) = spatial_index.get(&node) {
            if let Some((entity_b, _)) = entities_here
                .iter()
                .filter(|(entity_b, _)| *entity_b != entity)
                .find(|(_, box_b)| {
                    n += 1;
                    box_a.intersect(box_b)
                })
            {
                // A Collision occurred
                let (_, ball_a, _) = query.get(entity).unwrap();
                let (_, ball_b, _) = query.get(*entity_b).unwrap();
                bounce_on_collision(entity, ball_a.translation,
                                    ball_b.translation, &mut impulse);
            }
        }
    }

    // Store the time result
    collision_time.time = now.elapsed().as_millis();
    collision_time.checks = n;
}
