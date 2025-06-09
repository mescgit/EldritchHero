// mescgit/eldritchhero/EldritchHero-c197490fa863f6ebd3e83365f89cc741bfb8b804/src/weapon_systems.rs
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy::prelude::in_state;
use bevy::prelude::Name;
use std::time::Instant; // Added for log state

use crate::items::{
    StandardProjectileParams, ReturningProjectileParams, ChanneledBeamParams, ConeAttackParams,
    AutomaticWeaponId, AttackTypeData, AutomaticWeaponLibrary
};
use crate::components::{
    Velocity, Damage, Lifetime, Health, RootedComponent, HorrorLatchedByTetherComponent, PlayerRequestsOrbDeployment
};
use crate::survivor::{BASE_SURVIVOR_SPEED as BASE_PLAYER_SPEED, Survivor, SanityStrain as SurvivorSanityStrain, SURVIVOR_SIZE};
use crate::camera_systems::MainCamera;
use crate::horror::Horror;
use crate::game::AppState;
use crate::visual_effects;
use crate::audio::{PlaySoundEvent, SoundEffect}; // Re-added for orb pulse sound

// --- Chain Lightning Log State Resource ---
#[derive(Resource, Default)]
pub struct ChainLightningLogState { // Made public to resolve private_interfaces warning
    last_log_time: Option<Instant>,
    last_targets_hit_count: Option<usize>,
}

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
    pub hit_enemies_this_pass: Vec<Entity>, // Added
    pub projectile_size: Vec2,             // Added
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

pub fn lobbed_weapon_targeting_reticule_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    player_query: Query<(Entity, &GlobalTransform, &Survivor)>,
    mut reticule_query: Query<(Entity, &mut Transform, &mut LobbedWeaponTargetReticuleComponent, &Parent), With<LobbedWeaponTargetReticuleComponent>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok((player_entity, player_gtransform, player_stats)) = player_query.get_single() else {
        // If no player, despawn all lobbed reticles (they shouldn't exist anyway if parented)
        for (ret_entity, _, _, _) in reticule_query.iter_mut() {
            commands.entity(ret_entity).despawn_recursive();
        }
        return;
    };
    let player_pos_2d = player_gtransform.translation().truncate();

    let mut should_have_reticule = false;
    let mut current_lobbed_params_opt: Option<crate::items::LobbedAoEPoolParams> = None;

    let active_weapon_id = player_stats.inherent_weapon_id;
    if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
        if let AttackTypeData::LobbedAoEPool(params) = &weapon_def.attack_data {
            should_have_reticule = true;
            current_lobbed_params_opt = Some(params.clone());
        }
        // Future: Extend for other lobbed types by adding more `else if let` branches
        // else if let AttackTypeData::LobbedBouncingMagma(params) = &weapon_def.attack_data {
        //     should_have_reticule = true;
        //     // current_lobbed_params_opt = Some(params.clone_into_generic_lobbed_params()); // If needed
        //     // For now, we'll assume LobbedAoEPoolParams is the only one this reticle handles
        // }
    }

    if should_have_reticule {
        let params = current_lobbed_params_opt.expect("Lobbed params should be present if should_have_reticule is true");
        let window = windows.single();
        let (camera, camera_gtransform) = camera_q.single();

        let max_targeting_range = params.projectile_speed * 1.5; // Placeholder calculation
        let reticle_visual_size = Vec2::new(50.0, 50.0);
        let reticle_z_offset = 0.2; // Relative to player, ensure it's rendered above player/ground

        let mut cursor_world_pos_2d = player_pos_2d + player_stats.aim_direction.normalize_or_zero() * max_targeting_range * 0.5; // Default to half range
        if let Some(cursor_pos_screen) = window.cursor_position() {
            if let Some(cursor_world) = camera.viewport_to_world(camera_gtransform, cursor_pos_screen) {
                cursor_world_pos_2d = cursor_world.origin.truncate();
            }
        }

        let player_to_cursor_vector = cursor_world_pos_2d - player_pos_2d;
        let distance_to_cursor = player_to_cursor_vector.length();
        
        let clamped_distance = distance_to_cursor.min(max_targeting_range);

        let mut direction_to_cursor = player_to_cursor_vector.normalize_or_zero();
        if direction_to_cursor == Vec2::ZERO {
            // Fallback to player's aim direction if cursor is on player
            direction_to_cursor = player_stats.aim_direction.normalize_or_zero();
            if direction_to_cursor == Vec2::ZERO {
                // If player's aim is also zero, fallback to player's local right (X-axis)
                direction_to_cursor = (player_gtransform.compute_transform().right().truncate()).normalize_or_zero();
                if direction_to_cursor == Vec2::ZERO {
                    direction_to_cursor = Vec2::X; // Absolute fallback if all else fails
                }
            }
        }
        // Now, direction_to_cursor is a unit vector (or a default like Vec2::X)
        let local_reticle_pos = direction_to_cursor * clamped_distance;

        let mut reticule_updated_or_created = false;
        for (_ret_entity, mut ret_transform, mut ret_component, parent) in reticule_query.iter_mut() { // Changed ret_entity to _ret_entity
            if parent.get() == player_entity {
                ret_transform.translation = local_reticle_pos.extend(reticle_z_offset);
                ret_component.max_range = max_targeting_range;
                // visual_size is fixed for now, but could be updated from params if needed
                reticule_updated_or_created = true;
                break;
            }
        }

        if !reticule_updated_or_created {
            commands.entity(player_entity).with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/lobbed_reticle_placeholder.png"),
                        sprite: Sprite {
                            custom_size: Some(reticle_visual_size),
                            color: Color::rgba(0.8, 0.8, 0.2, 0.5),
                            ..default()
                        },
                        // Transform is local to the parent (player)
                        transform: Transform::from_translation(local_reticle_pos.extend(reticle_z_offset)),
                        ..default()
                    },
                    LobbedWeaponTargetReticuleComponent {
                        max_range: max_targeting_range,
                        visual_size: reticle_visual_size,
                    },
                    Name::new("LobbedTargetReticule"),
                ));
            });
        }
    } else {
        // If should_not_have_reticule, despawn any existing reticle parented to this player
        for (ret_entity, _, _, parent) in reticule_query.iter_mut() {
            if parent.get() == player_entity {
                commands.entity(ret_entity).despawn_recursive();
            }
        }
    }
}


#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct IsChannelingComponent {
    pub beam_entity: Option<Entity>,
    pub beam_params: ChanneledBeamParams,
    pub active_duration_timer: Option<Timer>,
    pub cooldown_timer: Option<Timer>,
}

// --- Lobbed AoE Pool Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct LobbedProjectileComponent {
    pub arc_height: f32,
    pub speed: f32,
    pub pool_params: crate::items::LobbedAoEPoolParams,
    pub initial_spawn_position: Vec3,
    pub target_position: Option<Vec3>, // Added field
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
    pub original_color: Color, // Added field
}

// --- Chain Lightning Definitions ---

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ChainLightningZapEffectComponent {
    pub duration_timer: Timer,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub color: Color,
    pub width: f32,
}

