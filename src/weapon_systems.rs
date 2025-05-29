// mescgit/eldritchhero/EldritchHero-77df6cd0b3e48857123b0971c9f30b59714a1b8a/src/weapon_systems.rs
use bevy::prelude::*;
use bevy::prelude::Name; 
use crate::items::{StandardProjectileParams, ReturningProjectileParams, ChanneledBeamParams, ConeAttackParams, AutomaticWeaponId};
use crate::components::{Velocity, Damage, Lifetime, Health, RootedComponent}; 
use crate::survivor::{BASE_SURVIVOR_SPEED as BASE_PLAYER_SPEED, Survivor};
use crate::camera_systems::MainCamera; 
use crate::horror::Horror; 
use crate::game::AppState; 

// --- Returning Projectile Definitions ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Component)]
#[reflect(Component)]
pub enum ReturningProjectileState {
    Outgoing,
    Returning,
}

impl Default for ReturningProjectileState {
    fn default() -> Self { Self::Outgoing }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ReturningProjectileComponent {
    pub state: ReturningProjectileState,
    pub start_position: Vec3,
    pub max_travel_distance: f32,
    pub speed: f32,
    pub piercing_left: u32,
}

// --- Channeled Beam Definitions ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ChanneledBeamComponent {
    pub damage_per_tick: i32,
    pub tick_timer: Timer, 
    pub range: f32,
    pub width: f32,
    pub color: Color,
    pub owner: Entity,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct IsChannelingComponent {
    pub beam_entity: Option<Entity>,
    pub beam_params: ChanneledBeamParams,
}

// --- Lobbed AoE Pool Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct LobbedProjectileComponent {
    pub arc_height: f32, 
    pub speed: f32, 
    pub pool_params: crate::items::LobbedAoEPoolParams, 
    pub initial_spawn_position: Vec3, 
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct IchorPoolComponent {
    pub damage_per_tick: i32,
    pub radius: f32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub color: Color, 
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ActiveIchorPools {
    pub pools: std::collections::VecDeque<Entity>, 
}

// --- Charge-Up Energy Shot Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ChargingWeaponComponent {
    pub weapon_id: AutomaticWeaponId,
    pub charge_timer: Timer,
    pub current_charge_level_index: usize,
    pub is_actively_charging: bool,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ExplodesOnFinalImpact {
    pub explosion_radius: f32,
    pub explosion_damage: i32,
}

// --- Trail of Fire Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TrailSpawningProjectileComponent {
    pub trail_params: crate::items::TrailOfFireParams,
    pub segment_spawn_timer: Timer,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct FireTrailSegmentComponent {
    pub damage_per_tick: i32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub width: f32,
    pub already_hit_this_tick: Vec<Entity>,
}

// --- Chain Lightning Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ChainLightningZapEffectComponent {
    pub duration_timer: Timer,
}

// --- Point-Blank Nova Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct NovaVisualComponent {
    pub initial_radius: f32,
    pub max_radius: f32,
    pub duration_timer: Timer,
    pub color: Color,
}

// --- Persistent Aura Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerPersistentAuraComponent {
    pub damage_per_tick: i32,
    pub tick_timer: Timer,
    pub radius: f32,
    pub aura_color: Color, 
    pub visual_entity: Option<Entity>,
    pub weapon_id: crate::items::AutomaticWeaponId, 
}

// --- Debuffing Aura/Cloud Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct DebuffCloudVisualComponent {
    pub params: crate::items::DebuffAuraParams, 
    pub duration_timer: Timer, 
    pub already_hit_horrors: Vec<Entity>, 
}

// --- Expanding Energy Bomb Definitions ---

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Default, Component)]
#[reflect(Component)]
pub enum SpiritBombState {
    #[default]
    Expanding,
    WaitingAtMaxRadius,
    Detonated, 
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ExpandingEnergyBombComponent {
    pub params: crate::items::ExpandingEnergyBombParams,
    pub current_radius: f32,
    pub expansion_timer: Timer,
    pub wait_at_max_radius_timer: Timer,
    pub state: SpiritBombState,
}

// --- Homing Debuff Projectile Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct HomingTargetComponent {
    pub target_entity: Option<Entity>,
    pub strength: f32, 
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct DebuffOnHitComponent {
    pub debuff_type: crate::items::ProjectileDebuffType,
    pub magnitude_per_stack: f32,
    pub max_stacks: u32,
    pub duration_secs: f32,
}

// --- Ground-Targeted Eruption Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct GroundTargetReticuleComponent {
    pub max_range: f32,
    pub visual_size: Vec2,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PendingGroundAoEComponent {
    pub position_of_impact: Vec3,
    pub params: crate::items::GroundTargetedAoEParams,
    pub eruption_timer: Timer,
    pub visual_eruption_effect_entity: Option<Entity>, 
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct EruptionVisualComponent {
    pub duration_timer: Timer,
    pub initial_radius: f32,
    pub max_radius: f32,
    pub color: Color,
}

// --- Dash Attack Definitions ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerDashingComponent {
    pub params: crate::items::LineDashAttackParams, // Changed from DashAttackParams
    pub initial_direction: Vec2, // Renamed from direction
    pub dash_timer: Timer, // Renamed from duration_timer
    pub already_hit_horrors: Vec<Entity>, // Renamed from hit_enemies_this_dash
    pub original_speed_if_modified: Option<f32>, // Renamed from original_speed
}

impl Default for PlayerDashingComponent {
    fn default() -> Self {
        let default_params = crate::items::LineDashAttackParams::default();
        Self {
            params: default_params.clone(), 
            initial_direction: Vec2::X, 
            dash_timer: Timer::from_seconds(default_params.dash_duration_secs, TimerMode::Once),
            already_hit_horrors: Vec::new(),
            original_speed_if_modified: None,
        }
    }
}

// --- Blink Strike Projectile Systems ---

pub fn spawn_blink_strike_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::BlinkStrikeProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    weapon_id: crate::items::AutomaticWeaponId,
) {
    let base_aim_direction_normalized = aim_direction.normalize_or_zero();
    let num_projectiles = params.num_projectiles_per_shot;

    // Example spread logic (e.g. 10 degrees total spread for 2 projectiles, 15 for 3, etc.)
    // This can be adjusted or made more sophisticated.
    let total_spread_degrees = if num_projectiles > 1 { (num_projectiles -1) as f32 * 7.5 } else { 0.0 };

    for i in 0..num_projectiles {
        let mut current_projectile_aim_direction = base_aim_direction_normalized;
        if num_projectiles > 1 {
            let total_spread_rad = total_spread_degrees.to_radians();
            // Calculate angle for this specific projectile, ensuring spread is centered
            let angle_offset_rad = if num_projectiles <= 1 {
                0.0
            } else {
                (i as f32 / (num_projectiles as f32 - 1.0)) * total_spread_rad - (total_spread_rad / 2.0)
            };
            
            let base_angle_rad = base_aim_direction_normalized.y.atan2(base_aim_direction_normalized.x);
            current_projectile_aim_direction = Vec2::new((base_angle_rad + angle_offset_rad).cos(), (base_angle_rad + angle_offset_rad).sin());
        }

        crate::automatic_projectiles::spawn_automatic_projectile(
            commands,
            asset_server,
            player_transform.translation,
            current_projectile_aim_direction,
            params.base_damage,
            params.projectile_speed,
            params.piercing,
            weapon_id,
            params.projectile_sprite_path,
            params.projectile_size,
            params.projectile_color,
            params.projectile_lifetime_secs,
            None, // opt_max_bounces
            None, // opt_dmg_loss_mult
            None, // opt_speed_loss_mult
            None, // opt_lifesteal_percentage
            None, // opt_tether_params_for_comp
            Some(params.clone()), // opt_blink_params
        );
    }
}

// --- Repositioning Tether Systems ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TetherProjectileComponent { // Marker component, params are on AutomaticProjectile via items.rs
    pub params_snapshot: crate::items::RepositioningTetherParams,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PlayerWaitingTetherActivationComponent {
    pub hit_horror_entity: Entity,
    pub horror_original_transform: Option<Transform>, 
    pub params: crate::items::RepositioningTetherParams,
    pub reactivation_window_timer: Timer,
    pub next_effect_mode: crate::items::RepositioningTetherMode,
}

impl Default for PlayerWaitingTetherActivationComponent {
    fn default() -> Self {
        // Note: This default is largely a placeholder as real values must be supplied.
        Self {
            hit_horror_entity: Entity::PLACEHOLDER,
            horror_original_transform: None,
            params: crate::items::RepositioningTetherParams { // This uses default() of RepositioningTetherParams
                base_fire_rate_secs: 1.0,
                tether_projectile_speed: 500.0,
                tether_range: 300.0,
                tether_sprite_path: "sprites/tether_placeholder.png",
                tether_color: Color::WHITE,
                tether_size: Vec2::new(5.0, 10.0),
                mode: crate::items::RepositioningTetherMode::default(),
                pull_strength: 100.0,
                push_strength: 100.0,
                reactivation_window_secs: 2.0,
                effect_duration_secs: 0.3,
            },
            reactivation_window_timer: Timer::from_seconds(2.0, TimerMode::Once),
            next_effect_mode: crate::items::RepositioningTetherMode::default(),
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct HorrorLatchedByTetherComponent {
    pub player_who_latched: Entity,
}

fn apply_tether_reposition_effect(
    // Removed Commands from here as it's not directly used for spawning, only for component removal by caller
    horror_transform: &mut Transform, // Directly pass the mutable horror transform
    player_transform: &Transform,   // Directly pass the player transform
    params: &crate::items::RepositioningTetherParams,
    mode: crate::items::RepositioningTetherMode,
) {
    let player_pos = player_transform.translation.truncate();
    let horror_pos = horror_transform.translation.truncate();
    
    let actual_mode = match mode {
        crate::items::RepositioningTetherMode::Alternate => {
            // This logic might need to be smarter if true alternation is desired on secondary activation
            // For now, if it's passed as Alternate, we'll pick one, e.g., Pull.
            // The PlayerWaitingTetherActivationComponent.next_effect_mode should ideally store the resolved mode.
            crate::items::RepositioningTetherMode::Pull 
        },
        _ => mode,
    };

    match actual_mode {
        crate::items::RepositioningTetherMode::Pull => {
            let direction_to_player = (player_pos - horror_pos).normalize_or_zero();
            if direction_to_player != Vec2::ZERO {
                horror_transform.translation += (direction_to_player * params.pull_strength).extend(0.0);
            }
        }
        crate::items::RepositioningTetherMode::Push => {
            let direction_from_player = (horror_pos - player_pos).normalize_or_zero();
             if direction_from_player != Vec2::ZERO {
                horror_transform.translation += (direction_from_player * params.push_strength).extend(0.0);
            } else {
                horror_transform.translation += (Vec2::X * params.push_strength).extend(0.0);
            }
        }
        _ => {} 
    }
    // Sound/visual effects for pull/push could be triggered here via event writers if needed,
    // or by the calling system.
}

pub fn spawn_repositioning_tether_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    // player_transform: &Transform, // Player transform will be queried
    aim_direction: Vec2, // Still needed for initial projectile direction
    weapon_params: &crate::items::RepositioningTetherParams, // Specific params for this weapon
    weapon_id: crate::items::AutomaticWeaponId,
    // Queries needed for reactivation logic
    mut player_waiting_query: Query<&mut PlayerWaitingTetherActivationComponent>, // Query for the specific player
    mut horror_query: Query<&mut Transform, With<Horror>>, // Query all horrors for their transforms
    player_transform_query: Query<&Transform, With<Survivor>>, // Query player transform
) {
    if let Ok(mut waiting_comp) = player_waiting_query.get_mut(player_entity) {
        if !waiting_comp.reactivation_window_timer.finished() {
            // Reactivation logic
            if let Ok(player_tform) = player_transform_query.get(player_entity) {
                if let Ok(mut horror_tform) = horror_query.get_mut(waiting_comp.hit_horror_entity) {
                    apply_tether_reposition_effect(
                        &mut horror_tform,
                        player_tform,
                        &waiting_comp.params, // Use params stored in the component
                        waiting_comp.next_effect_mode,
                    );
                }
            }
            // Clean up components after reactivation
            if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                 commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
            return; // Action performed, do not fire new projectile
        } else {
            // Timer finished, clean up old components before firing a new projectile
             if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
        }
    }

    // Primary Fire Logic: Spawn a new tether projectile
    // Need player_transform for initial projectile spawn position
    if let Ok(player_transform) = player_transform_query.get(player_entity) {
        let _projectile_entity = crate::automatic_projectiles::spawn_automatic_projectile(
            commands,
            asset_server,
            player_transform.translation, // Current player transform for spawn
            aim_direction,
            0, // Tether projectile damage (0 or very low)
            weapon_params.tether_projectile_speed,
            0, // Piercing
            weapon_id,
            weapon_params.tether_sprite_path,
            weapon_params.tether_size,
            weapon_params.tether_color,
            weapon_params.tether_range / weapon_params.tether_projectile_speed, // Lifetime
            None, None, None, None, // Bouncing, Lifesteal
            Some(weapon_params.clone()), // Pass RepositioningTetherParams for the projectile to carry
            None, // No blink params
        );
    }
    // Note: The logic to add TetherProjectileComponent to the projectile is now expected
    // to be handled within spawn_automatic_projectile or by the caller if spawn_automatic_projectile returns the entity.
    // Based on previous setup, spawn_automatic_projectile was modified to add it internally.
}

pub fn tether_reactivation_window_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerWaitingTetherActivationComponent)>,
) {
    for (player_entity, mut waiting_comp) in query.iter_mut() {
        waiting_comp.reactivation_window_timer.tick(time.delta());
        if waiting_comp.reactivation_window_timer.finished() {
            if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                 commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
        }
    }
}
        commands,
        asset_server,
        player_transform.translation,
        aim_direction,
        0, // Tether projectile damage (0 or very low)
        params.tether_projectile_speed,
        0, // Piercing
        weapon_id, // Pass the weapon_id
        params.tether_sprite_path,
        params.tether_size,
        params.tether_color,
        params.tether_range / params.tether_projectile_speed, // Lifetime based on range/speed
        None, None, None, // Bouncing params
        None, // Lifesteal
    );
    // Add TetherProjectileComponent to the spawned projectile
    // spawn_automatic_projectile returns void, so we need to query for it if we want to add component.
    // This is a limitation. For now, TetherProjectileComponent must be added IN spawn_automatic_projectile
    // or spawn_automatic_projectile must return the entity.
    // The prompt for automatic_projectiles.rs already added it to the projectile query there.
    // So, the component with params_snapshot should be added there.
    // Here, we just ensure the projectile is spawned.
}

