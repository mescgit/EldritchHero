use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, LobbedAoEPoolParams};
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, LobbedAoEPoolField};

pub fn define_primordial_ichor_blast() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(0),
        name: "Primordial Ichor Blast".to_string(),
        attack_data: AttackTypeData::LobbedAoEPool(LobbedAoEPoolParams {
            base_damage_on_impact: 5,
            pool_damage_per_tick: 3,
            base_fire_rate_secs: 0.6,
            projectile_speed: 400.0,
            projectile_sprite_path: "sprites/ichor_blast_placeholder.png".to_string(),
            projectile_size: Vec2::new(30.0, 30.0),
            projectile_color: Color::rgb(0.7, 0.5, 1.0),
            projectile_arc_height: 50.0,
            pool_radius: 100.0,
            pool_duration_secs: 3.0,
            pool_tick_interval_secs: 0.5,
            pool_color: Color::rgba(0.5, 0.3, 0.8, 0.5),
            max_active_pools: 3,
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![
        UpgradeCard {
            id: UpgradeId(20000),
            name: "Wider Ichor Pool".to_string(),
            description: "Increases the radius of the Ichor Blast pool by 10%.".to_string(),
            upgrade_type: UpgradeType::ModifyLobbedAoEPool {
                weapon_id: AutomaticWeaponId(0), // ID for Primordial Ichor Blast
                field: LobbedAoEPoolField::PoolRadius,
                change_value: 0.10, // 10% increase
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
        UpgradeCard {
            id: UpgradeId(20001),
            name: "Lingering Ichor".to_string(),
            description: "Increases the duration of the Ichor Blast pool by 0.5 seconds.".to_string(),
            upgrade_type: UpgradeType::ModifyLobbedAoEPool {
                weapon_id: AutomaticWeaponId(0),
                field: LobbedAoEPoolField::PoolDurationSecs,
                change_value: 0.5, // 0.5 seconds flat increase
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
        UpgradeCard {
            id: UpgradeId(20004), // New unique ID
            name: "Potent Ichor".to_string(),
            description: "Increases the damage per tick of the Ichor Blast pool by 2.".to_string(),
            upgrade_type: UpgradeType::ModifyLobbedAoEPool {
                weapon_id: AutomaticWeaponId(0),
                field: LobbedAoEPoolField::PoolDamagePerTick,
                change_value: 2.0, // Flat increase of 2
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
