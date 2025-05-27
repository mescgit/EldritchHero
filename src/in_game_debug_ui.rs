use bevy::prelude::*;
use bevy::reflect::Reflect; // Ensure Reflect is imported
// use crate::game::AppState; // Removed: AppState is not used in this file

// Imports for update_in_game_debug_ui
// Assuming Player (Survivor) and its constants are in crate::survivor
use crate::survivor::{Survivor as Player, SanityStrain as MindAffliction, BASE_PICKUP_RADIUS}; 
use crate::components::Health as ComponentHealth;
// Changed skills import as per request
use crate::skills::SkillLibrary; 
// Changed items import as per request
use crate::items::{ItemLibrary, AutomaticWeaponLibrary}; 
use crate::weapons::{CircleOfWarding, SwarmOfNightmares};
// Ensured GlyphLibrary import is active and other glyph types are removed
use crate::glyphs::GlyphLibrary; 

// Marker components (already defined in this file, but good to list for clarity in context)
// use crate::in_game_debug_ui::{
// InGameDebugUI, PlayerStatsDebugText, InherentWeaponDebugText, EquippedSkillsDebugText,
// CollectedItemsDebugText, SpecialWeaponsDebugText, GlyphsDebugText, DebugDisplayState
// };


// 1. A public struct `InGameDebugUI` that derives `Component` from `bevy::prelude::*`.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct InGameDebugUI;

// 2. A public struct `DebugDisplayState` that derives `Resource` from `bevy::prelude::*`.
// This struct should have a public field `visible: bool`.
// Implement `Default` for this struct, with `visible` defaulting to `false`.
#[derive(Resource)]
pub struct DebugDisplayState {
    pub visible: bool,
}

impl Default for DebugDisplayState {
    fn default() -> Self {
        Self { visible: false }
    }
}

// 3. Public structs for marking text fields. All should derive `Component`:
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerStatsDebugText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct InherentWeaponDebugText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct EquippedSkillsDebugText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CollectedItemsDebugText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SpecialWeaponsDebugText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GlyphsDebugText;

pub fn setup_in_game_debug_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    debug_state: Res<DebugDisplayState>,
) {
    let mut root_node_visibility = Visibility::Visible;
    if !debug_state.visible {
        root_node_visibility = Visibility::Hidden;
    }

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.75).into(),
            visibility: root_node_visibility,
            z_index: ZIndex::Global(100), // Ensure it's on top of other UI
            ..default()
        },
        InGameDebugUI,
        Name::new("InGameDebugUIRoot"),
    )).with_children(|parent| {
        let text_style = TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 14.0,
            color: Color::WHITE,
        };
        let margin_style = Style {
            margin: UiRect::bottom(Val::Px(5.0)),
            ..default()
        };

        parent.spawn((
            TextBundle::from_section("Player Stats: Loading...", text_style.clone())
                .with_style(margin_style.clone()),
            PlayerStatsDebugText,
            Name::new("PlayerStatsDebugText"),
        ));

        parent.spawn((
            TextBundle::from_section("Inherent Weapon: Loading...", text_style.clone())
                .with_style(margin_style.clone()),
            InherentWeaponDebugText,
            Name::new("InherentWeaponDebugText"),
        ));

        parent.spawn((
            TextBundle::from_section("Equipped Skills: Loading...", text_style.clone())
                .with_style(margin_style.clone()),
            EquippedSkillsDebugText,
            Name::new("EquippedSkillsDebugText"),
        ));

        parent.spawn((
            TextBundle::from_section("Collected Items: Loading...", text_style.clone())
                .with_style(margin_style.clone()),
            CollectedItemsDebugText,
            Name::new("CollectedItemsDebugText"),
        ));

        parent.spawn((
            TextBundle::from_section("Special Weapons: Loading...", text_style.clone())
                .with_style(margin_style.clone()),
            SpecialWeaponsDebugText,
            Name::new("SpecialWeaponsDebugText"),
        ));

        parent.spawn((
            TextBundle::from_section("Glyphs: Loading...", text_style.clone())
                .with_style(margin_style.clone()), // No margin for the last item, or keep it for consistency
            GlyphsDebugText,
            Name::new("GlyphsDebugText"),
        ));
    });
}