pub fn tether_reactivation_window_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerWaitingTetherActivationComponent)>,
) {
    for (player_entity, mut waiting_comp) in query.iter_mut() {
        waiting_comp.reactivation_window_timer.tick(time.delta());
        if waiting_comp.reactivation_window_timer.finished() {
            if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                 commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerInvulnerableComponent;

// --- Lobbed Bouncing Magma Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct LobbedBouncingProjectileComponent {
    pub params: crate::items::LobbedBouncingMagmaParams,
    pub bounces_left: u32,
    pub speed: f32, // Can change if bounces affect speed
    pub initial_spawn_position: Vec3, // For arc calculation
    // Potentially add:
    // pub last_bounce_position: Option<Vec3>,
    // pub lifetime_timer: Timer, // if lifetime is independent of bounces
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct MagmaPoolComponent {
    pub damage_per_tick: i32,
    pub radius: f32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub color: Color,
    pub already_hit_this_tick: Vec<Entity>, // To ensure one damage application per tick per enemy
}

// --- Orbiting Pet Definitions (New Implementation) ---

// Commenting out old OrbitingPetComponent and related structures/systems
/*
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct OrbitingPetComponent {
    pub params: crate::items::OrbitingPetParams, // This refers to the OLD OrbitingPetParams
    pub owner_entity: Entity,
    pub duration_timer: Timer,
    pub attack_timer: Timer,
    pub current_orbit_angle_rad: f32,
}

impl Default for OrbitingPetComponent {
    fn default() -> Self {
        // This default implementation refers to the old PetAttackTypeParams and old OrbitingPetParams structure
        // which are being replaced by the new monolithic OrbitingPetParams.
        // Keeping it commented for reference during transition.
        let default_pet_attack_params = crate::items::PetAttackTypeParams::PulseAoE { // Assuming PetAttackTypeParams might be commented out in items.rs
            damage: 5,
            radius: 50.0,
            tick_interval_secs: 1.0,
        };
        let default_orbiting_pet_params = crate::items::OrbitingPetParams { // This is the OLD params struct
            base_fire_rate_secs: 5.0, 
            pet_duration_secs: 10.0,
            orbit_radius: 100.0,
            orbit_speed_rad_per_sec: std::f32::consts::PI / 4.0,
            deployment_range: 0.0,
            num_pets_allowed: 1,
            pet_attack_params: default_pet_attack_params, 
            pet_sprite_path: "sprites/auto_shadow_orb.png", 
            pet_size: Vec2::new(32.0, 32.0),
            weapon_id_placeholder: Some(AutomaticWeaponId(7)), 
        };

        Self {
            params: default_orbiting_pet_params.clone(),
            owner_entity: Entity::PLACEHOLDER, 
            duration_timer: Timer::from_seconds(default_orbiting_pet_params.pet_duration_secs, TimerMode::Once),
            attack_timer: Timer::from_seconds(1.0, TimerMode::Repeating), // Placeholder, original logic was more complex
            current_orbit_angle_rad: 0.0,
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ActiveOrbitingPetsResource {
    pub active_pets: Vec<Entity>, 
}
*/

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct OrbitingPetComponent {
    pub params_snapshot: crate::items::OrbitingPetParams, // Cloned params
    pub orbit_angle_rad: f32,
    pub duration_timer: Timer,
    pub pulse_timer: Option<Timer>,
    pub bolt_timer: Option<Timer>,
    pub owner_player_entity: Entity,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerOrbControllerComponent {
    pub active_orb_entities: Vec<Entity>,
    pub max_orbs_allowed: u32,
    pub spawn_cooldown_timer: Timer,
}

// Helper function to spawn the beam entity
pub fn spawn_beam_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &ChanneledBeamParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    owner_entity: Entity, 
) -> Entity {
    let beam_transform = Transform::from_translation(player_transform.translation)
        .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x)));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/auto_arcane_ray.png"), 
            sprite: Sprite {
                custom_size: Some(Vec2::new(params.range, params.beam_width)), 
                color: params.beam_color,
                anchor: bevy::sprite::Anchor::CenterLeft, 
                ..default()
            },
            transform: beam_transform,
            ..default()
        },
        ChanneledBeamComponent {
            damage_per_tick: params.base_damage_per_tick,
            tick_timer: Timer::from_seconds(params.tick_rate_secs, TimerMode::Repeating),
            range: params.range,
            width: params.beam_width,
            color: params.beam_color,
            owner: owner_entity,
        },
        Name::new("ChanneledBeamEntity"),
    )).id()
}


pub struct WeaponSystemsPlugin;

impl Plugin for WeaponSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ReturningProjectileState>()
            .register_type::<ReturningProjectileComponent>()
            .register_type::<ChanneledBeamComponent>()
            .register_type::<IsChannelingComponent>()
            .register_type::<LobbedProjectileComponent>()
            .register_type::<IchorPoolComponent>()
            .init_resource::<ActiveIchorPools>()
            .register_type::<ChargingWeaponComponent>()
            .register_type::<ExplodesOnFinalImpact>()
            .register_type::<TrailSpawningProjectileComponent>()
            .register_type::<FireTrailSegmentComponent>()
            .register_type::<ChainLightningZapEffectComponent>() 
            .register_type::<NovaVisualComponent>() 
            .register_type::<PlayerPersistentAuraComponent>() 
            .register_type::<DebuffCloudVisualComponent>() 
            .register_type::<SpiritBombState>() 
            .register_type::<ExpandingEnergyBombComponent>() 
            .register_type::<HomingTargetComponent>() 
            .register_type::<DebuffOnHitComponent>() 
            .register_type::<GroundTargetReticuleComponent>()
            .register_type::<PendingGroundAoEComponent>()
            .register_type::<EruptionVisualComponent>()
            .register_type::<PlayerDashingComponent>()    
            .register_type::<PlayerInvulnerableComponent>() 
            .register_type::<LobbedBouncingProjectileComponent>() // New
            .register_type::<MagmaPoolComponent>()              // New
            .register_type::<OrbitingPetComponent>() 
            .register_type::<PlayerOrbControllerComponent>() 
            .register_type::<TetherProjectileComponent>()      // New
            .register_type::<PlayerWaitingTetherActivationComponent>() // New
            .register_type::<HorrorLatchedByTetherComponent>()   // New
            .add_systems(Update, (
                manage_player_orbs_system, 
                orbiting_pet_behavior_system, 
                tether_reactivation_window_system, 
                // repositioning_tether_firing_system, // Adding the new system for tether firing
                charge_weapon_system, 
                trail_spawning_projectile_system, 
                fire_trail_segment_system,      
                chain_lightning_visual_system, 
                nova_visual_system, 
                manage_persistent_aura_system, 
                debuff_cloud_system, 
                expanding_energy_bomb_system, 
                homing_projectile_system, 
                returning_projectile_system,
                lobbed_projectile_system,
                ichor_pool_system,
                player_is_channeling_effect_system,
                channeled_beam_update_system,
                channeled_beam_damage_system, 
                ground_targeting_reticule_system,
                pending_ground_aoe_system,
                eruption_visual_system,
                player_dashing_system, 
                lobbed_bouncing_projectile_system, 
                magma_pool_system,
                repositioning_tether_firing_system, // Added new system
            ).in_set(OnUpdate(AppState::InGame)));
    }
}

// New system to handle the actual firing logic for Repositioning Tether
pub fn repositioning_tether_firing_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Player related queries to check for firing readiness
    player_query: Query<(Entity, &Transform, &crate::player::Player, &crate::player::MindAffliction)>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    // Queries needed by spawn_repositioning_tether_attack
    player_waiting_query: Query<&mut PlayerWaitingTetherActivationComponent>, 
    horror_query: Query<&mut Transform, With<Horror>>, 
    survivor_transform_query: Query<&Transform, With<Survivor>>, // Specifically for the player's transform
    // Consider sound event writer if needed here, or if spawn_repositioning_tether_attack handles it
) {
    for (player_entity, player_transform, player_stats, mind_affliction) in player_query.iter() {
        if let Some(weapon_id) = player_stats.equipped_weapon_id {
            if let Some(weapon_def) = weapon_library.get_weapon_definition(weapon_id) {
                if let crate::items::AttackTypeData::RepositioningTether(ref params) = weapon_def.attack_data {
                    // Check if the weapon is ready to fire (based on MindAffliction timer)
                    // Note: player_shooting_system already updates the timer's duration.
                    // Here, we just check if it just finished.
                    if mind_affliction.fire_timer.just_finished() {
                        // Call the main logic function, passing all required queries
                        spawn_repositioning_tether_attack(
                            &mut commands,
                            &asset_server,
                            player_entity,
                            player_stats.aim_direction, // Pass aim direction from player_stats
                            params,
                            weapon_id,
                            player_waiting_query, // Pass the query
                            horror_query,         // Pass the query
                            survivor_transform_query, // Pass the query for player transform
                        );
                        // Potentially play sound here or ensure spawn_repositioning_tether_attack does
                    }
                }
            }
        }
    }
}

