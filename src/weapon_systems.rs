// mescgit/eldritchhero/EldritchHero-c197490fa863f6ebd3e83365f89cc741bfb8b804/src/weapon_systems.rs
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy::prelude::in_state;
use bevy::prelude::Name;
use crate::items::{
    StandardProjectileParams, ReturningProjectileParams, ChanneledBeamParams, ConeAttackParams,
    AutomaticWeaponId, AttackTypeData, OrbitingPetParams
};
use crate::components::{
    Velocity, Damage, Lifetime, Health, RootedComponent, HorrorLatchedByTetherComponent
};
use crate::survivor::{BASE_SURVIVOR_SPEED as BASE_PLAYER_SPEED, Survivor, SanityStrain as SurvivorSanityStrain};
use crate::camera_systems::MainCamera;
use crate::horror::Horror;
use crate::game::AppState;
use crate::visual_effects;

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
#[reflect(Component, Default)]
pub struct ChanneledBeamComponent {
    pub damage_per_tick: i32,
    pub tick_timer: Timer,
    pub range: f32,
    pub width: f32,
    pub color: Color,
    pub owner: Entity,
}

impl Default for ChanneledBeamComponent {
    fn default() -> Self {
        Self {
            damage_per_tick: 0,
            tick_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            range: 0.0,
            width: 0.0,
            color: Color::WHITE,
            owner: Entity::PLACEHOLDER,
        }
    }
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
    pub already_hit_this_tick: Vec<Entity>,
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

#[derive(Component, Debug, Reflect, Default, Clone)]
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

#[derive(Component, Debug, Reflect, Default, Clone)]
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
    pub params: crate::items::LineDashAttackParams,
    pub initial_direction: Vec2,
    pub dash_timer: Timer,
    pub already_hit_horrors: Vec<Entity>,
    pub original_speed_if_modified: Option<f32>,
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
    owner: Entity,
    params: &crate::items::BlinkStrikeProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    weapon_id: crate::items::AutomaticWeaponId,
) {
    let base_aim_direction_normalized = aim_direction.normalize_or_zero();
    let num_projectiles = params.num_projectiles_per_shot;
    let total_spread_degrees = if num_projectiles > 1 { (num_projectiles -1) as f32 * 7.5 } else { 0.0 };

    for i in 0..num_projectiles {
        let mut current_projectile_aim_direction = base_aim_direction_normalized;
        if num_projectiles > 1 {
            let total_spread_rad = total_spread_degrees.to_radians();
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
            owner,
            player_transform.translation,
            current_projectile_aim_direction,
            params.base_damage,
            params.projectile_speed,
            params.piercing,
            weapon_id,
            &params.projectile_sprite_path,
            params.projectile_size,
            params.projectile_color,
            params.projectile_lifetime_secs,
            None,
            None,
            None,
            None,
            None,
            Some(params.clone()),
        );
    }
}

pub fn blink_strike_projectile_weapon_fire_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_q: Query<(Entity, &Transform, &Survivor)>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    time: Res<Time>,
    mut sanity_strain_q: Query<&mut SurvivorSanityStrain>,
) {
    if let Ok((player_entity, player_transform, survivor_stats)) = player_q.get_single() {
        if let Ok(mut sanity_strain) = sanity_strain_q.get_single_mut() {
            let weapon_id = survivor_stats.inherent_weapon_id;
            if let Some(weapon_def) = weapon_library.get_weapon_definition(weapon_id) {
                if let AttackTypeData::BlinkStrikeProjectile(ref params) = weapon_def.attack_data {
                    if sanity_strain.fire_timer.tick(time.delta()).just_finished() {
                        spawn_blink_strike_projectile_attack(
                            &mut commands,
                            &asset_server,
                            player_entity,
                            params,
                            player_transform,
                            survivor_stats.aim_direction,
                            weapon_id,
                        );
                    }
                }
            }
        }
    }
}


// --- Repositioning Tether Systems ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TetherProjectileComponent {
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
        Self {
            hit_horror_entity: Entity::PLACEHOLDER,
            horror_original_transform: None,
            params: crate::items::RepositioningTetherParams::default(),
            reactivation_window_timer: Timer::from_seconds(2.0, TimerMode::Once),
            next_effect_mode: crate::items::RepositioningTetherMode::default(),
        }
    }
}

