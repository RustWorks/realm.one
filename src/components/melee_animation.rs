use amethyst::{
    ecs::{Component, DenseVecStorage, FlaggedStorage, Entity},
    core::transform::Transform,
};


use crate::components::{LifeformComponent};

#[derive(Debug, Clone, PartialEq)]
pub struct MeleeAnimation {
    end_stance: usize,
    at_stance: usize,
    pub sword_spr: usize,
    pub sword_pos: Transform,
    swing_time: f32,
    elapsed_time: f32,
    pec: f32,
    pub sword: Option<Entity>,
}

impl MeleeAnimation {
    pub fn new(pl: &LifeformComponent) -> Self { 
        Self {
            end_stance: pl.get_dir(),
            at_stance: pl.get_at(),
            sword_spr: pl.get_sword(),
            sword_pos: pl.get_sword_pos(),
            swing_time: 0.25,
            elapsed_time: 0.0,
            pec: 0.0,
            sword: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        self.pec = self.elapsed_time / self.swing_time;
    }
    
    /// Get the current animation sequence
    pub fn get_seq(&mut self) -> Option<usize> {
        //TODO: RATE LIMIT THIS (DON'T ALWAYS RETURN SOME)
        match self.pec > 0.9 {
            true => Some(self.end_stance),
            false => Some(self.at_stance),
        }
    }

    pub fn delete(&self) -> bool {
        self.pec >= 1.0
    }
}

impl Component for MeleeAnimation {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