// --- Orbiting Pet Systems (New Implementation) ---

pub fn spawn_orbiting_pet_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    player_transform: &Transform,
    params: &crate::items::OrbitingPetParams, // This is the NEW OrbitingPetParams
    orb_controller: &mut PlayerOrbControllerComponent,
) {
    // Initial spawn position can be right on the player; behavior system will adjust.
    // Or, apply an initial offset if desired.
    let initial_offset_angle = orb_controller.active_orb_entities.len() as f32 * (std::f32::consts::TAU / params.max_active_orbs as f32);
    let initial_pos = player_transform.translation + 
        Quat::from_rotation_z(initial_offset_angle) * Vec3::X * params.orbit_radius;

    let mut pulse_timer_opt = None;
    if params.pulses_aoe {
        pulse_timer_opt = Some(Timer::from_seconds(params.pulse_interval_secs, TimerMode::Repeating));
    }

    let mut bolt_timer_opt = None;
    if params.fires_seeking_bolts {
        bolt_timer_opt = Some(Timer::from_seconds(params.bolt_fire_interval_secs, TimerMode::Repeating));
    }

    let orb_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.orb_sprite_path),
            sprite: Sprite {
                custom_size: Some(params.orb_size),
                color: params.orb_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos),
            ..default()
        },
        OrbitingPetComponent { // New component
            params_snapshot: params.clone(),
            orbit_angle_rad: initial_offset_angle,
            duration_timer: Timer::from_seconds(params.orb_duration_secs, TimerMode::Once),
            pulse_timer: pulse_timer_opt,
            bolt_timer: bolt_timer_opt,
            owner_player_entity: player_entity,
        },
        Name::new("ShadowOrbInstance"),
    )).id();
    orb_controller.active_orb_entities.push(orb_entity);
}

pub fn manage_player_orbs_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, // For spawn_orbiting_pet_attack
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    mut player_query: Query<(Entity, &Transform, &mut Survivor, Option<&mut PlayerOrbControllerComponent>)>,
    orb_query: Query<Entity, With<OrbitingPetComponent>>, // To check if orb entities still exist
) {
    let Ok((player_entity, player_transform, mut player_stats, opt_orb_controller)) = player_query.get_single_mut() else { return; };

    let mut shadow_orb_params_opt: Option<crate::items::OrbitingPetParams> = None;
    if let Some(active_weapon_id) = player_stats.active_automatic_weapon_id {
        if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
            if let crate::items::AttackTypeData::OrbitingPet(params) = &weapon_def.attack_data {
                shadow_orb_params_opt = Some(params.clone());
            }
        }
    }

    if let Some(params) = shadow_orb_params_opt { // Shadow Orb is equipped
        let mut controller_exists_and_spawn = false;
        if let Some(mut controller) = opt_orb_controller {
            controller.spawn_cooldown_timer.tick(time.delta());
            if controller.spawn_cooldown_timer.finished() && controller.active_orb_entities.len() < controller.max_orbs_allowed {
                spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut controller);
                controller.spawn_cooldown_timer.reset();
            }
            // Clean up dead orb entities
            controller.active_orb_entities.retain(|&orb_e| orb_query.get(orb_e).is_ok());
            controller_exists_and_spawn = true;
        } else {
             // No controller, add one
            let mut new_controller = PlayerOrbControllerComponent {
                active_orb_entities: Vec::new(),
                max_orbs_allowed: params.max_active_orbs,
                spawn_cooldown_timer: Timer::from_seconds(params.base_fire_rate_secs, TimerMode::Repeating), // Repeating, will be reset on spawn
            };
            if new_controller.active_orb_entities.len() < new_controller.max_orbs_allowed {
                 spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut new_controller);
                 new_controller.spawn_cooldown_timer.reset(); // Start cooldown after first spawn
            }
            commands.entity(player_entity).insert(new_controller);
        }

    } else { // Shadow Orb is NOT equipped
        if let Some(mut controller) = opt_orb_controller {
            for orb_entity in controller.active_orb_entities.iter() {
                if orb_query.get(*orb_entity).is_ok() { // Check if entity still exists
                    commands.entity(*orb_entity).despawn_recursive();
                }
            }
            commands.entity(player_entity).remove::<PlayerOrbControllerComponent>();
        }
    }
}


