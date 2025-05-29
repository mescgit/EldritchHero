// mescgit/eldritchhero/EldritchHero-77df6cd0b3e48857123b0971c9f30b59714a1b8a/src/weapon_systems.rs
use bevy::prelude::*;
use bevy::prelude::Name; // Added Name import
use crate::items::{StandardProjectileParams, ReturningProjectileParams, ChanneledBeamParams, ConeAttackParams, AutomaticWeaponId};
use crate::components::{Velocity, Damage, Lifetime}; // Removed prelude::Name
use crate::survivor::{BASE_SURVIVOR_SPEED as BASE_PLAYER_SPEED}; // Removed Survivor as Player

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
    pub tick_timer: Timer, // Timer to apply damage ticks
    pub range: f32,
    pub width: f32,
    pub color: Color,
    // Stores the player entity that spawned this beam
    pub owner: Entity,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct IsChannelingComponent {
    // Entity of the spawned beam, to despawn it when channeling stops
    pub beam_entity: Option<Entity>,
    // Store the params for easy access by other systems if needed
    pub beam_params: ChanneledBeamParams,
    // pub last_updated: f32, // For later advanced termination logic
}

// --- Lobbed AoE Pool Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct LobbedProjectileComponent {
    // target_position: Vec3, // Simplified: projectile flies straight, lands after lifetime.
    pub arc_height: f32, // Not used in initial simplified movement.
    pub speed: f32, // Not directly used by this component if Velocity is used.
    // pub impact_damage: i32, // Removed, as Damage component on entity handles this.
    pub pool_params: crate::items::LobbedAoEPoolParams, // To know what kind of pool to spawn.
    pub initial_spawn_position: Vec3, // Used for range check or if Lifetime isn't precise enough.
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct IchorPoolComponent {
    pub damage_per_tick: i32,
    pub radius: f32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub color: Color, // For visual consistency if needed, though pool_params has it.
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ActiveIchorPools {
    pub pools: std::collections::VecDeque<Entity>, // Changed to VecDeque
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
    // Simplified: No start/end needed if we just spawn at target
    // pub start_pos: Vec3,
    // pub end_pos: Vec3,
    // pub color: Color, // Color will be set directly on the sprite
    // pub width: f32,   // Width will be the sprite's size
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
    pub aura_color: Color, // For debug or generic visuals
    pub visual_entity: Option<Entity>,
    pub weapon_id: crate::items::AutomaticWeaponId, // Tracks which weapon this aura is for
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
    Detonated, // Might be short-lived, for cleanup or post-detonation effects
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ExpandingEnergyBombComponent {
    pub params: crate::items::ExpandingEnergyBombParams,
    pub current_radius: f32,
    pub expansion_timer: Timer,
    pub wait_at_max_radius_timer: Timer,
    pub state: SpiritBombState,
    // No need for attached_to_player if it's always a child of the player while active
}

// --- Homing Debuff Projectile Definitions ---

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct HomingTargetComponent {
    pub target_entity: Option<Entity>,
    pub strength: f32, // Turn rate towards target
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct DebuffOnHitComponent {
    pub debuff_type: crate::items::ProjectileDebuffType,
    pub magnitude_per_stack: f32,
    pub max_stacks: u32,
    pub duration_secs: f32,
}


// Helper function to spawn the beam entity
pub fn spawn_beam_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &ChanneledBeamParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    owner_entity: Entity, // The player
) -> Entity {
    let beam_transform = Transform::from_translation(player_transform.translation)
        .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x)));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/auto_arcane_ray.png"), // Placeholder, consider making this part of params
            sprite: Sprite {
                custom_size: Some(Vec2::new(params.range, params.beam_width)), // Width is height for a rotated sprite
                color: params.beam_color,
                anchor: bevy::sprite::Anchor::CenterLeft, // Anchor so it extends from player
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
            .register_type::<SpiritBombState>() // Added new type
            .register_type::<ExpandingEnergyBombComponent>() 
            .register_type::<HomingTargetComponent>() // Added new component
            .register_type::<DebuffOnHitComponent>() // Added new component
            .add_systems(Update, (
                charge_weapon_system, 
                trail_spawning_projectile_system, 
                fire_trail_segment_system,      
                chain_lightning_visual_system, 
                nova_visual_system, 
                manage_persistent_aura_system, 
                debuff_cloud_system, 
                expanding_energy_bomb_system, 
                homing_projectile_system, // Added new system
                returning_projectile_system,
                lobbed_projectile_system,
                ichor_pool_system,
                player_is_channeling_effect_system,
                channeled_beam_update_system,
                channeled_beam_damage_system, // Added new system
            ));
        // Other attack type systems will be added here
    }
}

