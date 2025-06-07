// src/automatic_projectiles.rs
use bevy::prelude::*;
use bevy::prelude::in_state; 
// use rand::Rng; // Unused
use crate::{
    components::{Velocity, Damage, Lifetime, Health, HorrorLatchedByTetherComponent},
    visual_effects, 
    audio::{PlaySoundEvent, SoundEffect},
    horror::Horror,
    survivor::Survivor,
    items::{ItemLibrary, /*ItemEffect, ExplosionEffect,*/ AutomaticWeaponId}, // ItemEffect, ExplosionEffect unused
    game::AppState,
};
use crate::camera_systems::MainCamera;
use bevy::render::camera::OrthographicProjection;

// Define the CollisionAction struct here
#[derive(Debug, Clone)] // AutomaticProjectile will also need Clone
struct CollisionAction {
    projectile_entity: Entity,
    horror_entity: Entity,
    horror_gtransform: GlobalTransform,
    horror_local_transform: Transform,
    damage_to_apply: i32,
    original_projectile_stats: AutomaticProjectile, 
    projectile_explodes_params: Option<crate::weapon_systems::ExplodesOnFinalImpact>, 
    projectile_debuff_params: Option<crate::weapon_systems::DebuffOnHitComponent>,   
    projectile_tether_params: Option<crate::items::RepositioningTetherParams>,     
    horror_health_at_collision: i32,
}

// Make sure AutomaticProjectile can be cloned for storing in CollisionAction
#[derive(Component, Reflect, Debug, Clone)] // Added Clone here
#[reflect(Component, Default)]
pub struct AutomaticProjectile {
    pub owner: Option<Entity>, // Changed to Option<Entity>
    pub piercing_left: u32,
    pub weapon_id: AutomaticWeaponId,
    pub bounces_left: Option<u32>,
    pub damage_on_hit: i32,
    pub current_speed: f32,
    pub damage_loss_per_bounce_multiplier: Option<f32>,
    pub speed_loss_per_bounce_multiplier: Option<f32>,
    pub has_bounced_this_frame: bool,
    pub lifesteal_percentage: Option<f32>,
    pub blink_params_on_hit: Option<crate::items::BlinkStrikeProjectileParams>,
}

// Default implementation for the AutomaticProjectile struct (the one with Clone)
impl Default for AutomaticProjectile {
    fn default() -> Self {
        Self {
            owner: None, // Default to None
            piercing_left: 0,
            weapon_id: AutomaticWeaponId::default(),
            bounces_left: None,
            damage_on_hit: 0,
            current_speed: 0.0,
            damage_loss_per_bounce_multiplier: None,
            speed_loss_per_bounce_multiplier: None,
            has_bounced_this_frame: false,
            lifesteal_percentage: None,
            blink_params_on_hit: None,
        }
    }
}


pub struct AutomaticProjectilesPlugin;

impl Plugin for AutomaticProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Make sure we register the AutomaticProjectile that has Clone
            .register_type::<AutomaticProjectile>()
            .add_systems(Update, (
                projectile_movement_system,
                automatic_projectile_collision_system,
                projectile_screen_bounce_system.after(automatic_projectile_collision_system),
                automatic_projectile_lifetime_system,
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(PostUpdate, reset_projectile_bounce_flag_system.run_if(in_state(AppState::InGame)));
    }
}

// The redundant definition of AutomaticProjectile (without Clone) and its Default impl were here.
// They are removed by this diff. The Default impl above now correctly targets the struct with Clone.

fn handle_bounce_stat_updates(
    stats: &mut AutomaticProjectile,
    damage_comp: &mut Damage,
    velocity_comp: &mut Velocity,
) {
    if let Some(bounces) = stats.bounces_left.as_mut() {
        if *bounces > 0 {
            *bounces -= 1;

            if let Some(dmg_loss_mult) = stats.damage_loss_per_bounce_multiplier {
                stats.damage_on_hit = (stats.damage_on_hit as f32 * dmg_loss_mult).round() as i32;
                damage_comp.0 = stats.damage_on_hit;
            }

            if let Some(speed_loss_mult) = stats.speed_loss_per_bounce_multiplier {
                stats.current_speed *= speed_loss_mult;
                velocity_comp.0 = velocity_comp.0.normalize_or_zero() * stats.current_speed;
            }

            stats.has_bounced_this_frame = true;
        }
    }
}


