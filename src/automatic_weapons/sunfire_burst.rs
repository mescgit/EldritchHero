use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ConeAttackParams}; // Vec2 and Color are covered by bevy::prelude::*
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, ConeAttackField};

pub fn define_sunfire_burst() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(12),
        name: "Sunfire Burst".to_string(),
        attack_data: AttackTypeData::ConeAttack(ConeAttackParams {
            base_damage: 20,
            base_fire_rate_secs: 0.9,
            cone_angle_degrees: 60.0,
            cone_radius: 120.0,
            color: Color::rgb(1.0, 0.8, 0.2),
            visual_sprite_path: Some("sprites/sunfire_burst_effect_placeholder.png".to_string()),
            visual_size_scale_with_radius_angle: Some((1.0, 0.5)),
            visual_anchor_offset: None,
            // Burn parameters
            applies_burn: Some(true),
            burn_damage_per_tick: Some(5),
            burn_duration_secs: Some(3.0),
            burn_tick_interval_secs: Some(0.5),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![
        UpgradeCard {
            id: UpgradeId(20007), // New unique ID
            name: "Wider Sunfire Cone".to_string(),
            description: "Increases the angle of the Sunfire Burst cone by 10%.".to_string(),
            upgrade_type: UpgradeType::ModifyConeAttack {
                weapon_id: AutomaticWeaponId(12), // ID for Sunfire Burst
                field: ConeAttackField::ConeAngleDegrees,
                change_value: 0.10, // 10% increase
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