pub fn channeled_beam_damage_system(
    mut _commands: Commands, // For potential effects later, or if an enemy dies
    time: Res<Time>,
    mut beam_query: Query<(&mut ChanneledBeamComponent, &GlobalTransform)>, // Beam's global transform for accurate collision
    // Query for entities that can be damaged (e.g., enemies with Health)
    // Assuming enemies have a Health component from crate::components
    mut enemy_query: Query<(Entity, &Transform, &mut crate::components::Health), With<crate::horror::Horror>>, // Changed to horror
    // Optional: For spawning damage text or other effects
    // asset_server: Res<AssetServer>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (mut beam_comp, beam_gtransform) in beam_query.iter_mut() {
        beam_comp.tick_timer.tick(time.delta());
        if !beam_comp.tick_timer.just_finished() {
            continue;
        }

        // The beam's visual is a sprite starting at the player and extending outwards.
        // Its length is beam_comp.range, width is beam_comp.width.
        // The GlobalTransform gives its world position and rotation.
        let beam_start_pos = beam_gtransform.translation().truncate();
        let beam_rotation_quat = beam_gtransform.compute_transform().rotation;
        let beam_direction = (beam_rotation_quat * Vec3::X).truncate(); // Sprite is anchored CenterLeft, so X is its length direction

        // Simplified collision: Check entities within a certain distance along the beam's line.
        // A more accurate collision would use the beam's width (shape casting or multiple raycasts).
        // For now, iterate enemies and check if they are roughly along the beam's path and within range.
        for (_enemy_entity, enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
            let enemy_pos = enemy_transform.translation.truncate();
            
            // Vector from beam start to enemy
            let to_enemy = enemy_pos - beam_start_pos;
            
            // Project to_enemy onto beam_direction to find distance along beam
            let distance_along_beam = to_enemy.dot(beam_direction);
            
            // Check if enemy is within beam's length
            if distance_along_beam > 0.0 && distance_along_beam < beam_comp.range {
                // Check if enemy is close enough to the beam's centerline (perpendicular distance)
                let perpendicular_distance = (to_enemy - distance_along_beam * beam_direction).length();
                
                // Using a default enemy radius as ENEMY_SIZE is not directly accessible here.
                let enemy_radius = 16.0; // Default value as per instructions. Changed from HORROR_SIZE
                if perpendicular_distance < (beam_comp.width / 2.0) + enemy_radius { 
                    // Enemy is hit
                    enemy_health.0 -= beam_comp.damage_per_tick;
                    // info!("Beam hit enemy {:?}, health: {}", _enemy_entity, enemy_health.0); // For debugging

                    // Here you could spawn damage text or play hit sounds
                    // spawn_damage_text(&mut commands, &asset_server, enemy_transform.translation, beam_comp.damage_per_tick, &time);
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyHit)); // Changed to HorrorHit

                    if enemy_health.0 <= 0 {
                        // Potentially handle enemy death here or let another system do it
                        // commands.entity(enemy_entity).despawn_recursive();
                    }
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
    initial_aim_direction: Vec2, // Base aim direction from player
    weapon_id: crate::items::AutomaticWeaponId,
    all_horrors_query: &Query<(Entity, &GlobalTransform), With<crate::horror::Horror>>,
) {
    let player_pos = player_transform.translation.truncate();

    for i in 0..params.num_darts_per_shot {
        let mut current_aim_direction = initial_aim_direction;
        if params.num_darts_per_shot > 1 {
            // Simple spread: +/- 10 degrees for 2 shots, +/- 15 for 3, etc.
            let spread_angle_degrees = 10.0 * (params.num_darts_per_shot as f32 -1.0) / 2.0;
            let angle_offset_degrees = if params.num_darts_per_shot > 1 {
                (i as f32 * (spread_angle_degrees * 2.0) / (params.num_darts_per_shot as f32 - 1.0)) - spread_angle_degrees
            } else { 0.0 };
            current_aim_direction = Quat::from_rotation_z(angle_offset_degrees.to_radians()) * current_aim_direction.extend(0.0);
            current_aim_direction = current_aim_direction.truncate().normalize_or_zero();
        }
        
        // Find initial target (simplified: nearest in radius)
        let mut closest_target: Option<(Entity, f32)> = None; // (Entity, distance_sq)
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
                piercing_left: 0, // Debuff projectiles typically don't pierce
                weapon_id,
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
    horror_query: Query<&GlobalTransform, With<crate::horror::Horror>>,
) {
    for (mut velocity, proj_gtransform, mut homing_comp, _lifetime) in projectile_query.iter_mut() {
        if let Some(target_entity) = homing_comp.target_entity {
            if let Ok(target_gtransform) = horror_query.get(target_entity) {
                let proj_pos = proj_gtransform.translation().truncate();
                let target_pos = target_gtransform.translation().truncate();

                let dir_to_target = (target_pos - proj_pos).normalize_or_zero();
                let current_dir = velocity.0.normalize_or_zero();

                if dir_to_target == Vec2::ZERO || current_dir == Vec2::ZERO {
                    continue; // Avoid issues with zero vectors
                }
                
                // Simple linear interpolation for rotation (can be improved with slerp or more advanced turning)
                let angle_to_target = current_dir.angle_between(dir_to_target);
                let rotation_step = homing_comp.strength * time.delta_seconds(); // Max rotation in radians this frame
                
                let rotation_this_frame = angle_to_target.clamp(-rotation_step, rotation_step);
                
                let new_dir = Quat::from_rotation_z(rotation_this_frame) * current_dir.extend(0.0);
                velocity.0 = new_dir.truncate().normalize_or_zero() * velocity.0.length();

            } else {
                // Target lost
                homing_comp.target_entity = None;
            }
        }
        // If no target, continues straight (no re-acquisition logic for now)
    }
}

// --- Expanding Energy Bomb Systems ---

pub fn spawn_expanding_energy_bomb_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_entity: Entity,
    _player_transform: &Transform, // Player's world transform, used if bomb isn't a child
    params: &crate::items::ExpandingEnergyBombParams,
) {
    let initial_radius = 10.0; // Define a small initial radius for the bomb visual
    let sprite_path = params.visual_sprite_path.unwrap_or("sprites/spirit_bomb_effect_placeholder.png");

    let bomb_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(sprite_path),
            sprite: Sprite {
                color: params.bomb_color,
                custom_size: Some(Vec2::splat(initial_radius * 2.0)), // Initial size
                ..default()
            },
            // Spawned with local transform if it's a child. World transform if not.
            transform: Transform::from_translation(Vec3::ZERO), 
            ..default()
        },
        ExpandingEnergyBombComponent {
            params: params.clone(),
            current_radius: initial_radius,
            expansion_timer: Timer::from_seconds(params.expansion_duration_secs, TimerMode::Once),
            // Initialize wait_timer paused; it starts when max radius is reached.
            wait_at_max_radius_timer: Timer::from_seconds(params.auto_detonation_delay_after_max_expansion_secs, TimerMode::Paused),
            state: SpiritBombState::Expanding,
        },
        Name::new("SpiritBombField"),
    )).id();

    // Make the bomb a child of the player so it follows the player
    commands.entity(player_entity).add_child(bomb_entity);
}

