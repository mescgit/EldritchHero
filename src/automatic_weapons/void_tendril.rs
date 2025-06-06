use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ConeAttackParams};

pub fn define_void_tendril() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(15),
        name: "Void Tendril".to_string(),
        attack_data: AttackTypeData::ConeAttack(ConeAttackParams {
            base_damage: 18,
            base_fire_rate_secs: 0.65,
            cone_angle_degrees: 150.0,
            cone_radius: 100.0,
            color: Color::rgb(0.3, 0.0, 0.5),
            visual_sprite_path: Some("sprites/void_tendril_sweep_placeholder.png".to_string()),
            visual_size_scale_with_radius_angle: Some((1.0, 0.8)),
            visual_anchor_offset: Some(Vec2::new(0.0, 20.0)),
        }),
    }
}
