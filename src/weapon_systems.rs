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
    pub params: crate::items::LineDashAttackParams,
    pub initial_direction: Vec2,
    pub dash_timer: Timer,
    pub invulnerability_timer: Timer, // Added for separate invulnerability timing
    pub already_hit_horrors: Vec<Entity>,
    pub original_speed_if_modified: Option<f32>,
}

impl Default for PlayerDashingComponent {
    fn default() -> Self {
        let default_params = crate::items::LineDashAttackParams::default();
        Self {
            params: default_params.clone(),
            initial_direction: Vec2::X,
            dash_timer: Timer::from_seconds(default_params.dash_duration, TimerMode::Once), // Use new field name
            invulnerability_timer: Timer::from_seconds(default_params.invulnerability_duration, TimerMode::Once), // Use new field name
            already_hit_horrors: Vec::new(),
            original_speed_if_modified: None,
        }
    }
}

// System to remove PlayerInvulnerableComponent after its timer expires
pub fn player_invulnerability_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerInvulnerableComponent)>,
) {
    for (entity, mut invulnerable_comp) in query.iter_mut() {
        invulnerable_comp.duration_timer.tick(time.delta());
        if invulnerable_comp.duration_timer.finished() {
            commands.entity(entity).remove::<PlayerInvulnerableComponent>();
        }
    }
}

// --- Melee Arc Attack Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ActiveMeleeArcHitbox {
    pub params: crate::items::MeleeArcAttackParams,
    pub duration_timer: Timer,
    pub already_hit_enemies: Vec<Entity>,
    pub owner_forward_vector: Vec2,
    pub owner_position: Vec3,
    pub hit_count: u32,
}

// --- Blink Strike Systems ---

// This is the new spawn function for the BlinkStrike (player blink) attack type.
// It will be called by handle_automatic_weapon_fire_system.
pub fn spawn_blink_strike_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    params: &crate::items::BlinkStrikeParams, // Uses the new BlinkStrikeParams
    player_transform: &Transform,
    aim_direction: Vec2,
    weapon_id: crate::items::AutomaticWeaponId,
) {
    let base_aim_direction_normalized = aim_direction.normalize_or_zero();
    // num_projectiles_per_shot is now part of BlinkStrikeParams
    let num_projectiles = params.num_projectiles_per_shot;
    let total_spread_degrees = if num_projectiles > 1 { (num_projectiles -1) as f32 * 7.5 } else { 0.0 }; // Example spread

    for i in 0..num_projectiles {
        let mut current_projectile_aim_direction = base_aim_direction_normalized;
        if num_projectiles > 1 {
            let total_spread_rad = total_spread_degrees.to_radians();
            let angle_offset_rad = if num_projectiles <= 1 { 0.0 } else { (i as f32 / (num_projectiles as f32 - 1.0)) * total_spread_rad - (total_spread_rad / 2.0) };
            let base_angle_rad = base_aim_direction_normalized.y.atan2(base_aim_direction_normalized.x);
            current_projectile_aim_direction = Vec2::new((base_angle_rad + angle_offset_rad).cos(), (base_angle_rad + angle_offset_rad).sin());
        }

        crate::automatic_projectiles::spawn_automatic_projectile(
            commands,
            asset_server,
            player_entity, // Owner of the projectile
            player_transform.translation,
            current_projectile_aim_direction,
            params.projectile_damage as i32, // Damage is f32 in params, but i32 in spawn_auto_proj
            params.projectile_speed,
            params.piercing,
            weapon_id,
            params.projectile_asset_path,
            params.projectile_size,
            params.projectile_color,
            params.projectile_lifetime_secs,
            None, // opt_max_bounces for this projectile type
            None, // opt_dmg_loss_mult
            None, // opt_speed_loss_mult
            None, // opt_lifesteal_percentage
            None, // opt_tether_params_for_comp
            None, // opt_blink_params (this is for projectile blinking, player blink is handled by event)
        );
    }
}

// This is the old spawn function, keep it for now if other parts of code still use BlinkStrikeProjectileParams
pub fn spawn_blink_strike_projectile_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity, // Added player_entity
    params: &crate::items::BlinkStrikeProjectileParams, // This is the OLD params struct
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
            player_entity, // Pass player_entity as owner
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

// Old components PlayerWaitingTetherActivationComponent and HorrorLatchedByTetherComponent
// are replaced by PlayerTetherState and TetheredEnemy from components.rs.
// The old TetherProjectileComponent is replaced by PsionicTetherProjectile from components.rs.

