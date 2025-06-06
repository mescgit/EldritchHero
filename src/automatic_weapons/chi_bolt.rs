use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, LifestealProjectileParams};

pub fn define_chi_bolt() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(21),
        name: "Chi Bolt".to_string(),
        attack_data: AttackTypeData::LifestealProjectile(LifestealProjectileParams {
            base_fire_rate_secs: 0.45,
            base_damage: 18,
            projectile_speed: 750.0,
            projectile_sprite_path: "sprites/auto_chi_bolt.png".to_string(),
            projectile_size: Vec2::new(20.0, 20.0),
            projectile_color: Color::rgb(0.5, 0.9, 0.8),
            projectile_lifetime_secs: 1.5,
            piercing: 0,
            lifesteal_percentage: 0.10,
        }),
    }
}
