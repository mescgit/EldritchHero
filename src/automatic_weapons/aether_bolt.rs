use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, BlinkStrikeProjectileParams};

pub fn define_aether_bolt() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(23),
        name: "Aether Bolt".to_string(),
        attack_data: AttackTypeData::BlinkStrikeProjectile(BlinkStrikeProjectileParams {
            base_fire_rate_secs: 0.3,
            base_damage: 14,
            projectile_speed: 1000.0,
            projectile_sprite_path: "sprites/auto_aether_bolt.png".to_string(),
            projectile_size: Vec2::new(16.0, 16.0),
            projectile_color: Color::rgb(0.9,0.9,0.9),
            projectile_lifetime_secs: 1.4,
            piercing: 1,
            blink_chance_on_hit_percent: 0.25,
            blink_distance: 100.0,
            blink_to_target_behind: true,
            blink_requires_kill: false,
            num_projectiles_per_shot: 2,
            fire_sound_effect: None,
        }),
    }
}