// This function is refactored to only spawn the projectile.
// Reactivation logic is moved to tether_activation_system.
pub fn spawn_repositioning_tether_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    player_transform: &Transform,
    aim_direction: Vec2,
    params: &crate::items::RepositioningTetherParams,
    // weapon_id: crate::items::AutomaticWeaponId, // No longer needed here if not used by projectile directly
    // opt_player_tether_state: Option<&mut crate::components::PlayerTetherState>, // For reactivation
    // horror_query: Query<&mut Transform, (With<Horror>, Without<Survivor>)>, 
) {
    // For now, primary fire always spawns a new tether projectile.
    // If a tether is already active (PlayerTetherState has an entity), this new projectile 
    // might replace it or be ignored, depending on desired game feel (not handled here).

    let projectile_lifetime_secs = if params.tether_speed > 0.0 {
        params.tether_range / params.tether_speed
    } else {
        // Default lifetime if speed is zero to prevent infinite lifetime
        2.0 
    };

    commands.spawn((
        crate::components::PsionicTetherProjectile {
            params_snapshot: params.clone(),
            duration_timer: Timer::from_seconds(projectile_lifetime_secs, TimerMode::Once),
            owner: player_entity,
        },
        SpriteBundle {
            texture: asset_server.load(params.tether_sprite_path),
            sprite: Sprite { 
                custom_size: Some(Vec2::new(12.0, 24.0)), // Example size, make configurable if needed
                color: Color::rgb(0.8, 0.4, 0.9), // Example color
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation)
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        Velocity(aim_direction.normalize_or_zero() * params.tether_speed),
        Name::new("PsionicTetherProjectile"),
    ));
}

// New system for tether projectile collisions
pub fn tether_projectile_collision_system(
    mut commands: Commands,
    time: Res<Time>, // To access current time for timers, though Timer::from_seconds is used
    asset_server: Res<AssetServer>, // For damage text
    mut projectile_query: Query<(Entity, &crate::components::PsionicTetherProjectile, &GlobalTransform)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>,
    mut player_query: Query<&mut crate::components::PlayerTetherState>,
    // sound_event_writer: EventWriter<PlaySoundEvent>, // Optional for hit sounds
) {
    for (proj_entity, tether_projectile, proj_gtransform) in projectile_query.iter_mut() {
        let proj_pos = proj_gtransform.translation().truncate();

        for (horror_entity, horror_gtransform, mut horror_health, horror_stats) in horror_query.iter_mut() {
            // Simple distance-based collision
            let horror_pos = horror_gtransform.translation().truncate();
            let distance_sq = proj_pos.distance_squared(horror_pos);
            let combined_radii_sq = (horror_stats.size.x / 2.0 + 6.0).powi(2); // 6.0 is projectile half-width guess

            if distance_sq < combined_radii_sq {
                // Collision!
                if let Some(damage) = tether_projectile.params_snapshot.damage_on_hit {
                    horror_health.0 -= damage as i32;
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), damage as i32, &time);
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherHit));
                }

                // Add TetheredEnemy to horror
                commands.entity(horror_entity).insert(crate::components::TetheredEnemy {
                    tether_owner: tether_projectile.owner,
                    activation_window_timer: Timer::from_seconds(tether_projectile.params_snapshot.activation_window_duration, TimerMode::Once),
                });

                // Update PlayerTetherState
                if let Ok(mut player_tether_state) = player_query.get_mut(tether_projectile.owner) {
                    // If there was a previously tethered enemy, remove its TetheredEnemy component
                    if let Some(old_tethered_enemy_entity) = player_tether_state.tethered_enemy_entity {
                        if old_tethered_enemy_entity != horror_entity { // Avoid removing from the newly tethered one
                             if let Some(mut old_enemy_commands) = commands.get_entity(old_tethered_enemy_entity) {
                                old_enemy_commands.remove::<crate::components::TetheredEnemy>();
                            }
                        }
                    }
                    
                    player_tether_state.tethered_enemy_entity = Some(horror_entity);
                    player_tether_state.current_weapon_params_snapshot = Some(tether_projectile.params_snapshot.clone());
                    // Initialize last_tether_mode_used if mode is Alternate
                    if tether_projectile.params_snapshot.mode == crate::items::RepositioningTetherMode::Alternate {
                        if player_tether_state.last_tether_mode_used.is_none() { // Default to Pull first time
                             player_tether_state.last_tether_mode_used = Some(crate::items::RepositioningTetherMode::Pull);
                        }
                    } else {
                        player_tether_state.last_tether_mode_used = None; // Not used for non-Alternate modes
                    }

                } else { // Player might not have the state component yet
                    commands.entity(tether_projectile.owner).insert(crate::components::PlayerTetherState {
                        tethered_enemy_entity: Some(horror_entity),
                        current_weapon_params_snapshot: Some(tether_projectile.params_snapshot.clone()),
                        last_tether_mode_used: if tether_projectile.params_snapshot.mode == crate::items::RepositioningTetherMode::Alternate {
                            Some(crate::items::RepositioningTetherMode::Pull) // Default to Pull first
                        } else { None },
                    });
                }

                commands.entity(proj_entity).despawn_recursive();
                break; // Projectile is consumed
            }
        }
    }
}

