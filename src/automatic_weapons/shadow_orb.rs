use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, OrbitingPetParams};
use crate::upgrades::{UpgradeCard, UpgradeType, OrbitingPetField, UpgradeRarity, UpgradeId};

pub fn define_shadow_orb() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(7),
        name: "Shadow Orb".to_string(),
        attack_data: AttackTypeData::OrbitingPet(OrbitingPetParams {
            base_fire_rate_secs: 1.0,  // Reduced respawn cooldown to 1 second
            max_active_orbs: 1,          // Start with one orb
            orb_duration_secs: 15.0,     // Moderate duration
            orb_sprite_path: "sprites/auto_shadow_orb.png".to_string(),
            orb_size: Vec2::new(32.0, 32.0),
            orb_color: Color::rgb(0.2, 0.1, 0.3), // Keep or adjust
            orbit_radius: 80.0,          // Adjusted orbit radius
            orbit_speed_rad_per_sec: 0.8, // Adjusted orbit speed
            can_be_deployed_at_location: true, // Enable deployment
            deployment_range: 300.0,     // Sensible deployment range
            pulses_aoe: true,            // Pulse AoE is primary
            pulse_damage: 12,            // Adjusted pulse damage
            pulse_radius: 32.0,          // Damage radius now matches visual max radius (orb_size.x * 1.0 = 32.0)
            pulse_interval_secs: 1.8,    // Adjusted pulse interval
            pulse_color: Some(Color::rgba(0.3, 0.1, 0.5, 1.0)), // Made opaque for wave effect
            fires_seeking_bolts: false,  // Disable seeking bolts
            bolt_damage: 0,              // Zero out bolt damage
            bolt_speed: 400.0,           // Unused, can remain
            bolt_fire_interval_secs: 9999.0, // Effectively disable bolt firing
            bolt_sprite_path: Some("sprites/shadow_bolt_placeholder.png".to_string()), // Unused
            bolt_size: Some(Vec2::new(10.0, 15.0)), // Unused
            bolt_color: Some(Color::rgb(0.3, 0.1, 0.5)), // Unused
            bolt_lifetime_secs: Some(1.0), // Unused
            bolt_homing_strength: Some(0.5), // Unused
            spawn_sound_effect: Some("audio/shadow_orb_spawn.ogg".to_string()),
        }),
    }
}

pub fn get_shadow_orb_upgrade_cards() -> Vec<UpgradeCard> {
    let shadow_orb_weapon_id = AutomaticWeaponId(7); // Matches existing definition
    vec![
        // Upgrade MaxActiveOrbs
        UpgradeCard {
            id: UpgradeId(7000), // Ensure unique ID
            name: "Twin Orbs".to_string(),
            description: "Summon an additional Shadow Orb. Max Orbs +1.".to_string(),
            upgrade_type: UpgradeType::ModifyOrbitingPet {
                weapon_id: shadow_orb_weapon_id,
                field: OrbitingPetField::MaxActiveOrbs,
                change_value: 1.0, // Increase by 1
                is_percentage: false,
            },
            rarity: UpgradeRarity::Rare,
        },
        // Upgrade OrbDurationSecs
        UpgradeCard {
            id: UpgradeId(7001),
            name: "Lingering Shadows".to_string(),
            description: "Shadow Orbs persist for a longer duration. Duration +5s.".to_string(),
            upgrade_type: UpgradeType::ModifyOrbitingPet {
                weapon_id: shadow_orb_weapon_id,
                field: OrbitingPetField::OrbDurationSecs,
                change_value: 5.0, // Increase by 5 seconds
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
        // Upgrade PulseDamage
        UpgradeCard {
            id: UpgradeId(7002),
            name: "Deeper Pulse".to_string(),
            description: "Shadow Orb pulses deal increased damage. Pulse Damage +3.".to_string(),
            upgrade_type: UpgradeType::ModifyOrbitingPet {
                weapon_id: shadow_orb_weapon_id,
                field: OrbitingPetField::PulseDamage,
                change_value: 3.0, // Increase by 3
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
        // Upgrade PulseIntervalSecs (decrease for faster pulses)
        UpgradeCard {
            id: UpgradeId(7003),
            name: "Accelerated Pulse".to_string(),
            description: "Shadow Orbs pulse more frequently. Pulse Interval -0.3s (faster).".to_string(),
            upgrade_type: UpgradeType::ModifyOrbitingPet {
                weapon_id: shadow_orb_weapon_id,
                field: OrbitingPetField::PulseIntervalSecs,
                change_value: -0.3, // Decrease interval by 0.3s
                is_percentage: false,
            },
            rarity: UpgradeRarity::Regular,
        },
        // Upgrade PulseRadius
        UpgradeCard {
            id: UpgradeId(7004),
            name: "Widening Shadows".to_string(),
            description: "Shadow Orb pulse radius increased by 15%. Radius +15%.".to_string(),
            upgrade_type: UpgradeType::ModifyOrbitingPet {
                weapon_id: shadow_orb_weapon_id,
                field: OrbitingPetField::PulseRadius,
                change_value: 0.15, // Increase by 15%
                is_percentage: true,
            },
            rarity: UpgradeRarity::Regular,
        },
    ]
}
