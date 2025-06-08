use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChanneledBeamParams};

pub fn define_arcane_ray() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(6),
        name: "Arcane Ray".to_string(),
        attack_data: AttackTypeData::ChanneledBeam(ChanneledBeamParams {
            damage_per_tick: 5, // Renamed from base_damage_per_tick
            tick_interval_secs: 0.15, // Renamed from tick_rate_secs
            beam_range: 225.0, // Renamed from range
            beam_width: 20.0,
            color: Color::rgb(0.7, 0.2, 0.9), // Renamed from beam_color
            movement_penalty_multiplier: 0.6,
            max_duration_secs: Some(3.0),
            cooldown_secs: Some(5.0),
            is_automatic: true,
        }),
    }
}
