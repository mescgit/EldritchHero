// src/survivor.rs
use bevy::{prelude::*, window::PrimaryWindow};
use std::time::Duration;
use rand::Rng;
use crate::{
    components::{Velocity, Health as ComponentHealth},
    game::{AppState, ItemCollectedEvent, SelectedCharacter}, 
    automatic_projectiles::{spawn_automatic_projectile},
    horror::Horror,
    weapons::{CircleOfWarding, SwarmOfNightmares},
    audio::{PlaySoundEvent, SoundEffect},
    skills::{ActiveSkillInstance, SkillLibrary, SkillId, SurvivorBuffEffect, ActiveShield},
    items::{ItemId, ItemDrop, ItemLibrary, ItemEffect, RetaliationNovaEffect, AutomaticWeaponId, AutomaticWeaponLibrary, AttackTypeData},
};

pub const SURVIVOR_SIZE: Vec2 = Vec2::new(50.0, 50.0);
const XP_FOR_LEVEL: [u32; 10] = [100, 150, 250, 400, 600, 850, 1100, 1400, 1800, 2500];
pub const BASE_PICKUP_RADIUS: f32 = 100.0;
const PROJECTILE_SPREAD_ANGLE_DEGREES: f32 = 15.0; 
pub const INITIAL_SURVIVOR_MAX_HEALTH: i32 = 100;
pub const BASE_SURVIVOR_SPEED: f32 = 250.0;
const ITEM_COLLECTION_RADIUS: f32 = SURVIVOR_SIZE.x / 2.0 + crate::items::ITEM_DROP_SIZE.x / 2.0;
const MIND_STRAIN_DEBUFF_DURATION: f32 = 5.0;
const MIND_STRAIN_SPEED_REDUCTION_PER_STACK: f32 = 0.05; 
const MAX_MIND_STRAIN_STACKS: u32 = 4;


#[derive(Component)] pub struct SanityStrain { pub base_fire_rate_secs: f32, pub fire_timer: Timer, }

#[derive(Component, Debug)]
pub struct MindStrainDebuff {
    pub stacks: u32,
    pub timer: Timer, 
}

pub struct SurvivorPlugin;
#[derive(Component)]
pub struct Survivor {
    pub speed: f32, pub experience: u32, pub current_level_xp: u32, pub level: u32,
    pub aim_direction: Vec2, pub invincibility_timer: Timer,

    pub auto_weapon_damage_bonus: i32,
    pub auto_weapon_projectile_speed_multiplier: f32,
    pub auto_weapon_piercing_bonus: u32,
    pub auto_weapon_additional_projectiles_bonus: u32,

    pub xp_gain_multiplier: f32, pub pickup_radius_multiplier: f32,
    pub max_health: i32, pub health_regen_rate: f32,
    pub equipped_skills: Vec<ActiveSkillInstance>,
    pub collected_item_ids: Vec<ItemId>,
    pub inherent_weapon_id: AutomaticWeaponId, 

    // --- New Fields for Upgrades ---
    // Auto-Attack Focused
    pub auto_attack_bonus_fire_damage: u32,
    pub auto_attack_bonus_cold_damage: u32,
    pub auto_attack_bonus_lightning_damage: u32,
    pub auto_attack_poison_dps: u32,
    pub auto_attack_crit_chance: f32,
    pub auto_attack_crit_damage_multiplier: f32, // Base is e.g. 1.5x or 2.0x, this adds to it.
    pub auto_attack_execute_threshold: f32,
    pub auto_attack_lifesteal_percent: f32,
    pub auto_attack_chain_chance: f32,
    pub auto_attack_fork_chance: f32,
    pub auto_attack_chill_chance: f32,
    pub auto_attack_stun_chance: f32,
    pub auto_attack_burn_chance: f32,
    pub auto_attack_reduce_healing_chance: f32,
    pub auto_attack_aoe_on_hit_chance: f32,
    pub auto_attack_aoe_on_hit_damage: u32,
    pub auto_attack_projectile_duration_multiplier: f32,
    pub auto_attack_homing_strength: f32,
    pub auto_attack_ricochet_chance: f32,
    pub auto_attack_shield_penetration_percent: f32,
    pub auto_attack_cull_strike_chance: f32,