fn detonate_spirit_bomb(
    commands: &mut Commands,
    bomb_entity: Entity,
    bomb_comp: &ExpandingEnergyBombComponent,
    bomb_world_transform: &GlobalTransform, // Use GlobalTransform for world position
    horror_query: &mut Query<(Entity, &GlobalTransform, &mut crate::components::Health), With<crate::horror::Horror>>,
    asset_server: &Res<AssetServer>, // For damage text
    time: &Res<Time>,                // For damage text
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
            // Optional: Play detonation hit sound
        }
    }
    
    // Despawn the bomb entity itself after detonation effects
    commands.entity(bomb_entity).despawn_recursive(); 
}

pub fn expanding_energy_bomb_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, // For detonate_spirit_bomb
    mut bomb_query: Query<(Entity, &mut ExpandingEnergyBombComponent, &mut Transform, &mut Sprite, &GlobalTransform, Option<&Parent>)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut crate::components::Health), With<crate::horror::Horror>>,
    player_query: Query<(Entity, &Transform), With<crate::survivor::Survivor>>, // To get player's current transform if bomb is parented
    // Input handling for manual detonation is simplified to auto-detonation for now.
    // player_input: Res<ButtonInput<MouseButton>>, 
) {
    for (bomb_entity, mut bomb_comp, mut bomb_transform, mut bomb_sprite, bomb_gtransform, opt_parent) in bomb_query.iter_mut() {
        
        // Ensure bomb follows player if it's supposed to be parented (visual update)
        if bomb_comp.state == SpiritBombState::Expanding {
            if let Some(parent) = opt_parent {
                if let Ok((_player_entity, _player_w_transform)) = player_query.get(parent.get()) {
                    // If it's a child, its Transform is local. Ensure it's at player's center.
                    bomb_transform.translation = Vec3::ZERO; 
                }
            }
        }


        match bomb_comp.state {
            SpiritBombState::Expanding => {
                bomb_comp.expansion_timer.tick(time.delta());
                let progress = bomb_comp.expansion_timer.fraction();
                let initial_small_radius = 10.0; // Must match the initial_radius used in spawn function
                
                bomb_comp.current_radius = initial_small_radius + (bomb_comp.params.max_radius - initial_small_radius) * progress;
                
                // Assuming the sprite's native size is small (e.g., 1x1 or similar to initial_small_radius * 2)
                // For simplicity, let's say the placeholder sprite is designed to be 1 unit in size.
                // Then scale directly maps to diameter.
                // If sprite is e.g. 100x100, then scale by current_radius / 50.0
                // For a 1x1 sprite, scale by current_radius * 2.0
                // For now, let's use custom_size for clarity, assuming it's more direct.
                bomb_sprite.custom_size = Some(Vec2::splat(bomb_comp.current_radius * 2.0));

                if bomb_comp.expansion_timer.finished() {
                    bomb_comp.state = SpiritBombState::WaitingAtMaxRadius;
                    bomb_comp.current_radius = bomb_comp.params.max_radius; // Ensure it's exactly max
                    bomb_sprite.custom_size = Some(Vec2::splat(bomb_comp.current_radius * 2.0)); // Update visual to exact max
                    bomb_comp.wait_at_max_radius_timer.unpause(); // Start the wait timer
                }
            }
            SpiritBombState::WaitingAtMaxRadius => {
                bomb_comp.wait_at_max_radius_timer.tick(time.delta());
                let mut detonate_now = false;

                // Simplified: Auto-detonation based on timer. Manual detonation input is ignored for now.
                // if bomb_comp.params.detonation_can_be_manual && player_input.just_pressed(MouseButton::Left) {
                //     detonate_now = true;
                // }

                if bomb_comp.wait_at_max_radius_timer.finished() {
                    detonate_now = true;
                }

                if detonate_now {
                    // Determine the correct GlobalTransform to pass for detonation center
                    let detonation_center_gtransform = if let Some(parent) = opt_parent {
                        if let Ok((_player_entity, player_w_transform)) = player_query.get(parent.get()) {
                             // If parented, the bomb's GlobalTransform is already correct.
                             // However, if we want detonation centered on player at moment of detonation:
                             GlobalTransform::from(*player_w_transform) 
                        } else {
                            // Parent might have been despawned, use bomb's last known world pos
                            *bomb_gtransform 
                        }
                    } else {
                        // Not parented, use its own world transform
                        *bomb_gtransform 
                    };

                    detonate_spirit_bomb(&mut commands, bomb_entity, &bomb_comp, &detonation_center_gtransform, &mut horror_query, &asset_server, &time);
                    bomb_comp.state = SpiritBombState::Detonated; // Entity will be despawned by detonate_spirit_bomb
                }
            }
            SpiritBombState::Detonated => {
                // Entity should have been despawned by detonate_spirit_bomb.
                // If not, or if there are lingering effects to manage, do it here.
                // For now, assuming despawn happens in detonate_spirit_bomb.
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
            transform: Transform::from_translation(player_transform.translation), // Spawn at player's location
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
    horror_query: Query<(Entity, &GlobalTransform), With<crate::horror::Horror>>,
    // Assuming these components are defined in crate::components or crate::horror
    // If not, these lines would cause a compile error until they are defined.
    // For this subtask, we are just inserting them.
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
                // Horror is inside the cloud radius, apply debuff
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
                                tick_interval: 0.5, // Default tick interval
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
    asset_server: Res<AssetServer>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    player_query: Query<(Entity, &Transform, &crate::survivor::Survivor, Option<&PlayerPersistentAuraComponent>)>,
    // Using a more generic query for horrors for damage application
    mut horror_query: Query<(&GlobalTransform, &mut crate::components::Health), With<crate::horror::Horror>>,
    // Query for existing visual to potentially update it, though we despawn/respawn for simplicity now
    // aura_visual_query: Query<&mut Transform, With<Name>>, // Name can be too generic.
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
            // Aura should be active, but component doesn't exist: Add it
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
                        // Visual is child of player, transform is relative to player
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
            // Aura should be active and component exists: Update and do damage
            // First, check if it's for the same weapon or if params changed significantly
            if active_aura.weapon_id != weapon_id || active_aura.radius != params.radius {
                // Weapon changed or radius changed: Despawn old visual, create new one, update component
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
                // Update existing component instead of removing and re-adding
                commands.entity(player_entity).insert(PlayerPersistentAuraComponent {
                    damage_per_tick: params.damage_per_tick,
                    tick_timer: Timer::from_seconds(params.tick_interval_secs, TimerMode::Repeating), // Reset timer
                    radius: params.radius,
                    aura_color: params.aura_color,
                    visual_entity: new_visual_entity_opt,
                    weapon_id: weapon_id,
                });

            } else {
                 // Same weapon, same params, proceed with damage tick
                let mut mutable_aura_comp = commands.entity(player_entity).get_mut::<PlayerPersistentAuraComponent>().unwrap(); // Re-borrow to satisfy borrow checker
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
                            // Optional: Sound effect for aura damage
                        }
                    }
                }
            }
        }
        (false, Some(active_aura), _, _) => {
            // Aura should NOT be active, but component exists: Remove it and its visual
            if let Some(visual_entity) = active_aura.visual_entity {
                commands.entity(visual_entity).despawn_recursive();
            }
            commands.entity(player_entity).remove::<PlayerPersistentAuraComponent>();
        }
        _ => {
            // Other cases: (false, None, _, _) -> do nothing
            // (true, Some, None, _) -> should not happen if weapon_def is always found for active_weapon_id
        }
    }
}