fn apply_tether_reposition_effect(
    horror_transform: &mut Transform,
    player_transform: &Transform,
    params: &crate::items::RepositioningTetherParams,
    mode: crate::items::RepositioningTetherMode,
) {
    let player_pos = player_transform.translation.truncate();
    let horror_pos = horror_transform.translation.truncate();

    let actual_mode = match mode {
        crate::items::RepositioningTetherMode::Alternate => crate::items::RepositioningTetherMode::Pull,
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
}

pub fn spawn_repositioning_tether_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    aim_direction: Vec2,
    weapon_params: &crate::items::RepositioningTetherParams,
    weapon_id: crate::items::AutomaticWeaponId,
    player_waiting_query: &mut Query<&mut PlayerWaitingTetherActivationComponent>, 
    horror_query: &mut Query<&mut Transform, With<Horror>>,                   
    player_transform_query: &Query<&Transform, With<Survivor>>,               
) {
    if let Ok(waiting_comp) = player_waiting_query.get_mut(player_entity) {
        if !waiting_comp.reactivation_window_timer.finished() {
            if let Ok(player_tform) = player_transform_query.get(player_entity) {
                if let Ok(mut horror_tform) = horror_query.get_mut(waiting_comp.hit_horror_entity) {
                    apply_tether_reposition_effect(
                        &mut horror_tform,
                        player_tform,
                        &waiting_comp.params,
                        waiting_comp.next_effect_mode,
                    );
                }
            }
            if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                 commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
            return;
        } else {
             if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
        }
    }

    if let Ok(player_transform) = player_transform_query.get(player_entity) {
        let _projectile_entity = crate::automatic_projectiles::spawn_automatic_projectile(
            commands,
            asset_server,
            player_entity,
            player_transform.translation,
            aim_direction,
            0,
            weapon_params.tether_projectile_speed,
            0,
            weapon_id,
            &weapon_params.tether_sprite_path,
            weapon_params.tether_size,
            weapon_params.tether_color,
            weapon_params.tether_range / weapon_params.tether_projectile_speed,
            None,
            None,
            None,
            None,
            Some(weapon_params.clone()),
            None,
        );
    }
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
    pub speed: f32,
    pub initial_spawn_position: Vec3,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct MagmaPoolComponent {
    pub damage_per_tick: i32,
    pub radius: f32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub color: Color,
    pub already_hit_this_tick: Vec<Entity>,
}

// --- Orbiting Pet Definitions (New Implementation) ---

pub fn charge_weapon_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn trail_spawning_projectile_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn fire_trail_segment_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn chain_lightning_visual_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn nova_visual_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn manage_persistent_aura_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn debuff_cloud_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn expanding_energy_bomb_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn homing_projectile_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn lobbed_projectile_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn ichor_pool_system(mut _commands: Commands) {
    // TODO: Implement system
}

pub fn channeled_beam_update_system(mut _commands: Commands) {
    // TODO: Implement system
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct OrbitingPetComponent {
    pub params_snapshot: crate::items::OrbitingPetParams,
    pub orbit_angle_rad: f32,
    pub duration_timer: Timer,
    pub pulse_timer: Option<Timer>,
    pub bolt_timer: Option<Timer>,
    pub owner_player_entity: Entity,
}

impl Default for OrbitingPetComponent {
    fn default() -> Self {
        Self {
            params_snapshot: OrbitingPetParams::default(),
            orbit_angle_rad: 0.0,
            duration_timer: Timer::from_seconds(1.0, TimerMode::Once),
            pulse_timer: None,
            bolt_timer: None,
            owner_player_entity: Entity::PLACEHOLDER,
        }
    }
}


#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerOrbControllerComponent {
    pub active_orb_entities: Vec<Entity>,
    pub max_orbs_allowed: u32,
    pub spawn_cooldown_timer: Timer,
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
            .register_type::<LobbedBouncingProjectileComponent>()
            .register_type::<MagmaPoolComponent>()
            .register_type::<OrbitingPetComponent>()
            .register_type::<PlayerOrbControllerComponent>()
            .register_type::<TetherProjectileComponent>()
            .register_type::<PlayerWaitingTetherActivationComponent>()
            .register_type::<HorrorLatchedByTetherComponent>()
            .add_systems(Update, (
                manage_player_orbs_system,
                orbiting_pet_behavior_system,
                tether_reactivation_window_system,
                returning_projectile_system,
                player_is_channeling_effect_system,
                channeled_beam_damage_system,
                ground_targeting_reticule_system,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, pending_ground_aoe_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, eruption_visual_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, player_dashing_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                lobbed_bouncing_projectile_system,
                magma_pool_system,
                repositioning_tether_firing_system,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                charge_weapon_system,
                trail_spawning_projectile_system,
                fire_trail_segment_system,
                chain_lightning_visual_system,
                nova_visual_system,
                manage_persistent_aura_system,
                debuff_cloud_system,
                expanding_energy_bomb_system,
                homing_projectile_system,
                lobbed_projectile_system,
                ichor_pool_system,
                channeled_beam_update_system,
            ).run_if(in_state(AppState::InGame)));
    }
}

