// src/automatic_projectiles.rs
use bevy::prelude::*;
use rand::Rng; 
use crate::{
    components::{Velocity, Damage, Lifetime, Health},
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
    skills::SkillProjectile, 
    horror::HorrorProjectile, 
    survivor::Survivor, 
    items::{ItemLibrary, ItemEffect, ExplosionEffect, AutomaticWeaponId}, 
    game::AppState, 
};

pub struct AutomaticProjectilesPlugin;

impl Plugin for AutomaticProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AutomaticProjectile>()
            .add_systems(Update, (
                projectile_movement_system, 
                automatic_projectile_collision_system,
                projectile_screen_bounce_system.after(automatic_projectile_collision_system),
                automatic_projectile_lifetime_system,
            ).chain().in_set(OnUpdate(AppState::InGame)))
            .add_systems(PostUpdate, reset_projectile_bounce_flag_system.in_set(OnUpdate(AppState::InGame)));
    }
}

#[derive(Component, Reflect, Default, Debug)] 
#[reflect(Component)] 
pub struct AutomaticProjectile { 
    pub piercing_left: u32,
    pub weapon_id: AutomaticWeaponId, 
    // Bouncing fields
    pub bounces_left: Option<u32>,
    pub damage_on_hit: i32, 
    pub current_speed: f32, 
    pub damage_loss_per_bounce_multiplier: Option<f32>,
    pub speed_loss_per_bounce_multiplier: Option<f32>,
    pub has_bounced_this_frame: bool,
    // Lifesteal field
    pub lifesteal_percentage: Option<f32>,
    // Blink strike field
    pub blink_params_on_hit: Option<crate::items::BlinkStrikeProjectileParams>,
}

