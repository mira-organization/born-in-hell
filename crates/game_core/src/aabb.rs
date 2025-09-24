#![coverage(off)]

use bevy::prelude::*;

//axis aligned bounding box
#[derive(Default,Clone)]
pub struct AABB {
    pub center : Vec2,
    pub half_size : Vec2,
}

impl AABB {
    pub fn new(center : Vec2,size : Vec2) -> Self {
        Self {
            center,half_size: size
        }
    }
    pub fn right(&self) -> f32 {
        self.center.x + self.half_size.x
    }
    pub fn left(&self) -> f32 {
        self.center.x - self.half_size.x
    }
    pub fn top(&self) -> f32 {
        self.center.y + self.half_size.y
    }
    pub fn bottom(&self) -> f32 {
        self.center.y - self.half_size.y
    }
    pub fn get_intersection_depth(
        &self,
        aabb : &AABB
    ) -> Vec2 {
        let min_dist = self.half_size + aabb.half_size;
        let dist = self.center - aabb.center;
        let mut depth = Vec2::ZERO;

        if dist.x.abs() >= min_dist.x || dist.y.abs() >= min_dist.y {
            return depth;
        }

        if dist.x > 0. {
            //self is on the right and intersects negatively
            depth.x = min_dist.x - dist.x;
        }
        else {
            //self is on the left and intersects positively
            depth.x = -dist.x - min_dist.x;
        }

        if dist.y > 0. {
            //self is on top
            depth.y = min_dist.y - dist.y;
        }
        else {
            //self is on the bottom
            depth.y = -dist.y - min_dist.y;
        }

        return depth;
    }
}