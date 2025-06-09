use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, RepositioningTetherParams, RepositioningTetherMode};

pub fn define_psionic_lash() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(22),
        name: "Psionic Lash".to_string(),
        attack_data: AttackTypeData::RepositioningTether(RepositioningTetherParams {
            base_fire_rate_secs: 1.0,
            tether_projectile_speed: 800.0,
            tether_range: 500.0,
            tether_sprite_path: "sprites/auto_psionic_lash.png".to_string(),
            tether_color: Color::rgb(0.8, 0.4, 0.9),
            tether_size: Vec2::new(8.0, 20.0),
            mode: RepositioningTetherMode::Alternate,
            pull_strength: 100.0,
            push_strength: 100.0,
            reactivation_window_secs: 1.5,
            effect_duration_secs: 0.2,
            fire_sound_effect: Some("assets/audio/psionic_lash_fire.ogg".to_string()),
        }),
    }
}
