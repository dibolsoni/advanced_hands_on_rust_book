mod aabb;
mod rect2d;
mod static_quadtree;
pub use aabb::AxisAlignedBoundingBox;
pub use rect2d::Rect2D;
pub use static_quadtree::*;
use bevy::{prelude::*, platform::collections::HashMap};
use std::marker::PhantomData;

#[derive(Message)]
pub struct OnCollision<A, B>// (1)
where// (2)
  A: Component,
  B: Component,
{
  pub entity_a: Entity,// (3)
  pub entity_b: Entity,
  marker: PhantomData<(A, B)>,// (4)
}

pub fn check_collisions<A, B>(// (5)
  quad_tree: Res<StaticQuadTree>,// (6)
  query_a: Query<(Entity, &Transform, &AxisAlignedBoundingBox), With<A>>,// (7)
  query_b: Query<(Entity, &Transform, &AxisAlignedBoundingBox), With<B>>,
  mut sender: MessageWriter<OnCollision<A, B>>,// (8)
) where
  A: Component,
  B: Component,
{
  let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> =// (9)
    HashMap::new();

  query_b.iter().for_each(|(entity, transform, bbox)| {
    let bbox = bbox.as_rect(transform.translation.truncate());
    let in_node = quad_tree.smallest_node(&bbox);
    if let Some(contents) = spatial_index.get_mut(&in_node) {
      contents.push((entity, bbox));
    } else {
      spatial_index.insert(in_node, vec![(entity, bbox)]);
    }
  });

  query_a.iter().for_each(|(entity_a, transform_a, bbox_a)| {
    let bbox_a = bbox_a.as_rect(transform_a.translation.truncate());
    for node in quad_tree.intersecting_nodes(&bbox_a) {
      if let Some(contents) = spatial_index.get(&node) {
        for (entity_b, bbox_b) in contents {
          if entity_a != *entity_b && bbox_a.intersect(bbox_b) {
            sender.write(OnCollision {// (10)
              entity_a,
              entity_b: *entity_b,
              marker: PhantomData,// (11)
            });
          }
        }
      }
    }
  });
}
