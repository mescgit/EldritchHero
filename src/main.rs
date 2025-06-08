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
use eldritch_hero::echoing_soul;
use eldritch_hero::upgrades;
use eldritch_hero::level_event_effects;
use eldritch_hero::weapons;
use eldritch_hero::visual_effects;
use eldritch_hero::audio;
use eldritch_hero::camera_systems;
use eldritch_hero::background;
use eldritch_hero::debug_menu;
use eldritch_hero::in_game_debug_ui;
use eldritch_hero::skills;
use eldritch_hero::items;
use eldritch_hero::glyphs;
use eldritch_hero::weapon_systems;
use eldritch_hero::automatic_weapons;
use eldritch_hero::automatic_projectiles; // Ensure this use statement is present

// Modules specific to main.rs (if any)
// mod automatic_projectiles; // This line should be removed

use survivor::SurvivorPlugin;
use horror::HorrorPlugin;
use automatic_projectiles::AutomaticProjectilesPlugin; // Changed
use game::{GamePlugin, SCREEN_WIDTH, SCREEN_HEIGHT};
use level_event_effects::LevelEventEffectsPlugin;
use weapons::WeaponsPlugin;
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
        .add_plugins((
            GamePlugin,
            SurvivorPlugin,
            HorrorPlugin,
            AutomaticProjectilesPlugin, // Changed
            LevelEventEffectsPlugin,
            WeaponsPlugin,
            VisualEffectsPlugin,
            GameAudioPlugin,
            CameraSystemsPlugin,
            BackgroundPlugin,
            SkillsPlugin,
            ItemsPlugin,
            WeaponSystemsPlugin, // Added
            // crate::glyphs::GlyphsPlugin, // Removed as per instruction
        ))
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