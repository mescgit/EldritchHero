use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ReturningProjectileParams};
use crate::upgrades::{UpgradeCard, UpgradeType, UpgradeRarity, UpgradeId, ReturningProjectileField};

pub fn define_spectral_blades() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(3),
        name: "Spectral Blades".to_string(),
        attack_data: AttackTypeData::ReturningProjectile(ReturningProjectileParams {
            base_damage: 12,
            base_fire_rate_secs: 0.75,
            projectile_sprite_path: "sprites/spectral_blade_placeholder.png".to_string(),
            projectile_size: Vec2::new(50.0, 50.0),
            projectile_color: Color::rgb(0.6, 0.9, 1.0),
            projectile_speed: 400.0,
            travel_distance: 300.0,
            piercing: 999,
            fire_sound_effect: Some("assets/audio/spectral_blades_fire.ogg".to_string()),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![
        UpgradeCard {
            id: UpgradeId(20003),
            name: "Far-Reaching Blades".to_string(),
            description: "Increases the travel distance of Spectral Blades by 20%.".to_string(),
            upgrade_type: UpgradeType::ModifyReturningProjectile {
                weapon_id: AutomaticWeaponId(3), // ID for Spectral Blades
                field: ReturningProjectileField::TravelDistance,
                change_value: 0.20, // 20% increase
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