impl Default for ChainLightningZapEffectComponent {
    fn default() -> Self {
        Self {
            duration_timer: Timer::from_seconds(0.2, TimerMode::Once), // Default duration
            start_pos: Vec3::ZERO,
            end_pos: Vec3::ZERO,
            color: Color::WHITE,
            width: 5.0,
        }
    }
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
pub struct LobbedWeaponTargetReticuleComponent {
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
            None // opt_trail_params
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
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_entity, player_transform, survivor_stats)) = player_q.get_single() {
        if let Ok(mut sanity_strain) = sanity_strain_q.get_single_mut() {
            let weapon_id = survivor_stats.inherent_weapon_id;
            if let Some(weapon_def) = weapon_library.get_weapon_definition(weapon_id) {
                if let AttackTypeData::BlinkStrikeProjectile(ref params) = weapon_def.attack_data {
                    if sanity_strain.fire_timer.tick(time.delta()).just_finished() {
                        if let Some(sound_path) = &params.fire_sound_effect {
                            sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
                        }
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

pub fn charge_weapon_system(
    mut player_charging_query: Query<(Entity, &mut ChargingWeaponComponent, &Survivor)>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    mouse_button_input: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>, // Optional
) {
    for (_player_entity, mut charging_comp, _survivor_stats) in player_charging_query.iter_mut() {
        if !charging_comp.is_actively_charging {
            continue;
        }

        if mouse_button_input.pressed(MouseButton::Left) {
            // Player is holding the charge button
            charging_comp.charge_timer.tick(time.delta());

            if charging_comp.charge_timer.finished() {
                // Current charge level timer has completed
                if let Some(weapon_def) = weapon_library.get_weapon_definition(charging_comp.weapon_id) {
                    if let AttackTypeData::ChargeUpEnergyShot(ref shot_params) = weapon_def.attack_data {
                        let num_charge_levels = shot_params.charge_levels.len();
                        if num_charge_levels == 0 {
                            charging_comp.is_actively_charging = false; // Should not happen with valid params
                            continue;
                        }

                        if charging_comp.current_charge_level_index < num_charge_levels - 1 {
                            // Advance to the next charge level
                            charging_comp.current_charge_level_index += 1;
                            let next_level_params = &shot_params.charge_levels[charging_comp.current_charge_level_index];
                            charging_comp.charge_timer = Timer::from_seconds(next_level_params.charge_time_secs.max(0.01), TimerMode::Once); // Ensure non-zero
                            if let Some(sound_path) = &shot_params.charge_sound_effect {
                                sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
                            }
                        } else {
                            // Already at max charge level. Timer can stay finished/paused.
                            // charging_comp.charge_timer.pause(); // Or just let it be finished.
                        }
                    } else {
                        charging_comp.is_actively_charging = false; // Should not happen if weapon_id is correct
                    }
                } else {
                    charging_comp.is_actively_charging = false; // Weapon def not found
                }
            }
        } else {
            // Mouse button is NOT pressed.
            // Set is_actively_charging to false so survivor_casting_system can detect this state change
            // and then fire the shot and manage component removal.
            charging_comp.is_actively_charging = false; 
        }
    }
}

pub fn trail_spawning_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&GlobalTransform, &mut TrailSpawningProjectileComponent)>,
) {
    for (projectile_transform, mut trail_spawner) in query.iter_mut() {
        trail_spawner.segment_spawn_timer.tick(time.delta());

        if trail_spawner.segment_spawn_timer.just_finished() {
            let trail_params = &trail_spawner.trail_params; // Get a reference

            commands.spawn((
                SpriteBundle {
                    // Using hardcoded placeholder as trail_segment_sprite_path_placeholder is not in TrailOfFireParams yet
                    texture: asset_server.load("sprites/fire_trail_segment_placeholder.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(trail_params.trail_segment_width, trail_params.trail_segment_width)),
                        color: trail_params.trail_segment_color,
                        ..default()
                    },
                    // Use .translation() to get Vec3 from GlobalTransform
                    transform: Transform::from_translation(projectile_transform.translation()),
                    ..default()
                },
                FireTrailSegmentComponent {
                    damage_per_tick: trail_params.trail_segment_damage_per_tick,
                    tick_timer: Timer::from_seconds(trail_params.trail_segment_tick_interval_secs.max(0.01), TimerMode::Repeating),
                    duration_timer: Timer::from_seconds(trail_params.trail_segment_duration_secs.max(0.01), TimerMode::Once),
                    width: trail_params.trail_segment_width,
                    already_hit_this_tick: Vec::new(),
                    original_color: trail_params.trail_segment_color, // Initialize new field
                },
                Name::new("FireTrailSegment"),
            ));
        }
    }
}

pub fn fire_trail_segment_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut segment_query: Query<(Entity, &mut FireTrailSegmentComponent, &GlobalTransform, &mut Sprite)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>,
) {
    for (segment_entity, mut segment_comp, segment_gtransform, mut segment_sprite) in segment_query.iter_mut() {
        // Segment Lifetime & Fade Out
        segment_comp.duration_timer.tick(time.delta());
        if segment_comp.duration_timer.finished() {
            commands.entity(segment_entity).despawn_recursive();
            continue;
        }

        let remaining_percent = 1.0 - segment_comp.duration_timer.percent();
        segment_sprite.color.set_a(segment_comp.original_color.a() * remaining_percent);

        // Damage Application
        segment_comp.tick_timer.tick(time.delta());
        if segment_comp.tick_timer.just_finished() {
            segment_comp.already_hit_this_tick.clear();
            let segment_pos = segment_gtransform.translation().truncate(); // Use GlobalTransform for world position

            for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() {
                let horror_pos = horror_gtransform.translation().truncate(); // Use GlobalTransform for world position

                // Collision check (circular segment vs circular horror)
                // Assuming horror_radius is fixed for now, ideally get from Horror component stats if available
                let horror_radius = 16.0; // Placeholder radius for horrors
                let combined_radius_sq = (segment_comp.width / 2.0 + horror_radius).powi(2);
                let distance_sq = segment_pos.distance_squared(horror_pos);

                if distance_sq < combined_radius_sq {
                    if !segment_comp.already_hit_this_tick.contains(&horror_entity) {
                        horror_health.0 = horror_health.0.saturating_sub(segment_comp.damage_per_tick);

                        // Spawn damage text visual effect using the horror's GlobalTransform for position
                        visual_effects::spawn_damage_text(
                            &mut commands,
                            &asset_server,
                            horror_gtransform.translation(), // Position for damage text
                            segment_comp.damage_per_tick,
                            &time, // Pass time as a reference
                        );
                        segment_comp.already_hit_this_tick.push(horror_entity);
                    }
                }
            }
        }
    }
}

pub fn chain_lightning_visual_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut zap_query: Query<(Entity, &mut ChainLightningZapEffectComponent, Option<&Children>)>,
    // We might need QueryState if we iterate and then try to get a mutable child sprite via commands.get_entity().
    // For now, let's try direct query for children if possible, or simply manage sprites via their own entity.
    // A simpler approach for now: the visual system will spawn/despawn its own sprite entities
    // that are linked conceptually but not via Bevy parent/child hierarchy *if* direct child management becomes too complex
    // within a single system pass.
    // Re-evaluating: The parent/child for auto-despawn is good. The challenge is modifying the child sprite.
    // Let's assume we spawn a child sprite and then query for it if we need to update it.
    // For this version, we spawn it once and it gets despawned with the parent.
    // We will need a marker component on the child sprite if we want to query it specifically.
    // Or, if there's only one child, children.first() might work.
) {
    for (entity, mut zap, children_option) in zap_query.iter_mut() {
        zap.duration_timer.tick(time.delta());
        if zap.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Check if a visual sprite child already exists
        let mut has_sprite_child = false;
        if let Some(children) = children_option {
            for _child_entity in children.iter() {
                // This is a simplification. In a real scenario, you'd have a marker component
                // on the sprite child to ensure you're not interfering with other unrelated children.
                // For now, we assume any child is the visual sprite.
                has_sprite_child = true; 
                // TODO: If we needed to update the sprite every frame (e.g., for fading alpha independently),
                // we would query its components here using `commands.get_entity(*child_entity)` or a direct query.
                // For example: `if let Some(mut sprite) = sprite_query.get_mut(*child_entity) { sprite.color.set_a(...) }`
                break;
            }
        }

        if !has_sprite_child {
            let segment_vector = zap.end_pos - zap.start_pos;
            let length = segment_vector.length();

            if length < 1.0 { // Avoid issues with zero/tiny length
                continue;
            }

            let midpoint = zap.start_pos + segment_vector / 2.0;
            let angle = segment_vector.y.atan2(segment_vector.x);

            let sprite_entity = commands.spawn(SpriteBundle {
                texture: asset_server.load("sprites/chain_lightning_bolt_placeholder.png"), // Assuming 1x1 white pixel
                sprite: Sprite {
                    color: zap.color,
                    custom_size: Some(Vec2::new(1.0, 1.0)), // Base size of the pixel
                    anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(midpoint.x, midpoint.y, 0.9), // Z-ordering
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::new(length, zap.width, 1.0),
                },
                ..default()
            }).id();
            commands.entity(entity).add_child(sprite_entity);
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
        let progress = visual.duration_timer.percent();

        if visual.duration_timer.finished() {
            // Despawn immediately when max size/duration is reached for the "wave" effect
            commands.entity(entity).despawn_recursive();
        } else {
            // Scale animation:
            // NovaVisualComponent stores radius, sprite.custom_size is diameter
            let current_diameter = (visual.initial_radius + (visual.max_radius - visual.initial_radius) * progress) * 2.0;
            sprite.custom_size = Some(Vec2::splat(current_diameter.max(0.0))); // Ensure non-negative size

            // Keep fully opaque during expansion (use alpha from component's base color)
            sprite.color.set_a(visual.color.a()); 
        }
    }
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

// Helper function to spawn pool and despawn projectile
fn spawn_pool_and_despawn_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    projectile_entity: Entity,
    projectile_transform: &Transform,
    lob_comp: &LobbedProjectileComponent,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&lob_comp.pool_params.projectile_sprite_path), // Use pool sprite from params
            sprite: Sprite {
                custom_size: Some(Vec2::splat(lob_comp.pool_params.pool_radius * 2.0)),
                color: lob_comp.pool_params.pool_color,
                ..default()
            },
            transform: Transform::from_translation(projectile_transform.translation.truncate().extend(0.01)),
            ..default()
        },
        IchorPoolComponent {
            damage_per_tick: lob_comp.pool_params.pool_damage_per_tick,
            radius: lob_comp.pool_params.pool_radius,
            tick_timer: Timer::from_seconds(lob_comp.pool_params.pool_tick_interval_secs, TimerMode::Repeating),
            duration_timer: Timer::from_seconds(lob_comp.pool_params.pool_duration_secs, TimerMode::Once),
            color: lob_comp.pool_params.pool_color,
            already_hit_this_tick: Vec::new(),
        },
        Name::new("IchorPoolInstance (Targeted)"),
    ));
    commands.entity(projectile_entity).despawn_recursive();
}

