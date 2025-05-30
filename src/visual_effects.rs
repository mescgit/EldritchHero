// src/visual_effects.rs
use bevy::prelude::*;
use crate::components::Lifetime; // Added Lifetime import

const DAMAGE_TEXT_LIFETIME: f32 = 0.75;
const DAMAGE_TEXT_VELOCITY: f32 = 50.0;
const DAMAGE_TEXT_FONT_SIZE: f32 = 20.0;

pub struct VisualEffectsPlugin;

impl Plugin for VisualEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            damage_text_movement_system,
            damage_text_fade_system,
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
    _time: &Res<Time>, // Time might be used for staggering or animation later
) {
    let text_color = if amount > 0 { Color::RED } else if amount < 0 { Color::GREEN } else { Color::WHITE };
    let prefix = if amount > 0 { "" } else if amount < 0 { "+" } else { "" }; // For healing

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
            transform: Transform::from_xyz(position.x, position.y, position.z + 1.0), // Ensure it's above other sprites
            ..default()
        },
        DamageText,
        Lifetime { timer: Timer::from_seconds(DAMAGE_TEXT_LIFETIME, TimerMode::Once) },
        crate::components::Velocity(Vec2::new(0.0, DAMAGE_TEXT_VELOCITY)), // Use Velocity from components
        Name::new(format!("DamageText_{}", amount)),
    ));
}

fn damage_text_movement_system(
    mut query: Query<(&mut Transform, &crate::components::Velocity), With<DamageText>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.y += velocity.0.y * time.delta_seconds();
        // Optional: add some horizontal drift or use Velocity component more fully
    }
}

fn damage_text_fade_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Text, &Lifetime), With<DamageText>>,
    time: Res<Time>, // Not strictly needed if using Lifetime component only for despawn
) {
    for (entity, mut text, lifetime) in query.iter_mut() {
        let remaining_fraction = lifetime.timer.fraction_remaining();
        for section in text.sections.iter_mut() {
            let initial_alpha = section.style.color.a();
            section.style.color.set_a((initial_alpha * remaining_fraction).clamp(0.0, 1.0));
        }
        // Lifetime component will be handled by a generic lifetime system to despawn
        // If no generic system, add despawn logic here:
        // if lifetime.timer.finished() { commands.entity(entity).despawn(); }
    }
}