pub fn spawn_automatic_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    owner: Entity,
    position: Vec3,
    direction: Vec2,
    initial_damage: i32,
    initial_speed: f32,
    piercing: u32,
    weapon_id: AutomaticWeaponId,
    sprite_path: &str, 
    size: Vec2,
    color: Color,
    lifetime_secs: f32,
    opt_max_bounces: Option<u32>,
    opt_dmg_loss_mult: Option<f32>,
    opt_speed_loss_mult: Option<f32>,
    opt_lifesteal_percentage: Option<f32>,
    opt_tether_params_for_comp: Option<crate::items::RepositioningTetherParams>,
    opt_blink_params: Option<crate::items::BlinkStrikeProjectileParams>,
    opt_trail_params: Option<crate::items::TrailOfFireParams>,
) {
    let normalized_direction = direction.normalize_or_zero();
    
    let mut spawn_pos = position;
    spawn_pos.z += 0.1; // Increment Z

    let mut projectile_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(sprite_path.to_string()),
            sprite: Sprite {
                custom_size: Some(size),
                color,
                ..default()
            },
            transform: Transform::from_translation(spawn_pos) // Use modified spawn_pos
                .with_rotation(Quat::from_rotation_z(normalized_direction.y.atan2(normalized_direction.x))),
            visibility: Visibility::Visible, // Explicitly set visibility
            ..default()
        },
        AutomaticProjectile {
            owner: Some(owner), // Pass as Some(owner) since struct field is Option<Entity>
            piercing_left: piercing,
            weapon_id,
            bounces_left: opt_max_bounces,
            damage_on_hit: initial_damage,
            current_speed: initial_speed,
            damage_loss_per_bounce_multiplier: opt_dmg_loss_mult,
            speed_loss_per_bounce_multiplier: opt_speed_loss_mult,
            has_bounced_this_frame: false,
            lifesteal_percentage: opt_lifesteal_percentage,
            blink_params_on_hit: opt_blink_params.clone(),
        },
        Velocity(normalized_direction * initial_speed),
        Damage(initial_damage),
        Lifetime { timer: Timer::from_seconds(lifetime_secs, TimerMode::Once) },
        Name::new("AutomaticProjectile"),
    ));

    if let Some(tether_params) = opt_tether_params_for_comp {
        projectile_commands.insert(crate::weapon_systems::TetherProjectileComponent {
            params_snapshot: tether_params.clone(),
        });
    }

    if let Some(trail_params) = opt_trail_params {
        projectile_commands.insert(crate::weapon_systems::TrailSpawningProjectileComponent {
            trail_params: trail_params.clone(),
            segment_spawn_timer: Timer::from_seconds(trail_params.trail_segment_spawn_interval_secs.max(0.01), TimerMode::Repeating),
        });
    }

    // New Log
    info!(
        "Spawning projectile -- Path: '{}', Size: {:?}, Color (RGBA): ({},{},{},{}), SpawnXYZ: ({},{},{})",
        sprite_path,
        size,
        color.r(), color.g(), color.b(), color.a(),
        spawn_pos.x, spawn_pos.y, spawn_pos.z
    );
}

fn projectile_movement_system(
    mut query: Query<(&mut Transform, &Velocity, &AutomaticProjectile)>,
    time: Res<Time>,
) {
    for (mut transform, velocity, proj_stats) in query.iter_mut() {
        let speed = proj_stats.current_speed;
        transform.translation.x += velocity.0.normalize_or_zero().x * speed * time.delta_seconds();
        transform.translation.y += velocity.0.normalize_or_zero().y * speed * time.delta_seconds();
    }
}

