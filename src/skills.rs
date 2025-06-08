// src/skills.rs
use bevy::prelude::*;
use std::time::Duration;
use crate::{
    survivor::{Survivor, SURVIVOR_SIZE},
    game::AppState,
    components::{Velocity, Damage, Lifetime, Health},
    horror::Horror,
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Serialize, Deserialize)]
pub struct SkillId(pub u32);

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SkillEffectType {
    Projectile {
        base_damage: i32,
        speed: f32,
        size: Vec2,
        color: Color,
        lifetime_secs: f32,
        piercing: u32,
    },
    AreaOfEffect {
        base_damage_per_tick: i32,
        base_radius: f32,
        tick_interval_secs: f32,
        duration_secs: f32,
        color: Color,
    },
    SurvivorBuff {
        speed_multiplier_bonus: f32,
        fire_rate_multiplier_bonus: f32,
        duration_secs: f32,
    },
    SummonSentry {
        sentry_damage_per_tick: i32,
        sentry_radius: f32,
        sentry_tick_interval_secs: f32,
        sentry_duration_secs: f32,
        sentry_color: Color,
    },
    FreezingNova {
        damage: i32,
        radius: f32,
        nova_duration_secs: f32,
        slow_multiplier: f32,
        slow_duration_secs: f32,
        color: Color,
    },
    TemporaryShield {
        amount: i32,
        duration_secs: f32,
    },
    ChanneledBeam { 
        base_damage_per_tick: i32,
        tick_interval_secs: f32,
        duration_secs: f32,
        range: f32,
        width: f32,
        color: Color,
    },
}

#[derive(Debug, Clone, Reflect)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub base_cooldown: Duration,
    pub effect: SkillEffectType,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct ActiveSkillInstance {
    pub definition_id: SkillId,
    pub current_cooldown: Duration,
    pub current_level: u32,
    pub flat_damage_bonus: i32,
    pub cooldown_multiplier: f32,
    pub aoe_radius_multiplier: f32, 
}

impl ActiveSkillInstance {
    pub fn new(definition_id: SkillId ) -> Self {
        Self {
            definition_id,
            current_cooldown: Duration::ZERO,
            current_level: 1,
            flat_damage_bonus: 0,
            cooldown_multiplier: 1.0,
            aoe_radius_multiplier: 1.0,
        }
    }
    pub fn tick_cooldown(&mut self, delta: Duration) { if self.current_cooldown > Duration::ZERO { self.current_cooldown = self.current_cooldown.saturating_sub(delta); } }
    pub fn is_ready(&self) -> bool { self.current_cooldown == Duration::ZERO }
    pub fn trigger(&mut self, base_cooldown: Duration, effective_cooldown_multiplier: f32) {
        let modified_cooldown_secs = base_cooldown.as_secs_f32() * effective_cooldown_multiplier;
        self.current_cooldown = Duration::from_secs_f32(modified_cooldown_secs.max(0.1));
    }
}

#[derive(Component)]
pub struct SkillProjectile {
    pub skill_id: SkillId,
    pub piercing_left: u32,
    pub bounces_left: u32,
    pub already_hit_by_this_projectile: Vec<Entity>,
}

#[derive(Component)] pub struct ActiveSkillAoEEffect { pub skill_id: SkillId, pub actual_damage_per_tick: i32, pub actual_radius_sq: f32, pub tick_timer: Timer, pub lifetime_timer: Timer, pub already_hit_this_tick: Vec<Entity>, }
#[derive(Component, Debug)] pub struct SurvivorBuffEffect { pub speed_multiplier_bonus: f32, pub fire_rate_multiplier_bonus: f32, pub duration_timer: Timer, }