pub fn orbiting_pet_behavior_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut pet_query: Query<(Entity, &mut Transform, &mut OrbitingPetComponent)>,
    player_query: Query<&Transform, (With<Survivor>, Without<OrbitingPetComponent>)>, // Player's transform
    horror_query: Query<(Entity, &GlobalTransform), With<Horror>>, // For targeting bolts and AoE
    mut horror_health_query: Query<&mut Health, With<Horror>>, // For applying damage
    // weapon_id needed for spawn_automatic_projectile, can be sourced from params_snapshot if needed
) {
    for (orb_entity, mut orb_transform, mut orb_comp) in pet_query.iter_mut() {
        // Duration
        orb_comp.duration_timer.tick(time.delta());
        if orb_comp.duration_timer.finished() {
            commands.entity(orb_entity).despawn_recursive();
            continue;
        }

        // Movement
        if let Ok(owner_transform) = player_query.get(orb_comp.owner_player_entity) {
            orb_comp.orbit_angle_rad += orb_comp.params_snapshot.orbit_speed_rad_per_sec * time.delta_seconds();
            orb_comp.orbit_angle_rad %= std::f32::consts::TAU; // Keep angle within 0-2PI

            let offset = Vec2::from_angle(orb_comp.orbit_angle_rad) * orb_comp.params_snapshot.orbit_radius;
            orb_transform.translation = owner_transform.translation + offset.extend(0.1); // Ensure Z-ordering for visibility
        } else {
            // Owner despawned, despawn orb too
            commands.entity(orb_entity).despawn_recursive();
            continue;
        }

        // AoE Pulse
        if orb_comp.params_snapshot.pulses_aoe {
            if let Some(ref mut pulse_timer) = orb_comp.pulse_timer {
                pulse_timer.tick(time.delta());
                if pulse_timer.just_finished() {
                    let orb_position = orb_transform.translation;
                    // Optional: Spawn a visual pulse effect
                    if let Some(pulse_viz_color) = orb_comp.params_snapshot.pulse_color {
                         commands.spawn((
                            SpriteBundle {
                                texture: asset_server.load("sprites/pulse_effect_placeholder.png"), // Placeholder
                                sprite: Sprite {
                                    color: pulse_viz_color,
                                    custom_size: Some(Vec2::splat(orb_comp.params_snapshot.pulse_radius * 0.25)), // Initial small size
                                    ..default()
                                },
                                transform: Transform::from_translation(orb_position),
                                ..default()
                            },
                            NovaVisualComponent { // Using NovaVisual for simplicity to expand and fade
                                initial_radius: orb_comp.params_snapshot.pulse_radius * 0.25,
                                max_radius: orb_comp.params_snapshot.pulse_radius,
                                duration_timer: Timer::from_seconds(0.3, TimerMode::Once),
                                color: pulse_viz_color,
                            },
                            Name::new("OrbPulseVisual"),
                        ));
                    }

                    for (horror_entity, horror_gtransform) in horror_query.iter() {
                        if horror_gtransform.translation().distance_squared(orb_position) < orb_comp.params_snapshot.pulse_radius.powi(2) {
                            if let Ok(mut health) = horror_health_query.get_mut(horror_entity) {
                                health.0 -= orb_comp.params_snapshot.pulse_damage;
                                crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), orb_comp.params_snapshot.pulse_damage, &time);
                            }
                        }
                    }
                }
            }
        }

        // Fire Bolts
        if orb_comp.params_snapshot.fires_seeking_bolts {
            if let Some(ref mut bolt_timer) = orb_comp.bolt_timer {
                bolt_timer.tick(time.delta());
                if bolt_timer.just_finished() {
                    // Simplified: Find nearest horror
                    let mut closest_target: Option<(Entity, f32)> = None;
                    let orb_pos_2d = orb_transform.translation.truncate();

                    for (horror_entity, horror_gtransform) in horror_query.iter() {
                        let dist_sq = orb_pos_2d.distance_squared(horror_gtransform.translation().truncate());
                        // Define a reasonable detection range for bolts, e.g., 300 units
                        if dist_sq < 300.0f32.powi(2) {
                             if closest_target.is_none() || dist_sq < closest_target.unwrap().1 {
                                closest_target = Some((horror_entity, dist_sq));
                            }
                        }
                    }

                    if let Some((target_entity, _)) = closest_target {
                        if let Ok(target_gtransform) = horror_query.get_component::<GlobalTransform>(target_entity) {
                            let direction = (target_gtransform.translation().truncate() - orb_pos_2d).normalize_or_zero();
                            if direction != Vec2::ZERO {
                                // Use params_snapshot for bolt properties
                                let bolt_sprite = orb_comp.params_snapshot.bolt_sprite_path.unwrap_or("sprites/default_bolt.png");
                                let bolt_sz = orb_comp.params_snapshot.bolt_size.unwrap_or_else(|| Vec2::new(10.0,10.0));
                                let bolt_col = orb_comp.params_snapshot.bolt_color.unwrap_or(Color::WHITE);
                                let bolt_lt = orb_comp.params_snapshot.bolt_lifetime_secs.unwrap_or(1.0);
                                // weapon_id for spawn_automatic_projectile is tricky here. 
                                // Maybe use a generic/default or pass one through params_snapshot if needed for specific on-hit effects.
                                // For now, using a placeholder AutomaticWeaponId(u32::MAX) to signify it's from a pet.
                                crate::automatic_projectiles::spawn_automatic_projectile(
                                    &mut commands,
                                    &asset_server,
                                    orb_transform.translation,
                                    direction,
                                    orb_comp.params_snapshot.bolt_damage,
                                    orb_comp.params_snapshot.bolt_speed,
                                    0, // Piercing
                                    AutomaticWeaponId(u32::MAX), // Placeholder ID for pet-fired bolts
                                    bolt_sprite,
                                    bolt_sz,
                                    bolt_col,
                                    bolt_lt,
                                    None, None, None, None, // Bouncing params
                                    None, // Lifesteal
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

// --- Lobbed Bouncing Magma Systems ---

pub fn spawn_magma_ball_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::LobbedBouncingMagmaParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    weapon_id: AutomaticWeaponId,
) {
    let initial_pos = player_transform.translation;
    let projectile_velocity = aim_direction.normalize_or_zero() * params.speed;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.sprite_path),
            sprite: Sprite {
                custom_size: Some(params.sprite_size),
                color: params.sprite_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        LobbedBouncingProjectileComponent {
            params: params.clone(),
            bounces_left: params.max_bounces,
            speed: params.speed,
            initial_spawn_position: initial_pos,
        },
        Velocity(projectile_velocity),
        Damage(params.damage), // Direct hit damage
        Lifetime { timer: Timer::from_seconds(params.lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile { // For collision detection and lifetime
            piercing_left: 0, // Not typical piercing, bounce logic handles multiple hits
            weapon_id,
            // Max bounces handled by LobbedBouncingProjectileComponent, but AutomaticProjectile needs some values
            bounces_left: Some(params.max_bounces), 
            damage_on_hit: params.damage,
            current_speed: params.speed,
            damage_loss_per_bounce_multiplier: Some(1.0), // No damage loss on direct hit, pool does its own damage
            speed_loss_per_bounce_multiplier: Some(1.0), // No speed loss for now
            has_bounced_this_frame: false,
            lifesteal_percentage: None,
        },
        Name::new("MagmaBallProjectile"),
    ));
}

pub fn lobbed_bouncing_projectile_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut projectile_query: Query<(
        Entity,
        &mut LobbedBouncingProjectileComponent,
        &mut Velocity,
        &mut Damage,
        &Transform,
        &mut Lifetime,
        &mut crate::automatic_projectiles::AutomaticProjectile, // To update its bounce count
    )>,
    // Consider adding query for horrors if direct damage on bounce is needed,
    // or if environment collision should also trigger bounces.
    // For now, AutomaticProjectile's collision handles horror hits.
) {
    for (
        entity,
        mut bouncing_comp,
        mut velocity,
        mut damage,
        transform,
        mut lifetime,
        mut auto_proj_comp,
    ) in projectile_query.iter_mut()
    {
        // Lifetime check
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() && bouncing_comp.bounces_left == 0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Arc movement (simplified, similar to existing lobbed_projectile_system but without early detonation)
        // This part might need more refinement if complex arcing is desired.
        // For now, basic velocity handles straight line movement.
        // Gravity could be added here: velocity.0.y -= GRAVITY_CONSTANT * time.delta_seconds();
        // And then adjust transform.rotation accordingly if the sprite needs to rotate with trajectory.

        // Collision and bounce logic is primarily handled by automatic_projectile_collision_system.
        // When a collision occurs there, it should decrement auto_proj_comp.bounces_left.
        // Here, we react to that change.

        if auto_proj_comp.bounces_left.is_some() && auto_proj_comp.bounces_left.unwrap() < bouncing_comp.bounces_left {
            // A bounce occurred (detected by change in AutomaticProjectile's bounce count)
            bouncing_comp.bounces_left = auto_proj_comp.bounces_left.unwrap();

            // Spawn Magma Pool at collision point (current transform.translation)
            if rand::random::<f32>() < bouncing_comp.params.fire_pool_chance_per_bounce {
                spawn_magma_pool(
                    &mut commands,
                    &asset_server,
                    transform.translation,
                    &bouncing_comp.params,
                );
            }

            if bouncing_comp.bounces_left == 0 {
                // Despawn after last bounce's effects (like pool spawning)
                commands.entity(entity).despawn_recursive();
                continue;
            }
            // Optional: Modify damage/speed after a bounce if needed
            // damage.0 = (damage.0 as f32 * bouncing_comp.params.damage_loss_per_bounce_multiplier).round() as i32;
            // velocity.0 *= bouncing_comp.params.speed_loss_per_bounce_multiplier;
            // auto_proj_comp.current_speed = velocity.0.length();
            // auto_proj_comp.damage_on_hit = damage.0;
        }
    }
}

pub fn spawn_magma_pool(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    magma_params: &crate::items::LobbedBouncingMagmaParams,
) {
    // Consider if there's a max number of active magma pools, similar to ichor pools.
    // If so, implement a resource and queue like ActiveIchorPools.

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/magma_pool_placeholder.png"), // Placeholder
            sprite: Sprite {
                color: magma_params.fire_pool_color,
                custom_size: Some(Vec2::splat(magma_params.fire_pool_radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position.truncate().extend(0.01)), // Slightly above ground
            ..default()
        },
        MagmaPoolComponent {
            damage_per_tick: magma_params.fire_pool_damage_per_tick,
            radius: magma_params.fire_pool_radius,
            tick_timer: Timer::from_seconds(magma_params.fire_pool_tick_interval_secs, TimerMode::Repeating),
            duration_timer: Timer::from_seconds(magma_params.fire_pool_duration_secs, TimerMode::Once),
            color: magma_params.fire_pool_color,
            already_hit_this_tick: Vec::new(),
        },
        Name::new("MagmaPool"),
    ));
}

pub fn magma_pool_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pool_query: Query<(Entity, &mut MagmaPoolComponent, &GlobalTransform)>,
    mut horror_query: Query<(Entity, &Transform, &mut Health), With<Horror>>,
    asset_server: Res<AssetServer>, // For damage text
    // Consider adding ResMut<ActiveMagmaPools> if limiting pool count
) {
    for (pool_entity, mut pool_comp, pool_gtransform) in pool_query.iter_mut() {
        pool_comp.duration_timer.tick(time.delta());
        if pool_comp.duration_timer.finished() {
            commands.entity(pool_entity).despawn_recursive();
            // If using ActiveMagmaPools, remove from resource here
            continue;
        }

        pool_comp.tick_timer.tick(time.delta());
        if pool_comp.tick_timer.just_finished() {
            pool_comp.already_hit_this_tick.clear();
            let pool_center_pos = pool_gtransform.translation().truncate();
            let pool_radius_sq = pool_comp.radius.powi(2);

            for (horror_entity, horror_transform, mut horror_health) in horror_query.iter_mut() {
                if pool_comp.already_hit_this_tick.contains(&horror_entity) {
                    continue;
                }
                let horror_pos = horror_transform.translation.truncate();
                if horror_pos.distance_squared(pool_center_pos) < pool_radius_sq {
                    horror_health.0 -= pool_comp.damage_per_tick;
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_transform.translation, pool_comp.damage_per_tick, &time);
                    pool_comp.already_hit_this_tick.push(horror_entity);
                }
            }
        }
    }
}

// --- Line Dash Attack Systems ---

pub fn spawn_line_dash_attack(
    commands: &mut Commands,
    player_entity: Entity,
    player_stats: &mut crate::survivor::Survivor,
    player_transform: &Transform,
    params: &crate::items::LineDashAttackParams,
    // asset_server: Res<AssetServer>, // Needed if spawning sounds/visuals directly here
    // mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>, // For sounds
) {
    let mut dash_direction = player_stats.aim_direction.normalize_or_zero();
    if dash_direction == Vec2::ZERO {
        dash_direction = (player_transform.rotation * Vec3::X).truncate().normalize_or_zero();
        if dash_direction == Vec2::ZERO {
            dash_direction = Vec2::X; 
        }
    }

    let original_speed_val = player_stats.speed;
    // It's debatable if player speed should be modified here or if the dashing system should just ignore it.
    // For now, just storing it. The dashing system will use params.dash_speed.
    // player_stats.speed = 0.0; // Optionally reduce player's normal movement speed during dash

    commands.entity(player_entity).insert(PlayerDashingComponent {
        params: params.clone(),
        initial_direction: dash_direction,
        dash_timer: Timer::from_seconds(params.dash_duration_secs, TimerMode::Once),
        already_hit_horrors: Vec::new(),
        original_speed_if_modified: Some(original_speed_val),
    });

    if params.invulnerable_during_dash {
        commands.entity(player_entity).insert(PlayerInvulnerableComponent);
    }
    // Example: sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::PlayerDash));
}

// Renamed from player_dash_execution_system and updated for LineDashAttackParams
pub fn player_dashing_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, 
    mut player_query: Query<(Entity, &mut Transform, &mut Survivor, &mut PlayerDashingComponent), (With<Survivor>, Without<Horror>)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>, // Added Horror to get size
    // mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>,
) {
    if let Ok((player_entity, mut player_transform, mut player_stats, mut dashing_comp)) = player_query.get_single_mut() {
        dashing_comp.dash_timer.tick(time.delta());

        let movement_this_frame = dashing_comp.initial_direction * dashing_comp.params.dash_speed * time.delta_seconds();
        player_transform.translation += movement_this_frame.extend(0.0);

        let player_hitbox_center = player_transform.translation.truncate();
        let player_half_width = dashing_comp.params.hitbox_width / 2.0;

        for (horror_entity, horror_gtransform, mut horror_health, horror_data) in horror_query.iter_mut() {
            if dashing_comp.already_hit_horrors.len() >= dashing_comp.params.piercing_cap as usize {
                break; 
            }
            if dashing_comp.already_hit_horrors.contains(&horror_entity) {
                continue; 
            }

            let horror_pos = horror_gtransform.translation().truncate();
            // Using horror_data.size for AABB check
            let horror_half_width = horror_data.size.x / 2.0;
            let horror_half_height = horror_data.size.y / 2.0;

            let x_collision = (player_hitbox_center.x - horror_pos.x).abs() * 2.0 < (dashing_comp.params.hitbox_width + horror_data.size.x);
            let y_collision = (player_hitbox_center.y - horror_pos.y).abs() * 2.0 < (dashing_comp.params.hitbox_width + horror_data.size.y); // Assuming player hitbox is also somewhat square for y-axis checks.

            if x_collision && y_collision {
                horror_health.0 -= dashing_comp.params.damage_per_hit;
                crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), dashing_comp.params.damage_per_hit, &time);
                dashing_comp.already_hit_horrors.push(horror_entity);
                // sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::HorrorHit));
            }
        }
        
        if let Some(color) = dashing_comp.params.dash_trail_color {
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("sprites/dash_trail_placeholder.png"), 
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(dashing_comp.params.hitbox_width * 0.7, dashing_comp.params.hitbox_width * 0.7)), 
                        color: color, 
                        ..default()
                    },
                    transform: Transform::from_translation(player_transform.translation - movement_this_frame.extend(0.0) * 0.25), 
                    ..default()
                },
                Lifetime { timer: Timer::from_seconds(0.2, TimerMode::Once) }, 
                Name::new("LineDashTrailSegment"),
            ));
        }

        if dashing_comp.dash_timer.finished() {
            if let Some(original_speed) = dashing_comp.original_speed_if_modified {
                player_stats.speed = original_speed;
            } else {
                player_stats.speed = BASE_PLAYER_SPEED; 
            }
            commands.entity(player_entity).remove::<PlayerDashingComponent>();
            if dashing_comp.params.invulnerable_during_dash {
                commands.entity(player_entity).remove::<PlayerInvulnerableComponent>();
            }
        }
    }
}


// --- Ground-Targeted Eruption Systems ---

