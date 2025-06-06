use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, DebuffAuraParams, AuraDebuffType};

pub fn define_sand_blast() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(18),
        name: "Sand Blast".to_string(),
        attack_data: AttackTypeData::DebuffAura(DebuffAuraParams {
            base_fire_rate_secs: 1.5,
            cloud_radius: 120.0,
            cloud_duration_secs: 2.0,
            cloud_color: Color::rgba(0.9, 0.8, 0.5, 0.5),
            visual_sprite_path: Some("sprites/sand_cloud_placeholder.png".to_string()),
            debuff_type: AuraDebuffType::ReduceAccuracy,
            debuff_magnitude: 0.20,
            debuff_duration_secs: 3.0,
        }),
    }
}