    // Survivor Defensive Stats
    pub armor: u32,
    pub evasion_chance: f32,
    pub block_chance: f32,
    pub damage_reduction_percent: f32,
    pub tenacity_percent: f32,
    pub status_effect_resistance_percent: f32,
    pub healing_effectiveness_multiplier: f32,
    pub on_hit_temp_armor_bonus: u32,
    pub on_hit_temp_speed_bonus_percent: f32,
    pub after_being_hit_retaliation_nova_damage: i32,

    // Survivor Utility/Mobility
    pub max_dash_charges: u32,
    pub dash_cooldown_multiplier: f32,
    pub dash_range_multiplier: f32,
    pub dash_invulnerability_duration: f32,
    pub movement_speed_out_of_combat_multiplier: f32,
    pub slow_effectiveness_reduction_percent: f32,
    pub shield_on_kill_amount: u32,
    pub echoes_drop_rate_multiplier: f32,
    pub relic_drop_rate_multiplier: f32,
    pub free_skill_use_chance: f32,

    // Weapon-Specific (Aura/Orbiter)
    pub aura_size_per_kill_bonus_percent: f32,
    pub orbiter_speed_per_kill_bonus_percent: f32,
    pub aura_pull_enemies_chance: f32,
    pub orbiter_explode_on_kill_chance: f32,
    pub orbiter_explosion_damage: u32,
    pub aura_debuff_enemies_damage_increase_percent: f32,
}

impl Survivor {
    pub fn experience_to_next_level(&self) -> u32 { if self.level == 0 { return 0; } if (self.level as usize -1) < XP_FOR_LEVEL.len() { XP_FOR_LEVEL[self.level as usize - 1] } else { XP_FOR_LEVEL.last().unwrap_or(&2500) + (self.level - XP_FOR_LEVEL.len() as u32) * 500 } }
    pub fn add_experience( &mut self, amount: u32, next_state_value: &mut NextState<AppState>, sound_event_writer: &mut EventWriter<PlaySoundEvent>,) { let actual_xp_gained = (amount as f32 * self.xp_gain_multiplier).round() as u32; self.current_level_xp += actual_xp_gained; self.experience += actual_xp_gained; while self.current_level_xp >= self.experience_to_next_level() && self.level > 0 { let needed = self.experience_to_next_level(); self.current_level_xp -= needed; self.level += 1; sound_event_writer.send(PlaySoundEvent(SoundEffect::Revelation)); next_state_value.set(AppState::LevelUp); if next_state_value.0 == Some(AppState::LevelUp) { break; } } }
    pub fn get_effective_pickup_radius(&self) -> f32 { BASE_PICKUP_RADIUS * self.pickup_radius_multiplier }