pub fn ground_targeting_reticule_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    player_query: Query<(Entity, &GlobalTransform, &Survivor)>, 
    mut reticule_query: Query<(Entity, &mut Transform, &GroundTargetReticuleComponent, &Parent)>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((player_entity, player_gtransform, player_stats)) = player_query.get_single() else { return; };
    let player_pos_2d = player_gtransform.translation().truncate();

    let mut should_have_reticule = false;
    let mut current_reticule_params_opt: Option<crate::items::GroundTargetedAoEParams> = None;

    if let Some(active_weapon_id) = player_stats.active_automatic_weapon_id {
        if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
            if let crate::items::AttackTypeData::GroundTargetedAoE(params) = &weapon_def.attack_data {
                should_have_reticule = true;
                current_reticule_params_opt = Some(params.clone());
            }
        }
    }

    if should_have_reticule {
        let params = current_reticule_params_opt.unwrap(); 
        let window = windows.single();
        let (camera, camera_gtransform) = camera_q.single();
        
        let mut target_world_pos_2d = player_pos_2d + player_stats.aim_direction.normalize_or_zero() * params.targeting_range * 0.5; 
        if let Some(cursor_pos_screen) = window.cursor_position() {
            if let Some(cursor_world) = camera.viewport_to_world(camera_gtransform, cursor_pos_screen) {
                target_world_pos_2d = cursor_world.origin.truncate();
            }
        }
        
        let player_to_target = target_world_pos_2d - player_pos_2d;
        let distance = player_to_target.length();
        let clamped_distance = distance.min(params.targeting_range);
        let clamped_target_pos_2d = player_pos_2d + player_to_target.normalize_or_zero() * clamped_distance;

        let local_target_pos = clamped_target_pos_2d - player_pos_2d;

        if let Ok((_ret_entity, mut ret_transform, _ret_comp, _parent_comp)) = reticule_query.get_single_mut() {
            ret_transform.translation = local_target_pos.extend(0.2); 
        } else {
            let sprite_path = params.reticle_sprite_path.unwrap_or("sprites/ground_target_reticle_placeholder.png");
            let reticule_entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(sprite_path),
                    sprite: Sprite {
                        custom_size: Some(params.reticle_size),
                        color: params.aoe_color, 
                        ..default()
                    },
                    transform: Transform::from_translation(local_target_pos.extend(0.2)),
                    ..default()
                },
                GroundTargetReticuleComponent {
                    max_range: params.targeting_range,
                    visual_size: params.reticle_size,
                },
                Name::new("GroundTargetReticule"),
            )).id();
            commands.entity(player_entity).add_child(reticule_entity);
        }
    } else {
        for (ret_entity, _, _, _) in reticule_query.iter_mut() {
            commands.entity(ret_entity).despawn_recursive();
        }
    }
}


pub fn spawn_pending_ground_aoe_attack(
    commands: &mut Commands,
    params: &crate::items::GroundTargetedAoEParams,
    reticule_world_position: Vec3, 
) {
    commands.spawn((
        PendingGroundAoEComponent {
            position_of_impact: reticule_world_position,
            params: params.clone(),
            eruption_timer: Timer::from_seconds(params.delay_before_eruption_secs, TimerMode::Once),
            visual_eruption_effect_entity: None, 
        },
        TransformBundle::from_transform(Transform::from_translation(reticule_world_position)), 
        Name::new("PendingGroundAoE"),
    ));
}

pub fn pending_ground_aoe_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: &Res<AssetServer>, 
    mut pending_aoe_query: Query<(Entity, &mut PendingGroundAoEComponent)>, 
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &mut Velocity), With<Horror>>,
    // mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>,
) {
    for (pending_entity, mut pending_comp) in pending_aoe_query.iter_mut() { 
        pending_comp.eruption_timer.tick(time.delta());

        if pending_comp.eruption_timer.finished() {
            let sprite_path = pending_comp.params.visual_sprite_path.unwrap_or("sprites/eruption_effect_placeholder.png");
            let _eruption_visual_entity = commands.spawn(( 
                SpriteBundle {
                    texture: asset_server.load(sprite_path),
                    sprite: Sprite {
                        color: pending_comp.params.aoe_color,
                        custom_size: Some(Vec2::splat(10.0)), 
                        ..default()
                    },
                    transform: Transform::from_translation(pending_comp.position_of_impact.truncate().extend(0.1)), 
                    ..default()
                },
                EruptionVisualComponent {
                    duration_timer: Timer::from_seconds(pending_comp.params.aoe_visual_duration_secs, TimerMode::Once),
                    initial_radius: 10.0,
                    max_radius: pending_comp.params.eruption_radius,
                    color: pending_comp.params.aoe_color,
                },
                Name::new("EruptionVisual"),
            )).id();

            for (horror_entity, horror_gtransform, mut horror_health, mut horror_velocity) in horror_query.iter_mut() {
                if horror_gtransform.translation().distance_squared(pending_comp.position_of_impact) < pending_comp.params.eruption_radius.powi(2) {
                    let damage_to_apply = pending_comp.params.damage;
                    horror_health.0 -= damage_to_apply;
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), damage_to_apply, &time);
                    // sound_event_writer.send(crate::audio::PlaySoundEvent(SoundEffect::EarthShatterHit)); 

                    if pending_comp.params.knock_up_strength > 0.0 {
                        horror_velocity.0.y += pending_comp.params.knock_up_strength;
                    }

                    if let Some(root_duration) = pending_comp.params.root_duration_secs {
                        if root_duration > 0.0 {
                            commands.entity(horror_entity).insert(RootedComponent {
                                duration_timer: Timer::from_seconds(root_duration, TimerMode::Once),
                            });
                        }
                    }
                }
            }
            commands.entity(pending_entity).despawn_recursive();
        }
    }
}

pub fn eruption_visual_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EruptionVisualComponent, &mut Sprite)>, 
) {
    for (entity, mut visual, mut sprite) in query.iter_mut() { 
        visual.duration_timer.tick(time.delta());
        if visual.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            let progress = visual.duration_timer.fraction();
            let current_visual_radius = visual.initial_radius + (visual.max_radius - visual.initial_radius) * progress;
            sprite.custom_size = Some(Vec2::splat(current_visual_radius * 2.0));
            sprite.color.set_a(visual.color.a() * (1.0 - progress)); 
        }
    }
}


// --- Bouncing Projectile Systems ---

pub fn spawn_bouncing_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::BouncingProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2, 
    weapon_id: crate::items::AutomaticWeaponId,
) {
    let base_aim_direction_normalized = aim_direction.normalize_or_zero();

    for i in 0..params.num_shards_per_shot {
        let mut current_projectile_aim_direction = base_aim_direction_normalized;
        if params.num_shards_per_shot > 1 {
            let total_spread_rad = params.spread_angle_degrees.to_radians();
            let angle_offset_rad = if params.num_shards_per_shot <= 1 {
                0.0
            } else {
                (i as f32 / (params.num_shards_per_shot as f32 - 1.0)) * total_spread_rad - (total_spread_rad / 2.0)
            };
            
            let base_angle_rad = base_aim_direction_normalized.y.atan2(base_aim_direction_normalized.x);
            current_projectile_aim_direction = Vec2::new((base_angle_rad + angle_offset_rad).cos(), (base_angle_rad + angle_offset_rad).sin());
        }

        crate::automatic_projectiles::spawn_automatic_projectile(
            commands,
            asset_server,
            player_transform.translation, 
            current_projectile_aim_direction,
            params.base_damage,
            params.projectile_speed,
            0, 
            weapon_id,
            params.projectile_sprite_path,
            params.projectile_size,
            params.projectile_color,
            params.projectile_lifetime_secs,
            Some(params.max_bounces),
            Some(params.damage_loss_per_bounce_multiplier),
            Some(params.speed_loss_per_bounce_multiplier),
            None, 
        );
    }
}

pub fn channeled_beam_damage_system(
    mut _commands: Commands, 
    time: Res<Time>,
    mut beam_query: Query<(&mut ChanneledBeamComponent, &GlobalTransform)>, 
    mut enemy_query: Query<(Entity, &Transform, &mut Health), With<Horror>>, 
    // asset_server: Res<AssetServer>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (mut beam_comp, beam_gtransform) in beam_query.iter_mut() {
        beam_comp.tick_timer.tick(time.delta());
        if !beam_comp.tick_timer.just_finished() {
            continue;
        }

        let beam_start_pos = beam_gtransform.translation().truncate();
        let beam_rotation_quat = beam_gtransform.compute_transform().rotation;
        let beam_direction = (beam_rotation_quat * Vec3::X).truncate(); 

        for (_enemy_entity, enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
            let enemy_pos = enemy_transform.translation.truncate();
            let to_enemy = enemy_pos - beam_start_pos;
            let distance_along_beam = to_enemy.dot(beam_direction);
            
            if distance_along_beam > 0.0 && distance_along_beam < beam_comp.range {
                let perpendicular_distance = (to_enemy - distance_along_beam * beam_direction).length();
                let enemy_radius = 16.0; 
                if perpendicular_distance < (beam_comp.width / 2.0) + enemy_radius { 
                    enemy_health.0 -= beam_comp.damage_per_tick;
                }
            }
        }
    }
}

// --- Homing Debuff Projectile Systems ---

pub fn spawn_homing_debuff_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::HomingDebuffProjectileParams,
    player_transform: &Transform,
    initial_aim_direction: Vec2, 
    weapon_id: crate::items::AutomaticWeaponId,
    all_horrors_query: &Query<(Entity, &GlobalTransform), With<Horror>>,
) {
    let player_pos = player_transform.translation.truncate();

    for i in 0..params.num_darts_per_shot {
        let mut current_aim_direction = initial_aim_direction;
        if params.num_darts_per_shot > 1 {
            let spread_angle_degrees = 10.0 * (params.num_darts_per_shot as f32 -1.0) / 2.0;
            let angle_offset_degrees = if params.num_darts_per_shot > 1 {
                (i as f32 * (spread_angle_degrees * 2.0) / (params.num_darts_per_shot as f32 - 1.0)) - spread_angle_degrees
            } else { 0.0 };
            current_aim_direction = Quat::from_rotation_z(angle_offset_degrees.to_radians()) * current_aim_direction.extend(0.0);
            current_aim_direction = current_aim_direction.truncate().normalize_or_zero();
        }
        
        let mut closest_target: Option<(Entity, f32)> = None; 
        for (horror_entity, horror_gtransform) in all_horrors_query.iter() {
            let distance_sq = player_pos.distance_squared(horror_gtransform.translation().truncate());
            if distance_sq < params.homing_initial_target_search_radius.powi(2) {
                if closest_target.is_none() || distance_sq < closest_target.unwrap().1 {
                    closest_target = Some((horror_entity, distance_sq));
                }
            }
        }

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(params.projectile_sprite_path),
                sprite: Sprite {
                    custom_size: Some(params.projectile_size),
                    color: params.projectile_color,
                    ..default()
                },
                transform: Transform::from_translation(player_transform.translation)
                    .with_rotation(Quat::from_rotation_z(current_aim_direction.y.atan2(current_aim_direction.x))),
                ..default()
            },
            crate::automatic_projectiles::AutomaticProjectile {
                piercing_left: 0, 
                weapon_id,
                bounces_left: None, 
                damage_on_hit: params.base_damage,
                current_speed: params.projectile_speed,
                damage_loss_per_bounce_multiplier: None,
                speed_loss_per_bounce_multiplier: None,
                has_bounced_this_frame: false,
                lifesteal_percentage: None,
            },
            Velocity(current_aim_direction * params.projectile_speed),
            Damage(params.base_damage),
            Lifetime { timer: Timer::from_seconds(params.projectile_lifetime_secs, TimerMode::Once) },
            HomingTargetComponent {
                target_entity: closest_target.map(|(e, _)| e),
                strength: params.homing_strength,
            },
            DebuffOnHitComponent {
                debuff_type: params.debuff_type,
                magnitude_per_stack: params.debuff_magnitude_per_stack,
                max_stacks: params.max_debuff_stacks,
                duration_secs: params.debuff_duration_secs_on_target,
            },
            Name::new("MoonbeamDart"),
        ));
    }
}

