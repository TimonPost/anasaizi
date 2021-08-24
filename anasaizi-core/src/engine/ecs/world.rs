use hecs::Entity;
use nalgebra::Vector4;
use std::ops::{Deref, DerefMut};

pub struct World {
    world: hecs::World,
    last_click_world_space: Option<Vector4<f32>>,
}

impl World {
    pub(crate) fn set_last_click(&mut self, world_space: Vector4<f32>) {
        self.last_click_world_space = Some(world_space);
    }

    pub fn last_click(&self) -> Option<Vector4<f32>> {
        self.last_click_world_space
    }
}

impl World {
    pub fn new() -> World {
        World {
            world: hecs::World::new(),
            last_click_world_space: None,
        }
    }
}

impl Deref for World {
    type Target = hecs::World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}