    pub fn new_with_skills_items_and_weapon(
        initial_skills: Vec<ActiveSkillInstance>,
        initial_items: Vec<ItemId>,
        inherent_weapon_id: AutomaticWeaponId, 
        _weapon_library: &Res<AutomaticWeaponLibrary>, 
    ) -> Self {
        Self {
            speed: BASE_SURVIVOR_SPEED,
            experience: 0, current_level_xp: 0, level: 1,
            aim_direction: Vec2::X,
            invincibility_timer: Timer::from_seconds(1.0, TimerMode::Once),
            auto_weapon_damage_bonus: 0,
            auto_weapon_projectile_speed_multiplier: 1.0,
            auto_weapon_piercing_bonus: 0,
            auto_weapon_additional_projectiles_bonus: 0,
            xp_gain_multiplier: 1.0,
            pickup_radius_multiplier: 1.0,
            max_health: INITIAL_SURVIVOR_MAX_HEALTH,
            health_regen_rate: 0.0,
            equipped_skills: initial_skills,
            collected_item_ids: initial_items,
            inherent_weapon_id, 
            
            // Initialize new fields
            auto_attack_bonus_fire_damage: 0,
            auto_attack_bonus_cold_damage: 0,
            auto_attack_bonus_lightning_damage: 0,
            auto_attack_poison_dps: 0,
            auto_attack_crit_chance: 0.0,
            auto_attack_crit_damage_multiplier: 0.0, // This is a bonus, base crit multi (e.g. 2.0x) applied elsewhere
            auto_attack_execute_threshold: 0.0,
            auto_attack_lifesteal_percent: 0.0,
            auto_attack_chain_chance: 0.0,
            auto_attack_fork_chance: 0.0,
            auto_attack_chill_chance: 0.0,
            auto_attack_stun_chance: 0.0,
            auto_attack_burn_chance: 0.0,
            auto_attack_reduce_healing_chance: 0.0,
            auto_attack_aoe_on_hit_chance: 0.0,
            auto_attack_aoe_on_hit_damage: 0,
            auto_attack_projectile_duration_multiplier: 1.0,
            auto_attack_homing_strength: 0.0,
            auto_attack_ricochet_chance: 0.0,
            auto_attack_shield_penetration_percent: 0.0,
            auto_attack_cull_strike_chance: 0.0,

            armor: 0,
            evasion_chance: 0.0,
            block_chance: 0.0,
            damage_reduction_percent: 0.0,
            tenacity_percent: 0.0,
            status_effect_resistance_percent: 0.0,
            healing_effectiveness_multiplier: 1.0,
            on_hit_temp_armor_bonus: 0,
            on_hit_temp_speed_bonus_percent: 0.0,
            after_being_hit_retaliation_nova_damage: 0,

            max_dash_charges: 1, // Default to 1 dash charge
            dash_cooldown_multiplier: 1.0,
            dash_range_multiplier: 1.0,
            dash_invulnerability_duration: 0.0,
            movement_speed_out_of_combat_multiplier: 1.0,
            slow_effectiveness_reduction_percent: 0.0,
            shield_on_kill_amount: 0,
            echoes_drop_rate_multiplier: 1.0,
            relic_drop_rate_multiplier: 1.0,
            free_skill_use_chance: 0.0,

            aura_size_per_kill_bonus_percent: 0.0,
            orbiter_speed_per_kill_bonus_percent: 0.0,
            aura_pull_enemies_chance: 0.0,
            orbiter_explode_on_kill_chance: 0.0,
            orbiter_explosion_damage: 0,
            aura_debuff_enemies_damage_increase_percent: 0.0,
        }
    }
}

fn should_despawn_survivor(next_state: Res<NextState<AppState>>) -> bool { match next_state.0 { Some(AppState::GameOver) | Some(AppState::MainMenu) => true, _ => false, } }
fn no_survivor_exists(survivor_query: Query<(), With<Survivor>>) -> bool { survivor_query.is_empty() }

impl Plugin for SurvivorPlugin {
    fn build(&self, app: &mut App) {
        app .add_systems(OnEnter(AppState::InGame), spawn_survivor.run_if(no_survivor_exists))
            .add_systems(Update, (
                survivor_movement, 
                survivor_aiming,
                survivor_casting_system,
                survivor_health_regeneration_system,
                survivor_horror_collision_system.before(check_survivor_death_system),
                survivor_invincibility_system,
                check_survivor_death_system,
                survivor_item_drop_collection_system,
                mind_strain_debuff_update_system, 
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_survivor.run_if(should_despawn_survivor));
    }
}

fn spawn_survivor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    skill_library: Res<SkillLibrary>,
    weapon_library: Res<AutomaticWeaponLibrary>,
    selected_character: Res<SelectedCharacter>, 
) {
    let mut initial_skills = Vec::new();
    if let Some(_skill_def_bolt) = skill_library.get_skill_definition(SkillId(1)) {
        let bolt_instance = ActiveSkillInstance::new(SkillId(1));
        initial_skills.push(bolt_instance);
    }

    let chosen_inherent_weapon_id = selected_character.0;
    let mut initial_fire_rate = 0.5; 
    let mut survivor_name = "Survivor (Unknown Class)".to_string();

    if let Some(weapon_def) = weapon_library.get_weapon_definition(chosen_inherent_weapon_id) {
        if let AttackTypeData::StandardProjectile(params) = &weapon_def.attack_data {
            initial_fire_rate = params.base_fire_rate_secs;
        } else {
            // Log an error or use a default if it's not StandardProjectile
            // For now, let's use a default and log an error.
            error!("Chosen inherent weapon {:?} for survivor does not have StandardProjectile attack data! Using default fire rate.", weapon_def.id);
            // initial_fire_rate is already defaulted to 1.0, so no change needed here if error.
        }
        survivor_name = format!("Survivor ({})", weapon_def.name);
    }

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/survivor_placeholder.png"),
            sprite: Sprite { custom_size: Some(SURVIVOR_SIZE), ..default() },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Survivor::new_with_skills_items_and_weapon( 
            initial_skills,
            Vec::new(),
            chosen_inherent_weapon_id, 
            &weapon_library
        ),
        ComponentHealth(INITIAL_SURVIVOR_MAX_HEALTH),
        Velocity(Vec2::ZERO),
        SanityStrain {
            base_fire_rate_secs: initial_fire_rate,
            fire_timer: Timer::from_seconds(initial_fire_rate, TimerMode::Repeating),
        },
        CircleOfWarding::default(),
        SwarmOfNightmares::default(),
        Name::new(survivor_name), 
    ));
}
fn despawn_survivor(mut commands: Commands, survivor_query: Query<Entity, With<Survivor>>) { if let Ok(survivor_entity) = survivor_query.get_single() { commands.entity(survivor_entity).despawn_recursive(); } }
fn survivor_health_regeneration_system(time: Res<Time>, mut query: Query<(&Survivor, &mut ComponentHealth)>,) { for (survivor_stats, mut current_health) in query.iter_mut() { if survivor_stats.health_regen_rate > 0.0 && current_health.0 > 0 && current_health.0 < survivor_stats.max_health { let regen_amount = survivor_stats.health_regen_rate * time.delta_seconds(); current_health.0 = (current_health.0 as f32 + regen_amount).round() as i32; current_health.0 = current_health.0.min(survivor_stats.max_health); } } }