// --- Point-Blank Nova Systems ---

pub fn spawn_point_blank_nova_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    player_transform: &Transform,
    params: &crate::items::PointBlankNovaParams,
    // Query for Horrors to apply damage and slow effect. Note: Velocity might not be directly on Horror,
    // but on a child or needs to be handled by the Frozen system.
    // For now, we query for Health and Entity to insert Frozen component.
    all_horrors_query: &mut Query<(Entity, &GlobalTransform, &mut crate::components::Health), With<crate::horror::Horror>>,
    time: &Res<Time>, // For damage text
    sound_event_writer: &mut EventWriter<crate::audio::PlaySoundEvent>, // For sound effects
) {
    let player_pos = player_transform.translation;

    // Spawn Visual Effect
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/nova_effect_placeholder.png"), // Placeholder sprite
            sprite: Sprite {
                color: params.nova_color,
                custom_size: Some(Vec2::splat(10.0)), // Initial small size
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

    // Apply Damage and Slow to Horrors
    for (horror_entity, horror_gtransform, mut horror_health) in all_horrors_query.iter_mut() {
        let horror_pos = horror_gtransform.translation();
        let distance_sq = player_pos.distance_squared(horror_pos);

        if distance_sq < params.radius.powi(2) {
            // Apply Damage
            horror_health.0 -= params.damage;
            crate::visual_effects::spawn_damage_text(commands, asset_server, horror_pos, params.damage, time);
            sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::GlacialNovaHit));


            // Apply Slow Effect (using existing Frozen component)
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
    mut query: Query<(Entity, &mut NovaVisualComponent, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut visual, mut sprite, mut _transform) in query.iter_mut() {
        visual.duration_timer.tick(time.delta());

        if visual.duration_timer.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            let progress = visual.duration_timer.fraction();
            sprite.custom_size = Some(Vec2::splat(
                visual.initial_radius + (visual.max_radius - visual.initial_radius) * progress,
            ));
            sprite.color.set_a(visual.color.a() * (1.0 - progress)); 
            // Optional: Could scale transform instead of custom_size if preferred,
            // but custom_size is direct for non-uniform scaling if needed later.
        }
    }
}