pub fn lobbed_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, // For the helper function
    // Query for Lifetime as it's now the primary landing trigger
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &LobbedProjectileComponent, &mut Lifetime)>,
) {
    let g = 980.0; // pixels/sec^2, matches spawn function

    for (entity, mut transform, mut velocity, lob_comp, mut lifetime) in query.iter_mut() {
        // 1. Apply gravity
        velocity.0.y -= g * time.delta_seconds(); // Assuming positive g is downward, so subtract

        // 2. Update position based on velocity
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();

        // 3. Update rotation to match current velocity vector
        if velocity.0.length_squared() > 0.0 {
            transform.rotation = Quat::from_rotation_z(velocity.0.y.atan2(velocity.0.x));
        }

        // 4. Tick lifetime and check for landing
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            // Call the existing helper function to spawn pool and despawn projectile
            spawn_pool_and_despawn_projectile(&mut commands, &asset_server, entity, &transform, lob_comp);
            // The entity is despawned by the helper, so loop will not process it further.
        }
    }
}

pub fn ichor_pool_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, // For damage text
    mut pool_query: Query<(Entity, &mut IchorPoolComponent, &GlobalTransform)>,
    mut horror_query: Query<(Entity, &Transform, &mut crate::components::Health), With<crate::horror::Horror>>,
) {
    for (pool_entity, mut pool_comp, pool_gtransform) in pool_query.iter_mut() {
        // Tick duration timer and despawn if finished
        pool_comp.duration_timer.tick(time.delta());
        if pool_comp.duration_timer.finished() {
            commands.entity(pool_entity).despawn_recursive();
            continue;
        }

        // Tick damage timer
        pool_comp.tick_timer.tick(time.delta());
        if pool_comp.tick_timer.just_finished() {
            pool_comp.already_hit_this_tick.clear();
            let pool_center_pos = pool_gtransform.translation().truncate(); // Get pool's world position
            let pool_radius_sq = pool_comp.radius.powi(2);

            for (horror_entity, horror_transform, mut horror_health) in horror_query.iter_mut() {
                if pool_comp.already_hit_this_tick.contains(&horror_entity) {
                    continue;
                }
                
                let horror_pos = horror_transform.translation.truncate(); // Horror's world position
                if horror_pos.distance_squared(pool_center_pos) < pool_radius_sq {
                    horror_health.0 -= pool_comp.damage_per_tick;
                    // Spawn damage text visual effect
                    crate::visual_effects::spawn_damage_text(
                        &mut commands,
                        &asset_server,
                        horror_transform.translation, // Position for damage text
                        pool_comp.damage_per_tick,
                        &time
                    );
                    pool_comp.already_hit_this_tick.push(horror_entity);
                }
            }
        }
    }
}

pub fn channeled_beam_update_system(
    player_query: Query<(&Survivor, &Transform, &IsChannelingComponent)>, 
    mut beam_query: Query<&mut Transform, (With<ChanneledBeamComponent>, Without<Survivor>)>,
) {
    for (survivor_stats, player_transform, channeling_comp) in player_query.iter() {
        if let Some(beam_entity_id) = channeling_comp.beam_entity {
            if let Ok(mut beam_transform) = beam_query.get_mut(beam_entity_id) {
                let current_aim_direction = survivor_stats.aim_direction;

                if current_aim_direction == Vec2::ZERO {
                    // Optionally, keep last rotation or use a default if aim is zero.
                    // For now, we'll let it update, which might mean it points along X-axis if aim is zero.
                    // Or, we could skip updates if aim is zero:
                    // continue; 
                }

                let beam_width = channeling_comp.beam_params.beam_width;
                // Offset from survivor center, to edge, then half of beam width to align beam edge with survivor sprite edge
                let beam_spawn_offset = current_aim_direction * (crate::survivor::SURVIVOR_SIZE.y / 2.0 + beam_width / 4.0); 
                
                // Update beam position to follow player, applying offset
                beam_transform.translation = (player_transform.translation.truncate() + beam_spawn_offset)
                                             .extend(player_transform.translation.z + 0.1); // Ensure consistent Z-ordering

                // Update beam rotation to match player's aim
                beam_transform.rotation = Quat::from_rotation_z(current_aim_direction.y.atan2(current_aim_direction.x));
            }
        }
    }
}

#[derive(Component, Debug, Reflect)] // Default was removed as new fields need specific init
#[reflect(Component)]
pub struct OrbitingPetComponent {
    pub params_snapshot: crate::items::OrbitingPetParams,
    pub orbit_angle_rad: f32,
    pub duration_timer: Timer,
    pub pulse_timer: Option<Timer>,
    pub bolt_timer: Option<Timer>,
    pub owner_player_entity: Entity,
    pub is_deployed: bool, // New field
    pub deployed_position: Vec3, // New field
}

// Add a Default impl manually if Default derive was removed
impl Default for OrbitingPetComponent {
    fn default() -> Self {
        Self {
            params_snapshot: crate::items::OrbitingPetParams::default(),
            orbit_angle_rad: 0.0,
            duration_timer: Timer::from_seconds(1.0, TimerMode::Once),
            pulse_timer: None,
            bolt_timer: None,
            owner_player_entity: Entity::PLACEHOLDER,
            is_deployed: false, // Default to not deployed
            deployed_position: Vec3::ZERO, // Default position
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
            .init_resource::<ChainLightningLogState>() // Initialize the resource
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
            .register_type::<LobbedWeaponTargetReticuleComponent>() 
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
            // Added ChainZapParams to Reflect for potential editor usage if this system evolves.
            // Not strictly necessary if it's only passed as a direct param and not a component.
            .register_type::<crate::items::ChainZapParams>()
            .add_systems(Update, (
                manage_player_orbs_system,
                orbiting_pet_behavior_system,
                deploy_orbiting_pet_system, // Added new system
                tether_reactivation_window_system,
                returning_projectile_system,
                player_is_channeling_effect_system,
                channeled_beam_damage_system,
                ground_targeting_reticule_system,
                lobbed_weapon_targeting_reticule_system, // Added system to schedule
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, pending_ground_aoe_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, eruption_visual_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, player_dashing_system.run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                explode_on_lifetime_end_system, // Added new system
                generic_lifetime_system, // Existing system, now filtered
                lobbed_bouncing_projectile_system,
                magma_pool_system,
                repositioning_tether_firing_system,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                charge_weapon_system,
                // chain_lightning_visual_system, // Will be added below with other visual systems
                nova_visual_system,
                manage_persistent_aura_system,
                debuff_cloud_system,
                expanding_energy_bomb_system,
                homing_projectile_system,
                lobbed_projectile_system,
                ichor_pool_system,
                channeled_beam_update_system,
                // Added new systems here, ensuring they are not duplicated if already added by previous steps
                trail_spawning_projectile_system,
                fire_trail_segment_system,
                chain_lightning_visual_system, // Added the new visual system here
                chain_lightning_attack_system,
            ).run_if(in_state(AppState::InGame)));
    }
}

// --- Chain Lightning Attack System ---
pub fn chain_lightning_attack_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    time: Res<Time>, 
    player_query: Query<(&Transform, &Survivor)>, 
    sanity_strain_query: Query<&SurvivorSanityStrain, With<Survivor>>, 
    weapon_library: Res<AutomaticWeaponLibrary>, // Added
    mut horror_query: Query<(Entity, &Transform, &mut crate::components::Health), With<crate::horror::Horror>>, 
    mut log_state: ResMut<ChainLightningLogState>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    let Ok((player_transform, survivor_stats)) = player_query.get_single() else { return; }; 
    let Ok(sanity_strain) = sanity_strain_query.get_single() else { return; }; 

