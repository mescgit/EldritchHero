use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChanneledBeamParams};
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, ChanneledBeamField};

pub fn define_eldritch_gatling() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(1),
        name: "Eldritch Gatling".to_string(),
        attack_data: AttackTypeData::ChanneledBeam(ChanneledBeamParams {
            damage_per_tick: 2, // Renamed
            tick_interval_secs: 0.1, // Renamed
            beam_range: 400.0, // Renamed
            beam_width: 15.0,
            color: Color::rgb(0.3, 0.9, 0.4), // Renamed
            movement_penalty_multiplier: 0.7,
            max_duration_secs: None,
            cooldown_secs: None,
            is_automatic: false,
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![
        UpgradeCard {
            id: UpgradeId(20002),
            name: "Extended Gatling Beam".to_string(),
            description: "Increases the range of the Eldritch Gatling beam by 15%.".to_string(),
            upgrade_type: UpgradeType::ModifyChanneledBeam {
                weapon_id: AutomaticWeaponId(1), // ID for Eldritch Gatling
                field: ChanneledBeamField::Range,
                change_value: 0.15, // 15% increase
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