#[derive(Component, Debug, Reflect, Default)] #[reflect(Component)]
pub struct FreezingNovaEffect { pub damage: i32, pub radius_sq: f32, pub lifetime_timer: Timer, pub slow_multiplier: f32, pub slow_duration_secs: f32, pub already_hit_entities: Vec<Entity>, }

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct ActiveShield {
    pub amount: i32,
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct ActiveChanneledBeamComponent {
    pub skill_id: SkillId,
    pub direction: Vec2, 
    pub range: f32,
    pub width: f32,
    pub actual_damage_per_tick: i32,
    pub tick_timer: Timer,      
    pub lifetime_timer: Timer,  
    pub already_hit_this_tick: Vec<Entity>, 
    pub color: Color, 
}


#[derive(Resource, Default, Reflect)] #[reflect(Resource)]
pub struct SkillLibrary { pub skills: Vec<SkillDefinition>, }
impl SkillLibrary { pub fn get_skill_definition(&self, id: SkillId) -> Option<&SkillDefinition> { self.skills.iter().find(|def| def.id == id) } }

pub struct SkillsPlugin;
impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app .register_type::<SkillId>() .register_type::<SkillEffectType>() .register_type::<SkillDefinition>() .register_type::<ActiveSkillInstance>() .register_type::<SkillLibrary>()
            .register_type::<FreezingNovaEffect>()
            .register_type::<ActiveShield>()
            .init_resource::<SkillLibrary>()
            .add_systems(Startup, populate_skill_library)
            .add_systems(Update, (
                active_skill_cooldown_recharge_system,
                survivor_skill_input_system,
                skill_projectile_lifetime_system,
                skill_projectile_collision_system,
                active_skill_aoe_system,
                survivor_buff_management_system,
                freezing_nova_effect_damage_system,
                active_shield_timer_system,
                active_channeled_beam_system, 
            ).chain().run_if(in_state(AppState::InGame)) );
    }
}

fn populate_skill_library(mut library: ResMut<SkillLibrary>) {
    library.skills.push(SkillDefinition { id: SkillId(1), name: "Eldritch Bolt".to_string(), description: "Fires a bolt of arcane energy.".to_string(), base_cooldown: Duration::from_secs_f32(1.5), effect: SkillEffectType::Projectile { base_damage: 25, speed: 650.0, size: Vec2::new(12.0, 28.0), color: Color::rgb(0.6, 0.1, 0.9), lifetime_secs: 2.5, piercing: 0, }}); 
    library.skills.push(SkillDefinition { id: SkillId(2), name: "Mind Shatter".to_string(), description: "Unleashes a short-range psychic burst in a wide arc.".to_string(), base_cooldown: Duration::from_secs(4), effect: SkillEffectType::AreaOfEffect { base_damage_per_tick: 35, base_radius: 175.0, tick_interval_secs: 0.1, duration_secs: 0.2, color: Color::rgba(0.8, 0.2, 1.0, 0.7), }}); 
    library.skills.push(SkillDefinition { id: SkillId(3), name: "Void Lance".to_string(), description: "Projects a slow but potent lance of void energy that pierces foes.".to_string(), base_cooldown: Duration::from_secs_f32(2.5), effect: SkillEffectType::Projectile { base_damage: 40, speed: 400.0, size: Vec2::new(10.0, 40.0), color: Color::rgb(0.1, 0.0, 0.2), lifetime_secs: 3.0, piercing: 2, }}); 
    library.skills.push(SkillDefinition { id: SkillId(4), name: "Fleeting Agility".to_string(), description: "Briefly enhance your speed and reflexes.".to_string(), base_cooldown: Duration::from_secs(20), effect: SkillEffectType::SurvivorBuff { speed_multiplier_bonus: 0.30, fire_rate_multiplier_bonus: 0.25, duration_secs: 5.0, }}); 
    library.skills.push(SkillDefinition { id: SkillId(5), name: "Glacial Nova".to_string(), description: "Emits a chilling nova, damaging and slowing nearby foes.".to_string(), base_cooldown: Duration::from_secs(10), effect: SkillEffectType::FreezingNova { damage: 20, radius: 200.0, nova_duration_secs: 0.5, slow_multiplier: 0.5, slow_duration_secs: 3.0, color: Color::rgba(0.5, 0.8, 1.0, 0.6), }}); 
    library.skills.push(SkillDefinition { id: SkillId(6), name: "Psychic Sentry".to_string(), description: "Summons a stationary sentry that pulses with psychic energy.".to_string(), base_cooldown: Duration::from_secs(18), effect: SkillEffectType::SummonSentry { sentry_damage_per_tick: 15, sentry_radius: 100.0, sentry_tick_interval_secs: 0.75, sentry_duration_secs: 8.0, sentry_color: Color::rgba(0.2, 0.7, 0.9, 0.5), }}); 
    library.skills.push(SkillDefinition { id: SkillId(7), name: "Ethereal Ward".to_string(), description: "Briefly manifest an ethereal shield that absorbs incoming damage.".to_string(), base_cooldown: Duration::from_secs(25), effect: SkillEffectType::TemporaryShield { amount: 50, duration_secs: 5.0, }}); 
    library.skills.push(SkillDefinition { 
        id: SkillId(8), 
        name: "Gaze of the Abyss".to_string(), 
        description: "Channel a beam of dark energy, damaging foes in its path.".to_string(), 
        base_cooldown: Duration::from_secs(15), 
        effect: SkillEffectType::ChanneledBeam { 
            base_damage_per_tick: 8, 
            tick_interval_secs: 0.20, 
            duration_secs: 2.5, 
            range: 400.0, 
            width: 25.0, 
            color: Color::rgba(0.5, 0.0, 0.7, 0.7), 
        }
    });
}

