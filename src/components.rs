use bevy::prelude::*;
use crate::items::{
    OrbitingPetParams, BlinkStrikeProjectileParams, RepositioningTetherParams,
    DebuffAuraParams, LobbedAoEPoolParams, ProjectileDebuffType // Corrected and added necessary item param structs
};

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
    pub params_snapshot: OrbitingPetParams,
    pub active_pet_entities: Vec<Entity>,
    pub spawn_cooldown_timer: Timer,
}

impl Default for OrbitingPetController {
    fn default() -> Self {
        Self {
            params_snapshot: OrbitingPetParams::default(),
            active_pet_entities: Vec::new(),
            spawn_cooldown_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}


#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerBlinkState {
    pub blink_params: BlinkStrikeProjectileParams,
    pub charge_timer: Timer,
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
            blink_params: BlinkStrikeProjectileParams::default(),
            charge_timer: Timer::from_seconds(1.0, TimerMode::Once),
            state: BlinkState::Ready,
        }
    }
}

// Tether related component for player state
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerTetherState { // This component seems to be intended for use in automatic_projectiles.rs
    pub last_tether_mode_used: crate::items::RepositioningTetherMode,
    pub tethered_enemy_entity: Option<Entity>, // Field for the enemy hit by tether
    pub current_weapon_params_snapshot: Option<RepositioningTetherParams>,
    // Add other fields like reactivation timers if needed, or use PlayerWaitingTetherActivationComponent from weapon_systems
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

// Original LobbedCloudProjectile, PersistentAoECloud, ActiveDebuff were duplicated.
// Keeping one version and correcting param types.

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct LobbedCloudProjectile { // Assuming this is for a general debuff cloud
    pub params: DebuffAuraParams,
    pub duration_timer: Timer,
    pub initial_spawn_position: Vec3,
    pub target_position: Vec3,
    pub current_arc_time: f32,
}

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct PersistentAoECloud { // Assuming this is for a persistent damaging area
    pub params: LobbedAoEPoolParams, // Using LobbedAoEPoolParams if it's a damaging pool
    pub duration_timer: Timer,
    pub tick_timer: Timer,
    pub already_hit_entities: Vec<Entity>,
}

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct ActiveDebuff { // General debuff on an entity
    pub debuff_type: ProjectileDebuffType, // Using ProjectileDebuffType as an example, adjust if another type is needed
    pub magnitude: f32,
    pub duration_timer: Timer,
    pub stacks: u32,
}


// Component for entities rooted in place
#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct RootedComponent {
    pub duration_timer: Timer,
}


// Event for player blink
#[derive(Event, Debug)]
pub struct PlayerBlinkEvent {
    pub player_entity: Entity,
    pub hit_enemy_entity: Entity, 
    pub blink_params: crate::items::BlinkStrikeProjectileParams,
}


// General purpose debuffs, can be added to enemies
#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct DamageAmpDebuffComponent {
    pub current_stacks: u32,
    pub magnitude_per_stack: f32,
    pub max_stacks: u32,
    pub duration_timer: Timer,
}

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct AccuracyDebuffComponent {
    pub reduction_factor: f32, 
    pub duration_timer: Timer,
}

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct AttackSpeedDebuffComponent {
    pub multiplier: f32, 
    pub duration_timer: Timer,
}

#[derive(Component, Debug, Reflect, Default)] // Added Default derive
#[reflect(Component)]
pub struct ContinuousDamageComponent {
    pub damage_per_tick: f32, 
    pub tick_interval: f32,   
    pub duration_timer: Timer,
}

// Component for Horrors latched by a tether (used in weapon_systems.rs, better defined here)
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct HorrorLatchedByTetherComponent {
    pub player_who_latched: Option<Entity>, // Changed to Option<Entity> to allow Default derive
}

#[derive(Debug, Component, Reflect, Default)] // Added Default here
#[reflect(Component)]
pub struct BurnStatusEffect {
    pub damage_per_tick: i32,
    pub tick_interval_secs: f32,
    pub duration_timer: Timer,
    pub tick_timer: Timer,
    pub source_weapon_id: Option<u32>,
}