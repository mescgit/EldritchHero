use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ExpandingEnergyBombParams};
use crate::upgrades::UpgradeCard; // Added import

pub fn define_spirit_bomb() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(14),
        name: "Spirit Bomb".to_string(),
        attack_data: AttackTypeData::ExpandingEnergyBomb(ExpandingEnergyBombParams {
            base_fire_rate_secs: 2.5,
            max_radius: 300.0,
            expansion_duration_secs: 3.0,
            min_damage_at_min_radius: 20,
            max_damage_at_max_radius: 100,
            bomb_color: Color::rgba(0.6, 1.0, 0.9, 0.6),
            visual_sprite_path: Some("sprites/spirit_bomb_effect_placeholder.png".to_string()),
            detonation_can_be_manual: true,
            auto_detonation_delay_after_max_expansion_secs: 1.0,
            launch_sound_effect: Some("audio/spirit_bomb_launch.ogg".to_string()),
            detonation_sound_effect: Some("audio/spirit_bomb_detonate.ogg".to_string()),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![]
}
