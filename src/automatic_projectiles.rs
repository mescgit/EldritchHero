// src/automatic_projectiles.rs
use bevy::prelude::*;
use bevy::prelude::in_state; 
use rand::Rng;
use crate::{
    components::{Velocity, Damage, Lifetime, Health, HorrorLatchedByTetherComponent},
    visual_effects, 
    audio::{PlaySoundEvent, SoundEffect},
    horror::Horror,
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
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(PostUpdate, reset_projectile_bounce_flag_system.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
pub struct AutomaticProjectile {
    pub owner: Entity,
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

impl Default for AutomaticProjectile {
    fn default() -> Self {
        Self {
            owner: Entity::PLACEHOLDER,
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
) {
    let normalized_direction = direction.normalize_or_zero();
    let mut projectile_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(sprite_path.to_string()), // Changed to .to_string()
            sprite: Sprite {
                custom_size: Some(size),
                color,
                ..default()
            },
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(normalized_direction.y.atan2(normalized_direction.x))),
            ..default()
        },
        AutomaticProjectile {
            owner,
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
        &Transform,
        &mut Health,
        &crate::horror::Horror,
        Option<&mut crate::components::DamageAmpDebuffComponent>,
        Option<&mut HorrorLatchedByTetherComponent>,
    )>,
    mut player_tether_setup_query: Query<(Entity, Option<&mut crate::components::PlayerTetherState>), With<Survivor>>,
    mut player_effects_query: Query<(&mut Transform, &mut Health, &Survivor), (With<Survivor>, Without<Horror>, Without<AutomaticProjectile>)>,
    item_library: Res<ItemLibrary>,
    weapon_library: Res<crate::items::AutomaticWeaponLibrary>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    mut player_blink_event_writer: EventWriter<crate::components::PlayerBlinkEvent>,
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
        opt_tether_projectile,
    ) in projectile_query.iter_mut() {
        if proj_stats.has_bounced_this_frame { continue; }

        for (
            horror_entity,
            horror_gtransform,
            local_horror_transform,
            mut horror_health,
            horror_data,
            opt_existing_damage_amp_debuff,
            _opt_latched_by_tether,
        ) in horror_query.iter_mut() {
            let distance = proj_gtransform.translation().truncate().distance(horror_gtransform.translation().truncate());

            let projectile_radius = proj_sprite.custom_size.map_or(5.0, |s| s.x.max(s.y) / 2.0);
            let horror_radius = horror_data.size.x / 2.0;

            if distance < projectile_radius + horror_radius {
                if let Some(tether_comp_on_projectile) = opt_tether_projectile {
                    if let Ok((player_entity, opt_existing_player_tether_state)) = player_tether_setup_query.get_single_mut() { // Corrected variable name
                        if let Some(existing_player_tether_state) = opt_existing_player_tether_state {
                             if let Some(enemy_entity_in_state) = existing_player_tether_state.tethered_enemy_entity {
                                if commands.get_entity(enemy_entity_in_state).is_some() {
                                    commands.entity(enemy_entity_in_state).remove::<HorrorLatchedByTetherComponent>();
                                }
                            }
                        }
                        if commands.get_entity(player_entity).is_some() {
                            commands.entity(player_entity).remove::<crate::weapon_systems::PlayerWaitingTetherActivationComponent>();
                        }

                        let next_mode = if tether_comp_on_projectile.params_snapshot.mode == crate::items::RepositioningTetherMode::Alternate {
                            crate::items::RepositioningTetherMode::Pull
                        } else {
                            tether_comp_on_projectile.params_snapshot.mode
                        };

                        commands.entity(player_entity).insert(crate::weapon_systems::PlayerWaitingTetherActivationComponent {
                            hit_horror_entity: horror_entity,
                            horror_original_transform: Some(*local_horror_transform),
                            params: tether_comp_on_projectile.params_snapshot.clone(),
                            reactivation_window_timer: Timer::from_seconds(tether_comp_on_projectile.params_snapshot.reactivation_window_secs, TimerMode::Once),
                            next_effect_mode: next_mode,
                        });

                        commands.entity(horror_entity).insert(HorrorLatchedByTetherComponent {
                            player_who_latched: Some(player_entity)
                        });

                        sound_event_writer.send(PlaySoundEvent(SoundEffect::TetherHit));
                        commands.entity(projectile_entity).despawn_recursive();
                        break;
                    }
                } else {
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));

                    let actual_damage_dealt = proj_stats.damage_on_hit.min(horror_health.0);
                    horror_health.0 -= actual_damage_dealt;
                    visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), actual_damage_dealt, &time);

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

                    if let Some(weapon_def) = weapon_library.get_weapon_definition(proj_stats.weapon_id) {
                        if let crate::items::AttackTypeData::BlinkStrikeProjectile(ref blink_strike_params_from_weapon) = weapon_def.attack_data {
                            if rand::thread_rng().gen_bool(blink_strike_params_from_weapon.blink_chance_on_hit_percent as f64) {
                                player_blink_event_writer.send(crate::components::PlayerBlinkEvent {
                                    player_entity: proj_stats.owner,
                                    hit_enemy_entity: horror_entity,
                                    blink_params: blink_strike_params_from_weapon.clone(),
                                });
                            }
                        }
                    }

                    if let Some(ref blink_p_on_projectile) = proj_stats.blink_params_on_hit {
                        let killed_target = horror_health.0 <= 0;
                        if blink_p_on_projectile.blink_requires_kill && !killed_target {
                        } else {
                            if rand::thread_rng().gen_bool(blink_p_on_projectile.blink_chance_on_hit_percent as f64) {
                                info!("Projectile (ID: {:?}) itself blinking due to its own blink_params_on_hit for weapon ID: {:?}", projectile_entity, proj_stats.weapon_id);
                            }
                        }
                    }


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

                    if let Some(bounces) = proj_stats.bounces_left {
                        if bounces > 0 && !proj_stats.has_bounced_this_frame {
                            handle_bounce_stat_updates(&mut proj_stats, &mut proj_damage_component, &mut velocity_component);

                            let reflection_dir = (proj_gtransform.translation().truncate() - horror_gtransform.translation().truncate()).normalize_or_zero();
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
                                }
                            }
                        }

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
    explosion_query: Query<&crate::weapon_systems::ExplodesOnFinalImpact>,
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
                if explosion_query.get(entity).is_ok() {
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