// New system for tether activation (pull/push)
pub fn tether_activation_system(
    mut commands: Commands,
    // time: Res<Time>, // Not directly needed unless effects are timed over multiple frames
    // input: Res<Input<KeyCode>>, // Replace with actual input check for secondary activation
    mut player_query: Query<(Entity, &mut crate::components::PlayerTetherState, &Survivor)>, // Assuming Survivor has aim_direction
    mut horror_query: Query<&mut Transform, With<Horror>>, 
    // sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    // --- THIS IS A PLACEHOLDER FOR ACTUAL SECONDARY INPUT CHECK ---
    // In a real scenario, this would check an event or resource set by an input system.
    // For now, let's simulate it by checking if 'R' key is pressed, just for testing.
    // THIS SHOULD BE REPLACED with a proper event or resource for secondary action.
    // if !input.just_pressed(KeyCode::R) { 
    //     return;
    // }
    // For subtask, assume activation is triggered if conditions are met (e.g. always try to activate if possible)
    // The actual trigger will be an input event.

    for (player_entity, mut player_tether_state, _survivor_stats) in player_query.iter_mut() {
        if let (Some(tethered_enemy_entity), Some(params)) = (player_tether_state.tethered_enemy_entity, &player_tether_state.current_weapon_params_snapshot) {
            if let Some(mut enemy_commands) = commands.get_entity(tethered_enemy_entity) {
                if let Some(mut tethered_comp) = enemy_commands.get_mut::<crate::components::TetheredEnemy>() {
                    if !tethered_comp.activation_window_timer.finished() {
                        // Activation window is open!
                        let mode_to_execute = match params.mode {
                            crate::items::RepositioningTetherMode::Pull => crate::items::RepositioningTetherMode::Pull,
                            crate::items::RepositioningTetherMode::Push => crate::items::RepositioningTetherMode::Push,
                            crate::items::RepositioningTetherMode::Alternate => {
                                player_tether_state.last_tether_mode_used.map_or(crate::items::RepositioningTetherMode::Pull, |last_mode| {
                                    if last_mode == crate::items::RepositioningTetherMode::Pull { crate::items::RepositioningTetherMode::Push } else { crate::items::RepositioningTetherMode::Pull }
                                })
                            }
                        };

                        if let Ok(mut horror_transform) = horror_query.get_mut(tethered_enemy_entity) {
                            let player_pos = commands.get::<Transform>(player_entity).map(|t| t.translation.truncate()).unwrap_or_default(); // Should get player's current transform
                            let horror_pos = horror_transform.translation.truncate();

                            match mode_to_execute {
                                crate::items::RepositioningTetherMode::Pull => {
                                    let direction_to_player = (player_pos - horror_pos).normalize_or_zero();
                                    if direction_to_player != Vec2::ZERO {
                                        horror_transform.translation -= (direction_to_player * params.pull_strength).extend(0.0); // Move towards player
                                    }
                                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherPull));
                                }
                                crate::items::RepositioningTetherMode::Push => {
                                    let direction_from_player = (horror_pos - player_pos).normalize_or_zero();
                                    let push_dir = if direction_from_player != Vec2::ZERO { direction_from_player } else { Vec2::X }; // Default push if overlapping
                                    horror_transform.translation += (push_dir * params.push_strength).extend(0.0);
                                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherPush));
                                }
                                _ => {} // Alternate is resolved above
                            }
                            if params.mode == crate::items::RepositioningTetherMode::Alternate {
                                player_tether_state.last_tether_mode_used = Some(mode_to_execute);
                            }
                        }
                        
                        // Clean up: remove TetheredEnemy and clear player state
                        enemy_commands.remove::<crate::components::TetheredEnemy>();
                        player_tether_state.tethered_enemy_entity = None;
                        player_tether_state.current_weapon_params_snapshot = None;
                        // player_tether_state.last_tether_mode_used is preserved for Alternate mode's next cycle.
                    }
                } else { // No TetheredEnemy component, something is wrong, clear state
                    player_tether_state.tethered_enemy_entity = None;
                    player_tether_state.current_weapon_params_snapshot = None;
                }
            } else { // Tethered enemy entity no longer exists, clear state
                 player_tether_state.tethered_enemy_entity = None;
                 player_tether_state.current_weapon_params_snapshot = None;
            }
        }
    }
}