fn active_skill_cooldown_recharge_system(time: Res<Time>, mut player_query: Query<&mut Survivor>,) { if let Ok(mut player) = player_query.get_single_mut() { for skill_instance in player.equipped_skills.iter_mut() { skill_instance.tick_cooldown(time.delta()); } } }

fn survivor_skill_input_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button_input: Res<Input<MouseButton>>, // Changed ButtonInput to Input
    keyboard_input: Res<Input<KeyCode>>, // Changed ButtonInput to Input
    mut player_query: Query<(Entity, &mut Survivor, &Transform)>,
    skill_library: Res<SkillLibrary>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_entity, mut player, player_transform)) = player_query.get_single_mut() {
        let mut skill_to_trigger_idx: Option<usize> = None;
        if mouse_button_input.just_pressed(MouseButton::Right) { skill_to_trigger_idx = Some(0); }
        else if keyboard_input.just_pressed(KeyCode::Key1) { skill_to_trigger_idx = Some(0); }     // Digit1 -> Key1
        else if keyboard_input.just_pressed(KeyCode::Key2) { skill_to_trigger_idx = Some(1); }     // Digit2 -> Key2
        else if keyboard_input.just_pressed(KeyCode::Key3) { skill_to_trigger_idx = Some(2); }     // Digit3 -> Key3
        else if keyboard_input.just_pressed(KeyCode::E) { skill_to_trigger_idx = Some(3); }      // KeyE -> E
        else if keyboard_input.just_pressed(KeyCode::R) { skill_to_trigger_idx = Some(4); }      // KeyR -> R

        if let Some(idx) = skill_to_trigger_idx { if idx >= player.equipped_skills.len() { return; } let current_aim_direction = player.aim_direction; let skill_instance_snapshot = player.equipped_skills[idx].clone();
            if skill_instance_snapshot.is_ready() { if let Some(skill_def) = skill_library.get_skill_definition(skill_instance_snapshot.definition_id) {
                let mut effect_was_triggered = false;
                let mut projectile_damage = 0;
                let projectile_bounces: u32 = 0; 
                let mut effective_projectile_lifetime_secs = 0.0;
                let mut aoe_damage_per_tick = 0;
                let mut effective_aoe_radius = 0.0;
                let mut sentry_damage_val = 0;
                let mut effective_sentry_radius = 0.0;
                let mut nova_damage_val = 0;
                let mut effective_nova_radius = 0.0;
                let mut shield_amount = 0;
                let mut beam_damage_per_tick = 0;
                let mut beam_color_val = Color::WHITE; 

                let mut effective_cooldown_multiplier = skill_instance_snapshot.cooldown_multiplier;
                let mut effective_aoe_radius_multiplier = skill_instance_snapshot.aoe_radius_multiplier; 

                match &skill_def.effect {
                    SkillEffectType::Projectile { base_damage, lifetime_secs, .. } => { 
                        projectile_damage = base_damage + skill_instance_snapshot.flat_damage_bonus;
                        effective_projectile_lifetime_secs = *lifetime_secs;
                    }
                    SkillEffectType::AreaOfEffect { base_damage_per_tick, base_radius, .. } => {
                        aoe_damage_per_tick = base_damage_per_tick + skill_instance_snapshot.flat_damage_bonus;
                        effective_aoe_radius = *base_radius;
                    },
                    SkillEffectType::SummonSentry { sentry_damage_per_tick: sdpt, sentry_radius: sr, ..} => {
                        sentry_damage_val = sdpt + skill_instance_snapshot.flat_damage_bonus;
                        effective_sentry_radius = *sr;
                    }
                    SkillEffectType::FreezingNova { damage, radius, .. } => {
                        nova_damage_val = damage + skill_instance_snapshot.flat_damage_bonus;
                        effective_nova_radius = *radius;
                    }
                    SkillEffectType::TemporaryShield { amount, .. } => {
                        shield_amount = *amount + skill_instance_snapshot.flat_damage_bonus;
                    }
                    SkillEffectType::ChanneledBeam { base_damage_per_tick, color, ..} => { 
                        beam_damage_per_tick = base_damage_per_tick + skill_instance_snapshot.flat_damage_bonus;
                        beam_color_val = *color;
                    }
                    SkillEffectType::SurvivorBuff { .. } => {}
                }
                
                effective_cooldown_multiplier = effective_cooldown_multiplier.max(0.1);
                effective_aoe_radius_multiplier = effective_aoe_radius_multiplier.max(0.1); 

                if matches!(skill_def.effect, SkillEffectType::AreaOfEffect { .. }) { effective_aoe_radius *= effective_aoe_radius_multiplier; }
                if matches!(skill_def.effect, SkillEffectType::SummonSentry { .. }) { effective_sentry_radius *= effective_aoe_radius_multiplier; }
                if matches!(skill_def.effect, SkillEffectType::FreezingNova { .. }) { effective_nova_radius *= effective_aoe_radius_multiplier; }


                match &skill_def.effect {
                    SkillEffectType::Projectile { speed, size, color, piercing,.. } => { 
                        if current_aim_direction != Vec2::ZERO {
                            let projectile_spawn_position = player_transform.translation + current_aim_direction.extend(0.0) * (SURVIVOR_SIZE.y / 2.0 + size.y / 2.0);
                            commands.spawn((
                                SpriteBundle { texture: asset_server.load("sprites/eldritch_bolt_placeholder.png"), sprite: Sprite { custom_size: Some(*size), color: *color, ..default()}, transform: Transform::from_translation(projectile_spawn_position) .with_rotation(Quat::from_rotation_z(current_aim_direction.y.atan2(current_aim_direction.x))), ..default() },
                                SkillProjectile { skill_id: skill_def.id, piercing_left: *piercing, bounces_left: projectile_bounces, already_hit_by_this_projectile: Vec::new()}, 
                                Velocity(current_aim_direction * *speed),
                                Damage(projectile_damage),
                                Lifetime { timer: Timer::from_seconds(effective_projectile_lifetime_secs, TimerMode::Once) },
                                Name::new(format!("SkillProjectile_{}", skill_def.name)),
                            ));
                            effect_was_triggered = true;
                        }
                    }
                    SkillEffectType::AreaOfEffect { tick_interval_secs, duration_secs, color, .. } => {
                        if skill_def.id == SkillId(2) { 
                            let num_projectiles = 5;
                            let spread_angle_rad = 60.0f32.to_radians();
                            let angle_step = spread_angle_rad / (num_projectiles -1) as f32;
                            let base_angle = current_aim_direction.y.atan2(current_aim_direction.x) - spread_angle_rad / 2.0; // Changed to_angle()
                            for i in 0..num_projectiles {
                                let angle = base_angle + angle_step * i as f32;
                                let direction = Vec2::new(angle.cos(), angle.sin());
                                let projectile_spawn_position = player_transform.translation + direction.extend(0.0) * (SURVIVOR_SIZE.y / 2.0 + 10.0 / 2.0);
                                let mind_shatter_fragment_damage = 15 + skill_instance_snapshot.flat_damage_bonus; 
                                commands.spawn((
                                    SpriteBundle { texture: asset_server.load("sprites/mind_shatter_fragment_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::new(10.0, 10.0)), color: *color, ..default()}, transform: Transform::from_translation(projectile_spawn_position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))), ..default()},
                                    SkillProjectile { skill_id: skill_def.id, piercing_left: 0, bounces_left: 0, already_hit_by_this_projectile: Vec::new(),}, 
                                    Velocity(direction * 400.0), Damage(mind_shatter_fragment_damage), Lifetime { timer: Timer::from_seconds(0.4, TimerMode::Once) }, Name::new(format!("MindShatterFragment_{}", i)),
                                ));
                            }
                            effect_was_triggered = true;
                        } else { 
                            let aoe_spawn_position = player_transform.translation;
                            commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/generic_aoe_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(effective_aoe_radius * 2.0)), color: *color, ..default()}, transform: Transform::from_translation(aoe_spawn_position.truncate().extend(0.2)), ..default() }, ActiveSkillAoEEffect { skill_id: skill_def.id, actual_damage_per_tick: aoe_damage_per_tick, actual_radius_sq: effective_aoe_radius.powi(2), tick_timer: Timer::from_seconds(*tick_interval_secs, TimerMode::Repeating), lifetime_timer: Timer::from_seconds(*duration_secs, TimerMode::Once), already_hit_this_tick: Vec::new(), }, Name::new(format!("SkillAoE_{}", skill_def.name)), )); effect_was_triggered = true;
                        }
                    }
                    SkillEffectType::SurvivorBuff { speed_multiplier_bonus, fire_rate_multiplier_bonus, duration_secs } => { commands.entity(player_entity).insert(SurvivorBuffEffect { speed_multiplier_bonus: *speed_multiplier_bonus, fire_rate_multiplier_bonus: *fire_rate_multiplier_bonus, duration_timer: Timer::from_seconds(*duration_secs, TimerMode::Once), }); effect_was_triggered = true; }
                    SkillEffectType::SummonSentry { sentry_tick_interval_secs, sentry_duration_secs, sentry_color, .. } => { let sentry_spawn_position = player_transform.translation.truncate().extend(0.15); commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/psychic_sentry_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(effective_sentry_radius * 0.5)), color: *sentry_color, ..default() }, transform: Transform::from_translation(sentry_spawn_position), ..default() }, ActiveSkillAoEEffect { skill_id: skill_def.id, actual_damage_per_tick: sentry_damage_val, actual_radius_sq: effective_sentry_radius.powi(2), tick_timer: Timer::from_seconds(*sentry_tick_interval_secs, TimerMode::Repeating), lifetime_timer: Timer::from_seconds(*sentry_duration_secs, TimerMode::Once), already_hit_this_tick: Vec::new(), }, Name::new("PsychicSentry"), )); effect_was_triggered = true; }
                    SkillEffectType::FreezingNova { nova_duration_secs, slow_multiplier, slow_duration_secs, color, .. } => { let nova_spawn_position = player_transform.translation; commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/frost_nova_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(0.1)), color: *color, ..default() }, transform: Transform::from_translation(nova_spawn_position.truncate().extend(0.25)), ..default() }, FreezingNovaEffect { damage: nova_damage_val, radius_sq: effective_nova_radius.powi(2), lifetime_timer: Timer::from_seconds(*nova_duration_secs, TimerMode::Once), slow_multiplier: *slow_multiplier, slow_duration_secs: *slow_duration_secs, already_hit_entities: Vec::new(), }, Name::new("GlacialNovaEffect"), )); effect_was_triggered = true; sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); }
                    SkillEffectType::TemporaryShield { duration_secs, .. } => {
                        commands.entity(player_entity).insert(ActiveShield {
                            amount: shield_amount,
                            timer: Timer::from_seconds(*duration_secs, TimerMode::Once),
                        });
                        effect_was_triggered = true;
                    }
                    SkillEffectType::ChanneledBeam { tick_interval_secs, duration_secs, range, width, .. } => { 
                        if current_aim_direction != Vec2::ZERO {
                            let beam_spawn_offset = current_aim_direction * (SURVIVOR_SIZE.y / 2.0 + 1.0); 
                            let beam_spawn_position = player_transform.translation + beam_spawn_offset.extend(0.0);
                            
                            let beam_range = *range * effective_aoe_radius_multiplier;
                            let beam_width = *width * effective_aoe_radius_multiplier;

                            commands.spawn((
                                SpriteBundle {
                                    texture: asset_server.load("sprites/gaze_of_the_abyss_beam_placeholder.png"), 
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::new(beam_range, beam_width)), 
                                        color: beam_color_val, 
                                        anchor: bevy::sprite::Anchor::CenterLeft, 
                                        ..default()
                                    },
                                    transform: Transform::from_translation(beam_spawn_position.truncate().extend(0.18)) 
                                        .with_rotation(Quat::from_rotation_z(current_aim_direction.y.atan2(current_aim_direction.x))),
                                    ..default()
                                },
                                ActiveChanneledBeamComponent {
                                    skill_id: skill_def.id,
                                    direction: current_aim_direction,
                                    range: beam_range,
                                    width: beam_width,
                                    actual_damage_per_tick: beam_damage_per_tick,
                                    tick_timer: Timer::from_seconds(*tick_interval_secs, TimerMode::Repeating),
                                    lifetime_timer: Timer::from_seconds(*duration_secs, TimerMode::Once),
                                    already_hit_this_tick: Vec::new(),
                                    color: beam_color_val, 
                                },
                                Name::new("GazeOfTheAbyssBeam"),
                            ));
                            effect_was_triggered = true;
                            sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); 
                        }
                    }
                }
                if effect_was_triggered {
                    if let Some(skill_instance_mut) = player.equipped_skills.get_mut(idx) {
                        skill_instance_mut.trigger(skill_def.base_cooldown, effective_cooldown_multiplier);
                    }
                }
            }
        }
        }
    }
}

