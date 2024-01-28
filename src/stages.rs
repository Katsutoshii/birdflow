use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};

/// Stage of computation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SystemStage {
    Spawn,
    PreCompute,
    Compute,
    Apply,
    PostApply,
    Despawn,
}
impl SystemStage {
    pub fn get_config() -> SystemSetConfigs {
        (
            Self::Spawn,
            Self::PreCompute,
            Self::Compute,
            Self::Apply,
            Self::PostApply,
            Self::Despawn,
        )
            .chain()
    }
}
