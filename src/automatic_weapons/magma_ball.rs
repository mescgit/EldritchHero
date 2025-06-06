use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, LobbedBouncingMagmaParams};

pub fn define_magma_ball() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(17),
        name: "Magma Ball".to_string(),
        attack_data: AttackTypeData::LobbedBouncingMagma(LobbedBouncingMagmaParams::default()),
    }
}