fn survivor_movement( 
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&Survivor, &mut Transform, &mut Velocity, Option<&SurvivorBuffEffect>, Option<&MindStrainDebuff>)>, 
    time: Res<Time>,
) { 
    for (survivor, mut transform, mut velocity, buff_effect_opt, mind_strain_opt) in query.iter_mut() { 
        let mut direction = Vec2::ZERO; 
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; } 
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; } 
        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; } 
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; } 
        
        let mut current_speed = survivor.speed; 
        if let Some(buff) = buff_effect_opt { 
            current_speed *= 1.0 + buff.speed_multiplier_bonus; 
        }
        if let Some(debuff) = mind_strain_opt {
            current_speed *= 1.0 - (debuff.stacks as f32 * MIND_STRAIN_SPEED_REDUCTION_PER_STACK);
            current_speed = current_speed.max(BASE_SURVIVOR_SPEED * 0.1); 
        }

        velocity.0 = if direction != Vec2::ZERO { direction.normalize() * current_speed } else { Vec2::ZERO }; 
        transform.translation.x += velocity.0.x * time.delta_seconds(); 
        transform.translation.y += velocity.0.y * time.delta_seconds(); 
    } 
}

fn mind_strain_debuff_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MindStrainDebuff), With<Survivor>>,
) {
    for (survivor_entity, mut debuff) in query.iter_mut() {
        debuff.timer.tick(time.delta());
        if debuff.timer.finished() {
            commands.entity(survivor_entity).remove::<MindStrainDebuff>();
        }
    }
}


fn survivor_aiming(mut survivor_query: Query<(&mut Survivor, &Transform)>, window_query: Query<&Window, With<PrimaryWindow>>, camera_query: Query<(&Camera, &GlobalTransform)>,) { if let Ok((mut survivor, survivor_transform)) = survivor_query.get_single_mut() { if let Ok(primary_window) = window_query.get_single() { if let Ok((camera, camera_transform)) = camera_query.get_single() { if let Some(cursor_position) = primary_window.cursor_position() { if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) { let direction_to_mouse = (world_position - survivor_transform.translation.truncate()).normalize_or_zero(); if direction_to_mouse != Vec2::ZERO { survivor.aim_direction = direction_to_mouse; } } } } } } }