    if !sanity_strain.fire_timer.just_finished() { 
        // info!("SM_DEBUG_CL: Timer not finished. Skipping attack."); // Optional: Only if very verbose logging is needed
        return;
    }

    let actual_params = match weapon_library.get_weapon_definition(survivor_stats.inherent_weapon_id) {
        Some(def) => {
            if def.id == AutomaticWeaponId(5) { // Explicitly check if it's Chain Lightning
                match &def.attack_data {
                    AttackTypeData::ChainZap(params) => {
                        params.clone()
                    }
                    _ => {
                        return;
                    }
                }
            } else {
                // This system should only run for Chain Lightning. If inherent_weapon_id is different,
                // it implies an issue elsewhere or this system is being run too broadly.
                // For now, we just return if it's not the expected weapon.
                // This log can be removed later if it becomes too noisy once things are working.
                return;
            }
        }
        None => {
            return;
        }
    };

    if let Some(sound_path) = &actual_params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }

    let player_position = player_transform.translation.truncate();

    let mut initial_target_search_results: Vec<(Entity, f32, Transform)> = Vec::new();
    for (horror_entity, horror_transform, _health) in horror_query.iter() {
        let distance_sq = player_position.distance_squared(horror_transform.translation.truncate());
        if distance_sq < actual_params.initial_target_range.powi(2) { // Use actual_params
            initial_target_search_results.push((horror_entity, distance_sq, *horror_transform));
        }
    }
    
    let mut hit_targets: Vec<Entity> = Vec::new();

    let now = Instant::now();
    let mut log_this_specific_event = false;
    let mut initial_target_entity_opt: Option<Entity> = None;
    let mut initial_target_dist_sq_opt: Option<f32> = None;

    if !initial_target_search_results.is_empty() {
        initial_target_search_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let (initial_entity, initial_dist_sq, _) = initial_target_search_results[0];
        initial_target_entity_opt = Some(initial_entity);
        initial_target_dist_sq_opt = Some(initial_dist_sq);
        hit_targets.push(initial_entity); 
        // info!("SM_DEBUG_CL: Initial target found: {:?}, DistSq: {:?}", initial_target_entity_opt, initial_target_dist_sq_opt); // Moved this log

        if let Ok((_, target_actual_transform_ref, _)) = horror_query.get_mut(initial_entity) {
            let mut current_target_transform_for_chaining = *target_actual_transform_ref;
            let mut current_damage_for_chaining = actual_params.base_damage_per_zap; 
            for _chain_count in 1..=actual_params.max_chains { // Use actual_params
                let mut next_target_options_for_chaining: Vec<(Entity, f32, Transform)> = Vec::new();
                let current_search_origin_for_chaining = current_target_transform_for_chaining.translation.truncate();
                for (possible_next_entity, possible_next_transform, _health) in horror_query.iter() {
                    if hit_targets.contains(&possible_next_entity) { continue; }
                    let distance_sq_for_chaining = current_search_origin_for_chaining.distance_squared(possible_next_transform.translation.truncate());
                    if distance_sq_for_chaining < actual_params.chain_search_radius.powi(2) { // Use actual_params
                        next_target_options_for_chaining.push((possible_next_entity, distance_sq_for_chaining, *possible_next_transform));
                    }
                }
                if next_target_options_for_chaining.is_empty() { break; }
                next_target_options_for_chaining.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                let (next_target_entity_for_chaining, _, _) = next_target_options_for_chaining[0];
                current_damage_for_chaining = (current_damage_for_chaining as f32 * actual_params.damage_falloff_per_chain).round() as i32; // Use actual_params
                if current_damage_for_chaining == 0 { break; }
                if let Ok((_, next_transform_ref, _)) = horror_query.get_mut(next_target_entity_for_chaining) {
                    hit_targets.push(next_target_entity_for_chaining);
                    current_target_transform_for_chaining = *next_transform_ref;
                } else { break; }
            }
        }
    } else {
    }
    // Add this specific log for the option itself, after the if/else block
    
    if let Some(last_time) = log_state.last_log_time {
        if now.duration_since(last_time).as_secs_f32() >= 1.0 {
            if log_state.last_targets_hit_count.unwrap_or(0) > 0 && hit_targets.is_empty() { 
                log_this_specific_event = true;
            } else if log_state.last_targets_hit_count != Some(hit_targets.len()) && !hit_targets.is_empty() { 
                 log_this_specific_event = true;
            } else if log_state.last_targets_hit_count.is_none() && !hit_targets.is_empty() { 
                log_this_specific_event = true;
            }
        }
    } else if !hit_targets.is_empty() { 
        log_this_specific_event = true;
    }

    if log_this_specific_event {
        if initial_target_entity_opt.is_none() { 
            info!("Chain Lightning: No initial target found within range {}.", actual_params.initial_target_range); // Use actual_params
        } else {
            let initial_target_entity = initial_target_entity_opt.unwrap();
            let initial_target_dist_sq = initial_target_dist_sq_opt.unwrap();
            info!("Chain Lightning: Hit {} targets. Initial Range: {}. Search Radius: {}. Max Chains: {}.", 
                  hit_targets.len(), actual_params.initial_target_range, actual_params.chain_search_radius, actual_params.max_chains); // Use actual_params

            let player_actual_pos = player_transform.translation;
            if let Ok((_, target_actual_transform_ref, mut health)) = horror_query.get_mut(initial_target_entity) {
                let initial_target_actual_pos = target_actual_transform_ref.translation;
                health.0 = health.0.saturating_sub(actual_params.base_damage_per_zap); // Use actual_params
                crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, initial_target_actual_pos, actual_params.base_damage_per_zap, &time); // Use actual_params
                info!("  - Initial Target: {:?}, Damage: {}, Dist: {:.0}", initial_target_entity, actual_params.base_damage_per_zap, initial_target_dist_sq.sqrt()); // Use actual_params
                commands.spawn((
                    SpatialBundle::default(), // Added SpatialBundle
                    ChainLightningZapEffectComponent {
                        start_pos: player_actual_pos, end_pos: initial_target_actual_pos,
                        color: actual_params.zap_color, width: actual_params.zap_width, // Use actual_params
                        duration_timer: Timer::from_seconds(actual_params.zap_duration_secs, TimerMode::Once), // Use actual_params
                    }, Name::new("ChainLightningVisualSegment (Initial)"),
                ));

                let mut current_target_transform = *target_actual_transform_ref;
                let mut current_damage = actual_params.base_damage_per_zap; // Use actual_params
                let mut actual_hit_targets_in_chain_for_log = vec![initial_target_entity];

                for chain_count in 1..=actual_params.max_chains { // Use actual_params
                    let mut next_target_options: Vec<(Entity, f32, Transform)> = Vec::new();
                    let current_search_origin = current_target_transform.translation.truncate();
                    for (possible_next_entity, possible_next_transform, _health) in horror_query.iter() {
                        if actual_hit_targets_in_chain_for_log.contains(&possible_next_entity) { continue; }
                        let distance_sq = current_search_origin.distance_squared(possible_next_transform.translation.truncate());
                        if distance_sq < actual_params.chain_search_radius.powi(2) { // Use actual_params
                            next_target_options.push((possible_next_entity, distance_sq, *possible_next_transform));
                        }
                    }

                    if next_target_options.is_empty() {
                        info!("  - Chain broken after {} hits.", actual_hit_targets_in_chain_for_log.len());
                        break; 
                    }
                    next_target_options.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                    let (next_target_entity, _, _) = next_target_options[0];
                    current_damage = (current_damage as f32 * actual_params.damage_falloff_per_chain).round() as i32; // Use actual_params
                    if current_damage == 0 {
                        info!("  - Damage fell to zero at chain {}.", chain_count);
                        break;
                    }
                    
                    let zap_start_pos = current_target_transform.translation;
                    if let Ok((_, next_target_actual_transform_ref, mut health_chained)) = horror_query.get_mut(next_target_entity) {
                        let next_target_actual_pos = next_target_actual_transform_ref.translation;
                        health_chained.0 = health_chained.0.saturating_sub(current_damage);
                        crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, next_target_actual_pos, current_damage, &time);
                        info!("  - Chained Target {}: {:?}, Damage: {}", chain_count, next_target_entity, current_damage);
                        commands.spawn((
                            SpatialBundle::default(), // Added SpatialBundle
                            ChainLightningZapEffectComponent {
                                start_pos: zap_start_pos, end_pos: next_target_actual_pos,
                                color: actual_params.zap_color, width: actual_params.zap_width, // Use actual_params
                                duration_timer: Timer::from_seconds(actual_params.zap_duration_secs, TimerMode::Once), // Use actual_params
                            }, Name::new(format!("ChainLightningVisualSegment (Chain {})", chain_count)),
                        ));
                        current_target_transform = *next_target_actual_transform_ref;
                        actual_hit_targets_in_chain_for_log.push(next_target_entity);
                    } else {
                        error!("  - Chain link {}: Failed to get mutable health for target {:?}. Chain broken.", chain_count, next_target_entity);
                        break; 
                    }
                    if chain_count == actual_params.max_chains { // Use actual_params
                        info!("  - Max chains ({}) reached.", actual_params.max_chains); // Use actual_params
                    }
                }
            } else {
                 error!("Failed to get mutable health for initial target {:?} during logging/damage phase.", initial_target_entity);
            }
        }
        log_state.last_log_time = Some(now);
        log_state.last_targets_hit_count = Some(hit_targets.len()); 
    } else if initial_target_entity_opt.is_some() { 
        let initial_target_entity = initial_target_entity_opt.unwrap();
        let player_actual_pos = player_transform.translation;
        if let Ok((_, target_actual_transform_ref, mut health)) = horror_query.get_mut(initial_target_entity) {
            let initial_target_actual_pos = target_actual_transform_ref.translation;
            health.0 = health.0.saturating_sub(actual_params.base_damage_per_zap); // Use actual_params
            crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, initial_target_actual_pos, actual_params.base_damage_per_zap, &time); // Use actual_params
            commands.spawn((
                SpatialBundle::default(), // Added SpatialBundle
                ChainLightningZapEffectComponent {
                    start_pos: player_actual_pos, end_pos: initial_target_actual_pos,
                    color: actual_params.zap_color, width: actual_params.zap_width, // Use actual_params
                    duration_timer: Timer::from_seconds(actual_params.zap_duration_secs, TimerMode::Once), // Use actual_params
                }, Name::new("ChainLightningVisualSegment (Initial)"),
            ));

            let mut current_target_transform = *target_actual_transform_ref;
            let mut current_damage = actual_params.base_damage_per_zap; // Use actual_params
            let mut actual_hit_targets_in_chain = vec![initial_target_entity];

            for _chain_count in 1..=actual_params.max_chains { // Use actual_params
                let mut next_target_options: Vec<(Entity, f32, Transform)> = Vec::new();
                let current_search_origin = current_target_transform.translation.truncate();
                for (possible_next_entity, possible_next_transform, _health) in horror_query.iter() {
                    if actual_hit_targets_in_chain.contains(&possible_next_entity) { continue; }
                    let distance_sq = current_search_origin.distance_squared(possible_next_transform.translation.truncate());
                    if distance_sq < actual_params.chain_search_radius.powi(2) { // Use actual_params
                        next_target_options.push((possible_next_entity, distance_sq, *possible_next_transform));
                    }
                }
                if next_target_options.is_empty() { break; }
                next_target_options.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                let (next_target_entity, _, _) = next_target_options[0];
                current_damage = (current_damage as f32 * actual_params.damage_falloff_per_chain).round() as i32; // Use actual_params
                if current_damage == 0 { break; }
                
                let zap_start_pos = current_target_transform.translation;
                if let Ok((_, next_target_actual_transform_ref, mut health_chained)) = horror_query.get_mut(next_target_entity) {
                    let next_target_actual_pos = next_target_actual_transform_ref.translation;
                    health_chained.0 = health_chained.0.saturating_sub(current_damage);
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, next_target_actual_pos, current_damage, &time);
                    commands.spawn((
                        SpatialBundle::default(), // Added SpatialBundle
                        ChainLightningZapEffectComponent {
                            start_pos: zap_start_pos, end_pos: next_target_actual_pos,
                            color: actual_params.zap_color, width: actual_params.zap_width, // Use actual_params
                            duration_timer: Timer::from_seconds(actual_params.zap_duration_secs, TimerMode::Once), // Use actual_params
                        }, Name::new("ChainLightningVisualSegment (Chained)"),
                    ));
                    current_target_transform = *next_target_actual_transform_ref;
                    actual_hit_targets_in_chain.push(next_target_entity);
                } else { break; }
            }
        }
    }
}

