use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChanneledBeamParams};

pub fn define_arcane_ray() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(6),
        name: "Arcane Ray".to_string(),
        attack_data: AttackTypeData::ChanneledBeam(ChanneledBeamParams {
            base_damage_per_tick: 5,
            tick_rate_secs: 0.15,
            range: 225.0,
            beam_width: 20.0,
            beam_color: Color::rgb(0.7, 0.2, 0.9),
            movement_penalty_multiplier: 0.6,
            max_duration_secs: Some(3.0),
            cooldown_secs: Some(5.0),
            is_automatic: true,
        }),
    }
}
