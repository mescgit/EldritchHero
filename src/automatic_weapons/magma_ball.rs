use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, LobbedBouncingMagmaParams};
use bevy::prelude::{Color, Vec2}; // Added for Color and Vec2

pub fn define_magma_ball() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(17),
        name: "Magma Ball".to_string(),
        attack_data: AttackTypeData::LobbedBouncingMagma(LobbedBouncingMagmaParams {
            base_fire_rate_secs: 0.9,
            projectile_speed: 350.0,
            projectile_sprite_path: "sprites/magma_ball_placeholder.png".to_string(),
            projectile_size: Vec2::new(28.0, 28.0),
            projectile_color: Color::ORANGE_RED,
            projectile_arc_height: 60.0,
            num_bounces: 3,
            damage_per_bounce_impact: 15,
            bounce_impact_radius: 50.0,
            fire_pool_on_bounce_chance: 0.66,
            fire_pool_damage_per_tick: 8,
            fire_pool_radius: 60.0,
            fire_pool_duration_secs: 2.5,
            fire_pool_tick_interval_secs: 0.4,
            fire_pool_color: Color::rgba(1.0, 0.4, 0.0, 0.6),
            projectile_lifetime_secs: 10.0, 
            explosion_radius_on_final_bounce: 75.0,
            explosion_damage_on_final_bounce: 40,
            fire_sound_effect: Some("audio/magma_ball_fire.ogg".to_string()),
        }),
    }
}
