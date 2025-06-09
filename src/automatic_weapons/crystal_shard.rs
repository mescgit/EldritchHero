use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, BouncingProjectileParams};

pub fn define_crystal_shard() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(16),
        name: "Crystal Shard".to_string(),
        attack_data: AttackTypeData::BouncingProjectile(BouncingProjectileParams {
            base_fire_rate_secs: 0.3,
            num_shards_per_shot: 5,
            base_damage: 10,
            projectile_speed: 700.0,
            projectile_sprite_path: "sprites/auto_crystal_shard.png".to_string(),
            projectile_size: Vec2::new(18.0, 18.0),
            projectile_color: Color::rgb(0.8, 0.6, 1.0),
            projectile_lifetime_secs: 3.0,
            max_bounces: 2,
            damage_loss_per_bounce_multiplier: 0.75,
            speed_loss_per_bounce_multiplier: 0.9,
            spread_angle_degrees: 30.0,
            fire_sound_effect: Some("assets/audio/crystal_shard_fire.ogg".to_string()),
        }),
    }
}
