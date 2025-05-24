// src/game.rs
use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::{
    horror::{HorrorSpawnTimer, MaxHorrors},
    echoing_soul::{EchoingSoul, EchoingSoulPlugin},
    survivor::{Survivor, SanityStrain},
    components::Health,
    upgrades::{UpgradePlugin, UpgradePool, OfferedUpgrades, UpgradeCard, UpgradeType, UpgradeRarity}, // Added UpgradeRarity
    weapons::{CircleOfWarding, SwarmOfNightmares},
    audio::{PlaySoundEvent, SoundEffect},
    debug_menu::DebugMenuPlugin,
    items::{ItemId, ItemLibrary, AutomaticWeaponId, AutomaticWeaponLibrary}, 
    skills::{ActiveSkillInstance}, 
    automatic_projectiles::AutomaticProjectile,
};

pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;
const INITIAL_MAX_HORRORS: u32 = 20;
const INITIAL_SPAWN_INTERVAL_SECONDS: f32 = 2.0;
const DIFFICULTY_INCREASE_INTERVAL_SECONDS: f32 = 30.0;
const MAX_HORRORS_INCREMENT: u32 = 10;
const SPAWN_INTERVAL_DECREMENT_FACTOR: f32 = 0.9;
const MIN_SPAWN_INTERVAL_SECONDS: f32 = 0.3;
const COLLECTED_ITEM_ICON_SIZE: f32 = 32.0;
const COLLECTED_ITEM_UI_PADDING: f32 = 5.0;
const COLLECTED_ITEMS_TOP_MARGIN: f32 = 75.0; 


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default] MainMenu,
    InGame,
    LevelUp,
    GameOver,
    DebugUpgradeMenu,
}

#[derive(Resource, Default)]
struct PreviousGameState(Option<AppState>);

#[derive(Resource)]
pub struct SelectedCharacter(pub AutomaticWeaponId); 

impl Default for SelectedCharacter {
    fn default() -> Self {
        SelectedCharacter(AutomaticWeaponId(0)) 
    }
}

#[derive(Resource)]
pub struct GameConfig { pub width: f32, pub height: f32, pub spawn_area_padding: f32, }
impl Default for GameConfig { fn default() -> Self { Self { width: SCREEN_WIDTH, height: SCREEN_HEIGHT, spawn_area_padding: 50.0 } } }
pub struct GamePlugin;
#[derive(Resource, Default)]
pub struct GameState { pub score: u32, pub cycle_number: u32, pub horror_count: u32, pub game_over_timer: Timer, pub game_timer: Timer, pub difficulty_timer: Timer, }
#[derive(Event)] pub struct UpgradeChosenEvent(pub UpgradeCard);
#[derive(Event)] pub struct ItemCollectedEvent(pub ItemId);

#[derive(Component)] struct MainMenuUI;
#[derive(Component)] struct CharacterSelectButton(AutomaticWeaponId); 
#[derive(Component)] struct LevelUpUI;
#[derive(Component)] struct UpgradeButton(UpgradeCard);
#[derive(Component)] struct GameOverUI;
#[derive(Component)] struct InGameUI;
#[derive(Component)] struct CollectedItemsUI; 
#[derive(Component)] struct CollectedItemIcon(ItemId); 

#[derive(Component)] struct EnduranceText;
#[derive(Component)] struct InsightText;
#[derive(Component)] struct EchoesText;
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct TimerText;
#[derive(Component)] struct CycleText;


fn reset_for_new_game_session(
    mut game_state: ResMut<GameState>,
    mut horror_spawn_timer: ResMut<HorrorSpawnTimer>,
    mut max_horrors: ResMut<MaxHorrors>,
) {
    game_state.score = 0;
    game_state.cycle_number = 1;
    game_state.horror_count = 0;
    game_state.game_timer = Timer::from_seconds(3600.0, TimerMode::Once);
    game_state.game_timer.reset();
    game_state.game_timer.unpause();
    game_state.difficulty_timer = Timer::from_seconds(DIFFICULTY_INCREASE_INTERVAL_SECONDS, TimerMode::Repeating);
    game_state.difficulty_timer.reset();
    horror_spawn_timer.timer.set_duration(std::time::Duration::from_secs_f32(INITIAL_SPAWN_INTERVAL_SECONDS));
    horror_spawn_timer.timer.reset();
    max_horrors.0 = INITIAL_MAX_HORRORS;
}

fn on_enter_ingame_state_actions(mut game_state: ResMut<GameState>) {
    if game_state.game_timer.paused() { game_state.game_timer.unpause(); }
    if game_state.difficulty_timer.paused() { game_state.difficulty_timer.unpause(); }
}

fn on_enter_pause_like_state_actions(mut game_state: ResMut<GameState>, _current_app_state: Res<State<AppState>>) {
    if !game_state.game_timer.paused() { game_state.game_timer.pause(); }
    if !game_state.difficulty_timer.paused() { game_state.difficulty_timer.pause(); }
}
fn log_entering_debug_menu_state() {}
fn log_exiting_debug_menu_state() {}


impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app .add_event::<UpgradeChosenEvent>() .add_event::<ItemCollectedEvent>()
            .add_plugins((UpgradePlugin, DebugMenuPlugin)) .init_state::<AppState>()
            .init_resource::<GameConfig>() .init_resource::<GameState>()
            .init_resource::<PreviousGameState>()
            .init_resource::<SelectedCharacter>() 
            .insert_resource(HorrorSpawnTimer {timer: Timer::from_seconds(INITIAL_SPAWN_INTERVAL_SECONDS, TimerMode::Repeating)})
            .insert_resource(MaxHorrors(INITIAL_MAX_HORRORS)) .add_plugins(EchoingSoulPlugin)

            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu_ui)
            .add_systems(Update, character_select_button_interaction_system.run_if(in_state(AppState::MainMenu))) 
            .add_systems(OnExit(AppState::MainMenu), despawn_ui_by_marker::<MainMenuUI>)

            .add_systems(OnEnter(AppState::InGame), (
                on_enter_ingame_state_actions,
                setup_ingame_ui,
                setup_collected_items_ui, 
            ))
            .add_systems(Update, (
                update_ingame_ui,
                update_collected_items_ui, 
                update_game_timer,
                difficulty_scaling_system,
                global_key_listener,
                debug_character_switch_system, 
            ).chain().run_if(in_state(AppState::InGame).or_else(in_state(AppState::DebugUpgradeMenu))))
            .add_systems(OnExit(AppState::InGame), (
                cleanup_session_entities,
                despawn_ui_by_marker::<InGameUI>,
                despawn_ui_by_marker::<CollectedItemsUI>, 
            ))

            .add_systems(OnEnter(AppState::LevelUp), (setup_level_up_ui, on_enter_pause_like_state_actions))
            .add_systems(Update, handle_upgrade_choice_interaction.run_if(in_state(AppState::LevelUp)))
            .add_systems(Update, apply_chosen_upgrade.run_if(on_event::<UpgradeChosenEvent>()))
            .add_systems(OnExit(AppState::LevelUp), (despawn_ui_by_marker::<LevelUpUI>, on_enter_ingame_state_actions))

            .add_systems(OnEnter(AppState::DebugUpgradeMenu), (on_enter_pause_like_state_actions, log_entering_debug_menu_state))
            .add_systems(OnExit(AppState::DebugUpgradeMenu), (on_enter_ingame_state_actions, log_exiting_debug_menu_state));

            app.add_systems(OnEnter(AppState::GameOver), setup_game_over_ui)
            .add_systems(Update, game_over_input_system.run_if(in_state(AppState::GameOver)))
            .add_systems(OnExit(AppState::GameOver), despawn_ui_by_marker::<GameOverUI>);
    }
}

fn global_key_listener(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut prev_game_state: ResMut<PreviousGameState>,
) {
    if keyboard_input.just_pressed(KeyCode::Backquote) {
        match current_app_state.get() {
            AppState::InGame => {
                prev_game_state.0 = Some(AppState::InGame);
                next_app_state.set(AppState::DebugUpgradeMenu);
            }
            AppState::DebugUpgradeMenu => {
                if let Some(prev) = prev_game_state.0.take() {
                    next_app_state.set(prev);
                } else {
                    next_app_state.set(AppState::InGame);
                }
            }
            _ => {}
        }
    }
}

fn debug_character_switch_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Survivor, &mut SanityStrain, &mut Name)>,
    weapon_library: Res<AutomaticWeaponLibrary>,
    current_app_state: Res<State<AppState>>,
) {
    if !matches!(*current_app_state.get(), AppState::InGame | AppState::DebugUpgradeMenu) {
        return;
    }

    if let Ok((mut survivor, mut sanity_strain, mut name)) = player_query.get_single_mut() {
        let num_defined_weapons = weapon_library.weapons.len() as u32;
        if num_defined_weapons == 0 { return; }

        let mut current_weapon_idx = survivor.inherent_weapon_id.0; 

        let mut switched = false;
        if keyboard_input.just_pressed(KeyCode::F5) {
            current_weapon_idx = (current_weapon_idx + 1) % num_defined_weapons;
            switched = true;
        } else if keyboard_input.just_pressed(KeyCode::F6) {
            current_weapon_idx = if current_weapon_idx == 0 { num_defined_weapons - 1 } else { current_weapon_idx - 1};
            switched = true;
        }
        
        if switched {
            let new_inherent_weapon_id = AutomaticWeaponId(current_weapon_idx);
            if let Some(new_weapon_def) = weapon_library.get_weapon_definition(new_inherent_weapon_id) {
                survivor.inherent_weapon_id = new_inherent_weapon_id;
                sanity_strain.base_fire_rate_secs = new_weapon_def.base_fire_rate_secs;
                
                survivor.auto_weapon_damage_bonus = 0; 
                survivor.auto_weapon_piercing_bonus = 0;
                survivor.auto_weapon_additional_projectiles_bonus = 0;
                survivor.auto_weapon_projectile_speed_multiplier = 1.0;

                *name = Name::new(format!("Survivor ({})", new_weapon_def.name));
                sanity_strain.fire_timer.reset();
                sanity_strain.fire_timer.set_duration(std::time::Duration::from_secs_f32(new_weapon_def.base_fire_rate_secs.max(0.05)));

            }
        }
    }
}


fn despawn_ui_by_marker<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) { for entity in query.iter() { commands.entity(entity).despawn_recursive(); } }

fn setup_main_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    weapon_library: Res<AutomaticWeaponLibrary>
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ..default()
        },
        MainMenuUI,
    )).with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                "Echoes of the Abyss",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 70.0,
                    color: Color::WHITE,
                },
            ).with_text_justify(JustifyText::Center)
        );

        parent.spawn(
            TextBundle::from_section(
                "Choose your Vessel:",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                },
            ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default()})
        );

        let button_style = Style {
            width: Val::Px(300.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 24.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        };

        for weapon_def in weapon_library.weapons.iter() {
            parent.spawn((
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                },
                CharacterSelectButton(weapon_def.id),
            )).with_children(|button_parent| {
                button_parent.spawn(TextBundle::from_section(
                    weapon_def.name.clone(), 
                    button_text_style.clone(),
                ));
            });
        }
    });
}