pub fn repositioning_tether_firing_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    mut set: ParamSet<(
        Query<(Entity, &Transform, &Survivor, &SurvivorSanityStrain)>, // p0: Player data query
        Query<&mut Transform, With<Horror>>,                         // p1: Horror mutable transform query
        Query<&mut PlayerWaitingTetherActivationComponent>,          // p2: Player waiting component query
    )>,
) {
    // It's tricky to iterate player_query (p0) and pass horror_query (p1) and player_waiting_query (p2) mutably
    // to spawn_repositioning_tether_attack due to ParamSet limitations.
    // We need to avoid holding a borrow on set.p0() while trying to get mutable borrows for set.p1() or set.p2().
    // So, we collect the necessary player data first.

    let mut player_fire_requests: Vec<(Entity, Vec2, crate::items::RepositioningTetherParams, crate::items::AutomaticWeaponId, Transform)> = Vec::new();

    for (player_entity, player_transform, player_stats, sanity_strain) in set.p0().iter() {
        let weapon_id = player_stats.inherent_weapon_id;
        if let Some(weapon_def) = weapon_library.get_weapon_definition(weapon_id) {
            if let crate::items::AttackTypeData::RepositioningTether(ref params) = weapon_def.attack_data {
                if sanity_strain.fire_timer.just_finished() {
                    player_fire_requests.push((
                        player_entity,
                        player_stats.aim_direction,
                        params.clone(),
                        weapon_id,
                        *player_transform, // Store a copy of the transform
                    ));
                }
            }
        }
    }

    for (player_entity, aim_direction, params, weapon_id, player_transform_copy) in player_fire_requests {
        // Now we can pass parts of the ParamSet.
        // Note: spawn_repositioning_tether_attack will need to be refactored to accept these
        // or we query within it using the entity IDs.
        // For now, let's assume spawn_repositioning_tether_attack will be adapted.
        // We pass player_transform_copy directly.

        // The direct passing of p1 and p2 queries like this is problematic if spawn_repositioning_tether_attack
        // also tries to use ParamSet or expects full Query objects.
        // We will need to refactor spawn_repositioning_tether_attack significantly.
        // For now, this is a placeholder for how the call *might* look after spawn_repositioning_tether_attack is refactored.
        // The critical part is that we are not holding a borrow on set.p0() when we might need mutable access to set.p1() or set.p2().

        // Let's modify spawn_repositioning_tether_attack to take what it needs directly,
        // rather than entire Query objects.
        spawn_repositioning_tether_attack(
            &mut commands,
            &asset_server,
            player_entity,
            aim_direction,
            &params, // Cloned earlier, so this is fine
            weapon_id,
            &mut set.p2(), // PlayerWaitingTetherActivationComponent query
            &mut set.p1(), // Horror transform query
            &player_transform_copy, // Pass the copied transform
        );
    }
}

