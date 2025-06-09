use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, PointBlankNovaParams};

pub fn define_glacial_spike() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(10),
        name: "Glacial Spike".to_string(),
        attack_data: AttackTypeData::PointBlankNova(PointBlankNovaParams {
            base_fire_rate_secs: 0.9,
            damage: 22,
            radius: 150.0,
            nova_color: Color::rgba(0.4, 0.7, 1.0, 0.7),
            visual_duration_secs: 0.3,
            slow_effect_multiplier: 0.5,
            slow_duration_secs: 2.0,
            fire_sound_effect: Some("audio/glacial_spike_nova.ogg".to_string()),
        }),
    }
}