fn active_shield_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut shield_query: Query<(Entity, &mut ActiveShield)>,
) {
    for (entity, mut shield) in shield_query.iter_mut() {
        shield.timer.tick(time.delta());
        if shield.timer.finished() {
            commands.entity(entity).remove::<ActiveShield>();
        }
    }
}

fn survivor_buff_management_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut SurvivorBuffEffect)>,) { for (entity, mut buff) in query.iter_mut() { buff.duration_timer.tick(time.delta()); if buff.duration_timer.finished() { commands.entity(entity).remove::<SurvivorBuffEffect>(); } } }
fn skill_projectile_lifetime_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime), With<SkillProjectile>>,) { for (entity, mut lifetime) in query.iter_mut() { lifetime.timer.tick(time.delta()); if lifetime.timer.just_finished() { commands.entity(entity).despawn_recursive(); } } }

fn skill_projectile_collision_system(
    mut commands: Commands,
    mut skill_projectile_query: Query<(Entity, &GlobalTransform, &Damage, &mut SkillProjectile, &Sprite)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    skill_library: Res<SkillLibrary>,
    _player_query: Query<&Survivor>, 
) {
    for (proj_entity, proj_g_transform, proj_damage, mut skill_projectile_data, proj_sprite) in skill_projectile_query.iter_mut() {
        if skill_projectile_data.already_hit_by_this_projectile.len() > (skill_projectile_data.piercing_left + skill_projectile_data.bounces_left + 5) as usize { commands.entity(proj_entity).despawn_recursive(); continue; }
        let proj_pos = proj_g_transform.translation().truncate();
        let proj_radius = proj_sprite.custom_size.map_or(5.0, |s| (s.x.max(s.y)) / 2.0);
        for (horror_entity, horror_g_transform, mut horror_health, horror_data) in horror_query.iter_mut() {
            if skill_projectile_data.already_hit_by_this_projectile.contains(&horror_entity) { continue; }
            let horror_pos = horror_g_transform.translation().truncate();
            let horror_radius = horror_data.size.x / 2.0;
            if proj_pos.distance(horror_pos) < proj_radius + horror_radius {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                horror_health.0 -= proj_damage.0;
                spawn_damage_text(&mut commands, &asset_server, horror_g_transform.translation(), proj_damage.0, &time);
                skill_projectile_data.already_hit_by_this_projectile.push(horror_entity);
                if skill_projectile_data.piercing_left > 0 { skill_projectile_data.piercing_left -= 1; }
                else if skill_projectile_data.bounces_left > 0 {
                    skill_projectile_data.bounces_left -= 1;
                    let mut closest_new_target: Option<(Entity, f32)> = None;
                    let chain_search_radius_sq = 250.0 * 250.0;
                    for (potential_target_entity, potential_target_gtransform, _health, _horror_data) in horror_query.iter_mut() {
                        if potential_target_entity == horror_entity || skill_projectile_data.already_hit_by_this_projectile.contains(&potential_target_entity) { continue; }
                        let distance_sq = potential_target_gtransform.translation().truncate().distance_squared(horror_pos);
                        if distance_sq < chain_search_radius_sq { if closest_new_target.is_none() || distance_sq < closest_new_target.unwrap().1 { closest_new_target = Some((potential_target_entity, distance_sq)); } }
                    }
                    if let Some((target_entity, _)) = closest_new_target {
                        if let Ok((_t_ent, target_transform, _h, _horror_data_ref)) = horror_query.get(target_entity) {
                            let direction_to_new_target = (target_transform.translation().truncate() - horror_pos).normalize_or_zero();
                                if let Some(skill_def) = skill_library.get_skill_definition(skill_projectile_data.skill_id) {
                                    if let SkillEffectType::Projectile { speed, size, color, lifetime_secs, piercing, .. } = skill_def.effect {
                                        let chained_damage = proj_damage.0;
                                        commands.spawn((
                                            SpriteBundle { texture: asset_server.load("sprites/eldritch_bolt_placeholder.png"), sprite: Sprite { custom_size: Some(size), color, ..default()}, transform: Transform::from_translation(horror_pos.extend(proj_g_transform.translation().z)).with_rotation(Quat::from_rotation_z(direction_to_new_target.y.atan2(direction_to_new_target.x))), ..default() },
                                            SkillProjectile { skill_id: skill_projectile_data.skill_id, piercing_left: piercing, bounces_left: skill_projectile_data.bounces_left, already_hit_by_this_projectile: vec![target_entity], },
                                            Velocity(direction_to_new_target * speed), Damage(chained_damage), Lifetime { timer: Timer::from_seconds(lifetime_secs, TimerMode::Once) }, Name::new(format!("ChainedProjectile_{}", skill_def.name)),
                                        ));
                                    }
                                }
                        }
                    }
                    commands.entity(proj_entity).despawn_recursive(); break;
                } else { commands.entity(proj_entity).despawn_recursive(); break; }
            }
        }
    }
}