fn automatic_projectile_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), With<AutomaticProjectile>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn automatic_projectile_collision_system(
    mut commands: Commands,
    mut query_set: ParamSet<(
        Query<( // p0: Projectile Query
            Entity,
            &GlobalTransform,
            &mut Damage,
            &mut AutomaticProjectile,
            &mut Velocity,
            &Sprite,
            &mut Transform, // Projectile Transform (mutable)
            Option<&crate::weapon_systems::ExplodesOnFinalImpact>,
            Option<&crate::weapon_systems::DebuffOnHitComponent>,
            Option<&crate::weapon_systems::TetherProjectileComponent>,
        )>,
        Query<( // p1: Horror Query
            Entity,
            &GlobalTransform,
            &Transform, // Horror Transform (immutable)
            &mut Health,
            &crate::horror::Horror,
            Option<&crate::components::DamageAmpDebuffComponent>,
            Option<&mut HorrorLatchedByTetherComponent>,
        )>,
        Query<(&mut Transform, &mut Health, &Survivor), (With<Survivor>, Without<Horror>, Without<AutomaticProjectile>)>, // p2: Player Effects Query
        Query<(Entity, Option<&mut crate::components::PlayerTetherState>), With<Survivor>> // p3: Player Tether Setup Query
    )>,
    _item_library: Res<ItemLibrary>, // Unused
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    _player_blink_event_writer: EventWriter<crate::components::PlayerBlinkEvent>, // Not mutated
) {
    // Stage 1: Collect relevant information from projectiles and horrors
    let mut projectile_info_list = Vec::new();
    let p0_binding = query_set.p0(); 
    for (
        entity,
        gtransform,
        _damage, 
        stats,
        _velocity,
        sprite,
        _transform, 
        explodes_on_impact_comp,
        debuff_on_hit_comp,
        tether_projectile_comp
    ) in p0_binding.iter() { 
        if stats.has_bounced_this_frame { continue; }
        projectile_info_list.push((
            entity,
            *gtransform,
            stats.clone(), 
            sprite.custom_size,
            // Clone component data if it exists, not the Option<&Component>
            explodes_on_impact_comp.cloned(), 
            debuff_on_hit_comp.cloned(),     
            tether_projectile_comp.map(|t| t.params_snapshot.clone()) 
        ));
    }

    let mut horror_info_list = Vec::new();
    let p1_binding = query_set.p1(); 
    for (
        entity,
        gtransform,
        transform, 
        health,
        horror_stats,
            _damage_amp_debuff_comp, // Renamed, was _damage_amp_debuff
        _latched_by_tether  
    ) in p1_binding.iter() { 
        horror_info_list.push((
            entity,
            *gtransform,
            *transform, 
            health.0,   
            horror_stats.size 
        ));
    }

    // Stage 2: Identify actual collisions and prepare actions
    let mut collision_actions = Vec::new(); 

    for (
        proj_entity,
        proj_gtransform,
        proj_stats, 
        proj_sprite_custom_size,
        proj_opt_explodes_params, 
        proj_opt_debuff_params,   
        proj_opt_tether_params    
    ) in &projectile_info_list {
        
        let mut current_projectile_piercing_left = proj_stats.piercing_left;
        let mut projectile_would_be_consumed_by_hits = false;

        for (
            horror_entity,
            horror_gtransform,
            horror_local_transform, 
            initial_horror_health_value, 
            horror_size
        ) in &horror_info_list {
            if projectile_would_be_consumed_by_hits { break; }

            let distance = proj_gtransform.translation().truncate().distance(horror_gtransform.translation().truncate());
            let projectile_radius = proj_sprite_custom_size.map_or(5.0, |s| s.x.max(s.y) / 2.0);
            let horror_radius = horror_size.x / 2.0;

            if distance < projectile_radius + horror_radius {
                collision_actions.push(CollisionAction { // Corrected path to CollisionAction
                    projectile_entity: *proj_entity,
                    horror_entity: *horror_entity,
                    horror_gtransform: *horror_gtransform,
                    horror_local_transform: *horror_local_transform,
                    damage_to_apply: proj_stats.damage_on_hit,
                    original_projectile_stats: proj_stats.clone(),
                    projectile_explodes_params: proj_opt_explodes_params.clone(),
                    projectile_debuff_params: proj_opt_debuff_params.clone(),
                    projectile_tether_params: proj_opt_tether_params.clone(),
                    horror_health_at_collision: *initial_horror_health_value,
                });

                if proj_opt_tether_params.is_some() {
                    projectile_would_be_consumed_by_hits = true; 
                } else if current_projectile_piercing_left > 0 {
                    current_projectile_piercing_left -= 1;
                } else if !proj_stats.bounces_left.is_some() || proj_stats.bounces_left.unwrap_or(0) == 0 {
                    projectile_would_be_consumed_by_hits = true;
                }
            }
        }
    }
    
    // Stage 3: Apply mutations
    let mut processed_projectiles_this_frame = std::collections::HashSet::new(); // To handle piercing/bounce correctly once per frame

    for action in collision_actions {
        // --- Handle Tether (Exclusive) ---
        if let Some(tether_params_cloned) = action.projectile_tether_params {
            if let Ok((player_entity, mut opt_player_tether_state_comp)) = query_set.p3().get_single_mut() {
                if let Some(player_tether_state_comp) = opt_player_tether_state_comp.as_deref_mut() {
                    if let Some(enemy_in_state) = player_tether_state_comp.tethered_enemy_entity {
                        if commands.get_entity(enemy_in_state).is_some() { commands.entity(enemy_in_state).remove::<HorrorLatchedByTetherComponent>(); }
                    }
                }
                if commands.get_entity(player_entity).is_some() { commands.entity(player_entity).remove::<crate::weapon_systems::PlayerWaitingTetherActivationComponent>(); }
                let next_mode = if tether_params_cloned.mode == crate::items::RepositioningTetherMode::Alternate { crate::items::RepositioningTetherMode::Pull } else { tether_params_cloned.mode };
                commands.entity(player_entity).insert(crate::weapon_systems::PlayerWaitingTetherActivationComponent {
                    hit_horror_entity: action.horror_entity,
                    horror_original_transform: Some(action.horror_local_transform),
                    params: tether_params_cloned.clone(),
                    reactivation_window_timer: Timer::from_seconds(tether_params_cloned.reactivation_window_secs, TimerMode::Once),
                    next_effect_mode: next_mode,
                });
                commands.entity(action.horror_entity).insert(HorrorLatchedByTetherComponent { player_who_latched: Some(player_entity) });
                // sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherHit)); // Commented out as per subtask
            }
            commands.entity(action.projectile_entity).despawn_recursive();
            continue; 
        }

        // --- Regular Hit ---
        let mut projectile_should_despawn = false;
        let mut bounce_occurred_this_hit = false;

        if let Ok((_, _, _, mut horror_health, _, _opt_damage_amp_debuff, _)) = query_set.p1().get_mut(action.horror_entity) { // opt_damage_amp_debuff not mutated, prefixed with _
            sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
            let actual_damage_dealt = action.damage_to_apply.min(action.horror_health_at_collision);
            horror_health.0 = horror_health.0.saturating_sub(action.damage_to_apply);
            visual_effects::spawn_damage_text(&mut commands, &asset_server, action.horror_gtransform.translation(), action.damage_to_apply, &time);

            if let Some(lifesteal_pct) = action.original_projectile_stats.lifesteal_percentage {
                if lifesteal_pct > 0.0 && actual_damage_dealt > 0 {
                    if let Ok((_p_transform, mut p_health, p_stats)) = query_set.p2().get_single_mut() {
                        let heal_amount = (actual_damage_dealt as f32 * lifesteal_pct).round() as i32;
                        if heal_amount > 0 { p_health.0 = (p_health.0 + heal_amount).min(p_stats.max_health); }
                    }
                }
            }
            if let Some(_weapon_def) = weapon_library.get_weapon_definition(action.original_projectile_stats.weapon_id) { /* Blink from weapon */ } // Prefixed
            if let Some(ref _blink_p_on_projectile) = action.original_projectile_stats.blink_params_on_hit {  /* Blink from projectile */ } // Prefixed
            if let Ok((_player_transform, _player_h, _player_survivor_stats_for_items)) = query_set.p2().get_single() { /* Item effects */ } // Prefixed
            if let Some(_debuff_data) = action.projectile_debuff_params { /* Debuff application */ } // Prefixed
        }

        if let Ok((_proj_e, _proj_gt, _proj_dmg, mut proj_stats, _proj_vel, _proj_sprite, _proj_tf, _, _, _)) = query_set.p0().get_mut(action.projectile_entity) { // Prefixed several unused here
            // Simplified for brace checking
            if !processed_projectiles_this_frame.contains(&action.projectile_entity) {
                if proj_stats.bounces_left.is_some() && proj_stats.bounces_left.unwrap_or(0) > 0 && !proj_stats.has_bounced_this_frame {
                    bounce_occurred_this_hit = true;
                }
                if !bounce_occurred_this_hit {
                    if proj_stats.piercing_left > 0 {
                        proj_stats.piercing_left -= 1;
                    }
                }
                if proj_stats.piercing_left == 0 && !bounce_occurred_this_hit {
                     projectile_should_despawn = true;
                }
                if proj_stats.bounces_left.is_some() && proj_stats.bounces_left.unwrap() == 0 && bounce_occurred_this_hit {
                    projectile_should_despawn = true;
                }
                processed_projectiles_this_frame.insert(action.projectile_entity);
            } // Closes if !processed_projectiles_this_frame
        } // Closes if let Ok for p0
        // Removed the extra brace that was here, which incorrectly tried to close p1 block again.
    
        if projectile_should_despawn {
            if let Some(explosion_data_val) = action.projectile_explodes_params {
                // Call spawn_explosion_effect
                crate::weapon_systems::spawn_explosion_effect(
                    &mut commands,
                    &asset_server,
                    action.horror_gtransform.translation(), // Position of the horror hit
                    explosion_data_val.explosion_damage,
                    explosion_data_val.explosion_radius,
                    Color::ORANGE_RED, // Example color
                    String::from("sprites/explosion_placeholder.png"), // Changed to String::from()
                    0.3, // Example duration
                );

                // Apply damage to horrors in radius
                // The horror_info_list contains GlobalTransforms of horrors
                let explosion_center = action.horror_gtransform.translation();
                for (horror_entity_to_check, horror_gtransform_to_check, _, current_health_val, _) in &horror_info_list {
                    if *current_health_val <= 0 { continue; } // Skip already dead or pending despawn

                    if horror_gtransform_to_check.translation().distance_squared(explosion_center) < explosion_data_val.explosion_radius.powi(2) {
                        // It's tricky to mutate horrors directly here due to query_set borrowing.
                        // A common pattern is to send damage events or collect damage requests.
                        // For simplicity, we'll try to get mutable access if possible, but this might require restructuring
                        // or deferring damage application.
                        // However, the current structure of collision_actions processing after collection
                        // might allow for mutable access to p1 *after* iterating p0.
                        // Let's assume for now we can apply damage if we re-query or pass Health mutably.
                        // The current `action` struct doesn't hold mutable health.
                        // This part will need careful handling of mutable access.
                        // For this iteration, we will focus on spawning the visual and assume damage application
                        // might need a separate event or a refactor if direct mutation isn't feasible here.
                        // The most direct way if query_set.p1().get_mut(horror_entity) works here:
                        if let Ok(mut horror_health_comp) = query_set.p1().get_component_mut::<Health>(*horror_entity_to_check) {
                             if horror_health_comp.0 > 0 { // Check health again before applying
                                horror_health_comp.0 = horror_health_comp.0.saturating_sub(explosion_data_val.explosion_damage);
                                visual_effects::spawn_damage_text(
                                    &mut commands,
                                    &asset_server,
                                    horror_gtransform_to_check.translation(),
                                    explosion_data_val.explosion_damage,
                                    &time,
                                );
                            }
                        }
                    }
                }
            }
            commands.entity(action.projectile_entity).despawn_recursive();
        }
    }
}