// --- Chain Lightning Systems ---

// Simplified visual: spawn a small sprite at the target's location
pub fn spawn_zap_visual(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    target_pos: Vec3,
    params: &crate::items::ChainZapParams,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/zap_placeholder.png"), // Small circle or spark sprite
            sprite: Sprite {
                custom_size: Some(Vec2::new(params.zap_width * 4.0, params.zap_width * 4.0)), // Make it a bit larger than line width
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
    // It's often better to pass QueryState or individual query results if this is called from a system
    // that itself is iterating over many things, to avoid conflicting queries.
    // However, for a direct call from a weapon firing event, this direct query can be okay.
    horrors_query: &mut Query<(Entity, &GlobalTransform, &mut crate::components::Health), With<crate::horror::Horror>>,
    params: &crate::items::ChainZapParams,
    // Optional: for damage text and sound
    time: &Res<Time>, 
    sound_event_writer: &mut EventWriter<crate::audio::PlaySoundEvent>,
) {
    let mut already_zapped_entities: Vec<Entity> = Vec::new();
    let mut current_damage = params.base_damage_per_zap as f32;
    let mut last_zap_end_position = player_transform.translation; // For the first zap, it originates from the player
    let mut current_target_opt: Option<(Entity, Vec3)> = None; // (Entity, Position)

    // Find initial target
    let mut closest_initial_target: Option<(Entity, f32, Vec3)> = None; // (Entity, distance_sq, position)
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
        return; // No initial target found
    }

    for _chain_idx in 0..params.max_chains {
        if let Some((target_entity, target_pos)) = current_target_opt {
            // Apply damage to the current target
            if let Ok((_, _, mut health)) = horrors_query.get_mut(target_entity) {
                let damage_to_apply = current_damage.round() as i32;
                health.0 -= damage_to_apply;
                
                // Spawn damage text
                crate::visual_effects::spawn_damage_text(commands, asset_server, target_pos, damage_to_apply, time);
                // Play sound
                sound_event_writer.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::ChainLightningZap));

            } else { // Target might have been destroyed by another effect simultaneously
                current_target_opt = None; // Stop chaining
                break;
            }

            // Spawn visual for this zap
            spawn_zap_visual(commands, asset_server, target_pos, params);
            already_zapped_entities.push(target_entity);
            
            // Prepare for next chain
            last_zap_end_position = target_pos;
            current_damage *= params.damage_falloff_per_chain;
            current_target_opt = None; // Reset for next search

            // Find next target
            let mut closest_next_target: Option<(Entity, f32, Vec3)> = None;
            for (horror_entity, horror_gtransform, _health) in horrors_query.iter_mut() {
                if already_zapped_entities.contains(&horror_entity) {
                    continue; // Don't zap the same target twice in one chain
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
                break; // No next target found
            }
        } else {
            break; // No current target to process
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
    weapon_id: AutomaticWeaponId, // Added weapon_id
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
            piercing_left: 0, // Or a low value like 1 if it should hit one thing before dissipating
            weapon_id: weapon_id, // Pass the weapon_id
        },
        Name::new("InfernoBoltProjectile"),
    ));
}

