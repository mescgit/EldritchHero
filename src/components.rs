use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Damage(pub i32);

#[derive(Component)]
pub struct Cooldown { // Currently unused
    pub timer: Timer,
}

#[derive(Component)]
pub struct Target(pub Option<Entity>); // Currently unused

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Dashing {
    pub dash_timer: Timer,
    pub invulnerability_timer: Timer,
    pub damage: f32,
    pub hitbox_width: f32,
    pub already_hit_entities: Vec<Entity>,
    pub direction: Vec2,
}

// --- Shadow Orb Components ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ShadowOrb {
    pub params_snapshot: crate::items::DeployableOrbitingTurretParams, // Cloned params for this instance
    pub duration_timer: Timer,
    pub attack_timer: Timer,
    pub owner_entity: Entity,
}

// Default implementation might be tricky if DeployableOrbitingTurretParams doesn't have a simple default
// or if owner_entity cannot be default(). For now, let's assume it will be initialized explicitly.
// If a Default is strictly needed by Bevy's reflection/registration and causes issues,
// we might need to wrap params_snapshot in Option or provide a more sensible default for owner_entity (e.g. Entity::PLACEHOLDER).
// However, since this component will likely always be added with specific values, explicit initialization is better.

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct OrbitingMovement {
    pub center_entity: Entity,
    pub radius: f32,
    pub current_angle_rad: f32,
    pub speed_rad_per_sec: f32,
}

impl Default for OrbitingMovement {
    fn default() -> Self {
        Self {
            center_entity: Entity::PLACEHOLDER, // Placeholder, should be set on spawn
            radius: 100.0,
            current_angle_rad: 0.0,
            speed_rad_per_sec: std::f32::consts::PI / 2.0, // 1 rotation every 4 seconds
        }
    }
}

// --- Psionic Lash / Repositioning Tether Components ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PsionicTetherProjectile {
    pub params_snapshot: crate::items::RepositioningTetherParams,
    pub duration_timer: Timer, // For projectile lifetime
    pub owner: Entity,         // Player who fired it
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TetheredEnemy {
    pub tether_owner: Entity, // Player who applied the tether
    pub activation_window_timer: Timer,
    // Note: params for pull/push strength will be retrieved from PlayerTetherState.current_weapon_params_snapshot
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerTetherState {
    // Used if RepositioningTetherMode is Alternate, to flip between Pull and Push
    pub last_tether_mode_used: Option<crate::items::RepositioningTetherMode>, 
    pub tethered_enemy_entity: Option<Entity>,
    // Stores the params of the weapon that created the current tether connection
    pub current_weapon_params_snapshot: Option<crate::items::RepositioningTetherParams>, 
}

// --- Aether Bolt / Blink Strike Event ---

#[derive(Event, Debug, Clone)]
pub struct PlayerBlinkEvent {
    pub player_entity: Entity,
    pub hit_enemy_entity: Entity, // Needed for BlinkTarget::BehindEnemy
    pub blink_params: crate::items::BlinkStrikeParams, // Contains all details like distance, target type
}

// --- Venom Spit / Lobbed AoE Cloud Components ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LobbedCloudProjectile {
    pub params: crate::items::LobbedAoECloudParams,
    pub owner_entity: Entity,
    // Potentially add initial velocity or target point if gravity calculation is complex
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PersistentAoECloud {
    pub params: crate::items::LobbedAoECloudParams,
    pub duration_timer: Timer,
    pub tick_timer: Timer,
    pub already_hit_this_tick: Vec<Entity>,
    pub owner_entity: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ActiveDebuff {
    pub debuff_type: crate::items::DebuffType,
    pub intensity: f32,
    pub duration_timer: Timer,
}

// --- Venom Spit / Lobbed AoE Cloud Components ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LobbedCloudProjectile {
    pub params: crate::items::LobbedAoECloudParams,
    pub owner_entity: Entity,
    // Potentially add initial velocity or target point if gravity calculation is complex
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PersistentAoECloud {
    pub params: crate::items::LobbedAoECloudParams,
    pub duration_timer: Timer,
    pub tick_timer: Timer,
    pub already_hit_this_tick: Vec<Entity>,
    pub owner_entity: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ActiveDebuff {
    pub debuff_type: crate::items::DebuffType,
    pub intensity: f32,
    pub duration_timer: Timer,
    // pub source_weapon_type: Option<crate::items::AttackTypeData>, // Omitted for now for simplicity
    // pub source_weapon_id: Option<crate::items::AutomaticWeaponId>, // Alternative simpler tracking
}