pub fn homing_projectile_system(
    time: Res<Time>,
    mut projectile_query: Query<(&mut Velocity, &GlobalTransform, &mut HomingTargetComponent, &Lifetime)>,
    horror_query: Query<&GlobalTransform, With<Horror>>,
) {
    for (mut velocity, proj_gtransform, mut homing_comp, _lifetime) in projectile_query.iter_mut() {
        if let Some(target_entity) = homing_comp.target_entity {
            if let Ok(target_gtransform) = horror_query.get(target_entity) {
                let proj_pos = proj_gtransform.translation().truncate();
                let target_pos = target_gtransform.translation().truncate();

                let dir_to_target = (target_pos - proj_pos).normalize_or_zero();
                let current_dir = velocity.0.normalize_or_zero();

                if dir_to_target == Vec2::ZERO || current_dir == Vec2::ZERO {
                    continue; 
                }
                
                let angle_to_target = current_dir.angle_between(dir_to_target);
                let rotation_step = homing_comp.strength * time.delta_seconds(); 
                
                let rotation_this_frame = angle_to_target.clamp(-rotation_step, rotation_step);
                
                let new_dir = Quat::from_rotation_z(rotation_this_frame) * current_dir.extend(0.0);
                velocity.0 = new_dir.truncate().normalize_or_zero() * velocity.0.length();

            } else {
                homing_comp.target_entity = None;
            }
        }
    }
}

// --- Expanding Energy Bomb Systems ---

pub fn spawn_expanding_energy_bomb_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    _player_transform: &Transform, 
    params: &crate::items::ExpandingEnergyBombParams,
) {
    let initial_radius = 10.0; 
    let sprite_path = params.visual_sprite_path.unwrap_or("sprites/spirit_bomb_effect_placeholder.png");

    let bomb_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(sprite_path),
            sprite: Sprite {
                color: params.bomb_color,
                custom_size: Some(Vec2::splat(initial_radius * 2.0)), 
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO), 
            ..default()
        },
        ExpandingEnergyBombComponent {
            params: params.clone(),
            current_radius: initial_radius,
            expansion_timer: Timer::from_seconds(params.expansion_duration_secs, TimerMode::Once),
            wait_at_max_radius_timer: Timer::from_seconds(params.auto_detonation_delay_after_max_expansion_secs, TimerMode::Paused),
            state: SpiritBombState::Expanding,
        },
        Name::new("SpiritBombField"),
    )).id();

    commands.entity(player_entity).add_child(bomb_entity);
}

fn detonate_spirit_bomb(
    commands: &mut Commands,
    bomb_entity: Entity,
    bomb_comp: &ExpandingEnergyBombComponent,
    bomb_world_transform: &GlobalTransform, 
    horror_query: &mut Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>,
    asset_server: &Res<AssetServer>, 
    time: &Res<Time>,                
) {
    let bomb_center_pos = bomb_world_transform.translation();
    let damage_scale_factor = bomb_comp.current_radius / bomb_comp.params.max_radius;
    let damage = bomb_comp.params.min_damage_at_min_radius as f32 + 
                 (bomb_comp.params.max_damage_at_max_radius - bomb_comp.params.min_damage_at_min_radius) as f32 * damage_scale_factor;
    let damage_to_apply = damage.round() as i32;

    for (_horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() {
        if bomb_center_pos.distance_squared(horror_gtransform.translation()) < bomb_comp.current_radius.powi(2) {
            horror_health.0 -= damage_to_apply;
            crate::visual_effects::spawn_damage_text(commands, asset_server, horror_gtransform.translation(), damage_to_apply, time);
        }
    }
    
    commands.entity(bomb_entity).despawn_recursive(); 
}

pub fn expanding_energy_bomb_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, 
    mut bomb_query: Query<(Entity, &mut ExpandingEnergyBombComponent, &mut Transform, &mut Sprite, &GlobalTransform, Option<&Parent>)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>,
    player_query: Query<(Entity, &Transform), With<Survivor>>, 
) {
    for (bomb_entity, mut bomb_comp, mut bomb_transform, mut bomb_sprite, bomb_gtransform, opt_parent) in bomb_query.iter_mut() {
        
        if bomb_comp.state == SpiritBombState::Expanding {
            if let Some(parent) = opt_parent {
                if let Ok((_player_entity, _player_w_transform)) = player_query.get(parent.get()) {
                    bomb_transform.translation = Vec3::ZERO; 
                }
            }
        }


        match bomb_comp.state {
            SpiritBombState::Expanding => {
                bomb_comp.expansion_timer.tick(time.delta());
                let progress = bomb_comp.expansion_timer.fraction();
                let initial_small_radius = 10.0; 
                
                bomb_comp.current_radius = initial_small_radius + (bomb_comp.params.max_radius - initial_small_radius) * progress;
                bomb_sprite.custom_size = Some(Vec2::splat(bomb_comp.current_radius * 2.0));

                if bomb_comp.expansion_timer.finished() {
                    bomb_comp.state = SpiritBombState::WaitingAtMaxRadius;
                    bomb_comp.current_radius = bomb_comp.params.max_radius; 
                    bomb_sprite.custom_size = Some(Vec2::splat(bomb_comp.current_radius * 2.0)); 
                    bomb_comp.wait_at_max_radius_timer.unpause(); 
                }
            }
            SpiritBombState::WaitingAtMaxRadius => {
                bomb_comp.wait_at_max_radius_timer.tick(time.delta());
                let mut detonate_now = false;

                if bomb_comp.wait_at_max_radius_timer.finished() {
                    detonate_now = true;
                }

                if detonate_now {
                    let detonation_center_gtransform = if let Some(parent) = opt_parent {
                        if let Ok((_player_entity, player_w_transform)) = player_query.get(parent.get()) {
                             GlobalTransform::from(*player_w_transform) 
                        } else {
                            *bomb_gtransform 
                        }
                    } else {
                        *bomb_gtransform 
                    };

                    detonate_spirit_bomb(&mut commands, bomb_entity, &bomb_comp, &detonation_center_gtransform, &mut horror_query, &asset_server, &time);
                    bomb_comp.state = SpiritBombState::Detonated; 
                }
            }
            SpiritBombState::Detonated => {
            }
        }
    }
}

// --- Debuffing Aura/Cloud Systems ---

pub fn spawn_debuff_aura_cloud_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_transform: &Transform,
    params: &crate::items::DebuffAuraParams,
) {
    let sprite_path = params.visual_sprite_path.unwrap_or("sprites/sand_cloud_placeholder.png");

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(sprite_path),
            sprite: Sprite {
                color: params.cloud_color,
                custom_size: Some(Vec2::splat(params.cloud_radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation), 
            ..default()
        },
        DebuffCloudVisualComponent {
            params: params.clone(),
            duration_timer: Timer::from_seconds(params.cloud_duration_secs, TimerMode::Once),
            already_hit_horrors: Vec::new(),
        },
        Name::new("DebuffAuraCloud"),
    ));
}

pub fn debuff_cloud_system(
    mut commands: Commands,
    time: Res<Time>,
    mut cloud_query: Query<(Entity, &mut DebuffCloudVisualComponent, &GlobalTransform)>,
    horror_query: Query<(Entity, &GlobalTransform), With<Horror>>,
) {
    for (cloud_entity, mut cloud_visual, cloud_gtransform) in cloud_query.iter_mut() {
        cloud_visual.duration_timer.tick(time.delta());
        if cloud_visual.duration_timer.finished() {
            commands.entity(cloud_entity).despawn_recursive();
            continue;
        }

        let cloud_center_pos = cloud_gtransform.translation().truncate();
        let cloud_radius_sq = cloud_visual.params.cloud_radius.powi(2);

        for (horror_entity, horror_gtransform) in horror_query.iter() {
            if cloud_visual.already_hit_horrors.contains(&horror_entity) {
                continue;
            }

            let horror_pos = horror_gtransform.translation().truncate();
            if horror_pos.distance_squared(cloud_center_pos) < cloud_radius_sq {
                match cloud_visual.params.debuff_type {
                    crate::items::AuraDebuffType::ReduceAccuracy => {
                        commands.entity(horror_entity).insert(
                            crate::components::AccuracyDebuffComponent { 
                                reduction_factor: cloud_visual.params.debuff_magnitude,
                                duration_timer: Timer::from_seconds(cloud_visual.params.debuff_duration_secs, TimerMode::Once),
                            }
                        );
                    }
                    crate::items::AuraDebuffType::SlowAttackSpeed => {
                         commands.entity(horror_entity).insert(
                            crate::components::AttackSpeedDebuffComponent { 
                                multiplier: cloud_visual.params.debuff_magnitude,
                                duration_timer: Timer::from_seconds(cloud_visual.params.debuff_duration_secs, TimerMode::Once),
                            }
                        );
                    }
                    crate::items::AuraDebuffType::MinorDamageOverTime => {
                        commands.entity(horror_entity).insert(
                            crate::components::ContinuousDamageComponent { 
                                damage_per_tick: cloud_visual.params.debuff_magnitude, 
                                tick_interval: 0.5, 
                                duration_timer: Timer::from_seconds(cloud_visual.params.debuff_duration_secs, TimerMode::Once),
                            }
                        );
                    }
                }
                cloud_visual.already_hit_horrors.push(horror_entity);
            }
        }
    }
}

// --- Persistent Aura System ---

pub fn manage_persistent_aura_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: &Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    player_query: Query<(Entity, &Transform, &Survivor, Option<&PlayerPersistentAuraComponent>)>,
    mut horror_query: Query<(&GlobalTransform, &mut Health), With<Horror>>,
) {
    let Ok((player_entity, player_transform, player_stats, opt_aura_comp)) = player_query.get_single() else { return; };

    let mut should_have_aura = false;
    let mut current_aura_params_opt: Option<crate::items::PersistentAuraParams> = None;
    let mut current_weapon_id_opt: Option<AutomaticWeaponId> = None;

    if let Some(active_weapon_id) = player_stats.active_automatic_weapon_id {
        if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
            if let crate::items::AttackTypeData::PersistentAura(params) = &weapon_def.attack_data {
                should_have_aura = true;
                current_aura_params_opt = Some(params.clone());
                current_weapon_id_opt = Some(active_weapon_id);
            }
        }
    }

    match (should_have_aura, opt_aura_comp, current_aura_params_opt, current_weapon_id_opt) {
        (true, None, Some(params), Some(weapon_id)) => {
            let mut visual_entity_opt = None;
            if let Some(sprite_path) = params.visual_sprite_path {
                let visual_entity = commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load(sprite_path),
                        sprite: Sprite {
                            color: params.aura_color,
                            custom_size: Some(Vec2::splat(params.radius * 2.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::ZERO), 
                        ..default()
                    },
                    Name::new("PersistentAuraVisual"),
                )).id();
                commands.entity(player_entity).add_child(visual_entity);
                visual_entity_opt = Some(visual_entity);
            }

            commands.entity(player_entity).insert(PlayerPersistentAuraComponent {
                damage_per_tick: params.damage_per_tick,
                tick_timer: Timer::from_seconds(params.tick_interval_secs, TimerMode::Repeating),
                radius: params.radius,
                aura_color: params.aura_color,
                visual_entity: visual_entity_opt,
                weapon_id: weapon_id,
            });
        }
        (true, Some(mut active_aura), Some(params), Some(weapon_id)) => {
            if active_aura.weapon_id != weapon_id || active_aura.radius != params.radius {
                if let Some(old_visual) = active_aura.visual_entity {
                    commands.entity(old_visual).despawn_recursive();
                }
                let mut new_visual_entity_opt = None;
                if let Some(sprite_path) = params.visual_sprite_path {
                     let new_visual_entity = commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load(sprite_path),
                            sprite: Sprite {
                                color: params.aura_color,
                                custom_size: Some(Vec2::splat(params.radius * 2.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::ZERO),
                            ..default()
                        },
                        Name::new("PersistentAuraVisual"),
                    )).id();
                    commands.entity(player_entity).add_child(new_visual_entity);
                    new_visual_entity_opt = Some(new_visual_entity);
                }
                commands.entity(player_entity).insert(PlayerPersistentAuraComponent {
                    damage_per_tick: params.damage_per_tick,
                    tick_timer: Timer::from_seconds(params.tick_interval_secs, TimerMode::Repeating), 
                    radius: params.radius,
                    aura_color: params.aura_color,
                    visual_entity: new_visual_entity_opt,
                    weapon_id: weapon_id,
                });

            } else {
                let mut mutable_aura_comp = commands.entity(player_entity).get_mut::<PlayerPersistentAuraComponent>().unwrap(); 
                mutable_aura_comp.tick_timer.tick(time.delta());
                if mutable_aura_comp.tick_timer.just_finished() {
                    let player_world_pos = player_transform.translation;
                    for (horror_gtransform, mut horror_health) in horror_query.iter_mut() {
                        if player_world_pos.distance_squared(horror_gtransform.translation()) < mutable_aura_comp.radius.powi(2) {
                            horror_health.0 -= mutable_aura_comp.damage_per_tick;
                            crate::visual_effects::spawn_damage_text(
                                &mut commands, 
                                &asset_server, 
                                horror_gtransform.translation(), 
                                mutable_aura_comp.damage_per_tick, 
                                &time
                            );
                        }
                    }
                }
            }
        }
        (false, Some(active_aura), _, _) => {
            if let Some(visual_entity) = active_aura.visual_entity {
                commands.entity(visual_entity).despawn_recursive();
            }
            commands.entity(player_entity).remove::<PlayerPersistentAuraComponent>();
        }
        _ => {
        }
    }
}