pub fn trail_spawning_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>, // Needed for spawn_fire_trail_segment
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
            // Placeholder sprite for fire segment
            texture: asset_server.load("sprites/fire_segment_placeholder.png"), // Ensure this asset exists
            sprite: Sprite {
                custom_size: Some(Vec2::new(trail_params.trail_segment_width, trail_params.trail_segment_width)), // Square segment for now
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
    mut horror_query: Query<(Entity, &Transform, &mut crate::components::Health), With<crate::horror::Horror>>,
    // Optional: asset_server for damage text, sound_event_writer
    // asset_server: Res<AssetServer>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
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
            let segment_radius_sq = (segment.width / 2.0).powi(2); // Simple circular collision

            for (horror_entity, horror_transform, mut horror_health) in horror_query.iter_mut() {
                if segment.already_hit_this_tick.contains(&horror_entity) {
                    continue;
                }

                let horror_pos = horror_transform.translation.truncate();
                if horror_pos.distance_squared(segment_pos) < segment_radius_sq {
                    horror_health.0 -= segment.damage_per_tick;
                    segment.already_hit_this_tick.push(horror_entity);
                    
                    // Optional: Spawn damage text
                    // spawn_damage_text(&mut commands, &asset_server, horror_transform.translation, segment.damage_per_tick, &time);
                    // Optional: Play sound effect
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::FireTick));
                }
            }
        }
    }
}

// --- Charge-Up Energy Shot Systems ---

pub fn spawn_charge_shot_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    charge_params: &crate::items::ChargeUpEnergyShotParams, // Overall charge weapon params
    chosen_level_params: &crate::items::ChargeLevelParams, // Params for the specific charge level achieved
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
                color: charge_params.base_projectile_color, // Use base color for all charge levels for now
                ..default()
            },
            transform: Transform::from_translation(player_transform.translation) // Spawn at player center
                .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
            ..default()
        },
        Velocity(aim_direction.normalize_or_zero() * chosen_level_params.projectile_speed),
        Damage(chosen_level_params.projectile_damage),
        Lifetime { timer: Timer::from_seconds(charge_params.projectile_lifetime_secs, TimerMode::Once) },
        crate::automatic_projectiles::AutomaticProjectile { // Assuming this component exists and is used for collision logic
            piercing_left: chosen_level_params.piercing,
            already_hit_entities: Vec::new(), // Initialize empty
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
    mut query: Query<&mut ChargingWeaponComponent>, // Query for entities with ChargingWeaponComponent
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>, // To get weapon definitions
    // Player query to simulate input - this will be replaced by actual input later
    // For now, we assume if ChargingWeaponComponent exists, it's charging.
    // And if player releases fire, is_actively_charging is set to false elsewhere.
) {
    for mut charging_comp in query.iter_mut() {
        if !charging_comp.is_actively_charging {
            continue; // Not actively charging, so don't update charge level.
        }

        // Fetch the weapon's charge parameters
        if let Some(weapon_def) = weapon_library.get_weapon_definition(charging_comp.weapon_id) {
            if let crate::items::AttackTypeData::ChargeUpEnergyShot(charge_params) = &weapon_def.attack_data {
                
                charging_comp.charge_timer.tick(time.delta());

                // Determine current charge level based on timer
                let mut new_charge_level_index = 0; // Default to lowest (or tap-fire)
                for (idx, level_params) in charge_params.charge_levels.iter().enumerate() {
                    if charging_comp.charge_timer.elapsed_secs() >= level_params.charge_time_secs {
                        new_charge_level_index = idx;
                    } else {
                        // Found the first level not yet reached
                        break; 
                    }
                }
                charging_comp.current_charge_level_index = new_charge_level_index;
                
                // info!("Weapon {:?} charging. Time: {:.2}s, Level Index: {}", 
                //       charging_comp.weapon_id, 
                //       charging_comp.charge_timer.elapsed_secs(), 
                //       charging_comp.current_charge_level_index);

                // Visual feedback could be triggered here based on current_charge_level_index
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
            pool_params: params.clone(), // Clone the params for the pool
            initial_spawn_position: initial_pos,
            // impact_damage field removed
        },
        Velocity(projectile_velocity),
        Damage(params.base_damage_on_impact), // Direct impact damage
        // Adjusted lifetime to a fixed 2.0s for flight, or calculate based on a conceptual range.
        // For example, if range is 300 and speed is 150, lifetime is 2.0s.
        // Using a placeholder fixed lifetime for now.
        Lifetime { timer: Timer::from_seconds(2.0, TimerMode::Once) }, 
        Name::new("LobbedIchorProjectile"),
    ));
}

pub fn lobbed_projectile_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>, // Needed for spawn_ichor_pool
    mut active_pools_res: ResMut<ActiveIchorPools>, // Needed for spawn_ichor_pool
    time: Res<Time>,
    // Query for projectile: Entity, its specific component, Transform, and Lifetime.
    // Velocity is not directly needed here if movement is handled by a generic system.
    mut projectile_query: Query<(Entity, &LobbedProjectileComponent, &Transform, &mut Lifetime)>,
) {
    for (entity, lobbed_comp, transform, mut lifetime) in projectile_query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.just_finished() {
            // Projectile's lifetime has expired, it "lands".
            spawn_ichor_pool(
                &mut commands,
                &asset_server,
                transform.translation, // Land at current projectile position
                &lobbed_comp.pool_params, // Pass the pool parameters
                &mut active_pools_res,
            );
            commands.entity(entity).despawn_recursive(); // Despawn the projectile
        }
        // Note: Actual arcing movement logic would go here if not using a generic movement system
        // or if more sophisticated landing (e.g., ground collision) was implemented.
        // For now, simple lifetime-based landing is used.
    }
}