#[allow(clippy::too_many_arguments)] // Allow many arguments for this system
pub fn update_in_game_debug_ui(
    debug_state: Res<DebugDisplayState>,
    mut root_ui_query: Query<&mut Visibility, With<InGameDebugUI>>,
    player_query: Query<(
        &Player,
        &ComponentHealth,
        &MindAffliction,
        Option<&CircleOfWarding>,
        Option<&SwarmOfNightmares>,
    )>,
    skill_library: Res<SkillLibrary>,
    item_library: Res<ItemLibrary>,
    weapon_library: Res<AutomaticWeaponLibrary>,
    opt_glyph_library: Option<Res<GlyphLibrary>>,
    mut player_stats_text_query: Query<&mut Text, (With<PlayerStatsDebugText>, Without<InherentWeaponDebugText>, Without<EquippedSkillsDebugText>, Without<CollectedItemsDebugText>, Without<SpecialWeaponsDebugText>, Without<GlyphsDebugText>)>,
    mut inherent_weapon_text_query: Query<&mut Text, (With<InherentWeaponDebugText>, Without<PlayerStatsDebugText>, Without<EquippedSkillsDebugText>, Without<CollectedItemsDebugText>, Without<SpecialWeaponsDebugText>, Without<GlyphsDebugText>)>,
    mut skills_text_query: Query<&mut Text, (With<EquippedSkillsDebugText>, Without<PlayerStatsDebugText>, Without<InherentWeaponDebugText>, Without<CollectedItemsDebugText>, Without<SpecialWeaponsDebugText>, Without<GlyphsDebugText>)>,
    mut items_text_query: Query<&mut Text, (With<CollectedItemsDebugText>, Without<PlayerStatsDebugText>, Without<InherentWeaponDebugText>, Without<EquippedSkillsDebugText>, Without<SpecialWeaponsDebugText>, Without<GlyphsDebugText>)>,
    mut special_weapons_text_query: Query<&mut Text, (With<SpecialWeaponsDebugText>, Without<PlayerStatsDebugText>, Without<InherentWeaponDebugText>, Without<EquippedSkillsDebugText>, Without<CollectedItemsDebugText>, Without<GlyphsDebugText>)>,
    mut glyphs_text_query: Query<&mut Text, (With<GlyphsDebugText>, Without<PlayerStatsDebugText>, Without<InherentWeaponDebugText>, Without<EquippedSkillsDebugText>, Without<CollectedItemsDebugText>, Without<SpecialWeaponsDebugText>)>,
) {
    // 1. Visibility Toggle
    if let Ok(mut visibility) = root_ui_query.get_single_mut() {
        if debug_state.visible {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
            return; // Hidden, no need to update text
        }
    } else {
        // Root UI not found, nothing to do.
        return;
    }

    // 2. Data Fetching and Text Updates
    let player_data = player_query.get_single();

    if player_data.is_err() {
        let err_msg = "Player not found".to_string();
        if let Ok(mut text) = player_stats_text_query.get_single_mut() { text.sections[0].value = err_msg.clone(); }
        if let Ok(mut text) = inherent_weapon_text_query.get_single_mut() { text.sections[0].value = err_msg.clone(); }
        if let Ok(mut text) = skills_text_query.get_single_mut() { text.sections[0].value = err_msg.clone(); }
        if let Ok(mut text) = items_text_query.get_single_mut() { text.sections[0].value = err_msg.clone(); }
        if let Ok(mut text) = special_weapons_text_query.get_single_mut() { text.sections[0].value = err_msg.clone(); }
        if let Ok(mut text) = glyphs_text_query.get_single_mut() { text.sections[0].value = err_msg; }
        return;
    }

    let (player, health, sanity_strain, circle_opt, swarm_opt) = player_data.unwrap();

    // Update Player Stats Text
    if let Ok(mut text) = player_stats_text_query.get_single_mut() {
        text.sections[0].value = format!(
            "Player Stats:\n\
            HP: {:.0}/{:.0} (Regen: {:.1}/s)\n\
            Speed: {:.1}\n\
            Level: {} - XP: {:.0}/{:.0} (XP Mult: {:.2}x)\n\
            Pickup Rad: {:.1} (Base: {:.1}, Mult: {:.2}x)\n\
            Armor: {} | Evasion: {:.1}% | Block: {:.1}%\n\
            Dmg Reduction: {:.1}% | Tenacity: {:.1}% | Status Resist: {:.1}%\n\
            Healing Eff: {:.2}x | Free Skill Chance: {:.1}%\n\
            Dash Charges: {} (Max) (CD Mult: {:.2}x, Range Mult: {:.2}x, Invuln: {:.2}s)",
            health.0, player.max_health, player.health_regen_rate,
            player.speed, // Removed speed_multiplier
            player.level,
            player.current_level_xp, player.experience_to_next_level(), player.xp_gain_multiplier,
            BASE_PICKUP_RADIUS * player.pickup_radius_multiplier, BASE_PICKUP_RADIUS, player.pickup_radius_multiplier,
            player.armor,
            player.evasion_chance,
            player.block_chance,
            player.damage_reduction_percent,
            player.tenacity_percent,
            player.status_effect_resistance_percent,
            player.healing_effectiveness_multiplier,
            player.free_skill_use_chance,
            player.max_dash_charges, // Changed to show only max_dash_charges
            player.dash_cooldown_multiplier,
            player.dash_range_multiplier,
            player.dash_invulnerability_duration
        );
    }

    // Update Inherent Weapon Text
    if let Ok(mut text) = inherent_weapon_text_query.get_single_mut() {
        if let Some(weapon_def) = weapon_library.get_weapon_definition(player.inherent_weapon_id) {
            let effective_damage = weapon_def.base_damage as i32 + player.auto_weapon_damage_bonus;
            // Use mind_affliction (sanity_strain) for fire rate details
            let current_fire_rate = sanity_strain.fire_timer.duration().as_secs_f32();
            let base_fire_rate = sanity_strain.base_fire_rate_secs;
            let effective_piercing = weapon_def.base_piercing + player.auto_weapon_piercing_bonus;
            // Use additional_projectiles from weapon_def
            let effective_projectiles = weapon_def.additional_projectiles + player.auto_weapon_additional_projectiles_bonus; 
            text.sections[0].value = format!(
                "Inherent Weapon: {}\n\
                Dmg: {} (Base: {}, Bonus: {}) | Fire Rate: {:.2}s (Base: {:.2}s)\n\
                Proj.Speed Mult: {:.2}x | Pierce: {} (Base: {}, Bonus: {})\n\
                Proj#: {} (Base: {}, Bonus: {})\n\
                Crit: {:.1}% (Dmg Mult: {:.2}x) | Lifesteal: {:.1}% | Exec. Threshold: {:.1}%\n\
                Bonus Ele Dmg (F/C/L/P): {}/{}/{}/{}",
                weapon_def.name,
                effective_damage, weapon_def.base_damage, player.auto_weapon_damage_bonus,
                current_fire_rate, base_fire_rate, // Removed fire_rate_mult
                player.auto_weapon_projectile_speed_multiplier,
                effective_piercing, weapon_def.base_piercing, player.auto_weapon_piercing_bonus,
                effective_projectiles, weapon_def.additional_projectiles, player.auto_weapon_additional_projectiles_bonus, // Changed to additional_projectiles
                player.auto_attack_crit_chance, player.auto_attack_crit_damage_multiplier,
                player.auto_attack_lifesteal_percent, player.auto_attack_execute_threshold,
                player.auto_attack_bonus_fire_damage, player.auto_attack_bonus_cold_damage,
                player.auto_attack_bonus_lightning_damage, player.auto_attack_poison_dps
            );
        } else {
            text.sections[0].value = "Inherent Weapon: Not Found".to_string();
        }
    }

    // Update Equipped Skills Text
    if let Ok(mut text) = skills_text_query.get_single_mut() {
        let mut skills_str = "Equipped Skills:\n".to_string();
        if player.equipped_skills.is_empty() {
            skills_str.push_str("  None");
        } else {
            for (i, skill_instance) in player.equipped_skills.iter().enumerate() {
                if let Some(skill_def) = skill_library.get_skill_definition(skill_instance.definition_id) {
                    let base_cd = skill_def.base_cooldown.as_secs_f32(); // Use .as_secs_f32()
                    let actual_cd = base_cd * skill_instance.cooldown_multiplier;
                    let cd_timer_val = if skill_instance.current_cooldown.is_zero() {
                        "Ready".to_string()
                    } else {
                        format!("{:.1}s", skill_instance.current_cooldown.as_secs_f32())
                    };
                    
                    skills_str.push_str(&format!(
                        "  {}. {} (Lvl {}) - CD: {} / {:.1}s (Base: {:.1}s, Mult: {:.2}x)\n\
                        \tDmg Bonus: {}, AoE Mult: {:.2}x\n",
                        i + 1,
                        skill_def.name,
                        skill_instance.current_level,
                        cd_timer_val, // Updated cooldown display
                        actual_cd, base_cd, skill_instance.cooldown_multiplier,
                        skill_instance.flat_damage_bonus,
                        skill_instance.aoe_radius_multiplier
                    ));
                } else {
                    skills_str.push_str(&format!("  {}. Unknown Skill (ID: {:?})\n", i + 1, skill_instance.definition_id));
                }
            }
            if !player.equipped_skills.is_empty() {
                skills_str.pop(); 
            }
        }
        text.sections[0].value = skills_str;
    }

    // Update Collected Items Text
    if let Ok(mut text) = items_text_query.get_single_mut() {
        let mut items_str = "Collected Items:\n".to_string();
        if player.collected_item_ids.is_empty() {
            items_str.push_str("  None");
        } else {
            for item_id in &player.collected_item_ids {
                if let Some(item_def) = item_library.get_item_definition(*item_id) {
                    // Use item_def.description directly
                    items_str.push_str(&format!("  - {}: {}\n", item_def.name, item_def.description)); 
                } else {
                    items_str.push_str(&format!("  - Unknown Item (ID: {:?})\n", item_id));
                }
            }
            if !player.collected_item_ids.is_empty() {
                items_str.pop();
            }
        }
        text.sections[0].value = items_str;
    }

    // Update Special Weapons Text
    if let Ok(mut text) = special_weapons_text_query.get_single_mut() {
        let mut special_str = "Special Weapons:\n".to_string();
        let mut found_any = false;

        if let Some(circle) = circle_opt {
            if circle.is_active {
                found_any = true;
                special_str.push_str(&format!(
                    "  Circle: Active - Dmg/Tick: {:.1}, Radius: {:.1}, Tick Rate: {:.2}s\n",
                    circle.base_damage_per_tick, 
                    circle.current_radius,
                    circle.damage_tick_timer.duration().as_secs_f32()
                ));
            }
        }
        if let Some(swarm) = swarm_opt {
            if swarm.is_active {
                found_any = true;
                special_str.push_str(&format!(
                    "  Swarm: Active - Larvae: {}, Dmg/Hit: {}, Orbit Rad: {:.1}, Rot. Speed: {:.1}\n",
                    swarm.num_larvae,
                    swarm.damage_per_hit,
                    swarm.orbit_radius,
                    swarm.rotation_speed
                ));
            }
        }

        if !found_any {
            special_str.push_str("  None Active");
        } else {
            // Remove trailing newline if any specials were added
            special_str.pop();
        }
        text.sections[0].value = special_str;
    }

    // Update Glyphs Text
    if let Ok(mut text) = glyphs_text_query.get_single_mut() {
        // Set static message for glyphs
        text.sections[0].value = "Glyphs:\n  (Feature not currently available on Survivor)".to_string();
    }
}