fn character_select_button_interaction_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CharacterSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut selected_character: ResMut<SelectedCharacter>,
    game_state: ResMut<GameState>, 
    horror_spawn_timer: ResMut<HorrorSpawnTimer>, 
    max_horrors: ResMut<MaxHorrors>, 
    player_entity_query: Query<Entity, With<Survivor>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    let mut character_chosen_id: Option<AutomaticWeaponId> = None;

    for (interaction, button_data, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                character_chosen_id = Some(button_data.0);
                break; 
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.15, 0.15, 0.15).into();
            }
        }
    }

    if let Some(chosen_id) = character_chosen_id {
        sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted));
        selected_character.0 = chosen_id;

        for entity in player_entity_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        reset_for_new_game_session(game_state, horror_spawn_timer, max_horrors);
        
        next_app_state.set(AppState::InGame);
    }
}


fn setup_collected_items_ui(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(COLLECTED_ITEM_UI_PADDING),
                top: Val::Px(COLLECTED_ITEMS_TOP_MARGIN), 
                width: Val::Px(COLLECTED_ITEM_ICON_SIZE + COLLECTED_ITEM_UI_PADDING * 2.0),
                height: Val::Px(SCREEN_HEIGHT - COLLECTED_ITEMS_TOP_MARGIN - COLLECTED_ITEM_UI_PADDING),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart, 
                align_items: AlignItems::Center,
                row_gap: Val::Px(COLLECTED_ITEM_UI_PADDING),
                padding: UiRect { 
                    top: Val::Px(COLLECTED_ITEM_UI_PADDING),
                    bottom: Val::Px(COLLECTED_ITEM_UI_PADDING),
                    left: Val::Px(COLLECTED_ITEM_UI_PADDING),
                    right: Val::Px(COLLECTED_ITEM_UI_PADDING),
                },
                ..default()
            },
            z_index: ZIndex::Global(1),
            ..default()
        },
        CollectedItemsUI,
        Name::new("CollectedItemsPanel"),
    ));
}

fn update_collected_items_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Survivor>,
    item_library: Res<ItemLibrary>,
    ui_panel_query: Query<Entity, With<CollectedItemsUI>>,
    existing_icons_query: Query<(Entity, &CollectedItemIcon)>, 
) {
    if let Ok(player) = player_query.get_single() {
        if let Ok(panel_entity) = ui_panel_query.get_single() {
            let mut displayed_item_ids: Vec<ItemId> = Vec::new();
            for (_icon_entity, item_icon_data) in existing_icons_query.iter() {
                displayed_item_ids.push(item_icon_data.0);
            }

            for (icon_entity, item_icon_data) in existing_icons_query.iter() {
                if !player.collected_item_ids.contains(&item_icon_data.0) {
                    commands.entity(icon_entity).despawn_recursive();
                }
            }

            let mut children_to_add: Vec<Entity> = Vec::new();
            for &collected_item_id in &player.collected_item_ids {
                let mut already_displayed = false;
                for (_icon_entity, existing_icon_data) in existing_icons_query.iter() {
                    if existing_icon_data.0 == collected_item_id {
                        already_displayed = true;
                        break;
                    }
                }

                if !already_displayed {
                    if let Some(item_def) = item_library.get_item_definition(collected_item_id) {
                        let icon_entity = commands.spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(COLLECTED_ITEM_ICON_SIZE),
                                    height: Val::Px(COLLECTED_ITEM_ICON_SIZE),
                                    ..default()
                                },
                                image: asset_server.load(item_def.icon_path).into(),
                                ..default()
                            },
                            CollectedItemIcon(collected_item_id),
                            Name::new(format!("ItemIcon_{}", item_def.name)),
                        )).id();
                        children_to_add.push(icon_entity);
                    }
                }
            }
            commands.entity(panel_entity).push_children(&children_to_add);
        }
    }
}