pub fn spawn_repositioning_tether_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    aim_direction: Vec2,
    weapon_params: &crate::items::RepositioningTetherParams,
    weapon_id: crate::items::AutomaticWeaponId,
    player_waiting_query: &mut Query<&mut PlayerWaitingTetherActivationComponent>,
    horror_query: &mut Query<&mut Transform, With<Horror>>,
    player_transform: &Transform, // Changed from Query to direct &Transform
) {
    if let Ok(mut waiting_comp) = player_waiting_query.get_mut(player_entity) { // Add mut here
        if !waiting_comp.reactivation_window_timer.finished() {
            // No need to query player_transform again, it's passed directly
            if let Ok(mut horror_tform) = horror_query.get_mut(waiting_comp.hit_horror_entity) {
                apply_tether_reposition_effect(
                    &mut horror_tform,
                    player_transform, // Use the passed player_transform
                    &waiting_comp.params,
                    waiting_comp.next_effect_mode,
                );
            }
            if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                 commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
            return;
        } else {
             if commands.get_entity(waiting_comp.hit_horror_entity).is_some() {
                commands.entity(waiting_comp.hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
            }
            commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
        }
    }

    // No need to query player_transform again, it's passed directly
    let _projectile_entity = crate::automatic_projectiles::spawn_automatic_projectile(
        commands,
        asset_server,
        player_entity,
        player_transform.translation, // Use the passed player_transform
        aim_direction,
        0, // Damage for tether projectile is 0
        weapon_params.tether_projectile_speed,
        0, // Piercing for tether projectile is 0
        weapon_id,
        &weapon_params.tether_sprite_path,
        weapon_params.tether_size,
        weapon_params.tether_color,
        // Lifetime: range / speed
        weapon_params.tether_range / weapon_params.tether_projectile_speed,
        None, // No lifesteal
        None, // No bounce params
        None, // No explosion params
        None, // No trail params
        Some(weapon_params.clone()), // Tether params
        None, // No blink strike params
    );
}

pub fn spawn_orbiting_pet_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    player_transform: &Transform,
    params: &crate::items::OrbitingPetParams,
    orb_controller: &mut PlayerOrbControllerComponent,
) {
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
            texture: asset_server.load(params.orb_sprite_path.to_string()), // Use .to_string() for String
            sprite: Sprite {
                custom_size: Some(params.orb_size),
                color: params.orb_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos),
            ..default()
        },
        OrbitingPetComponent {
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
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    mut player_query: Query<(Entity, &Transform, &Survivor, Option<&mut PlayerOrbControllerComponent>)>,
    orb_query: Query<Entity, With<OrbitingPetComponent>>,
) {
    let Ok((player_entity, player_transform, player_stats, opt_orb_controller)) = player_query.get_single_mut() else { return; };

    let mut shadow_orb_params_opt: Option<crate::items::OrbitingPetParams> = None;
    let active_weapon_id = player_stats.inherent_weapon_id;
    if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
        if let AttackTypeData::OrbitingPet(params) = &weapon_def.attack_data {
            shadow_orb_params_opt = Some(params.clone());
        }
    }

    if let Some(params) = shadow_orb_params_opt {
        if let Some(mut controller) = opt_orb_controller { // Re-add mut here
            controller.spawn_cooldown_timer.tick(time.delta());
            if controller.spawn_cooldown_timer.finished() && controller.active_orb_entities.len() < controller.max_orbs_allowed as usize {
                spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut controller);
                controller.spawn_cooldown_timer.reset();
            }
            controller.active_orb_entities.retain(|&orb_e| orb_query.get(orb_e).is_ok());
        } else {
            let mut new_controller = PlayerOrbControllerComponent {
                active_orb_entities: Vec::new(),
                max_orbs_allowed: params.max_active_orbs,
                spawn_cooldown_timer: Timer::from_seconds(params.base_fire_rate_secs, TimerMode::Repeating),
            };
            if new_controller.active_orb_entities.len() < new_controller.max_orbs_allowed as usize {
                 spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut new_controller);
                 new_controller.spawn_cooldown_timer.reset();
            }
            commands.entity(player_entity).insert(new_controller);
        }
    } else {
        if let Some(controller) = opt_orb_controller { // Remove mut here
            for orb_entity in controller.active_orb_entities.iter() {
                if orb_query.get(*orb_entity).is_ok() {
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
    player_query: Query<&Transform, (With<Survivor>, Without<OrbitingPetComponent>)>,
    horror_query: Query<(Entity, &GlobalTransform), With<Horror>>,
    mut horror_health_query: Query<&mut Health, With<Horror>>,
) {
    for (orb_entity, mut orb_transform, mut orb_comp) in pet_query.iter_mut() {
        orb_comp.duration_timer.tick(time.delta());
        if orb_comp.duration_timer.finished() {
            commands.entity(orb_entity).despawn_recursive();
            continue;
        }

        if let Ok(owner_transform) = player_query.get(orb_comp.owner_player_entity) {
            orb_comp.orbit_angle_rad += orb_comp.params_snapshot.orbit_speed_rad_per_sec * time.delta_seconds();
            orb_comp.orbit_angle_rad %= std::f32::consts::TAU;

            let offset = Vec2::from_angle(orb_comp.orbit_angle_rad) * orb_comp.params_snapshot.orbit_radius;
            orb_transform.translation = owner_transform.translation + offset.extend(0.1);
        } else {
            commands.entity(orb_entity).despawn_recursive();
            continue;
        }

        if orb_comp.params_snapshot.pulses_aoe {
            if let Some(ref mut pulse_timer) = orb_comp.pulse_timer {
                pulse_timer.tick(time.delta());
                if pulse_timer.just_finished() {
                    let orb_position = orb_transform.translation;
                    if let Some(pulse_viz_color) = orb_comp.params_snapshot.pulse_color {
                         commands.spawn((
                            SpriteBundle {
                                texture: asset_server.load("sprites/pulse_effect_placeholder.png"),
                                sprite: Sprite {
                                    color: pulse_viz_color,
                                    custom_size: Some(Vec2::splat(orb_comp.params_snapshot.pulse_radius * 0.25)),
                                    ..default()
                                },
                                transform: Transform::from_translation(orb_position),
                                ..default()
                            },
                            NovaVisualComponent {
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
                                visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), orb_comp.params_snapshot.pulse_damage, &time);
                            }
                        }
                    }
                }
            }
        }

        if orb_comp.params_snapshot.fires_seeking_bolts {
            if let Some(ref mut bolt_timer) = orb_comp.bolt_timer {
                bolt_timer.tick(time.delta());
                if bolt_timer.just_finished() {
                    let mut closest_target: Option<(Entity, f32)> = None;
                    let orb_pos_2d = orb_transform.translation.truncate();

                    for (horror_entity, horror_gtransform) in horror_query.iter() {
                        let dist_sq = orb_pos_2d.distance_squared(horror_gtransform.translation().truncate());
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
                                let bolt_sprite_path_str = orb_comp.params_snapshot.bolt_sprite_path.as_deref().unwrap_or("sprites/default_bolt.png");
                                let bolt_sz = orb_comp.params_snapshot.bolt_size.unwrap_or_else(|| Vec2::new(10.0,10.0));
                                let bolt_col = orb_comp.params_snapshot.bolt_color.unwrap_or(Color::WHITE);
                                let bolt_lt = orb_comp.params_snapshot.bolt_lifetime_secs.unwrap_or(1.0);
                                crate::automatic_projectiles::spawn_automatic_projectile(
                                    &mut commands,
                                    &asset_server,
                                    orb_comp.owner_player_entity,
                                    orb_transform.translation,
                                    direction,
                                    orb_comp.params_snapshot.bolt_damage,
                                    orb_comp.params_snapshot.bolt_speed,
                                    0,
                                    AutomaticWeaponId(u32::MAX),
                                    bolt_sprite_path_str, // Pass as &str
                                    bolt_sz,
                                    bolt_col,
                                    bolt_lt,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
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
    let projectile_velocity = aim_direction.normalize_or_zero() * params.projectile_speed;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(params.projectile_sprite_path.to_string()), // Use .to_string()
            sprite: Sprite {
                custom_size: Some(params.projectile_size),
                color: params.projectile_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        LobbedBouncingProjectileComponent {
            params: params.clone(),
            bounces_left: params.num_bounces,
            speed: params.projectile_speed,
            initial_spawn_position: initial_pos,
        },
        Velocity(projectile_velocity),
        Damage(params.damage_per_bounce_impact),
        Lifetime { timer: Timer::from_seconds(params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile {
            owner: Entity::PLACEHOLDER, // FIXME: Pass actual owner
            piercing_left: 0,
            weapon_id,
            bounces_left: Some(params.num_bounces),
            damage_on_hit: params.damage_per_bounce_impact,
            current_speed: params.projectile_speed,
            damage_loss_per_bounce_multiplier: Some(1.0),
            speed_loss_per_bounce_multiplier: Some(1.0),
            has_bounced_this_frame: false,
            lifesteal_percentage: None,
            blink_params_on_hit: None,
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
        &mut Velocity, // Keep as it might be used by other systems or future logic
        &mut Damage,   // Keep for same reason
        &Transform,
        &mut Lifetime,
        & crate::automatic_projectiles::AutomaticProjectile,
    )>,
) {
    for (
        entity,
        mut bouncing_comp,
        _velocity, // Explicitly unused for now
        _damage,   // Explicitly unused for now
        transform,
        mut lifetime,
        auto_proj_comp,
    ) in projectile_query.iter_mut()
    {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() && bouncing_comp.bounces_left == 0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        if auto_proj_comp.bounces_left.is_some() && auto_proj_comp.bounces_left.unwrap() < bouncing_comp.bounces_left {
            bouncing_comp.bounces_left = auto_proj_comp.bounces_left.unwrap();

            if rand::random::<f32>() < bouncing_comp.params.fire_pool_on_bounce_chance {
                spawn_magma_pool(
                    &mut commands,
                    &asset_server,
                    transform.translation,
                    &bouncing_comp.params,
                );
            }

            if bouncing_comp.bounces_left == 0 {
                commands.entity(entity).despawn_recursive();
                continue;
            }
        }
    }
}

pub fn spawn_magma_pool(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    magma_params: &crate::items::LobbedBouncingMagmaParams,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/magma_pool_placeholder.png"),
            sprite: Sprite {
                color: magma_params.fire_pool_color,
                custom_size: Some(Vec2::splat(magma_params.fire_pool_radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position.truncate().extend(0.01)),
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
    asset_server: Res<AssetServer>,
) {
    for (pool_entity, mut pool_comp, pool_gtransform) in pool_query.iter_mut() {
        pool_comp.duration_timer.tick(time.delta());
        if pool_comp.duration_timer.finished() {
            commands.entity(pool_entity).despawn_recursive();
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
                    visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_transform.translation, pool_comp.damage_per_tick, &time);
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
) {
    let mut dash_direction = player_stats.aim_direction.normalize_or_zero();
    if dash_direction == Vec2::ZERO {
        dash_direction = (player_transform.rotation * Vec3::X).truncate().normalize_or_zero();
        if dash_direction == Vec2::ZERO {
            dash_direction = Vec2::X;
        }
    }

    let original_speed_val = player_stats.speed;
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
}

pub fn player_dashing_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut Transform, &mut Survivor, &mut PlayerDashingComponent), (With<Survivor>, Without<Horror>)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>,
) {
    if let Ok((player_entity, mut player_transform, mut player_stats, mut dashing_comp)) = player_query.get_single_mut() {
        dashing_comp.dash_timer.tick(time.delta());

        let movement_this_frame = dashing_comp.initial_direction * dashing_comp.params.dash_speed * time.delta_seconds();
        player_transform.translation += movement_this_frame.extend(0.0);

        let player_hitbox_center = player_transform.translation.truncate();

        for (horror_entity, horror_gtransform, mut horror_health, horror_data) in horror_query.iter_mut() {
            if dashing_comp.already_hit_horrors.len() >= dashing_comp.params.piercing_cap as usize {
                break;
            }
            if dashing_comp.already_hit_horrors.contains(&horror_entity) {
                continue;
            }

            let horror_pos = horror_gtransform.translation().truncate();
            let horror_half_width = horror_data.size.x / 2.0;
            let horror_half_height = horror_data.size.y / 2.0;

            let x_collision = (player_hitbox_center.x - horror_pos.x).abs() < (dashing_comp.params.hitbox_width / 2.0 + horror_half_width);
            let y_collision = (player_hitbox_center.y - horror_pos.y).abs() < (dashing_comp.params.hitbox_width / 2.0 + horror_half_height);


            if x_collision && y_collision {
                horror_health.0 -= dashing_comp.params.damage_per_hit;
                visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), dashing_comp.params.damage_per_hit, &time);
                dashing_comp.already_hit_horrors.push(horror_entity);
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

    let active_weapon_id = player_stats.inherent_weapon_id;
    if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
        if let AttackTypeData::GroundTargetedAoE(params) = &weapon_def.attack_data {
            should_have_reticule = true;
            current_reticule_params_opt = Some(params.clone());
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
            let path_to_load = params.reticle_sprite_path.as_deref().unwrap_or("sprites/ground_target_reticle_placeholder.png").to_string();
            let reticule_entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(path_to_load), // Use owned String
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
    asset_server: Res<AssetServer>,
    mut pending_aoe_query: Query<(Entity, &mut PendingGroundAoEComponent)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &mut Velocity), With<Horror>>,
) {
    for (pending_entity, mut pending_comp) in pending_aoe_query.iter_mut() {
        pending_comp.eruption_timer.tick(time.delta());

        if pending_comp.eruption_timer.finished() {
            let path_to_load = pending_comp.params.visual_sprite_path.as_deref().unwrap_or("sprites/eruption_effect_placeholder.png").to_string();
            let _eruption_visual_entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(path_to_load), // Use owned String
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
                    visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), damage_to_apply, &time);

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
            let progress = visual.duration_timer.percent();
            let current_visual_radius = visual.initial_radius + (visual.max_radius - visual.initial_radius) * progress;
            sprite.custom_size = Some(Vec2::splat(current_visual_radius * 2.0));
            sprite.color.set_a(visual.color.a() * (1.0 - progress));
        }
    }
}


pub fn channeled_beam_damage_system(
    mut commands: Commands, 
    time: Res<Time>,
    mut beam_query: Query<(&mut ChanneledBeamComponent, &GlobalTransform)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), With<Horror>>,
    asset_server: Res<AssetServer>, 
    mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>, 
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
                    visual_effects::spawn_damage_text( &mut commands, &asset_server, enemy_transform.translation, beam_comp.damage_per_tick, &time);
                    sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::HorrorHit));
                }
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
            texture: asset_server.load(params.projectile_sprite_path.to_string()), // Use .to_string()
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
    mut enemy_query: Query<(&Transform, &mut Health), With<Horror>>,
    time: &Res<Time>,
) {
    let player_pos = player_transform.translation.truncate();
    let forward_vector = aim_direction.normalize_or_zero();

    if let Some(sprite_path_str) = &params.visual_sprite_path {
        let path_to_load = sprite_path_str.to_string();
        let mut sprite_size = Vec2::new(params.cone_radius, params.cone_radius * 0.5);
        if let Some((radius_scale, angle_scale)) = params.visual_size_scale_with_radius_angle {
            sprite_size = Vec2::new(
                params.cone_radius * radius_scale,
                params.cone_radius * params.cone_angle_degrees.to_radians() * angle_scale
            );
        }

        let mut visual_transform = Transform::from_translation(player_transform.translation);
        visual_transform.rotation = Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x));

        let anchor = bevy::sprite::Anchor::CenterLeft;
        if let Some(offset) = params.visual_anchor_offset {
            let rotated_offset = visual_transform.rotation * offset.extend(0.0);
            visual_transform.translation = (visual_transform.translation.truncate() + rotated_offset.truncate()).extend(visual_transform.translation.z);
        }


        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(path_to_load),
                sprite: Sprite {
                    custom_size: Some(sprite_size),
                    color: params.color,
                    anchor,
                    ..default()
                },
                transform: visual_transform,
                ..default()
            },
            Lifetime { timer: Timer::from_seconds(0.25, TimerMode::Once) },
            Name::new("ConeAttackVisual"),
        ));
    }

    for (enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
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
                visual_effects::spawn_damage_text(commands, asset_server, enemy_transform.translation, params.base_damage, &time);
            }
        }
    }
}