fn survivor_casting_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(&Transform, &Survivor, &mut SanityStrain, Option<&SurvivorBuffEffect>)>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    weapon_library: Res<AutomaticWeaponLibrary>,
) {
    for (survivor_transform, survivor_stats, mut sanity_strain, buff_effect_opt) in query.iter_mut() {
        let weapon_def = match weapon_library.get_weapon_definition(survivor_stats.inherent_weapon_id) {
            Some(def) => def,
            None => { 
                weapon_library.get_weapon_definition(AutomaticWeaponId(0))
                    .expect("Default weapon ID 0 not found in library")
            }
        };

        let mut effective_fire_rate_secs = sanity_strain.base_fire_rate_secs; 

        if let Some(buff) = buff_effect_opt {
            effective_fire_rate_secs /= 1.0 + buff.fire_rate_multiplier_bonus;
        }

        let new_duration = Duration::from_secs_f32(effective_fire_rate_secs.max(0.05));
        if sanity_strain.fire_timer.duration() != new_duration {
            sanity_strain.fire_timer.set_duration(new_duration);
        }
        sanity_strain.fire_timer.tick(time.delta());

        if sanity_strain.fire_timer.just_finished() {
            if survivor_stats.aim_direction != Vec2::ZERO {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast));

                if let AttackTypeData::StandardProjectile(params) = &weapon_def.attack_data {
                    let current_damage = params.base_damage + survivor_stats.auto_weapon_damage_bonus;
                    // Applying the multiplier as it was likely intended for all auto attacks
                    let effective_projectile_lifetime_secs = params.projectile_lifetime_secs * survivor_stats.auto_attack_projectile_duration_multiplier;
                    let current_speed = params.base_projectile_speed * survivor_stats.auto_weapon_projectile_speed_multiplier;
                    let current_piercing = params.base_piercing + survivor_stats.auto_weapon_piercing_bonus;
                    let total_projectiles = 1 + params.additional_projectiles + survivor_stats.auto_weapon_additional_projectiles_bonus;

                    let base_angle = survivor_stats.aim_direction.to_angle();
                    // Use saturating_sub to prevent underflow if total_projectiles is 0, though it should be at least 1.
                    let spread_arc_degrees = PROJECTILE_SPREAD_ANGLE_DEGREES * (total_projectiles.saturating_sub(1)) as f32;
                    let start_angle_offset_rad = if total_projectiles > 1 { -spread_arc_degrees.to_radians() / 2.0 } else { 0.0 };
                    
                    for i in 0..total_projectiles {
                        let angle_offset_rad = if total_projectiles > 1 {
                            let step = if total_projectiles > 1 { spread_arc_degrees.to_radians() / (total_projectiles - 1) as f32 } else { 0.0 };
                            start_angle_offset_rad + (i as f32 * step)
                        } else {
                            0.0 // No offset if only one projectile
                        };
                        let projectile_direction = Vec2::from_angle(base_angle + angle_offset_rad);

                        spawn_automatic_projectile(
                            &mut commands,
                            &asset_server,
                            survivor_transform.translation,
                            projectile_direction,
                            current_damage,
                            current_speed,
                            current_piercing,
                            weapon_def.id, // weapon_def.id is still correct
                            params.projectile_sprite_path,
                            params.projectile_size,
                            params.projectile_color,
                            effective_projectile_lifetime_secs,
                        );
                    }
                } else {
                    error!("Weapon {:?} (ID: {}) does not have StandardProjectile attack data! Cannot fire.", weapon_def.name, weapon_def.id.0);
                }
            }
        }
    }
}
fn survivor_horror_collision_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut survivor_query: Query<(Entity, &Transform, &mut ComponentHealth, &mut Survivor, Option<&mut ActiveShield>, Option<&mut MindStrainDebuff>)>, 
    horror_query: Query<(Entity, &Transform, &Horror)>, 
    item_library: Res<ItemLibrary>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((survivor_entity, survivor_transform, mut survivor_health, mut survivor_component, 
               mut opt_active_shield, mut opt_mind_strain)) = survivor_query.get_single_mut() {
        if !survivor_component.invincibility_timer.finished() { return; }

        for (horror_entity, horror_transform, horror_stats) in horror_query.iter() {
            let distance = survivor_transform.translation.truncate().distance(horror_transform.translation.truncate());
            let survivor_radius = SURVIVOR_SIZE.x / 2.0;
            let horror_radius = horror_stats.size.x / 2.0;

            if distance < survivor_radius + horror_radius {
                if survivor_component.invincibility_timer.finished() {
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::SurvivorHit));
                    
                    if horror_stats.horror_type == crate::horror::HorrorType::MindLeech {
                        if let Some(debuff) = opt_mind_strain.as_mut() {
                            debuff.stacks = (debuff.stacks + 1).min(MAX_MIND_STRAIN_STACKS);
                            debuff.timer.reset(); 
                        } else {
                            commands.entity(survivor_entity).insert(MindStrainDebuff {
                                stacks: 1,
                                timer: Timer::from_seconds(MIND_STRAIN_DEBUFF_DURATION, TimerMode::Once),
                            });
                        }
                        commands.entity(horror_entity).despawn_recursive(); 
                    } else {
                        let mut damage_to_take = horror_stats.damage_on_collision;
                        if let Some(ref mut shield) = opt_active_shield {
                            if shield.amount > 0 {
                                let damage_absorbed = damage_to_take.min(shield.amount);
                                shield.amount -= damage_absorbed;
                                damage_to_take -= damage_absorbed;
                                if shield.amount <= 0 {
                                    commands.entity(survivor_entity).remove::<ActiveShield>();
                                }
                            }
                        }
                        if damage_to_take > 0 {
                            survivor_health.0 -= damage_to_take;
                        }
                    }

                    survivor_component.invincibility_timer.reset();

                    let mut rng = rand::thread_rng();
                    for item_id in survivor_component.collected_item_ids.iter() {
                        if let Some(item_def) = item_library.get_item_definition(*item_id) {
                            for effect in &item_def.effects {
                                if let ItemEffect::OnSurvivorHitRetaliate { chance, retaliation_damage, retaliation_radius, retaliation_color } = effect {
                                    if rng.gen_bool((*chance).into()) {
                                        commands.entity(survivor_entity).with_children(|parent| {
                                            parent.spawn((
                                                SpriteBundle { texture: asset_server.load("sprites/eldritch_nova_effect_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(0.1)), color: *retaliation_color, ..default() }, transform: Transform::from_xyz(0.0, 0.0, 0.3), ..default() },
                                                RetaliationNovaEffect { damage: *retaliation_damage, radius_sq: retaliation_radius.powi(2), timer: Timer::from_seconds(0.4, TimerMode::Once), already_hit_entities: Vec::new(), },
                                                Name::new("RetaliationNova"),
                                            ));
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
fn survivor_invincibility_system(time: Res<Time>, mut query: Query<(&mut Survivor, &mut Sprite, &ComponentHealth)>,) { for (mut survivor, mut sprite, health) in query.iter_mut() { if health.0 <= 0 { if sprite.color.a() != 1.0 { sprite.color.set_a(1.0); } continue; } if !survivor.invincibility_timer.finished() { survivor.invincibility_timer.tick(time.delta()); let alpha = (time.elapsed_seconds() * 20.0).sin() / 2.0 + 0.7; sprite.color.set_a(alpha.clamp(0.3, 1.0) as f32); } else { if sprite.color.a() != 1.0 { sprite.color.set_a(1.0); } } } }
fn check_survivor_death_system(survivor_query: Query<&ComponentHealth, With<Survivor>>, mut app_state_next: ResMut<NextState<AppState>>, mut sound_event_writer: EventWriter<PlaySoundEvent>, current_app_state: Res<State<AppState>>,) { if let Ok(survivor_health) = survivor_query.get_single() { if survivor_health.0 <= 0 && *current_app_state.get() == AppState::InGame { sound_event_writer.send(PlaySoundEvent(SoundEffect::MadnessConsumes)); app_state_next.set(AppState::GameOver); } } }
fn survivor_item_drop_collection_system(mut commands: Commands, survivor_query: Query<&Transform, With<Survivor>>, item_drop_query: Query<(Entity, &Transform, &ItemDrop)>, mut item_collected_event_writer: EventWriter<ItemCollectedEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { if let Ok(survivor_transform) = survivor_query.get_single() { let survivor_pos = survivor_transform.translation.truncate(); for (item_drop_entity, item_drop_transform, item_drop_data) in item_drop_query.iter() { let item_drop_pos = item_drop_transform.translation.truncate(); if survivor_pos.distance(item_drop_pos) < ITEM_COLLECTION_RADIUS { item_collected_event_writer.send(ItemCollectedEvent(item_drop_data.item_id)); sound_event_writer.send(PlaySoundEvent(SoundEffect::SoulCollect)); commands.entity(item_drop_entity).despawn_recursive(); } } } }