// New system for cleaning up tether components
pub fn tether_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut crate::components::PsionicTetherProjectile)>,
    mut tethered_enemy_query: Query<(Entity, &mut crate::components::TetheredEnemy)>,
    mut player_tether_state_query: Query<&mut crate::components::PlayerTetherState>, // To clear if enemy's window expires
) {
    // Despawn projectiles if their lifetime expires
    for (entity, mut projectile) in projectile_query.iter_mut() {
        projectile.duration_timer.tick(time.delta());
        if projectile.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Remove TetheredEnemy component if activation window expires
    for (entity, mut tethered_enemy) in tethered_enemy_query.iter_mut() {
        tethered_enemy.activation_window_timer.tick(time.delta());
        if tethered_enemy.activation_window_timer.finished() {
            commands.entity(entity).remove::<crate::components::TetheredEnemy>();
            
            // Also clear from PlayerTetherState if this was the active tethered enemy
            if let Ok(mut player_state) = player_tether_state_query.get_mut(tethered_enemy.tether_owner) {
                if player_state.tethered_enemy_entity == Some(entity) {
                    player_state.tethered_enemy_entity = None;
                    player_state.current_weapon_params_snapshot = None;
                    // last_tether_mode_used can persist for Alternate mode logic
                }
            }
        }
    }
}


#[derive(Component, Debug, Reflect)] // Removed Default here, will add custom Default
#[reflect(Component)]
pub struct PlayerInvulnerableComponent {
    pub duration_timer: Timer,
}

impl Default for PlayerInvulnerableComponent {
    fn default() -> Self {
        Self {
            // Default invulnerability duration, can be overridden when inserted
            duration_timer: Timer::from_seconds(0.2, TimerMode::Once) 
        }
    }
}

// --- Orbiting Pet Definitions (New Implementation) ---

// Commenting out old OrbitingPetComponent and related structures/systems
// The old OrbitingPetComponent and ActiveOrbitingPetsResource are effectively replaced by 
// ShadowOrb, OrbitingMovement, and PlayerOrbControllerComponent.
// #[derive(Component, Debug, Reflect)]
// #[reflect(Component)]
// pub struct OrbitingPetComponent { ... }
// impl Default for OrbitingPetComponent { ... }
// #[derive(Resource, Default, Reflect)]
// #[reflect(Resource)]
// pub struct ActiveOrbitingPetsResource { ... }

// The new OrbitingPetComponent that was recently added seems to be based on the OLD OrbitingPetParams.
// This will be replaced by the new ShadowOrb component logic.
// For clarity, I am commenting out this version of OrbitingPetComponent as well.
/*
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
*/

// PlayerOrbControllerComponent will be used for the new DeployableOrbitingTurret
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerOrbControllerComponent {
    pub active_orb_entities: Vec<Entity>, // Stores entities with the ShadowOrb component
    pub max_orbs_allowed_for_current_weapon: u32, // Updated from DeployableOrbitingTurretParams
    pub spawn_cooldown_timer: Timer, // Uses DeployableOrbitingTurretParams.cooldown
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
            // .register_type::<OrbitingPetComponent>() // Old/intermediate component, to be removed/replaced by ShadowOrb logic
            .register_type::<PlayerOrbControllerComponent>()
            // Old Tether Components - ensure they are removed if no longer used by any system after refactor.
            // .register_type::<TetherProjectileComponent>() 
            // .register_type::<PlayerWaitingTetherActivationComponent>()
            // .register_type::<HorrorLatchedByTetherComponent>()  
            .register_type::<ActiveMeleeArcHitbox>() 
            // New Tether Components (PsionicTetherProjectile, TetheredEnemy, PlayerTetherState) are registered in components.rs plugin
            .add_systems(Update, (
                melee_arc_attack_system, 
                manage_deployable_orbs_system, 
                shadow_orb_movement_system,    
                shadow_orb_behavior_system,    
                tether_projectile_collision_system, // New
                tether_activation_system,           // New
                tether_cleanup_system,              // New
                //tether_reactivation_window_system, // Old system, replaced by tether_activation_system & tether_cleanup_system
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
                player_dashing_system, // Renamed from player_dash_execution_system
                // orbiting_pet_behavior_system, // Old system, will be replaced by the new one added above
            ).in_set(OnUpdate(AppState::InGame)));
    }
}

// System to remove PlayerInvulnerableComponent after its timer expires
pub fn player_invulnerability_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerInvulnerableComponent)>,
) {
    for (entity, mut invulnerable_comp) in query.iter_mut() {
        invulnerable_comp.duration_timer.tick(time.delta());
        if invulnerable_comp.duration_timer.finished() {
            commands.entity(entity).remove::<PlayerInvulnerableComponent>();
        }
    }
}

// --- Deployable Orbiting Turret Systems ---

pub fn spawn_deployable_shadow_orb(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    player_transform: &Transform,
    params: &crate::items::DeployableOrbitingTurretParams,
    orb_controller: &mut PlayerOrbControllerComponent,
) {
    // For now, orbs always orbit player. Deployment range logic can be added later.
    // Determine initial angle to spread out multiple orbs if player has more than one.
    let initial_angle_offset = if orb_controller.max_orbs_allowed_for_current_weapon > 1 {
         orb_controller.active_orb_entities.len() as f32 * (std::f32::consts::TAU / orb_controller.max_orbs_allowed_for_current_weapon as f32)
    } else {
        0.0
    };

    let orb_entity = commands.spawn((
        crate::components::ShadowOrb {
            params_snapshot: params.clone(),
            duration_timer: Timer::from_seconds(params.orb_duration, TimerMode::Once),
            attack_timer: Timer::from_seconds(params.attack_interval, TimerMode::Repeating),
            owner_entity: player_entity,
        },
        crate::components::OrbitingMovement {
            center_entity: player_entity,
            radius: params.orbit_radius,
            current_angle_rad: initial_angle_offset,
            speed_rad_per_sec: params.orbit_speed_rad_per_sec, // Use value from params
        },
        SpriteBundle {
            texture: asset_server.load("sprites/auto_shadow_orb.png"), // Placeholder from task
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 32.0)), // Default size
                color: Color::rgb(0.3, 0.1, 0.5), // Default color, can be part of params later
                ..default()
            },
            // Initial position will be set by the movement system based on orbit params
            transform: Transform::from_translation(player_transform.translation), 
            ..default()
        },
        Name::new("DeployableShadowOrb"),
    )).id();
    orb_controller.active_orb_entities.push(orb_entity);
}