fn projectile_screen_bounce_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &GlobalTransform, &mut AutomaticProjectile, &mut Damage, &Sprite, &mut Transform)>,
    explosion_query: Query<&crate::weapon_systems::ExplodesOnFinalImpact>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<MainCamera>>,
) {
    let Ok((_camera, camera_gtransform, _projection)) = camera_query.get_single() else { return; };
    let camera_translation = camera_gtransform.translation();

    let world_min_x = -600.0 + camera_translation.x;
    let world_max_x = 600.0 + camera_translation.x;
    let world_min_y = -320.0 + camera_translation.y;
    let world_max_y = 320.0 + camera_translation.y;

    for (entity, mut velocity_comp, g_transform, mut proj_stats, mut damage_comp, sprite, mut transform_comp) in query.iter_mut() {
        if proj_stats.has_bounced_this_frame {
            continue;
        }

        let bounces_left = match proj_stats.bounces_left {
            Some(val) => val,
            None => continue,
        };

        if bounces_left == 0 {
            let proj_pos = g_transform.translation();
            let radius = sprite.custom_size.map_or(1.0, |s| s.x.max(s.y) / 2.0);
            if proj_pos.x - radius < world_min_x || proj_pos.x + radius > world_max_x ||
               proj_pos.y - radius < world_min_y || proj_pos.y + radius > world_max_y {
                    // Removed problematic: if explosion_query.get(entity).is_ok() {}
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        let proj_pos = g_transform.translation();
        let radius = sprite.custom_size.map_or(1.0, |s| s.x.max(s.y) / 2.0);

        let mut bounced = false;
        if (proj_pos.x - radius < world_min_x && velocity_comp.0.x < 0.0) || (proj_pos.x + radius > world_max_x && velocity_comp.0.x > 0.0) {
            velocity_comp.0.x *= -1.0;
            bounced = true;
        }
        if (proj_pos.y - radius < world_min_y && velocity_comp.0.y < 0.0) || (proj_pos.y + radius > world_max_y && velocity_comp.0.y > 0.0) {
            velocity_comp.0.y *= -1.0;
            bounced = true;
        }

        if bounced {
            handle_bounce_stat_updates(&mut proj_stats, &mut damage_comp, &mut velocity_comp);
            transform_comp.rotation = Quat::from_rotation_z(velocity_comp.0.y.atan2(velocity_comp.0.x));

            if proj_stats.bounces_left.is_some() && proj_stats.bounces_left.unwrap() == 0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn reset_projectile_bounce_flag_system(mut query: Query<&mut AutomaticProjectile>) {
    for mut proj_stats in query.iter_mut() {
        proj_stats.has_bounced_this_frame = false;
    }
}