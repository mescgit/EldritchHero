use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, OrbitingPetParams};

pub fn define_shadow_orb() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(7),
        name: "Shadow Orb".to_string(),
        attack_data: AttackTypeData::OrbitingPet(OrbitingPetParams {
            base_fire_rate_secs: 1.0,
            max_active_orbs: 2,
            orb_duration_secs: 10.0,
            orb_sprite_path: "sprites/auto_shadow_orb.png".to_string(),
            orb_size: Vec2::new(32.0, 32.0),
            orb_color: Color::rgb(0.2, 0.1, 0.3),
            orbit_radius: 100.0,
            orbit_speed_rad_per_sec: 1.0,
            can_be_deployed_at_location: false,
            deployment_range: 0.0,
            pulses_aoe: true,
            pulse_damage: 10,
            pulse_radius: 60.0,
            pulse_interval_secs: 2.0,
            pulse_color: Some(Color::rgba(0.3, 0.1, 0.5, 0.5)),
            fires_seeking_bolts: true,
            bolt_damage: 8,
            bolt_speed: 400.0,
            bolt_fire_interval_secs: 1.5,
            bolt_sprite_path: Some("sprites/shadow_bolt_placeholder.png".to_string()),
            bolt_size: Some(Vec2::new(10.0, 15.0)),
            bolt_color: Some(Color::rgb(0.3, 0.1, 0.5)),
            bolt_lifetime_secs: Some(1.0),
            bolt_homing_strength: Some(0.5),
        }),
    }
}