// --- Point-Blank Nova Systems ---

pub fn spawn_point_blank_nova_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_transform: &Transform,
    params: &crate::items::PointBlankNovaParams,
    all_horrors_query: &mut Query<(Entity, &GlobalTransform, &mut Health, &mut Velocity), With<Horror>>, 
    time: &Res<Time>, 
    sound_event_writer: &mut EventWriter<crate::audio::PlaySoundEvent>, 
) {
    let player_pos = player_transform.translation;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/nova_effect_placeholder.png"), 
            sprite: Sprite {
                color: params.nova_color,
                custom_size: Some(Vec2::splat(10.0)), 
                ..default()
            },
            transform: Transform::from_translation(player_pos),
            ..default()
        },
        NovaVisualComponent {
            initial_radius: 10.0,
            max_radius: params.radius,
            duration_timer: Timer::from_seconds(params.visual_duration_secs, TimerMode::Once),
            color: params.nova_color,
        },
        Name::new("GlacialNovaVisual"),
    ));

    for (horror_entity, horror_gtransform, mut horror_health, mut horror_velocity) in all_horrors_query.iter_mut() {
        let horror_pos = horror_gtransform.translation();
        let distance_sq = player_pos.distance_squared(horror_pos);

        if distance_sq < params.radius.powi(2) {
            horror_health.0 -= params.damage;
            crate::visual_effects::spawn_damage_text(commands, asset_server, horror_pos, params.damage, time);
            // sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::GlacialNovaHit)); 
            
            commands.entity(horror_entity).insert(crate::horror::Frozen { 
                timer: Timer::from_seconds(params.slow_duration_secs, TimerMode::Once),
                speed_multiplier: params.slow_effect_multiplier,
            });
        }
    }
}

pub fn nova_visual_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut NovaVisualComponent, &mut Sprite)>, 
) {
    for (entity, mut visual, mut sprite) in query.iter_mut() { 
        visual.duration_timer.tick(time.delta());

        if visual.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            let progress = visual.duration_timer.fraction();
            sprite.custom_size = Some(Vec2::splat(
                visual.initial_radius + (visual.max_radius - visual.initial_radius) * progress,
            ));
            sprite.color.set_a(visual.color.a() * (1.0 - progress)); 
        }
    }
}

// --- Chain Lightning Systems ---

pub fn spawn_zap_visual(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    target_pos: Vec3,
    params: &crate::items::ChainZapParams,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/zap_placeholder.png"), 
            sprite: Sprite {
                custom_size: Some(Vec2::new(params.zap_width * 4.0, params.zap_width * 4.0)), 
                color: params.zap_color,
                ..default()
            },
            transform: Transform::from_translation(target_pos),
            ..default()
        },
        ChainLightningZapEffectComponent {
            duration_timer: Timer::from_seconds(params.zap_duration_secs, TimerMode::Once),
        },
        Name::new("ChainLightningZapVisual"),
    ));
}

pub fn trigger_chain_lightning_zaps(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_transform: &Transform,
    horrors_query: &mut Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>,
    params: &crate::items::ChainZapParams,
    time: &Res<Time>, 
    sound_event_writer: &mut EventWriter<crate::audio::PlaySoundEvent>,
) {
    let mut already_zapped_entities: Vec<Entity> = Vec::new();
    let mut current_damage = params.base_damage_per_zap as f32;
    let mut last_zap_end_position = player_transform.translation; 
    let mut current_target_opt: Option<(Entity, Vec3)> = None; 

    let mut closest_initial_target: Option<(Entity, f32, Vec3)> = None; 
    for (horror_entity, horror_gtransform, _health) in horrors_query.iter_mut() {
        let distance_sq = player_transform.translation.distance_squared(horror_gtransform.translation());
        if distance_sq < params.initial_target_range.powi(2) {
            if closest_initial_target.is_none() || distance_sq < closest_initial_target.unwrap().1 {
                closest_initial_target = Some((horror_entity, distance_sq, horror_gtransform.translation()));
            }
        }
    }

    if let Some((initial_entity, _, initial_pos)) = closest_initial_target {
        current_target_opt = Some((initial_entity, initial_pos));
    } else {
        return; 
    }

    for _chain_idx in 0..params.max_chains {
        if let Some((target_entity, target_pos)) = current_target_opt {
            if let Ok((_, _, mut health)) = horrors_query.get_mut(target_entity) {
                let damage_to_apply = current_damage.round() as i32;
                health.0 -= damage_to_apply;
                
                crate::visual_effects::spawn_damage_text(commands, asset_server, target_pos, damage_to_apply, time);
                sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::ChainLightningZap));

            } else { 
                current_target_opt = None; 
                break;
            }

            spawn_zap_visual(commands, asset_server, target_pos, params);
            already_zapped_entities.push(target_entity);
            
            last_zap_end_position = target_pos;
            current_damage *= params.damage_falloff_per_chain;
            current_target_opt = None; 

            let mut closest_next_target: Option<(Entity, f32, Vec3)> = None;
            for (horror_entity, horror_gtransform, _health) in horrors_query.iter_mut() {
                if already_zapped_entities.contains(&horror_entity) {
                    continue; 
                }
                let distance_sq = last_zap_end_position.distance_squared(horror_gtransform.translation());
                if distance_sq < params.chain_search_radius.powi(2) {
                    if closest_next_target.is_none() || distance_sq < closest_next_target.unwrap().1 {
                        closest_next_target = Some((horror_entity, distance_sq, horror_gtransform.translation()));
                    }
                }
            }
            if let Some((next_entity, _, next_pos)) = closest_next_target {
                current_target_opt = Some((next_entity, next_pos));
            } else {
                break; 
            }
        } else {
            break; 
        }
    }
}

pub fn chain_lightning_visual_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ChainLightningZapEffectComponent)>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.duration_timer.tick(time.delta());
        if effect.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// --- Trail of Fire Systems ---

pub fn spawn_trail_of_fire_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::TrailOfFireParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    weapon_id: AutomaticWeaponId, 
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.projectile_sprite_path),
            sprite: Sprite {
                custom_size: Some(params.projectile_size),
                color: params.projectile_color,
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        TrailSpawningProjectileComponent {
            trail_params: params.clone(),
            segment_spawn_timer: Timer::from_seconds(params.trail_segment_spawn_interval_secs, TimerMode::Repeating),
        },
        Velocity(aim_direction.normalize_or_zero() * params.projectile_speed),
        Damage(params.base_damage_on_impact),
        Lifetime { timer: Timer::from_seconds(params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile {
            piercing_left: 0, 
            weapon_id: weapon_id, 
        },
        Name::new("InfernoBoltProjectile"),
    ));
}

pub fn trail_spawning_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, 
    mut query: Query<(&mut TrailSpawningProjectileComponent, &Transform)>,
) {
    for (mut projectile, transform) in query.iter_mut() {
        projectile.segment_spawn_timer.tick(time.delta());
        if projectile.segment_spawn_timer.just_finished() {
            spawn_fire_trail_segment(
                &mut commands,
                &asset_server,
                transform.translation,
                &projectile.trail_params,
            );
        }
    }
}

pub fn spawn_fire_trail_segment(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    trail_params: &crate::items::TrailOfFireParams,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/fire_segment_placeholder.png"), 
            sprite: Sprite {
                custom_size: Some(Vec2::new(trail_params.trail_segment_width, trail_params.trail_segment_width)), 
                color: trail_params.trail_segment_color,
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        FireTrailSegmentComponent {
            damage_per_tick: trail_params.trail_segment_damage_per_tick,
            tick_timer: Timer::from_seconds(trail_params.trail_segment_tick_interval_secs, TimerMode::Repeating),
            duration_timer: Timer::from_seconds(trail_params.trail_segment_duration_secs, TimerMode::Once),
            width: trail_params.trail_segment_width,
            already_hit_this_tick: Vec::new(),
        },
        Name::new("FireTrailSegment"),
    ));
}

pub fn fire_trail_segment_system(
    mut commands: Commands,
    time: Res<Time>,
    mut segment_query: Query<(Entity, &mut FireTrailSegmentComponent, &GlobalTransform)>,
    mut horror_query: Query<(Entity, &Transform, &mut Health), With<Horror>>,
) {
    for (segment_entity, mut segment, segment_gtransform) in segment_query.iter_mut() {
        segment.duration_timer.tick(time.delta());
        if segment.duration_timer.finished() {
            commands.entity(segment_entity).despawn_recursive();
            continue;
        }

        segment.tick_timer.tick(time.delta());
        if segment.tick_timer.just_finished() {
            segment.already_hit_this_tick.clear();
            let segment_pos = segment_gtransform.translation().truncate();
            let segment_radius_sq = (segment.width / 2.0).powi(2); 

            for (horror_entity, horror_transform, mut horror_health) in horror_query.iter_mut() {
                if segment.already_hit_this_tick.contains(&horror_entity) {
                    continue;
                }

                let horror_pos = horror_transform.translation.truncate();
                if horror_pos.distance_squared(segment_pos) < segment_radius_sq {
                    horror_health.0 -= segment.damage_per_tick;
                    segment.already_hit_this_tick.push(horror_entity);
                }
            }
        }
    }
}

// --- Charge-Up Energy Shot Systems ---

pub fn spawn_charge_shot_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    charge_params: &crate::items::ChargeUpEnergyShotParams, 
    chosen_level_params: &crate::items::ChargeLevelParams, 
    player_transform: &Transform,
    aim_direction: Vec2,
) {
    let projectile_sprite = chosen_level_params.projectile_sprite_path_override
        .unwrap_or(charge_params.base_projectile_sprite_path);

    let mut projectile_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(projectile_sprite),
            sprite: Sprite {
                custom_size: Some(chosen_level_params.projectile_size),
                color: charge_params.base_projectile_color, 
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation) 
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        Velocity(aim_direction.normalize_or_zero() * chosen_level_params.projectile_speed),
        Damage(chosen_level_params.projectile_damage),
        Lifetime { timer: Timer::from_seconds(charge_params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile { 
            piercing_left: chosen_level_params.piercing,
            already_hit_entities: Vec::new(), 
        },
        Name::new("ChargeShotProjectile"),
    ));

    if chosen_level_params.explodes_on_impact {
        projectile_commands.insert(ExplodesOnFinalImpact {
            explosion_radius: chosen_level_params.explosion_radius,
            explosion_damage: chosen_level_params.explosion_damage,
        });
    }
}