// Helper function for bounce stat updates
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
    position: Vec3, 
    direction: Vec2, 
    initial_damage: i32, 
    initial_speed: f32,  
    piercing: u32,
    weapon_id: AutomaticWeaponId, 
    sprite_path: &'static str, 
    size: Vec2,
    color: Color,
    lifetime_secs: f32,
    // Bouncing parameters
    opt_max_bounces: Option<u32>,
    opt_dmg_loss_mult: Option<f32>,
    opt_speed_loss_mult: Option<f32>,
    // Lifesteal parameter
    opt_lifesteal_percentage: Option<f32>,
    // Tether parameter
    opt_tether_params_for_comp: Option<crate::items::RepositioningTetherParams>,
    // Blink Strike parameter
    opt_blink_params: Option<crate::items::BlinkStrikeProjectileParams>,
) {
    let normalized_direction = direction.normalize_or_zero();
    let mut projectile_commands = commands.spawn(( 
        SpriteBundle { 
            texture: asset_server.load(sprite_path), 
            sprite: Sprite { 
                custom_size: Some(size), 
                color, 
                ..default() 
            }, 
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(normalized_direction.y.atan2(normalized_direction.x))), 
            ..default() 
        }, 
        AutomaticProjectile { 
            piercing_left: piercing, 
            weapon_id,
            bounces_left: opt_max_bounces,
            damage_on_hit: initial_damage,
            current_speed: initial_speed,
            damage_loss_per_bounce_multiplier: opt_dmg_loss_mult,
            speed_loss_per_bounce_multiplier: opt_speed_loss_mult,
            has_bounced_this_frame: false,
            lifesteal_percentage: opt_lifesteal_percentage,
            blink_params_on_hit: opt_blink_params.clone(), // Clone if it's Some
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
    mut projectile_query: Query<(
        Entity, 
        &GlobalTransform, 
        &mut Damage, 
        &mut AutomaticProjectile, 
        &mut Velocity,
        &Sprite, 
        &mut Transform,
        Option<&crate::weapon_systems::ExplodesOnFinalImpact>,
        Option<&crate::weapon_systems::DebuffOnHitComponent>,
        Option<&crate::weapon_systems::TetherProjectileComponent>, 
    )>, 
    mut horror_query: Query<(
        Entity, 
        &GlobalTransform,
        &Transform, // Added local Transform for horror
        &mut Health, 
        &crate::horror::Horror,
        Option<&mut crate::components::DamageAmpDebuffComponent>, 
        Option<&mut crate::weapon_systems::HorrorLatchedByTetherComponent>, 
    )>,
    // Specific query for player components needed by tether logic
    mut player_tether_setup_query: Query<(Entity, Option<&mut crate::weapon_systems::PlayerWaitingTetherActivationComponent>), With<Survivor>>,
    // Query for player health and stats for lifesteal AND blink (transform, health, stats)
    mut player_effects_query: Query<(&mut Transform, &mut Health, &Survivor), (With<Survivor>, Without<Horror>, Without<AutomaticProjectile>)>,
    item_library: Res<ItemLibrary>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (
        projectile_entity, 
        proj_gtransform, 
        mut proj_damage_component, 
        mut proj_stats, 
        mut velocity_component,   
        proj_sprite, 
        mut proj_transform, 
        opt_explodes_on_impact, 
        opt_debuff_on_hit,
        opt_tether_projectile, // Added tether projectile component to destructuring
    ) in projectile_query.iter_mut() {
        if proj_stats.has_bounced_this_frame { continue; }

        for (
            horror_entity, 
            horror_gtransform,
            local_horror_transform, // Added local_horror_transform
            mut horror_health, 
            horror_data, 
            opt_existing_damage_amp_debuff,
            opt_latched_by_tether, 
        ) in horror_query.iter_mut() {
            let distance = proj_gtransform.translation().truncate().distance(horror_gtransform.translation().truncate());
            
            let projectile_radius = proj_sprite.custom_size.map_or(5.0, |s| s.x.max(s.y) / 2.0);
            let horror_radius = horror_data.size.x / 2.0;

            if distance < projectile_radius + horror_radius {
                // Tether projectile logic
                if let Some(tether_comp_on_projectile) = opt_tether_projectile {
                    if let Ok((player_entity, mut opt_existing_activation_comp)) = player_tether_setup_query.get_single_mut() {
                        
                        // If player already has a waiting tether, clean it up first
                        if let Some(mut existing_activation_comp) = opt_existing_activation_comp.as_deref_mut() {
                            if commands.get_entity(existing_activation_comp.hit_horror_entity).is_some() {
                                commands.entity(existing_activation_comp.hit_horror_entity).remove::<crate::weapon_systems::HorrorLatchedByTetherComponent>();
                            }
                           // commands.entity(player_entity).remove::<crate::weapon_systems::PlayerWaitingTetherActivationComponent>(); // This is incorrect, need to use commands on the player entity directly
                        }
                         // Remove existing component from player if it exists, before adding a new one
                        if opt_existing_activation_comp.is_some() {
                            commands.entity(player_entity).remove::<crate::weapon_systems::PlayerWaitingTetherActivationComponent>();
                        }


                        let next_mode = if tether_comp_on_projectile.params_snapshot.mode == crate::items::RepositioningTetherMode::Alternate {
                            crate::items::RepositioningTetherMode::Pull // Default to Pull for Alternate on first hit
                        } else {
                            tether_comp_on_projectile.params_snapshot.mode
                        };

                        commands.entity(player_entity).insert(crate::weapon_systems::PlayerWaitingTetherActivationComponent {
                            hit_horror_entity: horror_entity,
                            horror_original_transform: Some(*local_horror_transform), // Use local transform
                            params: tether_comp_on_projectile.params_snapshot.clone(),
                            reactivation_window_timer: Timer::from_seconds(tether_comp_on_projectile.params_snapshot.reactivation_window_secs, TimerMode::Once),
                            next_effect_mode: next_mode,
                        });
                        
                        commands.entity(horror_entity).insert(crate::weapon_systems::HorrorLatchedByTetherComponent {
                            player_who_latched: player_entity
                        });
                        
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherHit)); 
                        commands.entity(projectile_entity).despawn_recursive();
                        break; // Tether projectile's job is done with this horror.
                    }
                } else { // Standard projectile logic (damage, lifesteal, on-hit effects, bounce, pierce)
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                    
                    let actual_damage_dealt = proj_stats.damage_on_hit.min(horror_health.0);
                    horror_health.0 -= actual_damage_dealt;
                    spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), actual_damage_dealt, &time);

                    // Lifesteal Logic
                    if let Some(lifesteal_pct) = proj_stats.lifesteal_percentage {
                        if lifesteal_pct > 0.0 && actual_damage_dealt > 0 {
                            if let Ok((_player_transform, mut player_health, player_survivor_stats)) = player_effects_query.get_single_mut() {
                                let lifesteal_amount = (actual_damage_dealt as f32 * lifesteal_pct).round() as i32;
                                if lifesteal_amount > 0 {
                                    player_health.0 = (player_health.0 + lifesteal_amount).min(player_survivor_stats.max_health);
                                }
                            }
                            }
                        }
                            }
                        }
                    }

                    // Blink Strike Logic (after damage and lifesteal, before piercing/despawn)
                    if let Some(ref blink_p) = proj_stats.blink_params_on_hit {
                        let killed_target = horror_health.0 <= 0;
                        if blink_p.blink_requires_kill && !killed_target {
                            // Blink requires kill, but target was not killed. Do nothing.
                        } else {
                            // Proceed with blink chance
                            if rand::thread_rng().gen_bool(blink_p.blink_chance_on_hit_percent as f64) {
                                if let Ok((mut player_transform, _player_health, survivor_stats)) = player_effects_query.get_single_mut() {
                                    let mut new_player_pos = player_transform.translation;
                                    if blink_p.blink_to_target_behind {
                                        let dir_from_player_to_horror = (horror_gtransform.translation() - player_transform.translation).truncate().normalize_or_else(|| survivor_stats.aim_direction.normalize_or_zero());
                                        // Place player 'blink_distance' along the line from player to horror, but starting from horror's position
                                        new_player_pos = horror_gtransform.translation() - dir_from_player_to_horror.extend(0.0) * blink_p.blink_distance;
                                    } else { // Blink in player's aim direction
                                        let aim_dir = survivor_stats.aim_direction.normalize_or_zero();
                                        let effective_aim_dir = if aim_dir == Vec2::ZERO { Vec2::X } else { aim_dir }; // Default if aim is zero
                                        new_player_pos = player_transform.translation + effective_aim_dir.extend(0.0) * blink_p.blink_distance;
                                    }
                                    player_transform.translation = new_player_pos;
                                    // Optional: Spawn blink visual effects here
                                     sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerBlink));
                                }
                            }
                        }
                    }
                    
                    // Item OnHit Effects
                    if let Ok((_player_transform, _player_h, player_survivor_stats_for_items)) = player_effects_query.get_single() {
                        for item_id in player_survivor_stats_for_items.collected_item_ids.iter() {
                            if let Some(item_def) = item_library.get_item_definition(*item_id) {
                                for effect in &item_def.effects {
                                    if let ItemEffect::OnAutomaticProjectileHitExplode { chance, explosion_damage, explosion_radius, explosion_color } = effect {
                                        let mut rng = rand::thread_rng();
                                        if rng.gen_bool((*chance).into()) {
                                            commands.spawn((
                                                SpriteBundle {
                                                    texture: asset_server.load("sprites/eldritch_nova_effect_placeholder.png"), 
                                                    sprite: Sprite {
                                                        custom_size: Some(Vec2::splat(0.1)), 
                                                        color: *explosion_color,
                                                        ..default()
                                                    },
                                                    transform: Transform::from_translation(horror_gtransform.translation().truncate().extend(0.3)),
                                                    ..default()
                                                },
                                                ExplosionEffect {
                                                    damage: *explosion_damage,
                                                    radius_sq: explosion_radius.powi(2),
                                                    timer: Timer::from_seconds(0.3, TimerMode::Once), 
                                                    already_hit_entities: vec![horror_entity], 
                                                },
                                                Name::new("ItemHitExplosion"),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Bouncing logic for horrors
                    if let Some(bounces) = proj_stats.bounces_left {
                        if bounces > 0 && !proj_stats.has_bounced_this_frame {
                            handle_bounce_stat_updates(&mut proj_stats, &mut proj_damage_component, &mut velocity_component);
                            
                            let reflection_dir = (proj_gtransform.translation().truncate() - horror_gtransform.translation().truncate()).normalize_or_else(Vec2::X);
                            if reflection_dir != Vec2::ZERO {
                               velocity_component.0 = reflection_dir * proj_stats.current_speed;
                               proj_transform.rotation = Quat::from_rotation_z(velocity_component.0.y.atan2(velocity_component.0.x));
                            }
                            break; 
                        }
                    }
                    
                    if proj_stats.piercing_left > 0 {
                        proj_stats.piercing_left -= 1;
                    } else {
                        // Debuff application if projectile has DebuffOnHitComponent
                        if let Some(debuff_comp) = opt_debuff_on_hit {
                            match debuff_comp.debuff_type {
                                crate::items::ProjectileDebuffType::DamageAmp => {
                                    if let Some(mut existing_debuff) = opt_existing_damage_amp_debuff {
                                        existing_debuff.current_stacks = (existing_debuff.current_stacks + 1).min(debuff_comp.max_stacks);
                                        existing_debuff.duration_timer = Timer::from_seconds(debuff_comp.duration_secs, TimerMode::Once);
                                    } else {
                                        commands.entity(horror_entity).insert(crate::components::DamageAmpDebuffComponent {
                                            current_stacks: 1,
                                            magnitude_per_stack: debuff_comp.magnitude_per_stack,
                                            max_stacks: debuff_comp.max_stacks,
                                            duration_timer: Timer::from_seconds(debuff_comp.duration_secs, TimerMode::Once),
                                        });
                                    }
                                }
                                crate::items::ProjectileDebuffType::Slow => {
                                    // Placeholder for slow debuff logic
                                }
                            }
                        }

                        // Explosion on final impact if applicable
                        if let Some(explosion_params) = opt_explodes_on_impact {
                            commands.spawn((
                                SpriteBundle {
                                    texture: asset_server.load("sprites/eldritch_nova_effect_placeholder.png"), 
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::splat(0.1)), 
                                        color: Color::rgba(1.0, 0.5, 0.0, 0.8), 
                                        ..default()
                                    },
                                    transform: Transform::from_translation(proj_gtransform.translation().truncate().extend(0.3)), 
                                    ..default()
                                },
                                ExplosionEffect {
                                    damage: explosion_params.explosion_damage,
                                    radius_sq: explosion_params.explosion_radius.powi(2),
                                    timer: Timer::from_seconds(0.3, TimerMode::Once), 
                                    already_hit_entities: vec![horror_entity], 
                                },
                                Name::new("ChargeShotExplosion"),
                            ));
                        }
                        commands.entity(projectile_entity).despawn_recursive();
                        break; 
                    }
                }
            }
        }
    }
}

fn projectile_screen_bounce_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &GlobalTransform, &mut AutomaticProjectile, &mut Damage, &Sprite, &mut Transform)>,
) {
    const SCREEN_MIN_X: f32 = -600.0;
    const SCREEN_MAX_X: f32 = 600.0;
    const SCREEN_MIN_Y: f32 = -320.0;
    const SCREEN_MAX_Y: f32 = 320.0;

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
            if proj_pos.x - radius < SCREEN_MIN_X || proj_pos.x + radius > SCREEN_MAX_X || 
               proj_pos.y - radius < SCREEN_MIN_Y || proj_pos.y + radius > SCREEN_MAX_Y {
                if let Some(_explosion_params) = commands.entity(entity).get::<crate::weapon_systems::ExplodesOnFinalImpact>() {
                    // Lifesteal projectiles typically don't explode on screen bounce unless specifically designed to.
                    // For now, this specific explosion on screen edge after all bounces is not implemented.
                 }
                commands.entity(entity).despawn_recursive();
            }
            continue; 
        }
        
        let proj_pos = g_transform.translation();
        let radius = sprite.custom_size.map_or(1.0, |s| s.x.max(s.y) / 2.0); 

        let mut bounced = false;
        if (proj_pos.x - radius < SCREEN_MIN_X && velocity_comp.0.x < 0.0) || (proj_pos.x + radius > SCREEN_MAX_X && velocity_comp.0.x > 0.0) {
            velocity_comp.0.x *= -1.0;
            bounced = true;
        }
        if (proj_pos.y - radius < SCREEN_MIN_Y && velocity_comp.0.y < 0.0) || (proj_pos.y + radius > SCREEN_MAX_Y && velocity_comp.0.y > 0.0) {
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