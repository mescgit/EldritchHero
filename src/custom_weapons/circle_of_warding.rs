use bevy::prelude::*;
use crate::{
    survivor::Survivor,
    horror::Horror,
    components::Health,
    game::AppState,
};

// --- Circle of Warding Aura Weapon ---
#[derive(Component, Debug)]
pub struct CircleOfWarding {
    pub damage_tick_timer: Timer,
    pub current_radius: f32,
    pub base_damage_per_tick: i32,
    pub is_active: bool,
    pub visual_entity: Option<Entity>,
}

impl Default for CircleOfWarding {
    fn default() -> Self {
        Self {
            damage_tick_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            current_radius: 75.0,
            base_damage_per_tick: 3,
            is_active: false,
            visual_entity: None,
        }
    }
}

#[derive(Component)]
struct CircleOfWardingVisual;

fn circle_of_warding_aura_system(
    _commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut CircleOfWarding), With<Survivor>>,
    mut horror_query: Query<(&Transform, &mut Health, &Horror), With<Horror>>,
) {
    for (player_transform, mut aura_weapon) in player_query.iter_mut() {
        if !aura_weapon.is_active { continue; }
        aura_weapon.damage_tick_timer.tick(time.delta());
        if aura_weapon.damage_tick_timer.just_finished() {
            let player_position = player_transform.translation.truncate();
            let aura_radius_sq = aura_weapon.current_radius.powi(2);
            for (horror_transform, mut horror_health, _horror_data) in horror_query.iter_mut() {
                let horror_position = horror_transform.translation.truncate();
                if player_position.distance_squared(horror_position) < aura_radius_sq {
                    horror_health.0 -= aura_weapon.base_damage_per_tick;
                }
            }
        }
    }
}

fn update_circle_of_warding_visual_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut CircleOfWarding), With<Survivor>>,
    mut visual_query: Query<(Entity, &mut Transform, &mut Sprite), With<CircleOfWardingVisual>>,
) {
    if let Ok((player_entity, mut aura_weapon)) = player_query.get_single_mut() {
        if aura_weapon.is_active {
            let diameter = aura_weapon.current_radius * 2.0;
            let target_scale = diameter;
            if let Some(visual_entity) = aura_weapon.visual_entity {
                if let Ok((_v_ent, mut visual_transform, _visual_sprite)) = visual_query.get_mut(visual_entity) {
                    visual_transform.scale = Vec3::splat(target_scale);
                } else { aura_weapon.visual_entity = None; }
            }
            if aura_weapon.visual_entity.is_none() {
                let visual_entity = commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/circle_of_warding_effect_placeholder.png"),
                        sprite: Sprite { custom_size: Some(Vec2::splat(1.0)), color: Color::rgba(0.4, 0.2, 0.6, 0.4), ..default() },
                        transform: Transform { translation: Vec3::new(0.0, 0.0, 0.1), scale: Vec3::splat(target_scale), ..default() },
                        visibility: Visibility::Visible, ..default()
                    }, CircleOfWardingVisual, Name::new("CircleOfWardingVisual"),
                )).id();
                commands.entity(player_entity).add_child(visual_entity);
                aura_weapon.visual_entity = Some(visual_entity);
            }
        } else {
            if let Some(visual_entity) = aura_weapon.visual_entity.take() {
                if visual_query.get_mut(visual_entity).is_ok() { commands.entity(visual_entity).despawn_recursive(); }
            }
        }
    }
}

fn cleanup_aura_visuals_on_weapon_remove(
    _commands: Commands,
    _removed_aura_weapons: RemovedComponents<CircleOfWarding>,
    _visual_query: Query<Entity, With<CircleOfWardingVisual>>,
) {
    // Placeholder
}

pub struct CircleOfWardingPlugin;

impl Plugin for CircleOfWardingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            (
                circle_of_warding_aura_system,
                update_circle_of_warding_visual_system,
            )
            .chain()
            .run_if(in_state(AppState::InGame))
        )
        .add_systems(PostUpdate, cleanup_aura_visuals_on_weapon_remove);
        // Potentially register components for reflection if needed
    }
}
