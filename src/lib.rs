// src/lib.rs

// Declare modules that should be part of the library and accessible to tests
pub mod skills;
pub mod upgrades;
pub mod survivor;
pub mod items;
pub mod components;
pub mod game; // Assuming AppState and other game-related items might be needed
pub mod automatic_projectiles; // Added this line
pub mod automatic_weapons; // For weapon definitions and specific upgrade functions
pub mod horror; // If any horror definitions/components are needed by tests
pub mod echoing_soul;
pub mod level_event_effects;
mod custom_weapons; // Added for refactored weapons
pub mod visual_effects;
pub mod audio;
pub mod camera_systems;
pub mod background;
pub mod debug_menu; // If tests interact with debug features
pub mod in_game_debug_ui; // If tests verify UI elements
pub mod glyphs;
pub mod weapon_systems; // If tests need to interact with these systems/components directly
pub mod player_input; // Added for player input systems

// You might also need to re-export specific items if you want shorter paths,
// but for now, just declaring the modules as public should be enough
// for tests to use `use eldritch_hero::module_name::ItemName;`
