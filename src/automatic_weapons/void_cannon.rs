use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChargeUpEnergyShotParams, ChargeLevelParams};

pub fn define_void_cannon() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(2),
        name: "Void Cannon".to_string(),
        attack_data: AttackTypeData::ChargeUpEnergyShot(ChargeUpEnergyShotParams {
            base_fire_rate_secs: 1.25,
            base_projectile_sprite_path: "sprites/void_cannon_projectile_placeholder.png".to_string(),
            base_projectile_color: Color::rgb(0.4, 0.1, 0.7),
            projectile_lifetime_secs: 2.5,
            charge_levels: vec![
                ChargeLevelParams {
                    charge_time_secs: 0.01,
                    damage: 10, // Renamed from projectile_damage
                    projectile_speed: 500.0,
                    projectile_size: Vec2::new(25.0, 25.0),
                    piercing: 0,
                    explodes_on_impact: false,
                    explosion_radius: 0.0,
                    explosion_damage: 0,
                    projectile_sprite_path: "".to_string(), // Use base_projectile_sprite_path from parent
                    projectile_color: Color::WHITE, // Use base_projectile_color from parent
                    aoe_radius_on_impact: None, // Added
                },
                ChargeLevelParams {
                    charge_time_secs: 0.75,
                    damage: 25, // Renamed
                    projectile_speed: 450.0,
                    projectile_size: Vec2::new(40.0, 40.0),
                    piercing: 1,
                    explodes_on_impact: false,
                    explosion_radius: 0.0,
                    explosion_damage: 0,
                    projectile_sprite_path: "".to_string(), // Use base_projectile_sprite_path from parent
                    projectile_color: Color::WHITE, // Use base_projectile_color from parent
                    aoe_radius_on_impact: None, // Added
                },
                ChargeLevelParams {
                    charge_time_secs: 1.5,
                    damage: 60, // Renamed
                    projectile_speed: 350.0,
                    projectile_size: Vec2::new(60.0, 60.0),
                    piercing: 2,
                    explodes_on_impact: true,
                    explosion_radius: 75.0,
                    explosion_damage: 30,
                    projectile_sprite_path: "sprites/void_cannon_projectile_placeholder.png".to_string(), // Specific override
                    projectile_color: Color::rgb(0.6, 0.3, 0.9), // Example color for charged shot
                    aoe_radius_on_impact: Some(50.0), // Example AoE
                },
            ],
            charge_sound_effect: None,
            release_sound_effect: None,
        }),
    }
}