fn active_skill_aoe_system(mut commands: Commands, time: Res<Time>, mut aoe_query: Query<(Entity, &mut ActiveSkillAoEEffect, &GlobalTransform, Option<&mut Sprite>)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (aoe_entity, mut aoe_effect, aoe_g_transform, opt_sprite) in aoe_query.iter_mut() { aoe_effect.lifetime_timer.tick(time.delta()); if let Some(mut sprite) = opt_sprite { let lifetime_remaining_fraction = 1.0 - aoe_effect.lifetime_timer.percent(); let initial_alpha = sprite.color.a(); sprite.color.set_a((initial_alpha * lifetime_remaining_fraction).clamp(0.0, initial_alpha)); } if aoe_effect.lifetime_timer.finished() { commands.entity(aoe_entity).despawn_recursive(); continue; } aoe_effect.tick_timer.tick(time.delta()); if aoe_effect.tick_timer.just_finished() { aoe_effect.already_hit_this_tick.clear(); let aoe_pos = aoe_g_transform.translation().truncate(); for (horror_entity, horror_g_transform, mut horror_health) in horror_query.iter_mut() { if aoe_effect.already_hit_this_tick.contains(&horror_entity) { continue; } let horror_pos = horror_g_transform.translation().truncate(); if horror_pos.distance_squared(aoe_pos) < aoe_effect.actual_radius_sq { sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); horror_health.0 -= aoe_effect.actual_damage_per_tick; spawn_damage_text(&mut commands, &asset_server, horror_g_transform.translation(), aoe_effect.actual_damage_per_tick, &time); aoe_effect.already_hit_this_tick.push(horror_entity); } } } } }