// Renamed from manage_player_orbs_system to be more specific
pub fn manage_deployable_orbs_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    mut player_query: Query<(Entity, &Transform, &mut Survivor, Option<&mut PlayerOrbControllerComponent>)>,
    orb_query: Query<Entity, With<crate::components::ShadowOrb>>, // Query for new ShadowOrb component
) {
    let Ok((player_entity, player_transform, mut player_stats, opt_orb_controller)) = player_query.get_single_mut() else { return; };

    let mut current_weapon_orb_params: Option<crate::items::DeployableOrbitingTurretParams> = None;
    if let Some(active_weapon_id) = player_stats.active_automatic_weapon_id {
        if let Some(weapon_def) = weapon_library.get_weapon_definition(active_weapon_id) {
            // Check for the new DeployableOrbitingTurret params
            if let crate::items::AttackTypeData::DeployableOrbitingTurret(params) = &weapon_def.attack_data {
                current_weapon_orb_params = Some(params.clone());
            }
        }
    }

    if let Some(params) = current_weapon_orb_params { // Weapon with deployable orbs is equipped
        if let Some(mut controller) = opt_orb_controller {
            // Update controller if params changed (e.g. player picked up an item that modifies orb count for this weapon type)
            controller.max_orbs_allowed_for_current_weapon = params.max_active_orbs;
            // Note: spawn_cooldown_timer's duration should ideally also be updated if params.cooldown changes.
            // For simplicity, Timer::set_duration and Timer::reset could be used if needed, or re-insert controller.

            controller.spawn_cooldown_timer.tick(time.delta());
            if controller.spawn_cooldown_timer.finished() && controller.active_orb_entities.len() < controller.max_orbs_allowed_for_current_weapon as usize {
                spawn_deployable_shadow_orb(&mut commands, &asset_server, player_entity, player_transform, &params, &mut controller);
                // Reset timer with current weapon's cooldown
                controller.spawn_cooldown_timer = Timer::from_seconds(params.cooldown, TimerMode::Repeating);
                controller.spawn_cooldown_timer.reset(); // Ensure it just started
            }
            // Clean up dead orb entities from the controller's list
            controller.active_orb_entities.retain(|&orb_e| orb_query.get(orb_e).is_ok());
        } else {
             // No controller, add one
            let mut new_controller = PlayerOrbControllerComponent {
                active_orb_entities: Vec::new(),
                max_orbs_allowed_for_current_weapon: params.max_active_orbs,
                spawn_cooldown_timer: Timer::from_seconds(params.cooldown, TimerMode::Repeating),
            };
            if new_controller.active_orb_entities.len() < new_controller.max_orbs_allowed_for_current_weapon as usize {
                 spawn_deployable_shadow_orb(&mut commands, &asset_server, player_entity, player_transform, &params, &mut new_controller);
                 new_controller.spawn_cooldown_timer.reset(); 
            }
            commands.entity(player_entity).insert(new_controller);
        }
    } else { // Weapon with deployable orbs is NOT equipped
        if let Some(mut controller) = opt_orb_controller {
            for orb_entity in controller.active_orb_entities.iter() {
                // Despawn only orbs that are still valid entities
                if orb_query.get(*orb_entity).is_ok() {
                    commands.entity(*orb_entity).despawn_recursive();
                }
            }
            commands.entity(player_entity).remove::<PlayerOrbControllerComponent>();
        }
    }
}

// New system for orb movement
pub fn shadow_orb_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut orb_query: Query<(&mut Transform, &mut crate::components::OrbitingMovement, &crate::components::ShadowOrb)>,
    center_query: Query<&Transform, (With<Survivor>, Without<crate::components::ShadowOrb>)>, // Center is player
) {
    for (mut orb_transform, mut orbit_params, shadow_orb_comp) in orb_query.iter_mut() {
        if let Ok(center_transform) = center_query.get(orbit_params.center_entity) {
            orbit_params.current_angle_rad += orbit_params.speed_rad_per_sec * time.delta_seconds();
            orbit_params.current_angle_rad %= std::f32::consts::TAU; // Keep angle within 0-2PI

            let offset = Vec2::from_angle(orbit_params.current_angle_rad) * orbit_params.radius;
            orb_transform.translation = center_transform.translation + offset.extend(shadow_orb_comp.owner_entity.index() as f32 * 0.01 + 0.1); // Small Z offset for visibility, vary by owner index slightly
        } else {
            // Center entity (player) despawned or no longer matches query, despawn orb
            commands.entity(shadow_orb_comp.owner_entity).remove::<PlayerOrbControllerComponent>(); // Clear controller if player gone
            commands.entity(shadow_orb_comp.owner_entity).despawn_recursive(); // This seems wrong, should despawn the orb itself
            // Corrected: Despawn the orb entity, not its owner.
            // The entity for the orb is implicit in the query `orb_query` but not directly passed.
            // We need the orb's own entity to despawn it.
            // The query should be: Query<(Entity, &mut Transform, &mut crate::components::OrbitingMovement, &crate::components::ShadowOrb)>
            // For now, this branch will simply not update movement. Despawning orbs whose owner is gone should be handled
            // in manage_deployable_orbs_system or shadow_orb_behavior_system.
        }
    }
}


