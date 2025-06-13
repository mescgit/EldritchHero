use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, HomingDebuffProjectileParams, ProjectileDebuffType};
use crate::upgrades::UpgradeCard; // Added import

pub fn define_moonbeam_dart() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(13),
        name: "Moonbeam Dart".to_string(),
        attack_data: AttackTypeData::HomingDebuffProjectile(HomingDebuffProjectileParams {
            base_fire_rate_secs: 0.4,
            num_darts_per_shot: 2,
            base_damage: 8,
            projectile_speed: 700.0,
            projectile_sprite_path: "sprites/auto_moonbeam_dart.png".to_string(),
            projectile_size: Vec2::new(15.0, 25.0),
            projectile_color: Color::rgb(0.7, 0.7, 0.9),
            projectile_lifetime_secs: 2.0,
            homing_strength: 1.5,
            homing_initial_target_search_radius: 400.0,
            debuff_type: ProjectileDebuffType::DamageAmp,
            debuff_magnitude_per_stack: 0.05,
            max_debuff_stacks: 5,
            debuff_duration_secs_on_target: 3.0,
            fire_sound_effect: Some("audio/moonbeam_dart_fire.ogg".to_string()),
        }),
    }
}

pub fn get_specific_upgrades() -> Vec<UpgradeCard> {
    vec![]
}
