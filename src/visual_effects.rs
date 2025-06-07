// src/visual_effects.rs
use bevy::prelude::*;
use rand::Rng; // Added import for gen_range
use crate::components::{Lifetime, Velocity, ExpandingWaveVisual}; // Added ExpandingWaveVisual

const DAMAGE_TEXT_LIFETIME: f32 = 0.75;
const DAMAGE_TEXT_VELOCITY_Y: f32 = 50.0;
const DAMAGE_TEXT_FONT_SIZE: f32 = 20.0; // Adjusted for visibility

pub struct VisualEffectsPlugin;

impl Plugin for VisualEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            damage_text_movement_system,
            damage_text_fade_despawn_system, // Combined fade and despawn
            expanding_wave_visual_system, // Added new system
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
            text: Text {
                sections: vec![TextSection::new(
                    format!("{}{}", prefix, amount.abs()),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: DAMAGE_TEXT_FONT_SIZE,
                        color: text_color,
                    },
                )],
                alignment: TextAlignment::Center, // Corrected
                ..default()
            },
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
        let remaining_fraction = lifetime.timer.percent_left(); // fraction_remaining() -> percent_left()
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

pub fn expanding_wave_visual_system(
    mut query: Query<(
        &mut Transform,
        &mut Sprite,
        &ExpandingWaveVisual,
        &Lifetime,
    )>,
) {
    for (mut transform, mut sprite, wave_params, lifetime) in query.iter_mut() {
        let progress = lifetime.timer.percent(); // Current progress of the lifetime timer (0.0 to 1.0)

        // Interpolate scale
        transform.scale = wave_params.initial_scale.lerp(wave_params.final_scale, progress);

        // Fade out alpha
        // Assuming initial alpha is 1.0 from the sprite's base color.
        // If the sprite.color itself has an alpha component, this will fade from that value.
        let initial_alpha = sprite.color.a(); // Get current alpha, which might be the base or already modified
        sprite.color.set_a((initial_alpha * (1.0 - progress)).clamp(0.0, 1.0));
    }
}