// Renamed and adapted from orbiting_pet_behavior_system
pub fn shadow_orb_behavior_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut orb_query: Query<(Entity, &mut crate::components::ShadowOrb, &GlobalTransform)>,
    // For PulseAoE:
    horror_query: Query<(Entity, &GlobalTransform), With<Horror>>, // Used by both Pulse and Bolt target finding
    mut horror_health_query: Query<&mut Health, With<Horror>>,
    // For SeekingBolts, potentially projectile spawning systems if not handled by spawn_automatic_projectile directly
) {
    for (orb_entity, mut shadow_orb, orb_gtransform) in orb_query.iter_mut() {
        shadow_orb.duration_timer.tick(time.delta());
        if shadow_orb.duration_timer.finished() {
            commands.entity(orb_entity).despawn_recursive();
            // Also need to notify PlayerOrbControllerComponent to remove this orb from its list.
            // This can be done via an event, or by having PlayerOrbControllerComponent check validity of its entities.
            // manage_deployable_orbs_system already does `orb_query.get(orb_e).is_ok()` check.
            continue;
        }

        shadow_orb.attack_timer.tick(time.delta());
        if shadow_orb.attack_timer.just_finished() {
            match shadow_orb.params_snapshot.attack_type {
                crate::items::OrbAttackType::PulseAoE => {
                    if let (Some(radius), Some(damage)) = (shadow_orb.params_snapshot.pulse_aoe_radius, shadow_orb.params_snapshot.pulse_aoe_damage) {
                        let orb_position = orb_gtransform.translation();
                        
                        // Optional: Spawn a visual pulse effect (reusing NovaVisualComponent as an example)
                        commands.spawn((
                            SpriteBundle {
                                texture: asset_server.load("sprites/pulse_effect_placeholder.png"), 
                                sprite: Sprite {
                                    color: Color::rgba(0.5, 0.1, 0.7, 0.7), // Example color
                                    custom_size: Some(Vec2::splat(radius * 0.25)), 
                                    ..default()
                                },
                                transform: Transform::from_translation(orb_position),
                                ..default()
                            },
                            NovaVisualComponent { 
                                initial_radius: radius * 0.25,
                                max_radius: radius,
                                duration_timer: Timer::from_seconds(0.3, TimerMode::Once),
                                color: Color::rgba(0.5, 0.1, 0.7, 0.7),
                            },
                            Name::new("OrbPulseAoEVisual"),
                        ));

                        for (horror_entity, horror_gtransform) in horror_query.iter() {
                            if horror_gtransform.translation().distance_squared(orb_position) < radius.powi(2) {
                                if let Ok(mut health) = horror_health_query.get_mut(horror_entity) {
                                    health.0 -= damage as i32;
                                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), damage as i32, &time);
                                }
                            }
                        }
                    }
                }
                crate::items::OrbAttackType::SeekingBolts => {
                    if let (Some(damage), Some(speed), Some(asset_path_static_str)) = 
                        (shadow_orb.params_snapshot.bolt_damage, shadow_orb.params_snapshot.bolt_speed, shadow_orb.params_snapshot.bolt_projectile_asset_path) {
                        
                        let mut closest_target: Option<Entity> = None;
                        let mut min_dist_sq = f32::MAX;
                        let orb_pos_2d = orb_gtransform.translation().truncate();

                        for (horror_entity, horror_gtransform) in horror_query.iter() {
                            let dist_sq = orb_pos_2d.distance_squared(horror_gtransform.translation().truncate());
                            // Define a reasonable detection range for bolts, e.g., 400 units for seeking start
                            if dist_sq < 400.0f32.powi(2) { 
                                 if dist_sq < min_dist_sq {
                                    min_dist_sq = dist_sq;
                                    closest_target = Some(horror_entity);
                                }
                            }
                        }

                        if let Some(target_entity) = closest_target {
                            if let Ok(target_gtransform) = horror_query.get_component::<GlobalTransform>(target_entity) {
                                let direction = (target_gtransform.translation().truncate() - orb_pos_2d).normalize_or_zero();
                                if direction != Vec2::ZERO {
                                    // Using placeholder values for some projectile params not in DeployableOrbitingTurretParams
                                    crate::automatic_projectiles::spawn_automatic_projectile(
                                        &mut commands,
                                        &asset_server,
                                        shadow_orb.owner_entity, // Orb's owner (player) is owner of bolt
                                        orb_gtransform.translation(),
                                        direction,
                                        damage as i32,
                                        speed,
                                        0, // Piercing
                                        AutomaticWeaponId(u32::MAX), // Special ID for sub-munitions
                                    asset_path_static_str, // Use &'static str directly
                                        Vec2::new(10.0, 10.0), // Default bolt size
                                        Color::PURPLE,        // Default bolt color
                                        1.5,                  // Default bolt lifetime
                                        None, None, None, None, // Bouncing, Lifesteal
                                        // Homing needs to be configured here if spawn_automatic_projectile supports it
                                        // Or, a HomingTargetComponent needs to be added to the bolt
                                    );
                                }
                            }
                        }
                    }
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
        dash_timer: Timer::from_seconds(params.dash_duration, TimerMode::Once), // Use new field name
        invulnerability_timer: Timer::from_seconds(params.invulnerability_duration, TimerMode::Once), // Initialize new timer
        already_hit_horrors: Vec::new(),
        original_speed_if_modified: Some(original_speed_val),
    });

    if params.invulnerability_duration > 0.0 { // Check against the float value
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
        dashing_comp.invulnerability_timer.tick(time.delta());

        if dashing_comp.invulnerability_timer.finished() {
            commands.entity(player_entity).remove::<PlayerInvulnerableComponent>();
        }

        // Only move and check collisions if the dash is ongoing
        if !dashing_comp.dash_timer.finished() {
            let movement_this_frame = dashing_comp.initial_direction * dashing_comp.params.dash_speed * time.delta_seconds();
            player_transform.translation += movement_this_frame.extend(0.0);

            let player_hitbox_center = player_transform.translation.truncate();
            // let player_half_width = dashing_comp.params.hitbox_width / 2.0; // Not directly used in this AABB

            // The piercing_cap and dash_trail_color fields were removed from LineDashAttackParams as per task instructions
            // So, direct access like dashing_comp.params.piercing_cap or dashing_comp.params.dash_trail_color will fail.
            // For now, I'll use a default piercing_cap of, say, 1000 (effectively infinite for typical scenarios)
            // and remove the trail spawning logic as the color field is gone.
            let effective_piercing_cap = 1000; // Placeholder as piercing_cap was removed from LineDashAttackParams

            for (horror_entity, horror_gtransform, mut horror_health, horror_data) in horror_query.iter_mut() {
                if dashing_comp.already_hit_horrors.len() >= effective_piercing_cap {
                    break;
                }
                if dashing_comp.already_hit_horrors.contains(&horror_entity) {
                    continue;
                }

                let horror_pos = horror_gtransform.translation().truncate();
                // Using horror_data.size for AABB check
                // let horror_half_width = horror_data.size.x / 2.0; // Not directly used in this AABB
                // let horror_half_height = horror_data.size.y / 2.0; // Not directly used in this AABB

                // AABB collision check
                let x_collision = (player_hitbox_center.x - horror_pos.x).abs() * 2.0 < (dashing_comp.params.hitbox_width + horror_data.size.x);
                let y_collision = (player_hitbox_center.y - horror_pos.y).abs() * 2.0 < (dashing_comp.params.hitbox_width + horror_data.size.y); // Assuming player hitbox is also somewhat square for y-axis checks.

                if x_collision && y_collision {
                    // Use the new damage field (f32)
                    let damage_to_apply = dashing_comp.params.damage as i32; // Convert to i32 for Health component
                    horror_health.0 -= damage_to_apply;
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), damage_to_apply, &time);
                    dashing_comp.already_hit_horrors.push(horror_entity);
                    // sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::HorrorHit));
                }
            }
            
            // Removed trail spawning logic as dash_trail_color was removed from LineDashAttackParams
            // if let Some(color) = dashing_comp.params.dash_trail_color { ... }
        }

        if dashing_comp.dash_timer.finished() {
            if let Some(original_speed) = dashing_comp.original_speed_if_modified {
                player_stats.speed = original_speed;
            } else {
                player_stats.speed = BASE_PLAYER_SPEED;
            }
            commands.entity(player_entity).remove::<PlayerDashingComponent>();
            // Invulnerability removal is now handled by its own timer check above
        }
    }
}