pub fn charge_weapon_system(
    time: Res<Time>,
    mut query: Query<&mut ChargingWeaponComponent>, 
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>, 
) {
    for mut charging_comp in query.iter_mut() {
        if !charging_comp.is_actively_charging {
            continue; 
        }

        if let Some(weapon_def) = weapon_library.get_weapon_definition(charging_comp.weapon_id) {
            if let crate::items::AttackTypeData::ChargeUpEnergyShot(charge_params) = &weapon_def.attack_data {
                
                charging_comp.charge_timer.tick(time.delta());

                let mut new_charge_level_index = 0; 
                for (idx, level_params) in charge_params.charge_levels.iter().enumerate() {
                    if charging_comp.charge_timer.elapsed_secs() >= level_params.charge_time_secs {
                        new_charge_level_index = idx;
                    } else {
                        break; 
                    }
                }
                charging_comp.current_charge_level_index = new_charge_level_index;
            }
        }
    }
}

// --- Lobbed AoE Pool Systems ---

pub fn spawn_lobbed_aoe_pool_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &crate::items::LobbedAoEPoolParams,
    player_transform: &Transform,
    aim_direction: Vec2,
) {
    let initial_pos = player_transform.translation;
    let projectile_velocity = aim_direction.normalize_or_zero() * params.projectile_speed;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.projectile_sprite_path),
            sprite: Sprite {
                custom_size: Some(params.projectile_size),
                color: params.projectile_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        LobbedProjectileComponent {
            arc_height: params.projectile_arc_height,
            speed: params.projectile_speed,
            pool_params: params.clone(), 
            initial_spawn_position: initial_pos,
        },
        Velocity(projectile_velocity),
        Damage(params.base_damage_on_impact), 
        Lifetime { timer: Timer::from_seconds(2.0, TimerMode::Once) }, 
        Name::new("LobbedIchorProjectile"),
    ));
}

pub fn lobbed_projectile_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>, 
    mut active_pools_res: ResMut<ActiveIchorPools>, 
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &LobbedProjectileComponent, &Transform, &mut Lifetime)>,
) {
    for (entity, lobbed_comp, transform, mut lifetime) in projectile_query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.just_finished() {
            spawn_ichor_pool(
                &mut commands,
                &asset_server,
                transform.translation, 
                &lobbed_comp.pool_params, 
                &mut active_pools_res,
            );
            commands.entity(entity).despawn_recursive(); 
        }
    }
}

pub fn spawn_ichor_pool(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>, 
    position: Vec3,
    pool_params: &crate::items::LobbedAoEPoolParams,
    active_pools_res: &mut ResMut<ActiveIchorPools>,
) {
    if active_pools_res.pools.len() >= pool_params.max_active_pools as usize {
        if let Some(oldest_pool_entity) = active_pools_res.pools.pop_front() {
            commands.entity(oldest_pool_entity).despawn_recursive();
        }
    }

    let pool_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/ichor_pool_placeholder.png"), 
            sprite: Sprite {
                color: pool_params.pool_color,
                custom_size: Some(Vec2::new(pool_params.pool_radius * 2.0, pool_params.pool_radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        IchorPoolComponent {
            damage_per_tick: pool_params.pool_damage_per_tick,
            radius: pool_params.pool_radius,
            tick_timer: Timer::from_seconds(pool_params.pool_tick_interval_secs, TimerMode::Repeating),
            duration_timer: Timer::from_seconds(pool_params.pool_duration_secs, TimerMode::Once),
            color: pool_params.pool_color, 
        },
        Name::new("IchorPool"),
    )).id();

    active_pools_res.pools.push_back(pool_entity); 
}

pub fn ichor_pool_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pool_query: Query<(Entity, &mut IchorPoolComponent, &GlobalTransform)>,
    mut horror_query: Query<(&Transform, &mut Health), With<Horror>>,
    mut active_pools_res: ResMut<ActiveIchorPools>,
) {
    for (pool_entity, mut pool_comp, pool_gtransform) in pool_query.iter_mut() {
        pool_comp.duration_timer.tick(time.delta());
        if pool_comp.duration_timer.finished() {
            commands.entity(pool_entity).despawn_recursive();
            active_pools_res.pools.retain(|&e| e != pool_entity);
            continue; 
        }

        pool_comp.tick_timer.tick(time.delta());
        if pool_comp.tick_timer.just_finished() {
            let pool_center_pos = pool_gtransform.translation().truncate();
            let pool_radius_sq = pool_comp.radius * pool_comp.radius;

            for (horror_transform, mut horror_health) in horror_query.iter_mut() {
                let horror_pos = horror_transform.translation.truncate();
                if horror_pos.distance_squared(pool_center_pos) < pool_radius_sq {
                    horror_health.0 -= pool_comp.damage_per_tick;
                }
            }
        }
    }
}

pub fn channeled_beam_update_system(
    player_query: Query<(&Transform, &Survivor), (With<Survivor>, Without<ChanneledBeamComponent>)>, 
    mut beam_query: Query<(&mut Transform, &ChanneledBeamComponent)>, 
) {
    for (mut beam_transform, beam_comp) in beam_query.iter_mut() {
        if let Ok((player_transform, player_stats)) = player_query.get(beam_comp.owner) {
            beam_transform.translation = player_transform.translation; 
            
            let aim_direction = player_stats.aim_direction;
            if aim_direction != Vec2::ZERO {
                beam_transform.rotation = Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x));
            }
        } 
    }
}

pub fn returning_projectile_system(
    mut commands: Commands,
    _time: Res<Time>, 
    mut query: Query<(Entity, &mut ReturningProjectileComponent, &mut Velocity, &Transform)>,
) {
    for (entity, mut projectile_comp, mut velocity, transform) in query.iter_mut() {
        match projectile_comp.state {
            ReturningProjectileState::Outgoing => {
                let distance_traveled = transform.translation.distance(projectile_comp.start_position);
                if distance_traveled >= projectile_comp.max_travel_distance {
                    projectile_comp.state = ReturningProjectileState::Returning;
                    
                    let direction_to_start = (projectile_comp.start_position - transform.translation).truncate().normalize_or_zero();
                    velocity.0 = direction_to_start * projectile_comp.speed;
                }
            }
            ReturningProjectileState::Returning => {
                let distance_to_target = transform.translation.distance(projectile_comp.start_position);
                if distance_to_target < 5.0 { 
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn spawn_standard_projectile_attack(
    _commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    params: &StandardProjectileParams, 
    _player_transform: &Transform,
    _aim_direction: Vec2,
    _weapon_id: AutomaticWeaponId 
) {
    info!("spawn_standard_projectile_attack called for sprite: {}, damage: {}, fire_rate: {}", params.projectile_sprite_path, params.base_damage, params.base_fire_rate_secs);
}

pub fn spawn_returning_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &ReturningProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2,
) {
    let start_pos = player_transform.translation;
    let projectile_velocity = aim_direction.normalize_or_zero() * params.projectile_speed;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.projectile_sprite_path),
            sprite: Sprite {
                custom_size: Some(params.projectile_size),
                color: params.projectile_color,
                ..default()
            },
            transform: Transform::from_translation(start_pos)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        ReturningProjectileComponent {
            state: ReturningProjectileState::Outgoing,
            start_position: start_pos,
            max_travel_distance: params.travel_distance,
            speed: params.projectile_speed,
            piercing_left: params.piercing,
        },
        Velocity(projectile_velocity), 
        Damage(params.base_damage),    
        Lifetime { 
            timer: Timer::from_seconds( (params.travel_distance / params.projectile_speed) * 2.5, TimerMode::Once) 
        },
        Name::new("ReturningProjectile"),
    ));
}

pub fn player_is_channeling_effect_system(
    mut player_query: Query<(Entity, &mut Survivor, Option<&IsChannelingComponent>)>, 
) {
    for (player_entity, mut player_stats, opt_is_channeling_comp) in player_query.iter_mut() {
        if let Some(is_channeling_comp) = opt_is_channeling_comp {
            let target_speed = BASE_PLAYER_SPEED * is_channeling_comp.beam_params.movement_penalty_multiplier;
            if player_stats.speed != target_speed {
                player_stats.speed = target_speed; 
                info!("Player {:?} speed set to {} due to channeling.", player_entity, target_speed);
            }
        } else {
            if player_stats.speed != BASE_PLAYER_SPEED { 
                player_stats.speed = BASE_PLAYER_SPEED; 
                info!("Player {:?} speed reset to {}.", player_entity, BASE_PLAYER_SPEED); 
            }
        }
    }
}

pub fn execute_cone_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>, 
    params: &ConeAttackParams,
    player_transform: &Transform,
    aim_direction: Vec2, 
    mut enemy_query: Query<(Entity, &Transform, &mut Health), With<Horror>>,
    // Added time for Lifetime component
    time: Res<Time>,
) {
    let player_pos = player_transform.translation.truncate();
    let forward_vector = aim_direction.normalize_or_zero();

    // Spawn visual effect for the cone
    if let Some(sprite_path) = params.visual_sprite_path {
        let mut sprite_size = Vec2::new(params.cone_radius, params.cone_radius * 0.5); // Default size if not scaled
        if let Some((radius_scale, angle_scale)) = params.visual_size_scale_with_radius_angle {
            sprite_size = Vec2::new(
                params.cone_radius * radius_scale, 
                params.cone_radius * params.cone_angle_degrees.to_radians() * angle_scale
            );
        }

        let mut visual_transform = Transform::from_translation(player_transform.translation);
        visual_transform.rotation = Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x));
        
        let mut anchor = bevy::sprite::Anchor::CenterLeft; // Default anchor
        if let Some(offset) = params.visual_anchor_offset {
            // Apply offset relative to player's facing direction
            let rotated_offset = visual_transform.rotation * offset.extend(0.0);
            visual_transform.translation += rotated_offset;
            // If an offset is used, anchor might need to be custom or adjusted based on sprite design
            // For simplicity, we'll keep CenterLeft but the user might need to adjust sprite or anchor for perfect alignment
        }
        
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(sprite_path),
                sprite: Sprite {
                    custom_size: Some(sprite_size),
                    color: params.color, // Apply tint
                    anchor,
                    ..default()
                },
                transform: visual_transform,
                ..default()
            },
            Lifetime { timer: Timer::from_seconds(0.25, TimerMode::Once) }, // Short lifetime for sweep visual
            Name::new("ConeAttackVisual"),
        ));
    }

    for (enemy_entity, enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        let vector_to_enemy = enemy_pos - player_pos;
        
        let distance_to_enemy_sq = vector_to_enemy.length_squared();
        if distance_to_enemy_sq > params.cone_radius * params.cone_radius {
            continue; 
        }

        if vector_to_enemy != Vec2::ZERO { 
            let angle_to_enemy_rad = forward_vector.angle_between(vector_to_enemy.normalize_or_zero());
            let half_cone_angle_rad = params.cone_angle_degrees.to_radians() / 2.0;

            if angle_to_enemy_rad.abs() <= half_cone_angle_rad {
                enemy_health.0 -= params.base_damage;
                // Consider spawning damage text here if not handled by a global system
                // crate::visual_effects::spawn_damage_text(commands, asset_server, enemy_transform.translation, params.base_damage, &time);
            }
        }
    }
}