// --- Generic Lifetime System ---

pub fn generic_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), Without<ExplodesOnFinalImpact>>, // MODIFIED query
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// --- New system for handling explosions on lifetime end ---
pub fn explode_on_lifetime_end_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    // Query for entities that have a lifetime, can explode, and optionally have damage for color reference
    mut query: Query<(Entity, &mut Lifetime, &GlobalTransform, &ExplodesOnFinalImpact, Option<&Damage>)>, 
    mut horror_query: Query<(&GlobalTransform, &mut Health), With<Horror>>, // For applying damage
) {
    for (entity, mut lifetime, g_transform, explodes_comp, _opt_damage_comp) in query.iter_mut() { // Changed opt_damage_comp to _opt_damage_comp
        // Important: We tick the timer here. If generic_lifetime_system also ticks it, it might double tick or cause issues.
        // This system now takes responsibility for ticking and acting for ExplodesOnFinalImpact entities.
        lifetime.timer.tick(time.delta()); 
        if lifetime.timer.just_finished() {
            // The opt_damage_comp is not used to determine color, Color::ORANGE_RED is hardcoded or could be from explodes_comp if it had a color field.
            let explosion_color = Color::ORANGE_RED; 

            spawn_explosion_effect(
                &mut commands,
                &asset_server,
                g_transform.translation(),
                explodes_comp.explosion_damage,
                explodes_comp.explosion_radius,
                explosion_color,
                String::from("sprites/explosion_placeholder.png"), // Placeholder sprite
                0.5,                                 // Duration
            );

            // Apply damage to horrors in radius
            let explosion_center = g_transform.translation();
            for (horror_gtransform, mut horror_health) in horror_query.iter_mut() {
                if horror_gtransform.translation().distance_squared(explosion_center) < explodes_comp.explosion_radius.powi(2) {
                    horror_health.0 = horror_health.0.saturating_sub(explodes_comp.explosion_damage);
                    visual_effects::spawn_damage_text(
                        &mut commands,
                        &asset_server,
                        horror_gtransform.translation(),
                        explodes_comp.explosion_damage,
                        &time,
                    );
                }
            }
            // Despawn the projectile entity since its lifetime is up and it has exploded.
            commands.entity(entity).despawn_recursive();
        }
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
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
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

    // The variable `params` from the loop is now `fire_params` to avoid collision with `effect_params`.
    for (player_entity, aim_direction, fire_params, weapon_id, player_transform_copy) in player_fire_requests {
        let mut tether_activated_this_iteration = false;

        // Use a block to limit the borrow scope of set.p2()
        // Correctly get PlayerWaitingTetherActivationComponent using p2 from ParamSet
        if let Ok(waiting_comp) = set.p2().get_mut(player_entity) {
            let hit_horror_entity = waiting_comp.hit_horror_entity; // Copy data
            let effect_params = waiting_comp.params.clone(); // Clone: component data for effect
            let next_effect_mode = waiting_comp.next_effect_mode;
            let timer_finished = waiting_comp.reactivation_window_timer.finished();

            if !timer_finished {
                // Timer has not finished, try to activate
                // Borrow from set.p1() now that borrow from set.p2() for waiting_comp is effectively done (data copied)
                // Correctly get Horror's Transform using p1 from ParamSet
                if let Ok(mut horror_transform) = set.p1().get_mut(hit_horror_entity) {
                    apply_tether_reposition_effect(
                        &mut horror_transform,
                        &player_transform_copy,
                        &effect_params, // Use the cloned params from the component
                        next_effect_mode,
                    );
                }
                // After effect, remove components using commands
                commands.entity(hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
                commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
                tether_activated_this_iteration = true;
            } else {
                // Timer has finished, just clean up components
                // Check if entity still exists before trying to remove component from it
                if commands.get_entity(hit_horror_entity).is_some() {
                    commands.entity(hit_horror_entity).remove::<HorrorLatchedByTetherComponent>();
                }
                commands.entity(player_entity).remove::<PlayerWaitingTetherActivationComponent>();
                // tether_activated_this_iteration remains false, so a new projectile will be fired.
            }
        }
        // Scopes of borrows from set.p1() and set.p2() must have ended before this point if they occurred.

        if !tether_activated_this_iteration {
            // Call the new function for spawning projectile
            // fire_params are the weapon definition params from the player_fire_requests loop
            spawn_actual_tether_projectile(
                &mut commands,
                &asset_server,
                player_entity,
                aim_direction,
                &fire_params, // These are the weapon's definition RepositioningTetherParams
                weapon_id,
                &player_transform_copy,
                &mut sound_event_writer,
            );
        }
    }
}

pub fn spawn_actual_tether_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    aim_direction: Vec2,
    weapon_params: &crate::items::RepositioningTetherParams, // This is the definition params
    weapon_id: crate::items::AutomaticWeaponId,
    player_transform: &Transform,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &weapon_params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
    // Spawns a new tether projectile
    let _projectile_entity = crate::automatic_projectiles::spawn_automatic_projectile(
        commands,
        asset_server,
        player_entity,
        player_transform.translation,
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
        Some(weapon_params.clone()), // Tether params (cloned from definition)
        None, // No blink strike params
        None // opt_trail_params
    );
}

pub fn spawn_lobbed_aoe_pool_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    _owner_entity: Entity, // The player entity
    player_transform: &Transform, // Player's current transform for initial position
    _aim_direction: Vec2, // May become unused if trajectory is solely based on target_world_pos
    weapon_params: &crate::items::LobbedAoEPoolParams,
    _weapon_id: crate::items::AutomaticWeaponId, // For potential tracking or specific logic, marked unused for now
    target_world_pos: Vec3, // New parameter
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &weapon_params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
    let g = 980.0; // pixels/sec^2, positive for downward influence
    let start_pos = player_transform.translation;

    let horizontal_distance = start_pos.truncate().distance(target_world_pos.truncate());

    let mut time_to_target = if weapon_params.projectile_speed > 0.01 {
        horizontal_distance / weapon_params.projectile_speed // projectile_speed is horizontal speed
    } else {
        1.0 // Default time if speed is effectively zero to avoid division by zero
    };

    // Refined handling for very short horizontal distances
    if horizontal_distance < 1.0 { // Target is (almost) directly above or below
        // Estimate time based on Y distance and some minimum speed, or fixed short time for lob
        let vertical_distance = (target_world_pos.y - start_pos.y).abs();
        // Use projectile_speed as a reference for how fast it *could* go vertically, capped by a reasonable minimum speed.
        // This attempts to make very vertical lobs take a sensible amount of time.
        time_to_target = (vertical_distance / weapon_params.projectile_speed.max(100.0)).max(0.1); // Ensure some time for arc
        time_to_target = time_to_target.max(0.05); // Absolute minimum time
    } else if time_to_target < 0.05 { // General minimum time for stability if target is very close
        time_to_target = 0.05;
    }
    
    let vx = (target_world_pos.x - start_pos.x) / time_to_target;
    // vy_initial = (delta_y / T) + (2*h_arc / T) + (g*T / 4)
    // This formula aims for h_arc above the chord midpoint.
    let vy = (target_world_pos.y - start_pos.y) / time_to_target +
             (2.0 * weapon_params.projectile_arc_height) / time_to_target +
             (0.25 * g * time_to_target);

    let initial_velocity = Vec2::new(vx, vy);

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&weapon_params.projectile_sprite_path),
            sprite: Sprite {
                custom_size: Some(weapon_params.projectile_size),
                color: weapon_params.projectile_color,
                ..default()
            },
            transform: Transform::from_translation(start_pos)
                .with_rotation(Quat::from_rotation_z(initial_velocity.y.atan2(initial_velocity.x))), // Rotate to initial launch vector
            ..default()
        },
        LobbedProjectileComponent {
            arc_height: weapon_params.projectile_arc_height, // Still stored for reference
            speed: weapon_params.projectile_speed, // Horizontal speed reference
            pool_params: weapon_params.clone(),
            initial_spawn_position: start_pos,
            target_position: Some(target_world_pos),
        },
        Velocity(initial_velocity),
        Damage(weapon_params.base_damage_on_impact),
        Lifetime { timer: Timer::from_seconds(time_to_target * 1.1, TimerMode::Once) }, // Lifetime slightly > time_to_target as failsafe
        Name::new("LobbedAoEPoolProjectile (Targeted)"),
    ));
}

