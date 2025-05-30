// src/visual_effects.rs
use bevy::prelude::*;
use rand::Rng; // Added import for gen_range
use crate::components::{Lifetime, Velocity}; // Added Velocity

const DAMAGE_TEXT_LIFETIME: f32 = 0.75;
const DAMAGE_TEXT_VELOCITY_Y: f32 = 50.0;
const DAMAGE_TEXT_FONT_SIZE: f32 = 20.0; // Adjusted for visibility

pub struct VisualEffectsPlugin;

impl Plugin for VisualEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            damage_text_movement_system,
            damage_text_fade_despawn_system, // Combined fade and despawn
        ));
    }
}

#[derive(Component)]
pub struct DamageText;

pub fn spawn_damage_text(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    amount: i32,
    _time: &Res<Time>, // Keep for potential future use (e.g., staggering animations)
) {
    let text_color = if amount > 0 { Color::TOMATO } else if amount < 0 { Color::GREEN } else { Color::WHITE };
    let prefix = if amount > 0 { "" } else if amount < 0 { "+" } else { "" };

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("{}{}", prefix, amount.abs()),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: DAMAGE_TEXT_FONT_SIZE,
                    color: text_color,
                },
            )
            .with_justify(JustifyText::Center),
            // Ensure Z-value is high enough to be visible over other elements
            transform: Transform::from_xyz(position.x, position.y, position.z + 5.0), 
            ..default()
        },
        DamageText,
        Lifetime { timer: Timer::from_seconds(DAMAGE_TEXT_LIFETIME, TimerMode::Once) },
        Velocity(Vec2::new(rand::thread_rng().gen_range(-10.0..10.0), DAMAGE_TEXT_VELOCITY_Y)), // Slight horizontal jitter
        Name::new(format!("DamageText_{}", amount)),
    ));
}

fn damage_text_movement_system(
    mut query: Query<(&mut Transform, &Velocity), With<DamageText>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
    }
}

fn damage_text_fade_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Text, &Lifetime), With<DamageText>>,
) {
    for (entity, mut text, lifetime) in query.iter_mut() {
        let remaining_fraction = lifetime.timer.fraction_remaining();
        // Ensure sections exist before trying to modify
        if let Some(section) = text.sections.get_mut(0) {
            let initial_alpha = section.style.color.a(); // Assuming initial alpha is 1.0
            section.style.color.set_a((initial_alpha * remaining_fraction).clamp(0.0, 1.0));
        }

        if lifetime.timer.just_finished() { // Despawn when lifetime is over
            commands.entity(entity).despawn();
        }
    }
}