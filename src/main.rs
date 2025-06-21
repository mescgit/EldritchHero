// src/main.rs
use bevy::prelude::*;
use bevy::log::info;
// use crate::game::AppState; // Removed as AppState is used with full path
use bevy::ecs::schedule::common_conditions::in_state;

// Modules managed by lib.rs now
use eldritch_hero::survivor;
use eldritch_hero::components;
use eldritch_hero::horror;
use eldritch_hero::game;


use eldritch_hero::level_event_effects;
// use eldritch_hero::weapons; // Removed
use eldritch_hero::custom_weapons::circle_of_warding::CircleOfWardingPlugin; // Added
use eldritch_hero::custom_weapons::swarm_of_nightmares::SwarmOfNightmaresPlugin; // Added
use eldritch_hero::visual_effects;
use eldritch_hero::audio;
use eldritch_hero::camera_systems;
use eldritch_hero::background;


use eldritch_hero::skills;
use eldritch_hero::items;

use eldritch_hero::weapon_systems;

use eldritch_hero::automatic_projectiles; // Ensure this use statement is present
use eldritch_hero::player_input::PlayerInputPlugin; // Added for the new player input plugin
use eldritch_hero::glyphs::GlyphsPlugin; // Added for Glyphs

// Modules specific to main.rs (if any)
// mod automatic_projectiles; // This line should be removed

use survivor::SurvivorPlugin;
use horror::HorrorPlugin;
use automatic_projectiles::AutomaticProjectilesPlugin; // Changed
use game::{GamePlugin, SCREEN_WIDTH, SCREEN_HEIGHT};
use level_event_effects::LevelEventEffectsPlugin;
// use weapons::WeaponsPlugin; // Removed
use visual_effects::VisualEffectsPlugin;
use audio::GameAudioPlugin;
use camera_systems::{CameraSystemsPlugin, MainCamera};
use background::BackgroundPlugin;
use skills::SkillsPlugin;
use items::{ItemsPlugin, AutomaticWeaponLibrary, AutomaticWeaponDefinition, AutomaticWeaponId};
use weapon_systems::WeaponSystemsPlugin; // Added
// use glyphs::GlyphsPlugin; // Commented out

fn log_on_enter_ingame() {
    info!("SM_DEBUG: Entered AppState::InGame");
}

fn log_in_ingame_update(mut first_run: Local<bool>) {
    if !*first_run {
        info!("SM_DEBUG: AppState::InGame first update tick");
        *first_run = true;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Echoes of the Abyss".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .register_type::<AutomaticWeaponId>()
        .register_type::<AutomaticWeaponDefinition>()
        .register_type::<AutomaticWeaponLibrary>()
        .register_type::<components::PlayerRequestsOrbDeployment>() // Added registration
        .add_event::<crate::components::PlayerBlinkEvent>()
        .add_plugins(GamePlugin)
        .add_plugins(SurvivorPlugin)
        .add_plugins(HorrorPlugin)
        .add_plugins(AutomaticProjectilesPlugin) // Changed
        .add_plugins(LevelEventEffectsPlugin)
        // WeaponsPlugin, // Removed
        .add_plugins(CircleOfWardingPlugin) // Added
        .add_plugins(SwarmOfNightmaresPlugin) // Added
        .add_plugins(VisualEffectsPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins(CameraSystemsPlugin)
        .add_plugins(BackgroundPlugin)
        .add_plugins(SkillsPlugin)
        .add_plugins(ItemsPlugin)
        .add_plugins(WeaponSystemsPlugin) // Added
        .add_plugins(PlayerInputPlugin) // Added new plugin
        .add_plugins(GlyphsPlugin) // Re-added GlyphsPlugin
        // crate::glyphs::GlyphsPlugin, // Removed as per instruction
        .add_systems(Startup,
            (
                setup_global_camera,
                crate::survivor::initialize_player_weapon_system
                    .after(crate::items::populate_automatic_weapon_library),
            )
        )
        .add_systems(OnEnter(crate::game::AppState::InGame), log_on_enter_ingame)
        .add_systems(Update, log_in_ingame_update.run_if(in_state(crate::game::AppState::InGame)))
        .run();
}

fn setup_global_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation.z = 999.0;
    commands.spawn((camera_bundle, MainCamera));
}