pub fn spawn_orbiting_pet_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    player_transform: &Transform,
    params: &crate::items::OrbitingPetParams,
    orb_controller: &mut PlayerOrbControllerComponent,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.spawn_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
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

    // Determine initial deployment state
    // Orb always starts orbiting. 'can_be_deployed_at_location' signifies capability.
    let initial_is_deployed = false; 
    let initial_deployed_position = Vec3::ZERO; // Default, not used if not deployed

    let orb_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&params.orb_sprite_path), // Ensure correct path usage
            sprite: Sprite {
                custom_size: Some(params.orb_size),
                color: params.orb_color,
                ..default()
            },
            transform: Transform::from_translation(initial_pos), // Initial position
            ..default()
        },
        OrbitingPetComponent {
            params_snapshot: params.clone(),
            orbit_angle_rad: initial_offset_angle, // Still useful if it can switch to orbiting
            duration_timer: Timer::from_seconds(params.orb_duration_secs, TimerMode::Once),
            pulse_timer: pulse_timer_opt, // Assuming pulse_timer_opt and bolt_timer_opt are defined earlier
            bolt_timer: bolt_timer_opt,
            owner_player_entity: player_entity,
            is_deployed: initial_is_deployed, // Should now be false
            deployed_position: initial_deployed_position, // Default Vec3::ZERO
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
    // Changed orb_query to all_orbs_query to reflect its new role and components
    all_orbs_query: Query<(Entity, &OrbitingPetComponent)>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
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
        if let Some(mut controller) = opt_orb_controller {
            // 1. Get the true count of live orbs owned by this player.
            let mut current_live_orbs_owned_by_player: Vec<Entity> = Vec::new();
            for (orb_entity, orb_data) in all_orbs_query.iter() {
                if orb_data.owner_player_entity == player_entity {
                    current_live_orbs_owned_by_player.push(orb_entity);
                }
            }
            let live_orb_count = current_live_orbs_owned_by_player.len();

            // 2. Synchronize the controller's list with reality.
            controller.active_orb_entities = current_live_orbs_owned_by_player;

            // 3. Tick the cooldown timer.
            controller.spawn_cooldown_timer.tick(time.delta());

            // 4. Check conditions (using the accurate live_orb_count) and spawn if necessary.
            if controller.spawn_cooldown_timer.finished() && live_orb_count < controller.max_orbs_allowed as usize {
                // spawn_orbiting_pet_attack will add the new (pending) orb ID to controller.active_orb_entities.
                // This is fine, as this list is reconstructed from reality at the start of the next frame/run.
                spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut controller, &mut sound_event_writer);
                controller.spawn_cooldown_timer.reset(); 
            }
        } else {
            let mut new_controller = PlayerOrbControllerComponent {
                active_orb_entities: Vec::new(),
                max_orbs_allowed: params.max_active_orbs,
                spawn_cooldown_timer: Timer::from_seconds(params.base_fire_rate_secs, TimerMode::Repeating),
            };
            // Spawn initial orb(s)
            // Spawn initial orb(s). This logic remains the same, as new_controller.active_orb_entities is empty.
            if new_controller.active_orb_entities.len() < new_controller.max_orbs_allowed as usize {
                 spawn_orbiting_pet_attack(&mut commands, &asset_server, player_entity, player_transform, &params, &mut new_controller, &mut sound_event_writer);
                 new_controller.spawn_cooldown_timer.reset(); // Reset timer after this initial spawn
            }
            commands.entity(player_entity).insert(new_controller);
        }
    } else {
        // If shadow orb params are not present (e.g. player changed weapon),
        // despawn all orbs owned by this player.
        if opt_orb_controller.is_some() { // Check if controller exists before trying to query orbs
            for (orb_entity, orb_data) in all_orbs_query.iter() {
                if orb_data.owner_player_entity == player_entity {
                    commands.entity(orb_entity).despawn_recursive();
                }
            }
            // Remove the controller component from the player
            commands.entity(player_entity).remove::<PlayerOrbControllerComponent>();
        }
    }
}