pub fn spawn_ichor_pool(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>, 
    position: Vec3,
    pool_params: &crate::items::LobbedAoEPoolParams,
    active_pools_res: &mut ResMut<ActiveIchorPools>,
) {
    // Manage max active pools: If current count is at max, despawn the oldest.
    if active_pools_res.pools.len() >= pool_params.max_active_pools as usize {
        if let Some(oldest_pool_entity) = active_pools_res.pools.pop_front() {
            commands.entity(oldest_pool_entity).despawn_recursive();
        }
    }

    let pool_entity = commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/ichor_pool_placeholder.png"), // Placeholder sprite
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

    active_pools_res.pools.push_back(pool_entity); // Add to the end of the VecDeque
}

pub fn ichor_pool_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pool_query: Query<(Entity, &mut IchorPoolComponent, &GlobalTransform)>,
    // Query for damagable entities (e.g., Horrors)
    mut horror_query: Query<(&Transform, &mut crate::components::Health), With<crate::horror::Horror>>,
    mut active_pools_res: ResMut<ActiveIchorPools>,
    // Optional: asset_server for damage text, sound_event_writer
    // asset_server: Res<AssetServer>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (pool_entity, mut pool_comp, pool_gtransform) in pool_query.iter_mut() {
        // Tick pool duration timer
        pool_comp.duration_timer.tick(time.delta());
        if pool_comp.duration_timer.finished() {
            commands.entity(pool_entity).despawn_recursive();
            // Remove from active_pools_res by finding and removing the specific entity
            active_pools_res.pools.retain(|&e| e != pool_entity);
            continue; // Skip further processing for this despawned pool
        }

        // Tick damage application timer
        pool_comp.tick_timer.tick(time.delta());
        if pool_comp.tick_timer.just_finished() {
            let pool_center_pos = pool_gtransform.translation().truncate();
            let pool_radius_sq = pool_comp.radius * pool_comp.radius;

            for (horror_transform, mut horror_health) in horror_query.iter_mut() {
                let horror_pos = horror_transform.translation.truncate();
                if horror_pos.distance_squared(pool_center_pos) < pool_radius_sq {
                    // Horror is inside the pool radius
                    horror_health.0 -= pool_comp.damage_per_tick;
                    // info!("Ichor pool damaged Horror {:?}, new health: {}", horror_entity, horror_health.0); // Assuming horror_entity is available

                    // Optional: Spawn damage text
                    // spawn_damage_text(&mut commands, &asset_server, horror_transform.translation, pool_comp.damage_per_tick, &time);
                    
                    // Optional: Play sound effect
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                }
            }
        }
    }
}

pub fn channeled_beam_update_system(
    player_query: Query<(&Transform, &crate::survivor::Survivor), (With<crate::survivor::Survivor>, Without<ChanneledBeamComponent>)>, // Player's transform and aim. Changed to survivor
    mut beam_query: Query<(&mut Transform, &ChanneledBeamComponent)>, // Beam's transform
) {
    for (mut beam_transform, beam_comp) in beam_query.iter_mut() {
        if let Ok((player_transform, player_stats)) = player_query.get(beam_comp.owner) {
            // Align beam position with player position (or an offset)
            beam_transform.translation = player_transform.translation; // Or player_transform.translation + offset;
            
            // Align beam rotation with player's aim_direction
            let aim_direction = player_stats.aim_direction;
            if aim_direction != Vec2::ZERO {
                beam_transform.rotation = Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x));
            }
        } else {
            // Owner player not found, perhaps despawn beam? Or handled by lifetime.
            // For now, do nothing.
        }
    }
}

pub fn returning_projectile_system(
    mut commands: Commands,
    _time: Res<Time>, // _time is kept as per signature, but not used with Velocity based movement
    mut query: Query<(Entity, &mut ReturningProjectileComponent, &mut Velocity, &Transform)>,
    // Optional: player_query: Query<&Transform, (With<crate::player::Player>, Without<ReturningProjectileComponent>)>, // Not needed for Option 1
) {
    for (entity, mut projectile_comp, mut velocity, transform) in query.iter_mut() {
        match projectile_comp.state {
            ReturningProjectileState::Outgoing => {
                let distance_traveled = transform.translation.distance(projectile_comp.start_position);
                if distance_traveled >= projectile_comp.max_travel_distance {
                    projectile_comp.state = ReturningProjectileState::Returning;
                    
                    // Option 1: Return towards start_position
                    let direction_to_start = (projectile_comp.start_position - transform.translation).truncate().normalize_or_zero();
                    velocity.0 = direction_to_start * projectile_comp.speed;
                }
            }
            ReturningProjectileState::Returning => {
                // Check if projectile is close to its original start position
                let distance_to_target = transform.translation.distance(projectile_comp.start_position);
                
                // Despawn if very close to the return target.
                if distance_to_target < 5.0 { 
                    commands.entity(entity).despawn_recursive();
                }
                // Lifetime component will handle it if it never reaches the target.
            }
        }
    }
}

