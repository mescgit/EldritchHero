use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, TrailOfFireParams};

pub fn define_inferno_bolt() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(4),
        name: "Inferno Bolt".to_string(),
        attack_data: AttackTypeData::TrailOfFire(TrailOfFireParams {
            base_damage_on_impact: 10,
            base_fire_rate_secs: 0.8,
            projectile_speed: 700.0,
            projectile_sprite_path: "sprites/auto_inferno_bolt.png".to_string(),
            projectile_size: Vec2::new(20.0, 20.0),
            projectile_color: Color::rgb(1.0, 0.3, 0.0),
            projectile_lifetime_secs: 1.5,
            segment_spawn_interval_secs: 0.1, // Renamed from trail_segment_spawn_interval_secs
            trail_segment_damage_per_tick: 5,
            trail_segment_tick_interval_secs: 0.5,
            trail_segment_duration_secs: 2.0,
            trail_segment_width: 30.0,
            trail_segment_color: Color::rgba(1.0, 0.5, 0.0, 0.7),
            fire_sound_effect: None,
        }),
    }
}
