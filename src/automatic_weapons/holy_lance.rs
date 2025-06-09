use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, LineDashAttackParams};

pub fn define_holy_lance() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(8),
        name: "Holy Lance".to_string(),
        attack_data: AttackTypeData::LineDashAttack(LineDashAttackParams {
            base_fire_rate_secs: 1.2,
            dash_speed: 900.0,
            dash_duration_secs: 0.25,
            damage_per_hit: 30,
            hitbox_width: 40.0,
            piercing_cap: 5,
            dash_trail_color: Some(Color::rgba(1.0, 1.0, 0.7, 0.5)),
            invulnerable_during_dash: true,
            fire_sound_effect: Some("audio/holy_lance_fire.ogg".to_string()),
        }),
    }
}
