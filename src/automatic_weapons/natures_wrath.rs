use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, GroundTargetedAoEParams};

pub fn define_natures_wrath() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(20),
        name: "Nature's Wrath".to_string(),
        attack_data: AttackTypeData::GroundTargetedAoE(GroundTargetedAoEParams {
            base_fire_rate_secs: 1.1,
            targeting_range: 350.0,
            reticle_sprite_path: Some("sprites/nature_reticle_placeholder.png".to_string()),
            visual_sprite_path: Some("sprites/nature_eruption_placeholder.png".to_string()),
            reticle_size: Vec2::new(80.0, 80.0),
            delay_before_eruption_secs: 0.4,
            eruption_radius: 80.0,
            damage: 5,
            aoe_color: Color::rgb(0.1, 0.6, 0.2),
            aoe_visual_duration_secs: 0.6,
            knock_up_strength: 0.0,
            root_duration_secs: Some(2.5),
            fire_sound_effect: Some("audio/natures_wrath_fire.ogg".to_string()),
        }),
    }
}