fn freezing_nova_effect_damage_system( 
    mut commands: Commands, 
    time: Res<Time>, 
    mut nova_query: Query<(Entity, &mut FreezingNovaEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, 
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &mut Velocity, &Horror)>, 
    asset_server: Res<AssetServer>, 
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) { 
    for (nova_entity, mut nova, nova_g_transform, mut sprite, mut vis_transform) in nova_query.iter_mut() { 
        nova.lifetime_timer.tick(time.delta()); 
        let progress = nova.lifetime_timer.percent(); // fraction() -> percent()
        let current_visual_radius = nova.radius_sq.sqrt() * 2.0 * progress; 
        vis_transform.scale = Vec3::splat(current_visual_radius); 
        sprite.color.set_a((1.0 - progress * progress).max(0.0)); 
        
        if nova.lifetime_timer.percent() < 0.5 && !nova.already_hit_entities.contains(&nova_entity) { // fraction() -> percent()
            let nova_pos = nova_g_transform.translation().truncate(); 
            for (horror_entity, horror_g_transform, mut horror_health, _horror_velocity, _horror_data) in horror_query.iter_mut() { 
                if nova.already_hit_entities.contains(&horror_entity) { continue; } 
                let horror_pos = horror_g_transform.translation().truncate(); 
                if horror_pos.distance_squared(nova_pos) < nova.radius_sq { 
                    horror_health.0 -= nova.damage; 
                    spawn_damage_text(&mut commands, &asset_server, horror_g_transform.translation(), nova.damage, &time); // Corrected typo: horror_gtransform to horror_g_transform
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); 
                    commands.entity(horror_entity).insert(crate::horror::Frozen { timer: Timer::from_seconds(nova.slow_duration_secs, TimerMode::Once), speed_multiplier: nova.slow_multiplier, }); 
                    nova.already_hit_entities.push(horror_entity); 
                } 
            } 
            if !nova.already_hit_entities.contains(&nova_entity) { nova.already_hit_entities.push(nova_entity); } 
        } 
        if nova.lifetime_timer.finished() { commands.entity(nova_entity).despawn_recursive(); } 
    } 
}

