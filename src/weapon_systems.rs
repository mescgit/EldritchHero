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
use crate::survivor::{BASE_SURVIVOR_SPEED as BASE_PLAYER_SPEED, Survivor, SanityStrain as SurvivorSanityStrain, SURVIVOR_SIZE};
use crate::camera_systems::MainCamera;
use crate::horror::Horror;
use crate::game::AppState;
use crate::visual_effects;
use crate::audio::{PlaySoundEvent, SoundEffect}; // Added

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
                            // Optional: Play charge level up sound
                            sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::RitualCast)); // Placeholder sound
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
            // Mouse button is NOT pressed, but ChargingWeaponComponent still exists and is_actively_charging.
            // This state should ideally be handled by survivor_casting_system to fire the shot and remove the component.
            // As a safeguard, or if survivor_casting_system's release logic somehow missed it: 
            charging_comp.is_actively_charging = false;
        }
    }
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
            .add_systems(Update, (
                manage_player_orbs_system,
                orbiting_pet_behavior_system,
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
) {
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
) {
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
    owner_entity: Entity, // New parameter
    survivor_stats: &Survivor // New parameter
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
        Damage(params.damage_per_bounce_impact + survivor_stats.auto_weapon_damage_bonus),
        Lifetime { timer: Timer::from_seconds(params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile {
            owner: owner_entity,
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
    game_time: Res<Time>, 
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut ReturningProjectileComponent, &mut Velocity, &Transform)>,
    projectile_damage_query: Query<&Damage, With<ReturningProjectileComponent>>,
    mut horror_query: Query<(Entity, &Transform, &mut Health, &Horror), (With<Horror>, Without<ReturningProjectileComponent>)>,
    mut sound_event_writer: EventWriter<crate::audio::PlaySoundEvent>,
) {
    for (entity, mut projectile_comp, mut velocity, transform) in query.iter_mut() {
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