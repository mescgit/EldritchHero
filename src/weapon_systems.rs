use bevy::prelude::*;
use crate::items::{StandardProjectileParams, ReturningProjectileParams, ChanneledBeamParams, ConeAttackParams, AutomaticWeaponId};
use crate::components::{Velocity, Damage, Lifetime, Name}; // Added Name
use crate::player::{Player, BASE_PLAYER_SPEED}; // Added BASE_PLAYER_SPEED

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

#[derive(Component, Debug, Reflect, Default)]
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
            .add_systems(Update, (
                returning_projectile_system,
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
    mut enemy_query: Query<(Entity, &Transform, &mut crate::components::Health, With<crate::enemy::Enemy>)>,
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
        for (_enemy_entity, enemy_transform, mut enemy_health, _with_enemy) in enemy_query.iter_mut() {
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
                let enemy_radius = 16.0; // Default value as per instructions.
                if perpendicular_distance < (beam_comp.width / 2.0) + enemy_radius { 
                    // Enemy is hit
                    enemy_health.0 -= beam_comp.damage_per_tick;
                    // info!("Beam hit enemy {:?}, health: {}", _enemy_entity, enemy_health.0); // For debugging

                    // Here you could spawn damage text or play hit sounds
                    // spawn_damage_text(&mut commands, &asset_server, enemy_transform.translation, beam_comp.damage_per_tick, &time);
                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyHit));

                    if enemy_health.0 <= 0 {
                        // Potentially handle enemy death here or let another system do it
                        // commands.entity(enemy_entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

pub fn channeled_beam_update_system(
    player_query: Query<(&Transform, &crate::player::Player), (With<crate::player::Player>, Without<ChanneledBeamComponent>)>, // Player's transform and aim
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
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    params: &StandardProjectileParams,
    player_transform: &Transform,
    aim_direction: Vec2,
    _weapon_id: AutomaticWeaponId // Kept weapon_id as per comment in prompt, though unused for now
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

pub fn start_channeled_beam_attack(
// Removed start_channeled_beam_attack function


pub fn player_is_channeling_effect_system(
    mut player_query: Query<(Entity, &mut crate::player::Player, Option<&IsChannelingComponent>)>,
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
            if player_stats.speed != BASE_PLAYER_SPEED {
                player_stats.speed = BASE_PLAYER_SPEED;
                info!("Player {:?} speed reset to {}.", player_entity, BASE_PLAYER_SPEED);
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
    mut enemy_query: Query<(Entity, &Transform, &mut crate::components::Health, With<crate::enemy::Enemy>)>,
    // Optional: For spawning visual effects or sounds
    // time: Res<Time>,
    // mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    // sound_event_writer.send(PlaySoundEvent(SoundEffect::ConeAttackSound)); // Placeholder for sound

    let player_pos = player_transform.translation.truncate();
    let forward_vector = aim_direction.normalize_or_dead(); // Ensure it's normalized

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


    for (enemy_entity, enemy_transform, mut enemy_health, _with_enemy) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();
        let vector_to_enemy = enemy_pos - player_pos;
        
        // Check distance
        let distance_to_enemy_sq = vector_to_enemy.length_squared();
        if distance_to_enemy_sq > params.cone_radius * params.cone_radius {
            continue; // Enemy is outside the radius
        }

        // Check angle
        if vector_to_enemy != Vec2::ZERO { // Avoid issues with dot product if enemy is at player's exact position
            let angle_to_enemy_rad = forward_vector.angle_between(vector_to_enemy.normalize_or_dead());
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