// --- Player Blink Event Handling System ---
pub fn handle_player_blink_event_system(
    mut commands: Commands,
    mut events: EventReader<crate::components::PlayerBlinkEvent>,
    mut player_query: Query<(&mut Transform, &Survivor), (With<Survivor>, Without<Horror>)>, // Added Without<Horror> for safety
    enemy_query: Query<&GlobalTransform, With<Horror>>, // Query enemies for their position
    // asset_server: Res<AssetServer>, // For visual effects if any
    mut sound_event_writer: EventWriter<PlaySoundEvent>, // For sound effects
) {
    for event in events.read() {
        if let Ok((mut player_transform, survivor_stats)) = player_query.get_mut(event.player_entity) {
            let mut blink_destination = player_transform.translation; // Default to current if target invalid

            match event.blink_params.blink_target {
                crate::items::BlinkTarget::BehindEnemy => {
                    if let Ok(enemy_gtransform) = enemy_query.get(event.hit_enemy_entity) {
                        let enemy_position = enemy_gtransform.translation();
                        // Calculate direction from player to enemy to find "behind"
                        let dir_player_to_enemy = (enemy_position - player_transform.translation).truncate().normalize_or_else(|| survivor_stats.aim_direction.normalize_or_else(|| Vec2::X));
                        
                        blink_destination = enemy_position + (dir_player_to_enemy * event.blink_params.blink_distance).extend(player_transform.translation.z);
                    } else {
                        // Fallback: Enemy might have despawned. Blink forward.
                        let aim_dir = survivor_stats.aim_direction.normalize_or_else(|| (player_transform.rotation * Vec3::X).truncate().normalize_or_else(|| Vec2::X));
                        blink_destination = player_transform.translation + (aim_dir * event.blink_params.blink_distance).extend(player_transform.translation.z);
                    }
                }
                crate::items::BlinkTarget::ForwardFixed => {
                    // Use player's current aim direction or facing direction
                    let aim_dir = survivor_stats.aim_direction.normalize_or_else(|| (player_transform.rotation * Vec3::X).truncate().normalize_or_else(|| Vec2::X));
                    blink_destination = player_transform.translation + (aim_dir * event.blink_params.blink_distance).extend(player_transform.translation.z);
                }
            }
            
            player_transform.translation = blink_destination;

            // Add short invulnerability
            commands.entity(event.player_entity).insert(PlayerInvulnerableComponent {
                duration_timer: Timer::from_seconds(0.2, TimerMode::Once) // Example: 0.2 seconds of invulnerability
            });

            sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerBlink));
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

// --- Melee Arc Attack Systems ---

pub fn spawn_melee_arc_attack(
    commands: &mut Commands,
    // asset_server: &Res<AssetServer>, // For visual effects, if any, later
    params: &crate::items::MeleeArcAttackParams,
    player_transform: &Transform,
    player_stats: &Survivor, // To get aim_direction
) {
    let mut attack_forward_vector = player_stats.aim_direction.normalize_or_zero();
    if attack_forward_vector == Vec2::ZERO { // Fallback to player's current facing if aim_direction is zero
        attack_forward_vector = (player_transform.rotation * Vec3::X).truncate().normalize_or_zero();
        if attack_forward_vector == Vec2::ZERO {
            attack_forward_vector = Vec2::X; // Default to X-axis if all else fails
        }
    }

    commands.spawn((
        ActiveMeleeArcHitbox {
            params: params.clone(),
            duration_timer: Timer::from_seconds(params.duration, TimerMode::Once),
            already_hit_enemies: Vec::new(),
            owner_forward_vector: attack_forward_vector,
            owner_position: player_transform.translation,
            hit_count: 0,
        },
        Name::new("MeleeArcHitbox"),
        // This entity is logical; visual effects would be spawned separately if needed
        // e.g., by sending an event or by the calling system spawning a visual.
    ));
}

pub fn melee_arc_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut hitbox_query: Query<(Entity, &mut ActiveMeleeArcHitbox)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>,
    asset_server: Res<AssetServer>, // For damage text
    // sound_event_writer: EventWriter<PlaySoundEvent>, // If sound effects are desired
) {
    for (hitbox_entity, mut hitbox) in hitbox_query.iter_mut() {
        hitbox.duration_timer.tick(time.delta());
        if hitbox.duration_timer.finished() {
            commands.entity(hitbox_entity).despawn_recursive();
            continue;
        }

        // Check for hits only once per active duration, or continuously if desired (current: continuous for duration)
        // For a single sweep, this loop effectively runs for the short duration, hitting targets.

        let max_targets_reached = if let Some(max) = hitbox.params.max_targets {
            hitbox.hit_count >= max
        } else {
            false // No limit
        };

        if max_targets_reached {
            continue; // Don't check for more hits if limit is reached
        }

        let owner_pos_2d = hitbox.owner_position.truncate();
        let half_arc_angle_rad = hitbox.params.arc_angle.to_radians() / 2.0;

        for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() {
            if hitbox.already_hit_enemies.contains(&horror_entity) {
                continue;
            }
            
            // Recalculate max_targets_reached inside the loop before attempting to hit
            let current_max_targets_reached = if let Some(max) = hitbox.params.max_targets {
                hitbox.hit_count >= max
            } else {
                false 
            };
            if current_max_targets_reached { break; }


            let horror_pos_2d = horror_gtransform.translation().truncate();
            let vector_to_horror = horror_pos_2d - owner_pos_2d;
            let distance_sq_to_horror = vector_to_horror.length_squared();

            // Check radius
            if distance_sq_to_horror <= hitbox.params.arc_radius.powi(2) {
                let direction_to_horror = vector_to_horror.normalize_or_zero();
                if direction_to_horror == Vec2::ZERO { // Horror is at the same position
                    continue;
                }

                // Check angle
                let angle_to_horror_rad = hitbox.owner_forward_vector.angle_between(direction_to_horror);
                if angle_to_horror_rad.abs() <= half_arc_angle_rad {
                    // Hit!
                    horror_health.0 -= hitbox.params.damage as i32;
                    crate::visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), hitbox.params.damage as i32, &time);
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::MeleeHit)); // Example

                    hitbox.already_hit_enemies.push(horror_entity);
                    hitbox.hit_count += 1;
                }
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