use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, StandardProjectileParams}; // Vec2 and Color are covered by bevy::prelude::*
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, StandardProjectileField};

pub fn define_venom_spit() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(9),
        name: "Venom Spit".to_string(),
        attack_data: AttackTypeData::StandardProjectile(StandardProjectileParams {
            base_damage: 10,
            base_fire_rate_secs: 0.4,
            projectile_speed: 500.0, // Renamed from base_projectile_speed
            piercing: 0, // Renamed from base_piercing
            additional_projectiles: 2,
            projectile_sprite_path: "sprites/auto_venom_spit.png".to_string(),
            projectile_size: Vec2::new(15.0, 15.0),
            projectile_color: Color::rgb(0.2, 0.8, 0.1),
            projectile_lifetime_secs: 1.8,
            fire_sound_effect: Some("assets/audio/venom_spit_fire.ogg".to_string()),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![
        UpgradeCard {
            id: UpgradeId(20005), // New unique ID
            name: "Extra Venom Projectile".to_string(),
            description: "Adds one additional projectile to Venom Spit.".to_string(),
            upgrade_type: UpgradeType::ModifyStandardProjectile {
                weapon_id: AutomaticWeaponId(9), // ID for Venom Spit
                field: StandardProjectileField::AdditionalProjectiles,
                change_value: 1.0, // Add 1 projectile
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
        UpgradeCard {
            id: UpgradeId(20006), // New unique ID
            name: "Faster Venom Bolts".to_string(),
            description: "Increases Venom Spit projectile speed by 15%.".to_string(),
            upgrade_type: UpgradeType::ModifyStandardProjectile {
                weapon_id: AutomaticWeaponId(9),
                field: StandardProjectileField::BaseProjectileSpeed,
                change_value: 0.15, // 15% increase
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
