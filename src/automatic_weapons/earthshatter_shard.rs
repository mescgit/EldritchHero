use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, GroundTargetedAoEParams};

pub fn define_earthshatter_shard() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(11),
        name: "EarthShatter Shard".to_string(),
        attack_data: AttackTypeData::GroundTargetedAoE(GroundTargetedAoEParams {
            base_fire_rate_secs: 1.8,
            targeting_range: 400.0,
            reticle_sprite_path: Some("sprites/ground_target_reticle_placeholder.png".to_string()),
            visual_sprite_path: Some("sprites/eruption_effect_placeholder.png".to_string()),
            reticle_size: Vec2::new(64.0, 64.0),
            delay_before_eruption_secs: 0.5,
            eruption_radius: 80.0,
            damage: 45,
            aoe_color: Color::rgb(0.6, 0.4, 0.2),
            aoe_visual_duration_secs: 0.5,
            knock_up_strength: 100.0,
            root_duration_secs: None,
            fire_sound_effect: Some("audio/earthshatter_shard_fire.ogg".to_string()),
        }),
    }
}
