// mescgit/bulletheavengame/bulletheavengame-a4c13a6183f1601049189db29b13bcfdace86153/src/audio.rs
use bevy::prelude::*;
use bevy::audio::Volume; // Added import for Volume
use crate::game::AppState;

#[derive(Event)]
pub struct PlaySoundEvent(pub SoundEffect);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    RitualCast,
    HorrorHit,
    HorrorDeath,
    SurvivorHit,
    Revelation,
    SoulCollect,
    MadnessConsumes,
    OmenAccepted,
    HorrorProjectile,
    TetherHit,
    PlayerBlink,
    GlacialNovaHit,
    ChainLightningZap,
    ShadowOrbPulse, // Added new sound effect
    // New variants for path-based and looped sounds
    Path(String),
    LoopPathStart(Entity, String), // Entity is the owner of the sound
    StopLoop(Entity),              // Entity is the owner of the sound
}

#[derive(Resource)]
pub struct GameAudioHandles {
    // Removed: ritual_cast, tether_hit, player_blink, glacial_nova_hit, chain_lightning_zap, shadow_orb_pulse
    pub horror_hit: Handle<AudioSource>,
    pub horror_death: Handle<AudioSource>,
    pub survivor_hit: Handle<AudioSource>,
    pub revelation: Handle<AudioSource>,
    pub soul_collect: Handle<AudioSource>,
    pub madness_consumes: Handle<AudioSource>,
    pub omen_accepted: Handle<AudioSource>,
    pub horror_projectile: Handle<AudioSource>,
    pub background_music: Handle<AudioSource>,
}

#[derive(Component)]
struct BackgroundMusicController;

// --- ActiveLoopingSounds Resource (Added as per Step 3) ---
#[derive(Resource, Default)]
pub struct ActiveLoopingSounds(pub std::collections::HashMap<Entity, Entity>);

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlaySoundEvent>()
            .init_resource::<ActiveLoopingSounds>() // Initialize the new resource
            .add_systems(Startup, setup_audio_handles)
            .add_systems(Update, play_sound_system)
            .add_systems(OnEnter(AppState::InGame), start_background_music)
            .add_systems(OnExit(AppState::InGame), stop_background_music);
    }
}

fn setup_audio_handles(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAudioHandles {
        // Removed: ritual_cast, tether_hit, player_blink, glacial_nova_hit, chain_lightning_zap, shadow_orb_pulse
        horror_hit: asset_server.load("audio/horror_hit_placeholder.ogg"),
        horror_death: asset_server.load("audio/horror_death_placeholder.ogg"),
        survivor_hit: asset_server.load("audio/survivor_hit_placeholder.ogg"),
        revelation: asset_server.load("audio/revelation_placeholder.ogg"),
        soul_collect: asset_server.load("audio/soul_collect_placeholder.ogg"),
        madness_consumes: asset_server.load("audio/madness_consumes_placeholder.ogg"),
        omen_accepted: asset_server.load("audio/omen_accepted_placeholder.ogg"),
        horror_projectile: asset_server.load("audio/horror_projectile_placeholder.ogg"),
        background_music: asset_server.load("audio/cyclopean_ruins_ambience_placeholder.ogg"),
    });
}

fn play_sound_system(
    mut commands: Commands,
    mut sound_events: EventReader<PlaySoundEvent>,
    audio_handles: Res<GameAudioHandles>,
    asset_server: Res<AssetServer>, // Added asset_server
    mut active_loops: ResMut<ActiveLoopingSounds>, // Added active_loops
) {
    for event in sound_events.read() {
        info!("Playing sound effect: {:?}", event.0); // Added logging line
        
        // Temporary variable for single-shot sources
        let mut source_to_play_once: Option<Handle<AudioSource>> = None;

        match event.0 {
            // Keep existing direct handle matches
            SoundEffect::HorrorHit => source_to_play_once = Some(audio_handles.horror_hit.clone()),
            SoundEffect::HorrorDeath => source_to_play_once = Some(audio_handles.horror_death.clone()),
            SoundEffect::SurvivorHit => source_to_play_once = Some(audio_handles.survivor_hit.clone()),
            SoundEffect::Revelation => source_to_play_once = Some(audio_handles.revelation.clone()),
            SoundEffect::SoulCollect => source_to_play_once = Some(audio_handles.soul_collect.clone()),
            SoundEffect::MadnessConsumes => source_to_play_once = Some(audio_handles.madness_consumes.clone()),
            SoundEffect::OmenAccepted => source_to_play_once = Some(audio_handles.omen_accepted.clone()),
            SoundEffect::HorrorProjectile => source_to_play_once = Some(audio_handles.horror_projectile.clone()),
            
            // Handle new path-based and loop sounds
            SoundEffect::Path(ref path_str) => { // path_str is &String
                let source = asset_server.load(path_str.clone()); // Clone path_str for loading
                commands.spawn(AudioBundle {
                    source,
                    settings: PlaybackSettings::DESPAWN.with_volume(Volume::new_relative(0.5)),
                });
            }
            SoundEffect::LoopPathStart(owner_entity, ref path_str) => { // owner_entity is Entity, path_str is &String
                if let Some(old_audio_entity) = active_loops.0.remove(&owner_entity) {
                    commands.entity(old_audio_entity).despawn_recursive();
                }
                let source = asset_server.load(path_str.clone()); // Clone path_str for loading
                let audio_entity = commands.spawn(AudioBundle {
                    source,
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: Volume::new_relative(0.4),
                        ..default()
                    },
                }).id();
                active_loops.0.insert(owner_entity, audio_entity);
            }
            SoundEffect::StopLoop(owner_entity) => { // owner_entity is Entity
                if let Some(audio_entity) = active_loops.0.remove(&owner_entity) {
                    commands.entity(audio_entity).despawn_recursive();
                }
            }
            // Default or removed sounds - these should ideally not be called if handles are removed
            // For safety during transition, we can log an error or do nothing.
            SoundEffect::RitualCast | SoundEffect::TetherHit | SoundEffect::PlayerBlink | 
            SoundEffect::GlacialNovaHit | SoundEffect::ChainLightningZap | SoundEffect::ShadowOrbPulse => {
                // These were removed from GameAudioHandles. If an event for these comes, it's an issue.
                // For now, let's just log it. In future, these variants might be removed or repurposed.
                error!("Attempted to play sound effect {:?} which no longer has a direct handle. It should be path-based.", event.0);
            }
        }

        if let Some(source) = source_to_play_once {
            commands.spawn(AudioBundle {
                source,
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::new_relative(0.5)),
			});
		}
    }
}

fn start_background_music(
    mut commands: Commands,
    audio_handles: Res<GameAudioHandles>,
    music_controller_query: Query<Entity, With<BackgroundMusicController>>,
) {
    if !music_controller_query.is_empty() {
        return;
    }
    commands.spawn((
        AudioBundle {
            source: audio_handles.background_music.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: Volume::new_relative(0.3),
                ..default()
            },
        },
        BackgroundMusicController,
    ));
}

fn stop_background_music(
    mut commands: Commands,
    music_controller_query: Query<Entity, With<BackgroundMusicController>>,
) {
    for entity in music_controller_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}