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
    pub auto_attack_bonus_fire_damage: u32,
    pub auto_attack_bonus_cold_damage: u32,
    pub auto_attack_bonus_lightning_damage: u32,
    pub auto_attack_poison_dps: u32,
    pub auto_attack_crit_chance: f32,
    pub auto_attack_crit_damage_multiplier: f32,
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

            auto_attack_bonus_fire_damage: 0,
            auto_attack_bonus_cold_damage: 0,
            auto_attack_bonus_lightning_damage: 0,
            auto_attack_poison_dps: 0,
            auto_attack_crit_chance: 0.0,
            auto_attack_crit_damage_multiplier: 0.0,
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

            max_dash_charges: 1,
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
        let bolt_instance = ActiveSkillInstance::new(SkillId(1)); // Removed base_glyph_slots
        initial_skills.push(bolt_instance);
    }

    let chosen_inherent_weapon_id = selected_character.0;
    let mut initial_fire_rate = 0.5;
    let mut survivor_name = "Survivor (Unknown Class)".to_string();

    if let Some(weapon_def) = weapon_library.get_weapon_definition(chosen_inherent_weapon_id) {
        // Extract base_fire_rate_secs based on AttackTypeData
        match &weapon_def.attack_data {
            AttackTypeData::StandardProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::ReturningProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::ChanneledBeam(params) => initial_fire_rate = params.tick_rate_secs, // Or a different logic for channeled
            AttackTypeData::ConeAttack(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::LobbedAoEPool(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::ChargeUpEnergyShot(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::TrailOfFire(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::ChainZap(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::PointBlankNova(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::PersistentAura(params) => initial_fire_rate = params.fire_rate_secs_placeholder, // Placeholder
            AttackTypeData::DebuffAura(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::ExpandingEnergyBomb(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::HomingDebuffProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::BouncingProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::LifestealProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::GroundTargetedAoE(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::LineDashAttack(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::OrbitingPet(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::RepositioningTether(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::BlinkStrikeProjectile(params) => initial_fire_rate = params.base_fire_rate_secs,
            AttackTypeData::LobbedBouncingMagma(params) => initial_fire_rate = params.base_fire_rate_secs,
            // Add other AttackTypeData variants as needed
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
        SanityStrain { // Changed from MindAffliction
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
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Survivor, &mut Transform, &mut Velocity, Option<&SurvivorBuffEffect>, Option<&MindStrainDebuff>)>,
    time: Res<Time>,
) {
    for (survivor, mut transform, mut velocity, buff_effect_opt, mind_strain_opt) in query.iter_mut() {
        let mut direction = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::A) { direction.x -= 1.0; }
        if keyboard_input.pressed(KeyCode::D) { direction.x += 1.0; }
        if keyboard_input.pressed(KeyCode::W) { direction.y += 1.0; }
        if keyboard_input.pressed(KeyCode::S) { direction.y -= 1.0; }

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
    mut player_query: Query<(Entity, &Transform, &Survivor, &mut SanityStrain, Option<&SurvivorBuffEffect>)>,
    mut channeling_status_query: Query<&mut crate::weapon_systems::IsChannelingComponent>, // Now mutable
    charging_comp_query: Query<&crate::weapon_systems::ChargingWeaponComponent>,
    reticule_query: Query<(&GlobalTransform, &Parent), With<crate::weapon_systems::LobbedWeaponTargetReticuleComponent>>, 
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    weapon_library: Res<AutomaticWeaponLibrary>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    for (survivor_entity, survivor_transform, survivor_stats, mut sanity_strain, buff_effect_opt) in player_query.iter_mut() {
        let weapon_def = match weapon_library.get_weapon_definition(survivor_stats.inherent_weapon_id) {
            Some(def) => def,
            None => {
                weapon_library.get_weapon_definition(AutomaticWeaponId(0))
                    .expect("Default weapon ID 0 not found in library")
            }
        };

        // --- Channeled Beam Logic ---
        if let AttackTypeData::ChanneledBeam(ref params) = weapon_def.attack_data {
            if params.is_automatic {
                // AUTOMATIC LOGIC
                if let Ok(mut channeling_comp) = channeling_status_query.get_mut(survivor_entity) {
                    // Player has IsChannelingComponent
                    if let Some(ref mut cd_timer) = channeling_comp.cooldown_timer {
                        cd_timer.tick(time.delta());
                        if !cd_timer.finished() { 
                            continue; // Still on cooldown
                        } else { 
                            channeling_comp.cooldown_timer = None; // Cooldown finished, ready to potentially start
                        }
                    }

                    if channeling_comp.beam_entity.is_some() { // Beam is active
                        if let Some(ref mut duration_timer) = channeling_comp.active_duration_timer {
                            duration_timer.tick(time.delta());
                            if duration_timer.finished() { // Duration ended
                                if let Some(beam_e) = channeling_comp.beam_entity.take() { commands.entity(beam_e).despawn_recursive(); }
                                if let Some(cd_secs) = params.cooldown_secs { // Must have cooldown for automatic cycle
                                    channeling_comp.cooldown_timer = Some(Timer::from_seconds(cd_secs, TimerMode::Once));
                                } else { 
                                    // Should not happen for automatic; error or default cooldown. For now, remove component.
                                    error!("Automatic Channeled Beam (ID: {:?}) ended duration but has no cooldown_secs defined. Removing IsChannelingComponent.", weapon_def.id);
                                    commands.entity(survivor_entity).remove::<crate::weapon_systems::IsChannelingComponent>(); 
                                }
                            }
                        } else { 
                            // No duration, implies continuous until externally stopped (should not happen for automatic)
                            error!("Automatic Channeled Beam (ID: {:?}) is active but has no active_duration_timer. This should not happen for automatic beams.", weapon_def.id);
                        }
                    } else { // Beam is NOT active (beam_entity is None), and cooldown is NOT active (or just finished)
                        // Spawn beam and set up timers (this is the "auto-activate" part)
                        let beam_aim_direction = survivor_stats.aim_direction;
                        if beam_aim_direction == Vec2::ZERO { continue; }

                        let beam_spawn_offset = beam_aim_direction * (SURVIVOR_SIZE.y / 2.0 + params.beam_width / 4.0);
                        let beam_spawn_position = survivor_transform.translation + beam_spawn_offset.extend(survivor_transform.translation.z + 0.1);
                        
                        let beam_entity_id = commands.spawn((
                            SpriteBundle { 
                                texture: asset_server.load(String::from("sprites/channeled_beam_placeholder.png")), 
                                sprite: Sprite { 
                                    custom_size: Some(Vec2::new(params.range, params.beam_width)), 
                                    color: params.beam_color, 
                                    anchor: bevy::sprite::Anchor::CenterLeft, 
                                    ..default() 
                                }, 
                                transform: Transform::from_translation(beam_spawn_position)
                                    .with_rotation(Quat::from_rotation_z(beam_aim_direction.y.atan2(beam_aim_direction.x))), 
                                ..default() 
                            },
                            crate::weapon_systems::ChanneledBeamComponent {
                                damage_per_tick: params.base_damage_per_tick + survivor_stats.auto_weapon_damage_bonus,
                                tick_timer: Timer::from_seconds(params.tick_rate_secs, TimerMode::Repeating),
                                range: params.range, 
                                width: params.beam_width, 
                                color: params.beam_color,
                                owner: survivor_entity,
                            },
                            Name::new("ChanneledBeamWeaponInstance (Automatic)"),
                        )).id();
                        channeling_comp.beam_entity = Some(beam_entity_id);
                        if let Some(max_duration) = params.max_duration_secs { // Must have duration for automatic cycle
                            channeling_comp.active_duration_timer = Some(Timer::from_seconds(max_duration, TimerMode::Once));
                        } else { 
                            // Should not happen for automatic; error or default duration
                            error!("Automatic Channeled Beam (ID: {:?}) started but has no max_duration_secs defined. Beam may not stop automatically.", weapon_def.id);
                        }
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast));
                    }
                } else { // Player does NOT have IsChannelingComponent (first activation for this weapon)
                    // Spawn beam and ADD IsChannelingComponent
                    let beam_aim_direction = survivor_stats.aim_direction;
                    if beam_aim_direction == Vec2::ZERO { continue; }
                    
                    let beam_spawn_offset = beam_aim_direction * (SURVIVOR_SIZE.y / 2.0 + params.beam_width / 4.0);
                    let beam_spawn_position = survivor_transform.translation + beam_spawn_offset.extend(survivor_transform.translation.z + 0.1);

                    let beam_entity_id = commands.spawn((
                        SpriteBundle { 
                            texture: asset_server.load(String::from("sprites/channeled_beam_placeholder.png")), 
                            sprite: Sprite { 
                                custom_size: Some(Vec2::new(params.range, params.beam_width)), 
                                color: params.beam_color, 
                                anchor: bevy::sprite::Anchor::CenterLeft, 
                                ..default() 
                            }, 
                            transform: Transform::from_translation(beam_spawn_position)
                                .with_rotation(Quat::from_rotation_z(beam_aim_direction.y.atan2(beam_aim_direction.x))), 
                            ..default() 
                        },
                        crate::weapon_systems::ChanneledBeamComponent {
                            damage_per_tick: params.base_damage_per_tick + survivor_stats.auto_weapon_damage_bonus,
                            tick_timer: Timer::from_seconds(params.tick_rate_secs, TimerMode::Repeating),
                            range: params.range, 
                            width: params.beam_width, 
                            color: params.beam_color,
                            owner: survivor_entity,
                        },
                        Name::new("ChanneledBeamWeaponInstance (Automatic)"),
                    )).id();
                    let mut new_channeling_comp = crate::weapon_systems::IsChannelingComponent {
                        beam_entity: Some(beam_entity_id), 
                        beam_params: params.clone(), 
                        active_duration_timer: None, 
                        cooldown_timer: None,
                    };
                    if let Some(max_duration) = params.max_duration_secs { 
                        new_channeling_comp.active_duration_timer = Some(Timer::from_seconds(max_duration, TimerMode::Once)); 
                    } else {
                        error!("Automatic Channeled Beam (ID: {:?}) first activation has no max_duration_secs defined. Beam may not stop automatically.", weapon_def.id);
                    }
                    commands.entity(survivor_entity).insert(new_channeling_comp);
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast));
                }
            } else {
                // MANUAL LOGIC (Existing press-and-hold logic)
                if let Ok(mut channeling_comp) = channeling_status_query.get_mut(survivor_entity) {
                    // 1. Tick and check Cooldown Timer
                    if let Some(ref mut cd_timer) = channeling_comp.cooldown_timer {
                        cd_timer.tick(time.delta());
                        if !cd_timer.finished() {
                            continue; // Still on cooldown, skip rest of beam logic for this player
                        } else {
                            channeling_comp.cooldown_timer = None; // Cooldown finished
                        }
                    }

                    // 2. Handle Active Beam (mouse pressed, beam exists)
                    if mouse_button_input.pressed(MouseButton::Left) {
                        if channeling_comp.beam_entity.is_some() { // Player is actively beaming
                            if let Some(ref mut duration_timer) = channeling_comp.active_duration_timer {
                                duration_timer.tick(time.delta());
                                if duration_timer.finished() {
                                    // Duration ended: Stop beam, start cooldown
                                    if let Some(beam_e) = channeling_comp.beam_entity.take() { // take() sets beam_entity to None
                                        commands.entity(beam_e).despawn_recursive();
                                    }
                                    if let Some(cd_secs) = params.cooldown_secs {
                                        channeling_comp.cooldown_timer = Some(Timer::from_seconds(cd_secs, TimerMode::Once));
                                    } else {
                                        // No cooldown defined, remove component if no beam and no cooldown
                                        commands.entity(survivor_entity).remove::<crate::weapon_systems::IsChannelingComponent>();
                                    }
                                    // sound_event_writer.send(PlaySoundEvent(SoundEffect::BeamEnd)); // Optional
                                    continue; // End processing for this player this frame
                                }
                            }
                            // If no duration_timer or it's not finished, beam continues.
                            continue; // Already beaming and button held, skip trying to start a new one.
                        }
                        // If mouse is pressed, but beam_entity is None (it means cooldown just finished)
                        // Fall through to "Start Channeling" logic below (for manual).
                    } else { // Mouse button is NOT pressed
                        if channeling_comp.beam_entity.is_some() {
                            // Button released: Stop beam, start cooldown
                            if let Some(beam_e) = channeling_comp.beam_entity.take() {
                                commands.entity(beam_e).despawn_recursive();
                            }
                            if let Some(cd_secs) = params.cooldown_secs {
                                channeling_comp.cooldown_timer = Some(Timer::from_seconds(cd_secs, TimerMode::Once));
                            } else {
                                // No cooldown defined, remove component
                                commands.entity(survivor_entity).remove::<crate::weapon_systems::IsChannelingComponent>();
                            }
                            // sound_event_writer.send(PlaySoundEvent(SoundEffect::BeamEnd)); // Optional
                        }
                        // If beam_entity is already None (already stopped or just finished cooldown and mouse not pressed), do nothing more here.
                        continue; // End processing for this player
                    }
                } // End of: if let Ok(mut channeling_comp) for manual

                // Start Channeling (manual: mouse pressed, no active beam, not on cooldown from above check)
                if mouse_button_input.pressed(MouseButton::Left) {
                    if channeling_status_query.get(survivor_entity).map_or(true, |comp| comp.beam_entity.is_none() && comp.cooldown_timer.is_none()) {
                        let beam_aim_direction = survivor_stats.aim_direction;
                        if beam_aim_direction == Vec2::ZERO { continue; }

                        let beam_spawn_offset = beam_aim_direction * (SURVIVOR_SIZE.y / 2.0 + params.beam_width / 4.0);
                        let beam_spawn_position = survivor_transform.translation + beam_spawn_offset.extend(survivor_transform.translation.z + 0.1);
                        
                        let beam_entity_id = commands.spawn((
                            SpriteBundle { 
                                texture: asset_server.load(String::from("sprites/channeled_beam_placeholder.png")), 
                                sprite: Sprite { 
                                    custom_size: Some(Vec2::new(params.range, params.beam_width)), 
                                    color: params.beam_color, 
                                    anchor: bevy::sprite::Anchor::CenterLeft, 
                                    ..default() 
                                }, 
                                transform: Transform::from_translation(beam_spawn_position)
                                    .with_rotation(Quat::from_rotation_z(beam_aim_direction.y.atan2(beam_aim_direction.x))), 
                                ..default() 
                            },
                            crate::weapon_systems::ChanneledBeamComponent {
                                damage_per_tick: params.base_damage_per_tick + survivor_stats.auto_weapon_damage_bonus,
                                tick_timer: Timer::from_seconds(params.tick_rate_secs, TimerMode::Repeating),
                                range: params.range, 
                                width: params.beam_width, 
                                color: params.beam_color,
                                owner: survivor_entity,
                            },
                            Name::new("ChanneledBeamWeaponInstance (Manual)"),
                        )).id();

                        let mut new_channeling_comp = crate::weapon_systems::IsChannelingComponent {
                            beam_entity: Some(beam_entity_id),
                            beam_params: params.clone(),
                            active_duration_timer: None,
                            cooldown_timer: None,
                        };
                        if let Some(max_duration) = params.max_duration_secs {
                            new_channeling_comp.active_duration_timer = Some(Timer::from_seconds(max_duration, TimerMode::Once));
                        }
                        commands.entity(survivor_entity).insert(new_channeling_comp);
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast));
                    }
                }
            }
            continue; // IMPORTANT: Skip the rest of the timed weapon logic
        }
        // --- End of Channeled Beam Logic ---

        // --- Handle ChargeUpEnergyShot ---
        if let AttackTypeData::ChargeUpEnergyShot(ref shot_params) = weapon_def.attack_data {
            // Manually tick the cooldown timer since we skip the generic logic below
            sanity_strain.fire_timer.tick(time.delta());
            let already_charging = charging_comp_query.get(survivor_entity).is_ok();
            info!(
                "ChargeUp: Attempting. FireTimerFinished: {}, Remaining: {:.2}s, AlreadyCharging: {}",
                sanity_strain.fire_timer.finished(),
                sanity_strain.fire_timer.remaining_secs(),
                already_charging
            );

            if mouse_button_input.just_pressed(MouseButton::Left) {
                info!("ChargeUp: Mouse Just Pressed.");
                let is_fire_timer_finished = sanity_strain.fire_timer.finished();
                // is_already_charging is 'already_charging' from above
                
                if !is_fire_timer_finished {
                    info!("ChargeUp: Blocked - Fire timer not finished (Remaining: {:.2}s).", sanity_strain.fire_timer.remaining_secs());
                }
                if already_charging { // Use the variable captured before the if block
                    info!("ChargeUp: Blocked - Already has ChargingWeaponComponent.");
                }

                if is_fire_timer_finished && !already_charging { // Use the variable
                    if shot_params.charge_levels.is_empty() {
                        info!("ChargeUp: Blocked - No charge levels defined for this weapon.");
                        continue; 
                    }
                    info!("ChargeUp: SUCCESS - Adding ChargingWeaponComponent.");
                    commands.entity(survivor_entity).insert(crate::weapon_systems::ChargingWeaponComponent {
                        weapon_id: weapon_def.id, // Store the actual weapon_id
                        charge_timer: Timer::from_seconds(shot_params.charge_levels[0].charge_time_secs.max(0.01), TimerMode::Once), // Ensure non-zero duration
                        current_charge_level_index: 0,
                        is_actively_charging: true,
                    });
                    // Reset and tick sanity_strain.fire_timer using shot_params.base_fire_rate_secs to start the "post-shot cooldown".
                    sanity_strain.fire_timer.set_duration(Duration::from_secs_f32(shot_params.base_fire_rate_secs));
                    sanity_strain.fire_timer.reset(); 
                    // (Optional: Play charge start sound)
                }
            }

            if mouse_button_input.just_released(MouseButton::Left) {
                if let Ok(charging_comp) = charging_comp_query.get(survivor_entity) {
                    if charging_comp.is_actively_charging { // Field exists on ChargingWeaponComponent
                        let current_level_index = charging_comp.current_charge_level_index;
                        if current_level_index < shot_params.charge_levels.len() {
                            let level_params = &shot_params.charge_levels[current_level_index];
                            let projectile_damage = level_params.projectile_damage + survivor_stats.auto_weapon_damage_bonus;
                            let projectile_speed = level_params.projectile_speed * survivor_stats.auto_weapon_projectile_speed_multiplier;
                            let projectile_piercing = level_params.piercing + survivor_stats.auto_weapon_piercing_bonus;
                            let sprite_path = level_params.projectile_sprite_path_override.as_deref().unwrap_or(&shot_params.base_projectile_sprite_path);
                            
                            crate::automatic_projectiles::spawn_automatic_projectile(
                                &mut commands, &asset_server, survivor_entity, survivor_transform.translation, survivor_stats.aim_direction,
                                projectile_damage, projectile_speed, projectile_piercing, weapon_def.id,
                                sprite_path, level_params.projectile_size, shot_params.base_projectile_color, shot_params.projectile_lifetime_secs,
                                None, None, None, None, None, None // No special params like bounce, lifesteal, tether for basic version
                            );
                            // Explosion logic deferred as per subtask instructions.
                        }
                        commands.entity(survivor_entity).remove::<crate::weapon_systems::ChargingWeaponComponent>();
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); // Or a specific shot fire sound
                    }
                }
            }
            continue; // Important: prevent falling into other weapon logic
        }
        // --- End of ChargeUpEnergyShot ---

        // --- Standard Timed Weapon Logic ---
        let mut effective_fire_rate_secs = sanity_strain.base_fire_rate_secs;
        if let Some(buff) = buff_effect_opt {
            effective_fire_rate_secs /= 1.0 + buff.fire_rate_multiplier_bonus;
        }

        let new_duration = Duration::from_secs_f32(effective_fire_rate_secs.max(0.05));
        if sanity_strain.fire_timer.duration() != new_duration {
            sanity_strain.fire_timer.set_duration(new_duration);
            sanity_strain.fire_timer.reset(); // Reset the timer so it's fresh for the new duration/weapon
        }
        sanity_strain.fire_timer.tick(time.delta());

        if sanity_strain.fire_timer.just_finished() {
            if survivor_stats.aim_direction != Vec2::ZERO {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast));

                match &weapon_def.attack_data {
                    AttackTypeData::StandardProjectile(params) => {
                        let current_damage = params.base_damage + survivor_stats.auto_weapon_damage_bonus;
                        let effective_projectile_lifetime_secs = params.projectile_lifetime_secs * survivor_stats.auto_attack_projectile_duration_multiplier;
                        let current_speed = params.base_projectile_speed * survivor_stats.auto_weapon_projectile_speed_multiplier;
                        let current_piercing = params.base_piercing + survivor_stats.auto_weapon_piercing_bonus;
                        let total_projectiles = 1 + params.additional_projectiles + survivor_stats.auto_weapon_additional_projectiles_bonus;

                        let base_angle = survivor_stats.aim_direction.y.atan2(survivor_stats.aim_direction.x);
                        let spread_arc_degrees = PROJECTILE_SPREAD_ANGLE_DEGREES * (total_projectiles.saturating_sub(1)) as f32;
                        let start_angle_offset_rad = if total_projectiles > 1 { -spread_arc_degrees.to_radians() / 2.0 } else { 0.0 };

                        for i in 0..total_projectiles {
                            let angle_offset_rad = if total_projectiles > 1 {
                                let step = if total_projectiles > 1 { spread_arc_degrees.to_radians() / (total_projectiles - 1) as f32 } else { 0.0 };
                                start_angle_offset_rad + (i as f32 * step)
                            } else {
                                0.0
                            };
                            let projectile_direction = Vec2::from_angle(base_angle + angle_offset_rad);

                            spawn_automatic_projectile( // This is crate::automatic_projectiles::spawn_automatic_projectile
                                &mut commands,
                                &asset_server,
                                survivor_entity,
                                survivor_transform.translation,
                                projectile_direction,
                                current_damage,
                                current_speed,
                                current_piercing,
                                weapon_def.id,
                                &params.projectile_sprite_path,
                                params.projectile_size,
                                params.projectile_color,
                                effective_projectile_lifetime_secs,
                                None, 
                                None, 
                                None, 
                                None, 
                                None, 
                                None, 
                            );
                        }
                    }
                    AttackTypeData::RepositioningTether(params) => {
                        crate::weapon_systems::spawn_actual_tether_projectile(
                            &mut commands,
                            &asset_server,
                            survivor_entity,
                            survivor_stats.aim_direction,
                            params, // These are &crate::items::RepositioningTetherParams
                            weapon_def.id,
                            survivor_transform,
                        );
                    }
                    AttackTypeData::PersistentAura(_params) => { // params might be unused here
                        // Log an info message, as persistent auras are typically managed by their own systems.
                        // The survivor_casting_system doesn't "fire" them on each timer tick.
                        // Their presence is usually detected by a dedicated aura management system.
                        info!(
                            "PersistentAura weapon type ({:?}) equipped. Actual aura management is handled by a dedicated system.",
                            weapon_def.name
                        );
                        // No action taken by survivor_casting_system for this type.
                    }
                    AttackTypeData::OrbitingPet(_params) => {
                        // Orbiting pets are managed by manage_player_orbs_system and orbiting_pet_behavior_system.
                        // survivor_casting_system doesn't "fire" them on each timer tick in the traditional sense.
                        info!(
                            "OrbitingPet weapon type ({:?}) equipped. Actual pet management is handled by dedicated systems.",
                            weapon_def.name
                        );
                        // No action taken by survivor_casting_system for this type.
                    }
                    AttackTypeData::LobbedAoEPool(params) => {
                        let mut final_target_pos = survivor_transform.translation + survivor_stats.aim_direction.extend(0.0) * (params.projectile_speed * 1.5); // Default/fallback

                        for (reticule_g_transform, parent) in reticule_query.iter() {
                            if parent.get() == survivor_entity {
                                final_target_pos = reticule_g_transform.translation();
                                break;
                            }
                        }

                        crate::weapon_systems::spawn_lobbed_aoe_pool_attack(
                            &mut commands,
                            &asset_server,
                            survivor_entity,
                            survivor_transform,
                            survivor_stats.aim_direction, // Pass original aim_direction, spawn func might ignore it or use for fallback
                            params, // This is &LobbedAoEPoolParams
                            weapon_def.id,
                            final_target_pos, // Pass the determined target position
                        );
                    }
                    AttackTypeData::LifestealProjectile(params) => {
                        let current_damage = params.base_damage + survivor_stats.auto_weapon_damage_bonus;
                        let effective_projectile_lifetime_secs = params.projectile_lifetime_secs * survivor_stats.auto_attack_projectile_duration_multiplier;
                        let current_speed = params.projectile_speed * survivor_stats.auto_weapon_projectile_speed_multiplier;
                        let current_piercing = params.piercing + survivor_stats.auto_weapon_piercing_bonus;
                        
                        // Assume base additional_projectiles for Lifesteal is 0, but player can have bonus
                        let total_projectiles = 1 + survivor_stats.auto_weapon_additional_projectiles_bonus;

                        let base_angle = survivor_stats.aim_direction.y.atan2(survivor_stats.aim_direction.x);
                        let spread_arc_degrees = PROJECTILE_SPREAD_ANGLE_DEGREES * (total_projectiles.saturating_sub(1)) as f32;
                        let start_angle_offset_rad = if total_projectiles > 1 { -spread_arc_degrees.to_radians() / 2.0 } else { 0.0 };

                        for i in 0..total_projectiles {
                            let angle_offset_rad = if total_projectiles > 1 {
                                let step = if total_projectiles > 1 { spread_arc_degrees.to_radians() / (total_projectiles - 1) as f32 } else { 0.0 };
                                start_angle_offset_rad + (i as f32 * step)
                            } else {
                                0.0
                            };
                            let projectile_direction = Vec2::from_angle(base_angle + angle_offset_rad);

                            spawn_automatic_projectile(
                                &mut commands,
                                &asset_server,
                                survivor_entity,
                                survivor_transform.translation,
                                projectile_direction,
                                current_damage,
                                current_speed,
                                current_piercing,
                                weapon_def.id,
                                &params.projectile_sprite_path,
                                params.projectile_size,
                                params.projectile_color,
                                effective_projectile_lifetime_secs,
                                // Corrected order for optional arguments:
                                None,                              // opt_max_bounces (LifestealProjectiles don't have inherent bounce stats)
                                None,                              // opt_dmg_loss_mult
                                None,                              // opt_speed_loss_mult
                                Some(params.lifesteal_percentage), // opt_lifesteal_percentage
                                None,                              // opt_tether_params
                                None,                              // opt_blink_params
                            );
                        }
                    }
                    AttackTypeData::BlinkStrikeProjectile(params) => {
                        crate::weapon_systems::spawn_blink_strike_projectile_attack(
                            &mut commands,
                            &asset_server,
                            survivor_entity,
                            params, // These are &crate::items::BlinkStrikeProjectileParams
                            survivor_transform,
                            survivor_stats.aim_direction,
                            weapon_def.id,
                        );
                    }
                    AttackTypeData::LobbedBouncingMagma(params) => {
                        crate::weapon_systems::spawn_magma_ball_attack(
                            &mut commands,
                            &asset_server,
                            params,
                            survivor_transform,
                            survivor_stats.aim_direction,
                            weapon_def.id,
                            survivor_entity,
                            survivor_stats,
                        );
                    }
                    _ => {
                        error!(
                            "Weapon {:?} (ID: {}) has AttackTypeData variant {:?} which is not yet handled by survivor_casting_system's timed logic.",
                            weapon_def.name,
                            weapon_def.id.0,
                            weapon_def.attack_data 
                        );
                    }
                }
            }
        }
        // --- End of Standard Timed Weapon Logic ---
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