fn setup_ingame_ui(mut commands: Commands, asset_server: Res<AssetServer>) { commands.spawn(( NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, padding: UiRect::all(Val::Px(10.0)), position_type: PositionType::Absolute, ..default() }, z_index: ZIndex::Global(1), ..default() }, InGameUI, )).with_children(|parent| { parent.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), justify_content: JustifyContent::SpaceAround, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(5.0)), ..default() }, background_color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(), ..default() }).with_children(|top_bar| { top_bar.spawn((TextBundle::from_section( "Endurance: 100", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::GREEN, }, ), EnduranceText)); top_bar.spawn((TextBundle::from_section( "Insight: 1", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::CYAN, }, ), InsightText)); top_bar.spawn((TextBundle::from_section( "Echoes: 0/100", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::YELLOW, }, ), EchoesText)); top_bar.spawn((TextBundle::from_section( "Cycle: 1", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::ORANGE_RED, }, ), CycleText)); }); parent.spawn(NodeBundle { style: Style { width: Val::Percent(100.0), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::FlexEnd, padding: UiRect::all(Val::Px(5.0)), ..default() }, ..default() }).with_children(|bottom_bar| { bottom_bar.spawn((TextBundle::from_section( "Score: 0", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::WHITE, }, ), ScoreText)); bottom_bar.spawn((TextBundle::from_section( "Time: 00:00", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::WHITE, }, ), TimerText)); }); }); }
fn update_game_timer(mut game_state: ResMut<GameState>, time: Res<Time>) { if !game_state.game_timer.paused() { game_state.game_timer.tick(time.delta()); } }
fn difficulty_scaling_system(time: Res<Time>, mut game_state: ResMut<GameState>, mut horror_spawn_timer: ResMut<HorrorSpawnTimer>, mut max_horrors: ResMut<MaxHorrors>,) { if game_state.difficulty_timer.paused() { return; } game_state.difficulty_timer.tick(time.delta()); if game_state.difficulty_timer.just_finished() { game_state.cycle_number += 1; max_horrors.0 = (INITIAL_MAX_HORRORS + (game_state.cycle_number -1) * MAX_HORRORS_INCREMENT).min(200); let current_duration = horror_spawn_timer.timer.duration().as_secs_f32(); let new_duration = (current_duration * SPAWN_INTERVAL_DECREMENT_FACTOR).max(MIN_SPAWN_INTERVAL_SECONDS); horror_spawn_timer.timer.set_duration(std::time::Duration::from_secs_f32(new_duration)); } }
fn update_ingame_ui(player_query: Query<(&Survivor, &Health)>, game_state: Res<GameState>, mut ui_texts: ParamSet< ( Query<&mut Text, With<EnduranceText>>, Query<&mut Text, With<InsightText>>, Query<&mut Text, With<EchoesText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<TimerText>>, Query<&mut Text, With<CycleText>>, )>,) { if let Ok((player_stats, player_health)) = player_query.get_single() { if let Ok(mut text) = ui_texts.p0().get_single_mut() { text.sections[0].value = format!("Endurance: {}/{}", player_health.0, player_stats.max_health); if player_health.0 < player_stats.max_health / 3 { text.sections[0].style.color = Color::RED; } else if player_health.0 < player_stats.max_health * 2 / 3 { text.sections[0].style.color = Color::YELLOW; } else { text.sections[0].style.color = Color::GREEN; } } if let Ok(mut text) = ui_texts.p1().get_single_mut() { text.sections[0].value = format!("Insight: {}", player_stats.level); } if let Ok(mut text) = ui_texts.p2().get_single_mut() { text.sections[0].value = format!("Echoes: {}/{}", player_stats.current_level_xp, player_stats.experience_to_next_level()); } } else { if let Ok(mut text) = ui_texts.p0().get_single_mut() { text.sections[0].value = "Endurance: --/--".to_string(); } if let Ok(mut text) = ui_texts.p1().get_single_mut() { text.sections[0].value = "Insight: --".to_string(); } if let Ok(mut text) = ui_texts.p2().get_single_mut() { text.sections[0].value = "Echoes: --/--".to_string(); } } if let Ok(mut text) = ui_texts.p3().get_single_mut() { text.sections[0].value = format!("Score: {}", game_state.score); } if let Ok(mut text) = ui_texts.p4().get_single_mut() { let elapsed_seconds = game_state.game_timer.elapsed().as_secs(); let minutes = elapsed_seconds / 60; let seconds = elapsed_seconds % 60; text.sections[0].value = format!("Time: {:02}:{:02}", minutes, seconds); } if let Ok(mut text) = ui_texts.p5().get_single_mut() { text.sections[0].value = format!("Cycle: {}", game_state.cycle_number); } }
fn setup_level_up_ui(mut commands: Commands, asset_server: Res<AssetServer>, player_query: Query<&Survivor>, upgrade_pool: Res<UpgradePool>,) { let player_level = if let Ok(player) = player_query.get_single() { player.level } else { 0 }; let current_offered_upgrades = OfferedUpgrades { choices: upgrade_pool.get_random_upgrades(3) }; commands.spawn(( NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, row_gap: Val::Px(30.0), ..default() }, background_color: Color::rgba(0.1, 0.1, 0.2, 0.9).into(), z_index: ZIndex::Global(10), ..default() }, LevelUpUI, current_offered_upgrades.clone(), )).with_children(|parent| { parent.spawn( TextBundle::from_section( format!("Revelation! Insight: {}", player_level), TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 50.0, color: Color::GOLD, }, ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default()}) ); for (index, card) in current_offered_upgrades.choices.iter().enumerate() { 
            let border_color_val = match card.rarity {
                UpgradeRarity::Regular => Color::rgb(0.75, 0.75, 0.75), // Light Gray
                UpgradeRarity::Rare => Color::PURPLE,
                UpgradeRarity::Legendary => Color::GOLD,
            };
            
            parent.spawn(( ButtonBundle { style: Style { width: Val::Px(400.0), height: Val::Px(120.0), padding: UiRect::all(Val::Px(10.0)), justify_content: JustifyContent::Center, align_items: AlignItems::FlexStart, flex_direction: FlexDirection::Column, border: UiRect::all(Val::Px(3.0)), // Increased border width for visibility margin: UiRect::bottom(Val::Px(10.0)), ..default() }, border_color: BorderColor(border_color_val), background_color: Color::GRAY.into(), ..default() }, UpgradeButton(card.clone()), Name::new(format!("Upgrade Button {}", index + 1)), )).with_children(|button_parent| { button_parent.spawn(TextBundle::from_section( &card.name, TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 24.0, color: Color::WHITE, }, ).with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() })); button_parent.spawn(TextBundle::from_section( &card.description, TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 18.0, color: Color::rgb(0.9, 0.9, 0.9), }, )); }); } }); }
fn handle_upgrade_choice_interaction(mut interaction_query: Query< (&Interaction, &UpgradeButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>), >, mut upgrade_chosen_event: EventWriter<UpgradeChosenEvent>, mut next_app_state: ResMut<NextState<AppState>>, keyboard_input: Res<ButtonInput<KeyCode>>, level_up_ui_query: Query<&OfferedUpgrades, With<LevelUpUI>>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (interaction, upgrade_button_data, mut bg_color) in interaction_query.iter_mut() { match *interaction { Interaction::Pressed => { sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted)); upgrade_chosen_event.send(UpgradeChosenEvent(upgrade_button_data.0.clone())); next_app_state.set(AppState::InGame); return; } Interaction::Hovered => { *bg_color = Color::DARK_GREEN.into(); } Interaction::None => { *bg_color = Color::GRAY.into(); } } } if let Ok(offered) = level_up_ui_query.get_single() { let choice_made = if keyboard_input.just_pressed(KeyCode::Digit1) && offered.choices.len() > 0 { Some(offered.choices[0].clone()) } else if keyboard_input.just_pressed(KeyCode::Digit2) && offered.choices.len() > 1 { Some(offered.choices[1].clone()) } else if keyboard_input.just_pressed(KeyCode::Digit3) && offered.choices.len() > 2 { Some(offered.choices[2].clone()) } else { None }; if let Some(chosen_card) = choice_made { sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted)); upgrade_chosen_event.send(UpgradeChosenEvent(chosen_card)); next_app_state.set(AppState::InGame); } } }

fn apply_chosen_upgrade(
    mut events: EventReader<UpgradeChosenEvent>,
    mut player_query: Query<(&mut Survivor, &mut SanityStrain, &mut Health, &mut CircleOfWarding, &mut SwarmOfNightmares)>,
    item_library: Res<ItemLibrary>,
    _weapon_library: Res<AutomaticWeaponLibrary>, // Renamed from weapon_library to avoid unused warning, as per simpler interpretation.
    mut item_collected_writer: EventWriter<ItemCollectedEvent>,
    skill_library: Res<crate::skills::SkillLibrary>,
) {
    for event in events.read() {
        let Ok((mut player_stats, mut sanity_strain, mut health_stats, mut circle_aura, mut nightmare_swarm)) = player_query.get_single_mut() else { continue; };
        
        let rarity = event.0.rarity; // Get the rarity

        match &event.0.upgrade_type {
            UpgradeType::SurvivorSpeed(percentage) => { player_stats.speed *= 1.0 + (*percentage as f32 / 100.0); }
            UpgradeType::MaxEndurance(amount) => { player_stats.max_health += *amount; health_stats.0 += *amount; health_stats.0 = health_stats.0.min(player_stats.max_health); }

            UpgradeType::IncreaseAutoWeaponDamage(bonus_amount) => { player_stats.auto_weapon_damage_bonus += *bonus_amount; }
            UpgradeType::IncreaseAutoWeaponFireRate(percentage) => {
                let increase_factor = *percentage as f32 / 100.0;
                sanity_strain.base_fire_rate_secs /= 1.0 + increase_factor;
                sanity_strain.base_fire_rate_secs = sanity_strain.base_fire_rate_secs.max(0.05);
            }
            UpgradeType::IncreaseAutoWeaponProjectileSpeed(percentage_increase) => { player_stats.auto_weapon_projectile_speed_multiplier *= 1.0 + (*percentage_increase as f32 / 100.0); }
            UpgradeType::IncreaseAutoWeaponPiercing(amount) => { player_stats.auto_weapon_piercing_bonus += *amount; }
            UpgradeType::IncreaseAutoWeaponProjectiles(amount) => { player_stats.auto_weapon_additional_projectiles_bonus += *amount; }
            
            // New Auto-Attack Upgrade Types with Rarity Scaling
            UpgradeType::AutoAttackDamagePercent(base_val) => {
                let actual_bonus = match rarity {
                    UpgradeRarity::Regular => *base_val as i32,
                    UpgradeRarity::Rare => (*base_val * 2.0) as i32,
                    UpgradeRarity::Legendary => (*base_val * 3.0) as i32,
                };
                player_stats.auto_weapon_damage_bonus += actual_bonus;
            }
            UpgradeType::AutoAttackSpeedPercent(base_val) => {
                let percent_increase = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_weapon_projectile_speed_multiplier *= 1.0 + (percent_increase / 100.0);
            }
            UpgradeType::AutoAttackFireRatePercent(base_val) => {
                let percent_increase = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                sanity_strain.base_fire_rate_secs /= 1.0 + (percent_increase / 100.0);
                sanity_strain.base_fire_rate_secs = sanity_strain.base_fire_rate_secs.max(0.05); 
            }
            UpgradeType::AutoAttackAddProjectiles(base_val) => {
                let actual_add = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_weapon_additional_projectiles_bonus += actual_add;
            }
            UpgradeType::AutoAttackAddPiercing(base_val) => {
                let actual_add = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_weapon_piercing_bonus += actual_add;
            }

            UpgradeType::EchoesGainMultiplier(percentage) => { player_stats.xp_gain_multiplier *= 1.0 + (*percentage as f32 / 100.0); }
            UpgradeType::SoulAttractionRadius(percentage) => { player_stats.pickup_radius_multiplier *= 1.0 + (*percentage as f32 / 100.0); }

            UpgradeType::InscribeCircleOfWarding => { if !circle_aura.is_active { circle_aura.is_active = true; } else { circle_aura.base_damage_per_tick += 1; circle_aura.current_radius *= 1.1; }}
            UpgradeType::IncreaseCircleRadius(percentage) => { if circle_aura.is_active { circle_aura.current_radius *= 1.0 + (*percentage as f32 / 100.0); }}
            UpgradeType::IncreaseCircleDamage(amount) => { if circle_aura.is_active { circle_aura.base_damage_per_tick += *amount; }}
            UpgradeType::DecreaseCircleTickRate(percentage) => { if circle_aura.is_active { let reduction_factor = *percentage as f32 / 100.0; let current_tick_duration = circle_aura.damage_tick_timer.duration().as_secs_f32(); let new_tick_duration = (current_tick_duration * (1.0 - reduction_factor)).max(0.1); circle_aura.damage_tick_timer.set_duration(std::time::Duration::from_secs_f32(new_tick_duration)); } }
            UpgradeType::EnduranceRegeneration(amount) => { player_stats.health_regen_rate += *amount; }
            UpgradeType::ManifestSwarmOfNightmares => { if !nightmare_swarm.is_active { nightmare_swarm.is_active = true; nightmare_swarm.num_larvae = nightmare_swarm.num_larvae.max(2); } else { nightmare_swarm.num_larvae += 1; nightmare_swarm.damage_per_hit += 1; }}
            UpgradeType::IncreaseNightmareCount(count) => { if nightmare_swarm.is_active { nightmare_swarm.num_larvae += *count; }}
            UpgradeType::IncreaseNightmareDamage(damage) => { if nightmare_swarm.is_active { nightmare_swarm.damage_per_hit += *damage; }}
            UpgradeType::IncreaseNightmareRadius(radius_increase) => { if nightmare_swarm.is_active { nightmare_swarm.orbit_radius += *radius_increase; }}
            UpgradeType::IncreaseNightmareRotationSpeed(speed_increase) => { if nightmare_swarm.is_active { nightmare_swarm.rotation_speed += *speed_increase; }}
            UpgradeType::IncreaseSkillDamage { slot_index, amount } => { if let Some(skill_instance) = player_stats.equipped_skills.get_mut(*slot_index) { skill_instance.flat_damage_bonus += *amount; skill_instance.current_level += 1; } }
            UpgradeType::GrantRandomRelic => { if !item_library.items.is_empty() { let mut rng = rand::thread_rng(); if let Some(random_item_def) = item_library.items.choose(&mut rng) { item_collected_writer.send(ItemCollectedEvent(random_item_def.id)); } } }
            UpgradeType::GrantSkill(skill_id_to_grant) => { let already_has_skill = player_stats.equipped_skills.iter().any(|s| s.definition_id == *skill_id_to_grant); if !already_has_skill { if player_stats.equipped_skills.len() < 5 { if let Some(_skill_def) = skill_library.get_skill_definition(*skill_id_to_grant) { player_stats.equipped_skills.push(ActiveSkillInstance::new(*skill_id_to_grant )); } } } }
            UpgradeType::ReduceSkillCooldown { slot_index, percent_reduction } => { if let Some(skill_instance) = player_stats.equipped_skills.get_mut(*slot_index) { skill_instance.cooldown_multiplier *= 1.0 - percent_reduction; skill_instance.cooldown_multiplier = skill_instance.cooldown_multiplier.max(0.1); skill_instance.current_level +=1; } }
            UpgradeType::IncreaseSkillAoERadius { slot_index, percent_increase } => { if let Some(skill_instance) = player_stats.equipped_skills.get_mut(*slot_index) { skill_instance.aoe_radius_multiplier *= 1.0 + percent_increase; skill_instance.current_level +=1; } }

            // --- Auto-Attack Focused (New Batch) ---
            UpgradeType::AutoAttackAddFireDamage(base_val) => {
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_attack_bonus_fire_damage += actual_value;
            }
            UpgradeType::AutoAttackAddColdDamage(base_val) => {
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_attack_bonus_cold_damage += actual_value;
            }
            UpgradeType::AutoAttackAddLightningDamage(base_val) => {
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_attack_bonus_lightning_damage += actual_value;
            }
            UpgradeType::AutoAttackAddPoisonDamage(base_val) => { // Base DPS
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.auto_attack_poison_dps += actual_value;
                // TO-DO: Implement poison application system on hit (duration, ticking)
            }
            UpgradeType::AutoAttackCritChance(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_crit_chance += actual_value;
            }
            UpgradeType::AutoAttackCritDamage(base_val) => { // Percent bonus
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_crit_damage_multiplier += actual_value / 100.0; // Assuming it's a multiplier bonus e.g. 0.2 for +20%
            }
            UpgradeType::AutoAttackExecuteLowHealth(base_val) => { // Percent health threshold
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5, // Scaled differently as it's a threshold
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_execute_threshold = player_stats.auto_attack_execute_threshold.max(actual_value); // Take the highest threshold
                // TO-DO: Implement execute logic on hit
            }
            UpgradeType::AutoAttackLifeSteal(base_val) => { // Percent of damage
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_lifesteal_percent += actual_value;
                // TO-DO: Implement lifesteal application on damage dealt
            }
            UpgradeType::AutoAttackChainChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5, // Scaled differently for chances
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_chain_chance += actual_value;
                // TO-DO: Implement chaining logic on projectile hit
            }
            UpgradeType::AutoAttackForkChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_fork_chance += actual_value;
                // TO-DO: Implement forking logic on projectile spawn/hit
            }
            UpgradeType::AutoAttackChillChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_chill_chance += actual_value;
                // TO-DO: Implement chill application on hit (slow effect on enemy)
            }
            UpgradeType::AutoAttackStunChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_stun_chance += actual_value;
                // TO-DO: Implement stun application on hit (short disable on enemy)
            }
            UpgradeType::AutoAttackBurnChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_burn_chance += actual_value;
                // TO-DO: Implement burn DoT application on hit
            }
            UpgradeType::AutoAttackReduceHealingChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_reduce_healing_chance += actual_value;
                // TO-DO: Implement healing reduction debuff on enemy
            }
            UpgradeType::AutoAttackAreaDamageOnHitChance(base_aoe_damage) => { // base_val is AoE damage
                let (actual_chance, actual_aoe_damage) = match rarity {
                    UpgradeRarity::Regular => (10.0, *base_aoe_damage), // Example chance: 10%
                    UpgradeRarity::Rare => (15.0, *base_aoe_damage * 2),   // Example chance: 15%
                    UpgradeRarity::Legendary => (20.0, *base_aoe_damage * 3),// Example chance: 20%
                };
                player_stats.auto_attack_aoe_on_hit_chance = player_stats.auto_attack_aoe_on_hit_chance.max(actual_chance); // Take best chance
                player_stats.auto_attack_aoe_on_hit_damage = player_stats.auto_attack_aoe_on_hit_damage.max(actual_aoe_damage); // Take best damage
                // TO-DO: Implement AoE spawn on projectile hit
            }
            UpgradeType::AutoAttackIncreaseDuration(base_val) => { // Percent increase
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_projectile_duration_multiplier *= 1.0 + (actual_value / 100.0);
            }
            UpgradeType::AutoAttackHomingStrength(base_val) => { // Flat increase
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_homing_strength += actual_value;
                // TO-DO: Implement homing logic in projectile movement
            }
            UpgradeType::AutoAttackRicochetChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_ricochet_chance += actual_value;
                // TO-DO: Implement ricochet logic on projectile hit
            }
            UpgradeType::AutoAttackShieldPenetration(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.auto_attack_shield_penetration_percent += actual_value;
                // TO-DO: Implement shield penetration in damage calculation against shielded enemies
            }
            UpgradeType::AutoAttackCullStrikeChance(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.auto_attack_cull_strike_chance += actual_value;
                // TO-DO: Implement cull strike logic (massive damage vs low health non-elites)
            }

            // --- Survivor Defensive Stats (New Batch) ---
            UpgradeType::IncreaseArmor(base_val) => {
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.armor += actual_value;
            }
            UpgradeType::IncreaseEvasionChance(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.evasion_chance += actual_value;
            }
            UpgradeType::IncreaseBlockChance(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.block_chance += actual_value;
                // TO-DO: Implement block logic in damage mitigation
            }
            UpgradeType::IncreaseDamageReduction(base_val) => { // Flat Percent reduction
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.damage_reduction_percent += actual_value;
            }
            UpgradeType::IncreaseTenacity(base_val) => { // Percent reduction to CC duration
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.tenacity_percent += actual_value;
                // TO-DO: Apply tenacity when CC effects are applied to player
            }
            UpgradeType::IncreaseStatusEffectResistance(base_val) => { // Percent chance to resist
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.status_effect_resistance_percent += actual_value;
                // TO-DO: Check resistance when status effects are attempted on player
            }
            UpgradeType::IncreaseHealingEffectiveness(base_val) => { // Percent bonus
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.healing_effectiveness_multiplier *= 1.0 + (actual_value / 100.0);
            }
            UpgradeType::OnHitGainTemporaryArmor(base_val) => { // Flat armor
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.on_hit_temp_armor_bonus = player_stats.on_hit_temp_armor_bonus.max(actual_value); // Take best
                // TO-DO: Implement system to grant temporary armor buff when player is hit
            }
            UpgradeType::OnHitGainTemporarySpeed(base_val) => { // Percent speed
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.on_hit_temp_speed_bonus_percent = player_stats.on_hit_temp_speed_bonus_percent.max(actual_value); // Take best
                // TO-DO: Implement system to grant temporary speed buff when player is hit
            }
            UpgradeType::AfterBeingHitSpawnRetaliationNova(base_val) => { // Flat damage
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.after_being_hit_retaliation_nova_damage = player_stats.after_being_hit_retaliation_nova_damage.max(actual_value); // Take best
                // TO-DO: System to trigger this nova (already partially exists for an item, might need generalization)
            }

            // --- Survivor Utility/Mobility (New Batch) ---
            UpgradeType::IncreaseDashCharges(base_val) => {
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.max_dash_charges += actual_value;
                // TO-DO: Dash system needs to exist and use this
            }
            UpgradeType::ReduceDashCooldown(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.dash_cooldown_multiplier *= 1.0 - (actual_value / 100.0);
            }
            UpgradeType::IncreaseDashRange(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.dash_range_multiplier *= 1.0 + (actual_value / 100.0);
            }
            UpgradeType::DashGrantsInvulnerability(base_val) => { // Duration in seconds
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.dash_invulnerability_duration = player_stats.dash_invulnerability_duration.max(actual_value);
                // TO-DO: Dash system needs to grant invulnerability for this duration
            }
            UpgradeType::IncreaseMovementOutOfCombat(base_val) => { // Percent
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.movement_speed_out_of_combat_multiplier *= 1.0 + (actual_value / 100.0);
                // TO-DO: System to check if player is "in combat" to apply this
            }
            UpgradeType::ReduceSlowEffectiveness(base_val) => { // Percent reduction on slows
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.slow_effectiveness_reduction_percent += actual_value;
                // TO-DO: Apply this reduction when slow effects are calculated for player
            }
            UpgradeType::GainShieldOnKill(base_val) => { // Flat shield amount
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2,
                    UpgradeRarity::Legendary => *base_val * 3,
                };
                player_stats.shield_on_kill_amount = player_stats.shield_on_kill_amount.max(actual_value);
                // TO-DO: System to grant shield on kill
            }
            UpgradeType::IncreaseEchoesDropRate(base_val) => { // Percent more echoes orbs
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.echoes_drop_rate_multiplier *= 1.0 + (actual_value / 100.0);
            }
            UpgradeType::IncreaseRelicDropRate(base_val) => { // Percent higher chance for relics
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.relic_drop_rate_multiplier *= 1.0 + (actual_value / 100.0);
            }
            UpgradeType::ChanceForFreeSkillUse(base_val) => { // Percent chance
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.free_skill_use_chance += actual_value;
                // TO-DO: Check this chance when skills are used
            }

            // --- Weapon-Specific (Aura/Orbiter - Circle of Warding / Swarm of Nightmares) (New Batch) ---
            UpgradeType::AuraIncreaseSizePerKill(base_val) => { // Percent size increase stack
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.aura_size_per_kill_bonus_percent += actual_value;
                // TO-DO: System for CircleOfWarding to track kills and temporarily increase radius
            }
            UpgradeType::OrbiterIncreaseSpeedPerKill(base_val) => { // Percent speed increase stack
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.orbiter_speed_per_kill_bonus_percent += actual_value;
                // TO-DO: System for SwarmOfNightmares to track kills and temporarily increase speed
            }
            UpgradeType::AuraPullEnemiesChance(base_val) => { // Percent chance per tick
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 1.5,
                    UpgradeRarity::Legendary => *base_val * 2.0,
                };
                player_stats.aura_pull_enemies_chance += actual_value;
                // TO-DO: CircleOfWarding system to attempt pull on enemies within aura
            }
            UpgradeType::OrbiterExplodeOnKillChance(base_explosion_damage) => { // base_val is explosion damage
                let (actual_chance, actual_damage) = match rarity {
                    UpgradeRarity::Regular => (10.0, *base_explosion_damage), // Example chance 10%
                    UpgradeRarity::Rare => (15.0, *base_explosion_damage * 2), // Example chance 15%
                    UpgradeRarity::Legendary => (20.0, *base_explosion_damage * 3), // Example chance 20%
                };
                player_stats.orbiter_explode_on_kill_chance = player_stats.orbiter_explode_on_kill_chance.max(actual_chance);
                player_stats.orbiter_explosion_damage = player_stats.orbiter_explosion_damage.max(actual_damage);
                // TO-DO: SwarmOfNightmares system to check this chance on kill and spawn explosion
            }
            UpgradeType::AuraDebuffEnemies(base_val) => { // Percent increased damage taken
                let actual_value = match rarity {
                    UpgradeRarity::Regular => *base_val,
                    UpgradeRarity::Rare => *base_val * 2.0,
                    UpgradeRarity::Legendary => *base_val * 3.0,
                };
                player_stats.aura_debuff_enemies_damage_increase_percent += actual_value;
                // TO-DO: CircleOfWarding system to apply debuff to enemies in aura, damage system to check for this debuff
            }
        }
    }
}
fn setup_game_over_ui(mut commands: Commands, game_state: Res<GameState>, asset_server: Res<AssetServer>) { commands.spawn(( NodeBundle { style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, row_gap: Val::Px(20.0), ..default() }, ..default() }, GameOverUI, )).with_children(|parent| { parent.spawn( TextBundle::from_section( "Consumed by Madness!", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 80.0, color: Color::RED, }, ).with_text_justify(JustifyText::Center) ); parent.spawn( TextBundle::from_section( format!("Score: {}", game_state.score), TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 50.0, color: Color::WHITE, }, ).with_text_justify(JustifyText::Center) ); parent.spawn( TextBundle::from_section( "Succumb Again? (R)", TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 40.0, color: Color::rgba(0.8,0.8,0.8,1.0), }, ).with_text_justify(JustifyText::Center) ); }); }
fn game_over_input_system(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>, mut next_app_state: ResMut<NextState<AppState>>, game_state: ResMut<GameState>, horror_spawn_timer: ResMut<HorrorSpawnTimer>, max_horrors: ResMut<MaxHorrors>, player_entity_query: Query<Entity, With<Survivor>>,) { if keyboard_input.just_pressed(KeyCode::KeyR) { for entity in player_entity_query.iter() { commands.entity(entity).despawn_recursive(); } reset_for_new_game_session(game_state, horror_spawn_timer, max_horrors); next_app_state.set(AppState::MainMenu); } }

fn cleanup_session_entities(
    mut commands: Commands,
    projectiles_query: Query<Entity, With<AutomaticProjectile>>,
    orbs_query: Query<Entity, With<EchoingSoul>>,
    skill_projectiles_query: Query<Entity, With<crate::skills::SkillProjectile>>,
    skill_aoe_query: Query<Entity, With<crate::skills::ActiveSkillAoEEffect>>,
) {
    for entity in projectiles_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in orbs_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in skill_projectiles_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in skill_aoe_query.iter() { commands.entity(entity).despawn_recursive(); }
}