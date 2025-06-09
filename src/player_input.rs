// This file will contain player input systems.
use bevy::prelude::*;
use crate::survivor::Survivor; // Assuming Survivor is the main player marker component
use crate::weapon_systems::PlayerOrbControllerComponent; // Component to check if player can use orbs
use crate::components::PlayerRequestsOrbDeployment; // The component to add

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_orb_deployment_input_system.run_if(in_state(crate::game::AppState::InGame)));
    }
}

// System to handle player input for deploying orbs
pub fn player_orb_deployment_input_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>, // Changed back to KeyCode
    // Query for player entity that has Survivor and PlayerOrbControllerComponent
    player_query: Query<Entity, (With<Survivor>, With<PlayerOrbControllerComponent>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) { // Changed to Spacebar
        if let Ok(player_entity) = player_query.get_single() {
            // Add PlayerRequestsOrbDeployment(true) to the player entity
            // This will be picked up by the deploy_orbiting_pet_system
            commands.entity(player_entity).insert(PlayerRequestsOrbDeployment(true));
            info!("Player {:?} requested orb deployment with Spacebar.", player_entity); // Updated log message
        } else {
            // Log if no suitable player entity is found (e.g. player doesn't have orb controller)
            // This might be noisy if the player can exist without an orb controller.
            // Consider the game's design for when this log is appropriate.
            // if player_query.get_single().is_err() { // More specific check for logging
                 // info!("Spacebar pressed, but no player entity with Survivor and PlayerOrbControllerComponent found to deploy orb.");
            // }
        }
    }
}
