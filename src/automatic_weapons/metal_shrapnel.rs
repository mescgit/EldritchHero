use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, PersistentAuraParams};
use crate::upgrades::UpgradeCard; // Added import

pub fn define_metal_shrapnel() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(19),
        name: "Metal Shrapnel".to_string(),
        attack_data: AttackTypeData::PersistentAura(PersistentAuraParams {
            is_active_by_default: true,
            damage_per_tick: 2,
            tick_interval_secs: 0.25,
            radius: 75.0,
            aura_color: Color::rgba(0.6, 0.6, 0.6, 0.4),
            visual_sprite_path: Some("sprites/metal_shrapnel_aura_placeholder.png".to_string()),
            fire_rate_secs_placeholder: 0.25,
            activation_sound_effect: Some("audio/metal_shrapnel_activate.ogg".to_string()),
            deactivation_sound_effect: Some("audio/metal_shrapnel_deactivate.ogg".to_string()),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![]
}
