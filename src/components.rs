use bevy::prelude::*;
use crate::items::{DebuffAuraParams, ProjectileDebuffType, OrbitingPetParams, BlinkStrikeProjectileParams, RepositioningTetherParams, LobbedAoEPoolParams}; // Adjusted imports

#[derive(Component, Deref, DerefMut, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Health(pub i32);

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Damage(pub i32);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Cooldown {
    pub timer: Timer,
}

impl Default for Cooldown {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(1.0, TimerMode::Once) }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Target(pub Option<Entity>);

impl Default for Target {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}
impl Default for Lifetime {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(1.0, TimerMode::Once) }
    }
}

// Player specific components (can be moved to player.rs or survivor.rs if preferred)
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerStats {
    // Placeholder for actual player stats
    pub movement_speed: f32,
    pub max_health: i32,
}
impl Default for PlayerStats {
    fn default() -> Self {
        Self { movement_speed: 200.0, max_health: 100 }
    }
}


#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct OrbitingPetController {
    pub params_snapshot: OrbitingPetParams, // Cloned params for this instance
    pub active_pet_entities: Vec<Entity>,
    pub spawn_cooldown_timer: Timer,
}

impl Default for OrbitingPetController {
    fn default() -> Self {
        Self {
            params_snapshot: OrbitingPetParams::default(), // Assuming OrbitingPetParams derives Default
            active_pet_entities: Vec::new(),
            spawn_cooldown_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}


#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerBlinkState {
    pub blink_params: BlinkStrikeProjectileParams, // Contains all details like distance, target type
    pub charge_timer: Timer, // For charge-up before blink, or cooldown
    pub state: BlinkState,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum BlinkState {
    #[default]
    Ready,
    Charging,
    Blinking,
    Cooldown,
}

impl Default for PlayerBlinkState {
    fn default() -> Self {
        Self {
            blink_params: BlinkStrikeProjectileParams::default(), // Assuming it derives default
            charge_timer: Timer::from_seconds(1.0, TimerMode::Once),
            state: BlinkState::Ready,
        }
    }
}

// Tether related component for player state
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerTetherState {
    pub last_tether_mode_used: crate::items::RepositioningTetherMode,
    pub tethered_enemy_entity: Option<Entity>,
    pub current_weapon_params_snapshot: Option<RepositioningTetherParams>, // If params change with weapon
                                                                 // Add other fields like reactivation timers if needed
}

impl Default for PlayerTetherState {
    fn default() -> Self {
        Self {
            last_tether_mode_used: crate::items::RepositioningTetherMode::default(),
            tethered_enemy_entity: None,
            current_weapon_params_snapshot: None,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LobbedVenomCloudComponent { // Renamed from LobbedCloudProjectile
    pub params: DebuffAuraParams, // Assuming venom cloud uses DebuffAuraParams for its effect
    pub duration_timer: Timer,
    pub initial_spawn_position: Vec3,
    pub target_position: Vec3, // For arcing logic
    pub current_arc_time: f32, // For arcing logic
}
impl Default for LobbedVenomCloudComponent {
    fn default() -> Self {
        Self {
            params: DebuffAuraParams::default(), // Assuming DebuffAuraParams derives Default
            duration_timer: Timer::from_seconds(5.0, TimerMode::Once),
            initial_spawn_position: Vec3::ZERO,
            target_position: Vec3::ZERO,
            current_arc_time: 0.0,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PersistentHolyDamageCloud { // Renamed from PersistentAoECloud
    pub params: LobbedAoEPoolParams, // Example: Using pool params if it's a damaging area
    pub duration_timer: Timer,
    pub tick_timer: Timer,
    pub already_hit_entities: Vec<Entity>,
}
impl Default for PersistentHolyDamageCloud {
    fn default() -> Self {
        Self {
            params: LobbedAoEPoolParams::default(), // Assuming it derives Default
            duration_timer: Timer::from_seconds(10.0, TimerMode::Once),
            tick_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            already_hit_entities: Vec::new(),
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ActiveDamageDebuff { // Renamed from ActiveDebuff
    pub debuff_type: ProjectileDebuffType, // Example: Using ProjectileDebuffType
    pub magnitude: f32,
    pub duration_timer: Timer,
    pub stacks: u32,
}
impl Default for ActiveDamageDebuff {
    fn default() -> Self {
        Self {
            debuff_type: ProjectileDebuffType::default(),
            magnitude: 0.0,
            duration_timer: Timer::from_seconds(5.0, TimerMode::Once),
            stacks: 1,
        }
    }
}


// Component for entities rooted in place
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct RootedComponent {
    pub duration_timer: Timer,
}

impl Default for RootedComponent {
    fn default() -> Self {
        Self { duration_timer: Timer::from_seconds(1.0, TimerMode::Once) }
    }
}

// Event for player blink
#[derive(Event, Debug)]
pub struct PlayerBlinkEvent {
    pub player_entity: Entity,
    pub hit_enemy_entity: Entity, // The enemy that triggered the blink (if applicable)
    pub blink_params: crate::items::BlinkStrikeProjectileParams,
}


// General purpose debuffs, can be added to enemies
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct DamageAmpDebuffComponent {
    pub current_stacks: u32,
    pub magnitude_per_stack: f32,
    pub max_stacks: u32,
    pub duration_timer: Timer,
}

impl Default for DamageAmpDebuffComponent {
    fn default() -> Self {
        Self {
            current_stacks: 0,
            magnitude_per_stack: 0.05, // Default 5%
            max_stacks: 5,             // Default max 5 stacks
            duration_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}


#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct AccuracyDebuffComponent {
    pub reduction_factor: f32, // e.g., 0.8 for 20% reduction
    pub duration_timer: Timer,
}
impl Default for AccuracyDebuffComponent {
    fn default() -> Self { Self { reduction_factor: 1.0, duration_timer: Timer::from_seconds(1.0, TimerMode::Once) } }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct AttackSpeedDebuffComponent {
    pub multiplier: f32, // e.g., 1.2 for 20% slower attack speed (increased interval)
    pub duration_timer: Timer,
}
impl Default for AttackSpeedDebuffComponent {
    fn default() -> Self { Self { multiplier: 1.0, duration_timer: Timer::from_seconds(1.0, TimerMode::Once) } }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ContinuousDamageComponent {
    pub damage_per_tick: f32, // Changed to f32 to allow fractional damage if needed from magnitude
    pub tick_interval: f32,   // Seconds
    // pub tick_timer: Timer, // Implicitly handled by duration_timer and tick_interval in system
    pub duration_timer: Timer,
}
impl Default for ContinuousDamageComponent {
    fn default() -> Self {
        Self {
            damage_per_tick: 1.0,
            tick_interval: 1.0,
            duration_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}