pub fn deploy_orbiting_pet_system(
    _commands: Commands,
    mut player_query: Query<(Entity, &mut PlayerRequestsOrbDeployment), With<Survivor>>,
    // Query for orbs that are NOT currently deployed and belong to the player.
    // We might need to refine this query or iterate through all player orbs.
    // For simplicity, let's get all orbs of the player and find one that is not deployed.
    mut orb_query: Query<(Entity, &mut OrbitingPetComponent, &Transform)>,
) {
    if let Ok((player_entity, mut deploy_request)) = player_query.get_single_mut() {
        if deploy_request.0 { // If true, a deployment is requested
            let mut deployed_an_orb = false;
            for (_orb_entity, mut orb_comp, orb_transform) in orb_query.iter_mut() {
                // Check if this orb belongs to the player who made the request
                // AND if it's not already deployed.
                if orb_comp.owner_player_entity == player_entity && !orb_comp.is_deployed {
                    orb_comp.is_deployed = true;
                    orb_comp.deployed_position = orb_transform.translation; // Set deployed position to current
                    
                    // Optional: If you want the orb to stop moving immediately
                    // commands.entity(orb_entity).remove::<Velocity>(); // If orbs have a velocity component for orbiting
                    
                    deployed_an_orb = true;
                    break; // Deploy one orb at a time
                }
            }

            if deployed_an_orb {
                deploy_request.0 = false; // Reset the request
                info!("Player {:?} deployed an orb.", player_entity);
            } else {
                // Optional: Log if no orb was available to deploy
                // info!("Player {:?} requested orb deployment, but no suitable orb found or already deploying.", player_entity);
                // Reset request even if no orb found, to prevent spamming if held true
                deploy_request.0 = false; 
            }
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
    mut sound_event_writer: EventWriter<PlaySoundEvent>, // Added to play sounds
) {
    for (orb_entity, mut orb_transform, mut orb_comp) in pet_query.iter_mut() {
        orb_comp.duration_timer.tick(time.delta());
        if orb_comp.duration_timer.finished() {
            commands.entity(orb_entity).despawn_recursive();
            continue;
        }

        // Determine orb position based on deployment status
        if orb_comp.is_deployed {
            // If deployed, stay at the deployed position
            // Ensure deployed_position is Vec3. The Z component might need adjustment for visibility.
            orb_transform.translation = orb_comp.deployed_position; 
                                       // Make sure Z is appropriate, e.g. orb_comp.deployed_position.truncate().extend(0.1)
                                       // Or, if deployed_position is already Vec3 with correct Z: orb_transform.translation = orb_comp.deployed_position;
            // No need to update orbit_angle_rad if it's stationary.
        } else {
            // If not deployed, orbit the player
            if let Ok(owner_transform) = player_query.get(orb_comp.owner_player_entity) {
                orb_comp.orbit_angle_rad += orb_comp.params_snapshot.orbit_speed_rad_per_sec * time.delta_seconds();
                orb_comp.orbit_angle_rad %= std::f32::consts::TAU;

                let offset = Vec2::from_angle(orb_comp.orbit_angle_rad) * orb_comp.params_snapshot.orbit_radius;
                // Ensure Z-offset is consistent, e.g., player's Z + small offset, or a fixed Z for orbs.
                let new_translation_z = owner_transform.translation.z + 0.1; // Example Z adjustment
                orb_transform.translation = (owner_transform.translation.truncate() + offset).extend(new_translation_z);
            } else {
                // Owner (player) not found, despawn the orb
                commands.entity(orb_entity).despawn_recursive();
                continue; // Skip further processing for this orb
            }
        }

        // Pulse AoE logic (should use orb_transform.translation, which is now correctly set)
        if orb_comp.params_snapshot.pulses_aoe {
            if let Some(ref mut pulse_timer) = orb_comp.pulse_timer {
                pulse_timer.tick(time.delta());
                if pulse_timer.just_finished() {
                    let orb_position = orb_transform.translation; // World position of the orb
                    if let Some(pulse_viz_color) = orb_comp.params_snapshot.pulse_color {
                        // Define wave parameters before spawning
                        let wave_initial_radius_val = orb_comp.params_snapshot.orb_size.x * 0.1; // e.g., 32.0 * 0.1 = 3.2 radius
                        let wave_max_radius_val = orb_comp.params_snapshot.orb_size.x * 1.0;    // e.g., 32.0 * 1.0 = 32.0 radius (for 64.0 diameter)
                        let wave_duration_secs = 0.25; // Short duration for quick wave expansion
                        
                        // Spawn the pulse visual as a child of the orb entity
                        let pulse_visual_entity = commands.spawn((
                            SpriteBundle {
                                texture: asset_server.load("sprites/pulse_effect_placeholder.png"), // This sprite should ideally be a circle/ring
                                sprite: Sprite {
                                    color: pulse_viz_color, // Should be opaque from params now
                                    custom_size: Some(Vec2::splat(wave_initial_radius_val * 2.0)), // Initial diameter
                                    ..default()
                                },
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)), 
                                ..default()
                            },
                            NovaVisualComponent {
                                initial_radius: wave_initial_radius_val,
                                max_radius: wave_max_radius_val,
                                duration_timer: Timer::from_seconds(wave_duration_secs, TimerMode::Once),
                                color: pulse_viz_color, // Passed to system, which will use its alpha
                            },
                            Name::new("OrbWavePulseVisual"), // Renamed for clarity
                        )).id();
                        commands.entity(orb_entity).add_child(pulse_visual_entity);
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::ShadowOrbPulse)); // Play pulse sound
                    }

                    // Damage application logic remains unchanged
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
                        // Corrected: Use query.get() and destructure the tuple.
                        if let Ok((_target_entity_from_get, target_gtransform)) = horror_query.get(target_entity) {
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
                                    None // opt_trail_params
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
    owner_entity: Entity, // New parameter
    survivor_stats: &Survivor, // New parameter
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
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
        Damage(params.damage_per_bounce_impact + survivor_stats.auto_weapon_damage_bonus),
        Lifetime { timer: Timer::from_seconds(params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile {
            owner: Some(owner_entity), // Corrected: owner -> Some(owner)
            piercing_left: 0,
            weapon_id,
            bounces_left: Some(params.num_bounces),
            damage_on_hit: params.damage_per_bounce_impact + survivor_stats.auto_weapon_damage_bonus,
            current_speed: params.projectile_speed,
            damage_loss_per_bounce_multiplier: Some(1.0),
            speed_loss_per_bounce_multiplier: Some(1.0),
            has_bounced_this_frame: false,
            lifesteal_percentage: None,
            blink_params_on_hit: None,
        },
        ExplodesOnFinalImpact { // Added component
            explosion_radius: params.explosion_radius_on_final_bounce,
            explosion_damage: params.explosion_damage_on_final_bounce,
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
        &Transform,    // Local transform
        &GlobalTransform, // Added for world-space explosion position
        &mut Lifetime,
        &crate::automatic_projectiles::AutomaticProjectile,
        Option<&ExplodesOnFinalImpact>, // To check for explosion data
    )>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, // For applying damage
) {
    for (
        entity,
        mut bouncing_comp,
        _velocity, 
        _damage,   
        transform,      // Local transform for existing logic (like pool spawning)
        g_transform,    // Global transform for explosion
        mut lifetime,
        auto_proj_comp,
        explodes_comp_opt,
    ) in projectile_query.iter_mut()
    {
        lifetime.timer.tick(time.delta());
        // This system primarily handles bounce logic. Lifetime despawn with explosion is now handled by explode_on_lifetime_end_system.
        // However, if a bounce *results* in bounces_left == 0, it should explode *then*.

        if auto_proj_comp.bounces_left.is_some() && auto_proj_comp.bounces_left.unwrap() < bouncing_comp.bounces_left {
            bouncing_comp.bounces_left = auto_proj_comp.bounces_left.unwrap();

            if rand::random::<f32>() < bouncing_comp.params.fire_pool_on_bounce_chance {
                spawn_magma_pool(
                    &mut commands,
                    &asset_server,
                    transform.translation, // Pool uses local transform as before
                    &bouncing_comp.params,
                );
            }

            if bouncing_comp.bounces_left == 0 {
                if let Some(explodes_comp) = explodes_comp_opt {
                    spawn_explosion_effect(
                        &mut commands,
                        &asset_server,
                        g_transform.translation(), // Explosion at global position
                        explodes_comp.explosion_damage,
                        explodes_comp.explosion_radius,
                        bouncing_comp.params.projectile_color, // Use projectile color for now
                        String::from("sprites/explosion_placeholder.png"),   // Placeholder sprite
                        0.5,                                   // Duration
                    );
                    // Apply damage to horrors in radius
                    for (_horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() {
                        if horror_gtransform.translation().distance_squared(g_transform.translation()) < explodes_comp.explosion_radius.powi(2) {
                            horror_health.0 = horror_health.0.saturating_sub(explodes_comp.explosion_damage);
                            visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), explodes_comp.explosion_damage, &time);
                        }
                    }
                }
                commands.entity(entity).despawn_recursive(); // Despawn after explosion
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

// --- Explosion Effect Spawning ---

pub fn spawn_explosion_effect(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    _damage: i32, // Damage application will be handled by the calling system
    radius: f32,
    color: Color,
    visual_sprite_path: String, // Changed to String
    duration_secs: f32,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(visual_sprite_path), // Handles String correctly
            sprite: Sprite {
                custom_size: Some(Vec2::splat(radius * 0.2)), // Initial small size
                color: color,
                ..default()
            },
            transform: Transform::from_translation(position.truncate().extend(0.3)), // Ensure Z-level is appropriate
            ..default()
        },
        NovaVisualComponent {
            initial_radius: radius * 0.1,
            max_radius: radius,
            duration_timer: Timer::from_seconds(duration_secs, TimerMode::Once),
            color: color, // Use the main color for nova, or could be different
        },
        Lifetime { timer: Timer::from_seconds(duration_secs, TimerMode::Once) },
        Name::new("ExplosionEffectVisual"),
    ));
}

// --- Line Dash Attack Systems ---

pub fn spawn_line_dash_attack(
    commands: &mut Commands,
    player_entity: Entity,
    player_stats: &mut crate::survivor::Survivor,
    player_transform: &Transform,
    params: &crate::items::LineDashAttackParams,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
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
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
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
    game_time: Res<Time>, 
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut ReturningProjectileComponent, &mut Velocity, &mut Transform)>,
    projectile_damage_query: Query<&Damage, With<ReturningProjectileComponent>>,
    mut horror_query: Query<(Entity, &Transform, &mut Health, &Horror), (With<Horror>, Without<ReturningProjectileComponent>)>,
    mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>,
) {
    for (entity, mut projectile_comp, mut velocity, mut transform) in query.iter_mut() {
        // Apply movement based on velocity
        transform.translation.x += velocity.0.x * game_time.delta_seconds();
        transform.translation.y += velocity.0.y * game_time.delta_seconds();
        // Rotate projectile to face direction of movement
        if velocity.0.length_squared() > 0.0 {
            transform.rotation = Quat::from_rotation_z(velocity.0.y.atan2(velocity.0.x));
        }

        let projectile_world_transform = *transform;

        if let Ok(projectile_damage) = projectile_damage_query.get(entity) {
            let projectile_radius = projectile_comp.projectile_size.x / 2.0;

            for (horror_entity, horror_transform, mut horror_health, horror_stats) in horror_query.iter_mut() {
                if projectile_comp.hit_enemies_this_pass.contains(&horror_entity) {
                    continue; 
                }

                let distance = projectile_world_transform.translation.distance(horror_transform.translation);
                let horror_radius = horror_stats.size.x / 2.0; // Assuming Horror struct has 'size: Vec2'

                if distance < projectile_radius + horror_radius {
                    horror_health.0 = horror_health.0.saturating_sub(projectile_damage.0);
                    crate::visual_effects::spawn_damage_text(
                        &mut commands,
                        &asset_server,
                        horror_transform.translation,
                        projectile_damage.0,
                        &game_time,
                    );
                    sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::HorrorHit));

                    projectile_comp.hit_enemies_this_pass.push(horror_entity);
                    projectile_comp.piercing_left = projectile_comp.piercing_left.saturating_sub(1);

                    if projectile_comp.piercing_left == 0 {
                        commands.entity(entity).despawn_recursive();
                        break; 
                    }
                }
            }
        } else {
            error!("Returning projectile {:?} has no Damage component!", entity);
        }

        if commands.get_entity(entity).is_none() { // Check if despawned by piercing
            continue;
        }

        match projectile_comp.state {
            ReturningProjectileState::Outgoing => {
                let distance_traveled = transform.translation.distance(projectile_comp.start_position);
                if distance_traveled >= projectile_comp.max_travel_distance {
                    projectile_comp.state = ReturningProjectileState::Returning;
                    projectile_comp.hit_enemies_this_pass.clear(); // Add this line
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
    _weapon_id: AutomaticWeaponId,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
    info!("spawn_standard_projectile_attack called for sprite: {}, damage: {}, fire_rate: {}", params.projectile_sprite_path, params.base_damage, params.base_fire_rate_secs);
}

pub fn spawn_returning_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &ReturningProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
    let spawn_offset_distance = SURVIVOR_SIZE.x / 2.0 + params.projectile_size.x / 4.0; // Spawn slightly ahead of player's edge + projectile's own radius
    let offset_vector = aim_direction.normalize_or_zero() * spawn_offset_distance;

    let mut start_pos = player_transform.translation;
    start_pos.x += offset_vector.x;
    start_pos.y += offset_vector.y;
    start_pos.z += 0.1; // Ensure it's slightly above the player/other sprites on the same plane

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
            hit_enemies_this_pass: Vec::new(), // Initialize
            projectile_size: params.projectile_size, // Initialize
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
    enemy_query: &mut Query<(Entity, &Transform, &mut Health, &Horror), With<Horror>>,
    time: &Res<Time>,
    sound_event_writer: &mut EventWriter<PlaySoundEvent>,
) {
    if let Some(sound_path) = &params.fire_sound_effect {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::Path(sound_path.clone())));
    }
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

    for (_enemy_entity, enemy_transform, mut enemy_health, _horror_tag) in enemy_query.iter_mut() {
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