fn active_channeled_beam_system(
    mut commands: Commands,
    time: Res<Time>,
    mut beam_query: Query<(Entity, &mut ActiveChanneledBeamComponent, &GlobalTransform, &mut Transform, &mut Sprite)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>, 
    asset_server: Res<AssetServer>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    player_query: Query<&Transform, (With<Survivor>, Without<ActiveChanneledBeamComponent>)>, 
) {
    let Ok(player_transform) = player_query.get_single() else { return; };

    for (beam_entity, mut beam, _beam_g_transform, mut beam_vis_transform, mut beam_sprite) in beam_query.iter_mut() {
        beam.lifetime_timer.tick(time.delta());

        let beam_spawn_offset = beam.direction * (SURVIVOR_SIZE.y / 2.0 + 1.0);
        beam_vis_transform.translation = (player_transform.translation.truncate() + beam_spawn_offset).extend(0.18);
        beam_vis_transform.rotation = Quat::from_rotation_z(beam.direction.y.atan2(beam.direction.x));

        let lifetime_remaining_fraction = 1.0 - beam.lifetime_timer.percent(); // fraction() -> percent()
        let initial_alpha = beam.color.a(); 
        beam_sprite.color.set_a((initial_alpha * lifetime_remaining_fraction).clamp(0.0, initial_alpha));


        if beam.lifetime_timer.finished() {
            commands.entity(beam_entity).despawn_recursive();
            continue;
        }

        beam.tick_timer.tick(time.delta());
        if beam.tick_timer.just_finished() {
            beam.already_hit_this_tick.clear();
            
            let beam_start_pos = beam_vis_transform.translation.truncate(); 

            for (horror_entity, horror_g_transform, mut horror_health, horror_data) in horror_query.iter_mut() { 
                if beam.already_hit_this_tick.contains(&horror_entity) { continue; }

                let horror_pos = horror_g_transform.translation().truncate();
                let horror_radius = horror_data.size.x / 2.0; 

                let vec_to_horror = horror_pos - beam_start_pos;
                let projection_len = vec_to_horror.dot(beam.direction);

                if projection_len >= 0.0 && projection_len <= beam.range {
                    let perpendicular_dist = (vec_to_horror - projection_len * beam.direction).length();
                    if perpendicular_dist < (beam.width / 2.0 + horror_radius) {
                        sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); 
                        horror_health.0 -= beam.actual_damage_per_tick;
                        spawn_damage_text(&mut commands, &asset_server, horror_g_transform.translation(), beam.actual_damage_per_tick, &time);
                        beam.already_hit_this_tick.push(horror_entity);
                    }
                }
            }
        }
    }
}