pub fn spawn_standard_projectile_attack(
    _commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    params: &StandardProjectileParams, // params is used
    _player_transform: &Transform,
    _aim_direction: Vec2,
    _weapon_id: AutomaticWeaponId // _weapon_id is already correctly prefixed
) {
    info!("spawn_standard_projectile_attack called for sprite: {}, damage: {}, fire_rate: {}", params.projectile_sprite_path, params.base_damage, params.base_fire_rate_secs);
    // This function will eventually contain logic similar to the old `spawn_automatic_projectile`
    // but using the fields from `params`. For now, it just logs.
    // Example of spawning a placeholder:
    /*
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::RED, custom_size: Some(Vec2::new(10.0, 10.0)), ..default()},
            transform: Transform::from_translation(player_transform.translation + aim_direction.extend(0.0) * 50.0), // Offset slightly
            ..default()
        },
        Name::new("StandardProjectilePlaceholder"),
    ));
    */
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
    mut player_query: Query<(Entity, &mut crate::survivor::Survivor, Option<&IsChannelingComponent>)>, // Changed to survivor
    // To despawn beam if IsChannelingComponent is removed for some reason by other logic
    // mut commands: Commands, 
) {
    for (player_entity, mut player_stats, opt_is_channeling_comp) in player_query.iter_mut() {
        if let Some(is_channeling_comp) = opt_is_channeling_comp {
            // Player IS channeling
            let target_speed = BASE_PLAYER_SPEED * is_channeling_comp.beam_params.movement_penalty_multiplier;
            if player_stats.speed != target_speed {
                // Store original speed if not already stored elsewhere, or just set.
                // For simplicity, we just set. Reverting relies on this system running when component is removed.
                player_stats.speed = target_speed; 
                info!("Player {:?} speed set to {} due to channeling.", player_entity, target_speed);
            }
        } else {
            // Player is NOT channeling (or component was just removed)
            if player_stats.speed != BASE_PLAYER_SPEED { // Changed from BASE_SURVIVOR_SPEED
                player_stats.speed = BASE_PLAYER_SPEED; // Changed from BASE_SURVIVOR_SPEED
                info!("Player {:?} speed reset to {}.", player_entity, BASE_PLAYER_SPEED); // Changed from BASE_SURVIVOR_SPEED
            }
        }
    }
}

pub fn execute_cone_attack(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>, // For visual effects
    params: &ConeAttackParams,
    player_transform: &Transform,
    aim_direction: Vec2, // Normalized direction player is aiming
    // Query for entities that can be damaged
    mut enemy_query: Query<(Entity, &Transform, &mut crate::components::Health), With<crate::horror::Horror>>, // Changed to horror
    // Optional: For spawning visual effects or sounds
    // time: Res<Time>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    // sound_event_writer.send(PlaySoundEvent(SoundEffect::ConeAttackSound)); // Placeholder for sound

    let player_pos = player_transform.translation.truncate();
    let forward_vector = aim_direction.normalize_or_zero(); // Ensure it's normalized

    // Optional: Spawn a visual representation of the cone attack
    // This could be a sprite that quickly appears and disappears.
    // For simplicity in this step, we'll focus on the damage logic.
    // Example:
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load("sprites/cone_attack_effect.png"), // Placeholder sprite
    //     sprite: Sprite { color: params.color, custom_size: Some(Vec2::new(params.cone_radius, params.cone_radius * params.cone_angle_degrees.to_radians())), ..default()},
    //     transform: Transform::from_translation(player_transform.translation)
    //         .with_rotation(Quat::from_rotation_z(aim_direction.y.atan2(aim_direction.x))),
    // ..default()
    // }).insert(Lifetime { timer: Timer::from_seconds(0.2, TimerMode::Once) });


    for (enemy_entity, enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        let vector_to_enemy = enemy_pos - player_pos;
        
        // Check distance
        let distance_to_enemy_sq = vector_to_enemy.length_squared();
        if distance_to_enemy_sq > params.cone_radius * params.cone_radius {
            continue; // Enemy is outside the radius
        }

        // Check angle
        if vector_to_enemy != Vec2::ZERO { // Avoid issues with dot product if enemy is at player's exact position
            let angle_to_enemy_rad = forward_vector.angle_between(vector_to_enemy.normalize_or_zero());
            let half_cone_angle_rad = params.cone_angle_degrees.to_radians() / 2.0;

            if angle_to_enemy_rad.abs() <= half_cone_angle_rad {
                // Enemy is within the cone
                enemy_health.0 -= params.base_damage;
                // info!("Cone attack hit enemy {:?}, health: {}", enemy_entity, enemy_health.0);

                // Spawn damage text, play hit sound, etc.
                // if enemy_health.0 <= 0 { commands.entity(enemy_entity).despawn_recursive(); }
            }
        }
    }
}