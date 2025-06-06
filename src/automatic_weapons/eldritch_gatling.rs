use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChanneledBeamParams};
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, ChanneledBeamField};

pub fn define_eldritch_gatling() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(1),
        name: "Eldritch Gatling".to_string(),
        attack_data: AttackTypeData::ChanneledBeam(ChanneledBeamParams {
            base_damage_per_tick: 2,
            tick_rate_secs: 0.1,
            range: 400.0,
            beam_width: 15.0,
            beam_color: Color::rgb(0.3, 0.9, 0.4),
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
