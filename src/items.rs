// src/items.rs
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::{
    survivor::Survivor,
    components::Health,
    game::{AppState, ItemCollectedEvent},
    horror::Horror,
    visual_effects, // Import the module
    audio::{PlaySoundEvent, SoundEffect},
    skills::{SkillId, SkillLibrary, ActiveSkillInstance},
    weapons::{CircleOfWarding, SwarmOfNightmares},
};
use crate::automatic_weapons;

// --- Standard Items (Relics) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum SurvivorTemporaryBuff { HealthRegen { rate: f32, duration_secs: f32 }, }

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ItemEffect {
    PassiveStatBoost {
        max_health_increase: Option<i32>,
        speed_multiplier: Option<f32>,
        damage_increase: Option<i32>,
        xp_gain_multiplier: Option<f32>,
        pickup_radius_increase: Option<f32>,
        auto_weapon_projectile_speed_multiplier_increase: Option<f32>,
    },
    OnAutomaticProjectileHitExplode {
        chance: f32,
        explosion_damage: i32,
        explosion_radius: f32,
        explosion_color: Color,
    },
    OnSurvivorHitRetaliate {
        chance: f32,
        retaliation_damage: i32,
        retaliation_radius: f32,
        retaliation_color: Color,
    },
    OnHorrorKillTrigger { // Changed from OnEnemyKillTrigger
        chance: f32,
        effect: SurvivorTemporaryBuff,
    },
    GrantSpecificSkill { skill_id: SkillId, },
    ActivateCircleOfWarding { base_damage: i32, base_radius: f32, base_tick_interval: f32 },
    ActivateSwarmOfNightmares { num_larvae: u32, base_damage: i32, base_orbit_radius: f32, base_rotation_speed: f32 },
}

// --- Automatic Weapon ID ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub struct AutomaticWeaponId(pub u32);


// --- Parameter Struct Definitions for AttackTypeData ---

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct StandardProjectileParams {
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub base_damage: i32,
    pub projectile_speed: f32, // Keep this as the main speed field
    pub projectile_lifetime_secs: f32,
    pub piercing: u32, // Keep this as the main piercing field
    pub base_fire_rate_secs: f32, // Added
    pub additional_projectiles: u32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct ReturningProjectileParams {
    pub base_fire_rate_secs: f32, // Added
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub base_damage: i32, 
    pub projectile_speed: f32,
    pub travel_distance: f32,
    pub piercing: u32,
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)] // Assuming Default is handled by impl
pub struct ChanneledBeamParams {
    // pub base_damage_per_tick: i32, // Renamed in subtask to damage_per_tick
    pub damage_per_tick: i32, // From subtask
    pub tick_interval_secs: f32, // From subtask (was tick_rate_secs)
    // pub range: f32, // Renamed in subtask to beam_range
    pub beam_range: f32, // From subtask
    pub beam_width: f32,
    // pub beam_color: Color, // Renamed in subtask to color
    pub color: Color, // From subtask
    pub movement_penalty_multiplier: f32,
    pub max_duration_secs: Option<f32>, // Added
    pub cooldown_secs: Option<f32>, // Added
    pub is_automatic: bool, // Added
    pub fire_sound_effect: Option<String>,
    pub loop_sound_effect: Option<String>,
    pub stop_sound_effect: Option<String>,
}
impl Default for ChanneledBeamParams {
    fn default() -> Self {
        Self {
            damage_per_tick: 1, tick_interval_secs: 0.1, beam_range: 300.0, beam_width: 10.0,
            color: Color::WHITE, movement_penalty_multiplier: 0.5,
            max_duration_secs: None, // Added default
            cooldown_secs: None, // Added default
            is_automatic: false, // Added default
            fire_sound_effect: None,
            loop_sound_effect: None,
            stop_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct ConeAttackParams {
    // pub base_damage: i32, // Already present
    pub base_fire_rate_secs: f32, // Added
    pub base_damage: i32, 
    pub cone_angle_degrees: f32,
    pub cone_radius: f32,
    pub color: Color,
    pub visual_sprite_path: Option<String>,
    pub visual_size_scale_with_radius_angle: Option<(f32, f32)>, 
    pub visual_anchor_offset: Option<Vec2>, 
    pub applies_burn: Option<bool>, // Added
    pub burn_damage_per_tick: Option<i32>, // Added
    pub burn_duration_secs: Option<f32>, // Added
    pub burn_tick_interval_secs: Option<f32>, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct LobbedAoEPoolParams {
    // pub base_damage_on_impact: i32, // Already present
    // pub pool_damage_per_tick: i32, // Already present
    pub base_fire_rate_secs: f32, // Added
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_arc_height: f32, 
    pub projectile_speed: f32, 
    pub base_damage_on_impact: i32, 
    pub pool_radius: f32, 
    pub pool_duration_secs: f32, 
    pub pool_damage_per_tick: i32, 
    pub pool_tick_interval_secs: f32, 
    pub pool_color: Color, 
    pub max_active_pools: u32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct ChargeLevelParams {
    pub charge_time_secs: f32,
    // pub projectile_damage: i32, // Renamed to damage in subtask
    pub damage: i32, 
    pub projectile_speed: f32,
    pub projectile_size: Vec2,
    pub piercing: u32, // Added
    pub explodes_on_impact: bool, // Added
    pub explosion_radius: f32, // Added
    pub explosion_damage: i32, // Added
    pub projectile_sprite_path: String, 
    pub projectile_color: Color, 
    pub aoe_radius_on_impact: Option<f32>, 
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct ChargeUpEnergyShotParams {
    pub base_fire_rate_secs: f32, 
    pub base_projectile_sprite_path: String, 
    pub base_projectile_color: Color, 
    pub charge_levels: Vec<ChargeLevelParams>, 
    pub projectile_lifetime_secs: f32, 
    pub charge_sound_effect: Option<String>,
    pub release_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct TrailOfFireParams {
    pub base_damage_on_impact: i32, // Added
    pub base_fire_rate_secs: f32, // Added
    pub projectile_speed: f32, // Added
    pub projectile_sprite_path: String, // Added
    pub projectile_size: Vec2, // Added
    pub projectile_color: Color, // Added
    pub projectile_lifetime_secs: f32, // Added
    pub segment_spawn_interval_secs: f32, 
    pub trail_segment_damage_per_tick: i32,
    pub trail_segment_tick_interval_secs: f32,
    pub trail_segment_duration_secs: f32,
    pub trail_segment_width: f32,
    pub trail_segment_color: Color,
    pub fire_sound_effect: Option<String>,
}


#[derive(Debug, Clone, Copy, Reflect, PartialEq, Default, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub enum RepositioningTetherMode {
    #[default]
    Pull,
    Push,
    Alternate,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)] // Default is custom
pub struct RepositioningTetherParams {
    pub base_fire_rate_secs: f32, // Added
    pub tether_projectile_speed: f32,
    pub tether_range: f32,
    pub tether_sprite_path: String,
    pub tether_color: Color,
    pub tether_size: Vec2,
    pub mode: RepositioningTetherMode,
    pub pull_strength: f32,
    pub push_strength: f32,
    pub reactivation_window_secs: f32,
    pub effect_duration_secs: f32,
    pub fire_sound_effect: Option<String>,
}
impl Default for RepositioningTetherParams {
    fn default() -> Self {
        Self {
            base_fire_rate_secs: 1.0, // Added default
            tether_projectile_speed: 600.0,
            tether_range: 400.0,
            tether_sprite_path: "sprites/tether_placeholder.png".to_string(),
            tether_color: Color::WHITE,
            tether_size: Vec2::new(10.0, 10.0),
            mode: RepositioningTetherMode::Pull,
            pull_strength: 50.0,
            push_strength: 50.0,
            reactivation_window_secs: 2.0,
            effect_duration_secs: 0.3,
            fire_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)] // Default is custom
pub struct OrbitingPetParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub max_active_orbs: u32,
    pub orb_duration_secs: f32,
    pub orb_sprite_path: String, // Not in subtask list, but seems essential
    pub orb_size: Vec2, // Not in subtask list, but seems essential
    pub orb_color: Color, // Not in subtask list, but seems essential
    pub orbit_radius: f32,
    pub orbit_speed_rad_per_sec: f32,
    pub base_fire_rate_secs: f32, // Added
    pub can_be_deployed_at_location: bool, // Added
    pub deployment_range: f32, // Added
    pub pulses_aoe: bool, // Added
    pub pulse_damage: i32, // Added
    pub pulse_radius: f32, // Added
    pub pulse_interval_secs: f32, // Added
    pub pulse_color: Option<Color>, // Added
    pub fires_seeking_bolts: bool, // Added
    pub bolt_damage: i32, // Added
    pub bolt_speed: f32, // Added
    pub bolt_fire_interval_secs: f32, // Added
    pub bolt_sprite_path: Option<String>, // Added
    pub bolt_size: Option<Vec2>, // Added
    pub bolt_color: Option<Color>, // Added
    pub bolt_lifetime_secs: Option<f32>, // Added
    pub bolt_homing_strength: Option<f32>, // Added
    pub spawn_sound_effect: Option<String>,
}
impl Default for OrbitingPetParams {
    fn default() -> Self {
        Self {
            max_active_orbs: 1, orb_duration_secs: 10.0,
            orb_sprite_path: "sprites/auto_shadow_orb.png".to_string(), orb_size: Vec2::new(32.0, 32.0),
            orb_color: Color::PURPLE, orbit_radius: 75.0, orbit_speed_rad_per_sec: 1.0,
            base_fire_rate_secs: 1.0, // Added default
            can_be_deployed_at_location: false, // Added default
            deployment_range: 0.0, // Added default
            pulses_aoe: true, // Added default
            pulse_damage: 5, // Added default
            pulse_radius: 50.0, // Added default
            pulse_interval_secs: 1.5, // Added default
            pulse_color: Some(Color::rgba(0.5, 0.2, 0.8, 0.4)), // Added default
            fires_seeking_bolts: false, // Added default
            bolt_damage: 0, // Added default
            bolt_speed: 0.0, // Added default
            bolt_fire_interval_secs: 0.0, // Added default
            bolt_sprite_path: None, // Added default
            bolt_size: None, // Added default
            bolt_color: None, // Added default
            bolt_lifetime_secs: None, // Added default
            bolt_homing_strength: None, // Added default
            spawn_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct GroundTargetedAoEParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub targeting_range: f32,
    pub reticle_sprite_path: Option<String>,
    pub visual_sprite_path: Option<String>,
    pub reticle_size: Vec2,
    pub delay_before_eruption_secs: f32,
    pub eruption_radius: f32,
    pub damage: i32,
    pub aoe_color: Color,
    pub aoe_visual_duration_secs: f32,
    pub base_fire_rate_secs: f32, // Added
    pub knock_up_strength: f32, // Added
    pub root_duration_secs: Option<f32>, // Added
    pub fire_sound_effect: Option<String>,
}


#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct LifestealProjectileParams {
    pub base_fire_rate_secs: f32, // Added
    pub base_damage: i32,
    pub projectile_speed: f32,
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_lifetime_secs: f32,
    pub piercing: u32,
    pub lifesteal_percentage: f32,
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct BouncingProjectileParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    // pub num_shards_per_shot: u32, // Not in subtask list
    pub base_damage: i32,
    pub projectile_speed: f32,
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_lifetime_secs: f32,
    pub max_bounces: u32,
    pub base_fire_rate_secs: f32, // Added
    pub num_shards_per_shot: u32, // Added
    pub damage_loss_per_bounce_multiplier: f32, // Added
    pub speed_loss_per_bounce_multiplier: f32, // Added
    pub spread_angle_degrees: f32, // Added
    pub fire_sound_effect: Option<String>,
}


#[derive(Debug, Clone, Copy, Reflect, PartialEq, Default, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub enum ProjectileDebuffType {
    #[default]
    DamageAmp,
    Slow,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct HomingDebuffProjectileParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    // pub num_darts_per_shot: u32, // Not in subtask list
    pub base_damage: i32,
    pub projectile_speed: f32,
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_lifetime_secs: f32,
    pub homing_strength: f32,
    pub debuff_type: ProjectileDebuffType,
    pub base_fire_rate_secs: f32, // Added
    pub num_darts_per_shot: u32, // Added
    pub homing_initial_target_search_radius: f32, // Added
    pub debuff_magnitude_per_stack: f32, // Added
    pub max_debuff_stacks: u32, // Added
    pub debuff_duration_secs_on_target: f32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)] // Default is custom
pub struct ExpandingEnergyBombParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub max_radius: f32,
    pub expansion_duration_secs: f32,
    // pub min_damage_at_min_radius: i32, // Not in subtask list
    pub bomb_color: Color,
    pub visual_sprite_path: Option<String>,
    pub base_fire_rate_secs: f32, // Added
    pub min_damage_at_min_radius: i32, // Added
    pub max_damage_at_max_radius: i32, // Added
    pub detonation_can_be_manual: bool, // Added
    pub auto_detonation_delay_after_max_expansion_secs: f32, // Added
    pub launch_sound_effect: Option<String>,
    pub detonation_sound_effect: Option<String>,
}
impl Default for ExpandingEnergyBombParams {
    fn default() -> Self {
        Self {
            max_radius: 250.0, expansion_duration_secs: 2.5,
            bomb_color: Color::CYAN,
            visual_sprite_path: Some("sprites/spirit_bomb_effect_placeholder.png".to_string()),
            base_fire_rate_secs: 2.0, // Added default
            min_damage_at_min_radius: 10, // Added default
            max_damage_at_max_radius: 80, // Added default
            detonation_can_be_manual: true, // Added default
            auto_detonation_delay_after_max_expansion_secs: 1.0, // Added default
            launch_sound_effect: None,
            detonation_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Copy, Reflect, PartialEq, Default, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub enum AuraDebuffType {
    #[default]
    ReduceAccuracy,
    SlowAttackSpeed,
    MinorDamageOverTime,
}


#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)] // Default is custom
pub struct DebuffAuraParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub cloud_radius: f32,
    pub cloud_duration_secs: f32,
    pub cloud_color: Color,
    pub visual_sprite_path: Option<String>,
    pub debuff_type: AuraDebuffType,
    pub debuff_magnitude: f32,
    pub base_fire_rate_secs: f32, // Added
    pub debuff_duration_secs: f32, // Added
    pub activation_sound_effect: Option<String>,
}
impl Default for DebuffAuraParams {
    fn default() -> Self {
        Self {
            base_fire_rate_secs: 1.0, // Added default
            cloud_radius: 100.0, cloud_duration_secs: 3.0,
            cloud_color: Color::GRAY, visual_sprite_path: Some("sprites/debuff_cloud_placeholder.png".to_string()),
            debuff_type: AuraDebuffType::ReduceAccuracy, debuff_magnitude: 0.2, 
            debuff_duration_secs: 2.0, // Added default
            activation_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct PersistentAuraParams {
    pub is_active_by_default: bool, // Added
    pub damage_per_tick: i32,
    pub tick_interval_secs: f32,
    pub radius: f32,
    pub aura_color: Color,
    pub visual_sprite_path: Option<String>,
    pub fire_rate_secs_placeholder: f32, // Added
    pub activation_sound_effect: Option<String>,
    pub deactivation_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct PointBlankNovaParams {
    pub base_fire_rate_secs: f32, // Added
    pub damage: i32,
    pub radius: f32,
    pub nova_color: Color,
    pub visual_duration_secs: f32, // Added
    pub slow_effect_multiplier: f32, // Added
    pub slow_duration_secs: f32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
pub struct ChainZapParams {
    pub base_fire_rate_secs: f32, // Restored and ensuring it's present
    pub initial_target_range: f32,
    pub max_chains: u32,
    pub chain_search_radius: f32,
    pub base_damage_per_zap: i32,
    pub damage_falloff_per_chain: f32, // Added
    pub zap_color: Color,
    pub zap_width: f32, // Added
    pub zap_duration_secs: f32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)] // Default is custom
pub struct LineDashAttackParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub dash_speed: f32,
    pub dash_duration_secs: f32,
    pub damage_per_hit: i32,
    pub hitbox_width: f32,
    pub base_fire_rate_secs: f32, // Added
    pub piercing_cap: u32, // Added
    pub dash_trail_color: Option<Color>, // Added
    pub invulnerable_during_dash: bool, // Added
    pub fire_sound_effect: Option<String>,
}
impl Default for LineDashAttackParams {
    fn default() -> Self {
        Self {
            base_fire_rate_secs: 1.0, // Added default
            dash_speed: 1000.0,
            dash_duration_secs: 0.3,
            damage_per_hit: 10,
            hitbox_width: 50.0,
            piercing_cap: 3, // Added default
            dash_trail_color: Some(Color::WHITE), // Added default
            invulnerable_during_dash: false, // Added default
            fire_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, Default, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Default, PartialEq)]
pub struct BlinkStrikeProjectileParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub base_damage: i32,
    pub projectile_speed: f32,
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_lifetime_secs: f32,
    pub piercing: u32,
    pub base_fire_rate_secs: f32, // Added
    pub blink_chance_on_hit_percent: f32, // Added
    pub blink_distance: f32, // Added
    pub blink_to_target_behind: bool, // Added
    pub blink_requires_kill: bool, // Added
    pub num_projectiles_per_shot: u32, // Added
    pub fire_sound_effect: Option<String>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq)] // Default is custom
pub struct LobbedBouncingMagmaParams {
    // pub base_fire_rate_secs: f32, // Not in subtask list
    pub projectile_speed: f32,
    pub projectile_sprite_path: String,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_arc_height: f32,
    pub num_bounces: u32,
    pub damage_per_bounce_impact: i32,
    pub bounce_impact_radius: f32,
    pub projectile_lifetime_secs: f32, 
    pub base_fire_rate_secs: f32, // Added
    pub explosion_radius_on_final_bounce: f32, // Added
    pub explosion_damage_on_final_bounce: i32, // Added
    pub fire_pool_on_bounce_chance: f32, // Added
    pub fire_pool_color: Color, // Added
    pub fire_pool_radius: f32, // Added
    pub fire_pool_damage_per_tick: i32, // Added
    pub fire_pool_tick_interval_secs: f32, // Added
    pub fire_pool_duration_secs: f32, // Added
    pub fire_sound_effect: Option<String>,
}

impl Default for LobbedBouncingMagmaParams {
    fn default() -> Self {
        Self {
            base_fire_rate_secs: 0.9, // Added default
            projectile_speed: 350.0,
            projectile_sprite_path: "sprites/magma_ball_placeholder.png".to_string(),
            projectile_size: Vec2::new(28.0, 28.0),
            projectile_color: Color::ORANGE_RED,
            projectile_arc_height: 60.0,
            num_bounces: 3,
            damage_per_bounce_impact: 15,
            bounce_impact_radius: 50.0,
            fire_pool_on_bounce_chance: 0.66, // Added default
            fire_pool_damage_per_tick: 8, // Added default
            fire_pool_radius: 60.0, // Added default
            fire_pool_duration_secs: 2.5, // Added default
            fire_pool_tick_interval_secs: 0.4, // Added default
            fire_pool_color: Color::rgba(1.0, 0.4, 0.0, 0.6), // Added default
            projectile_lifetime_secs: 10.0, 
            explosion_radius_on_final_bounce: 75.0, // Added default
            explosion_damage_on_final_bounce: 40,   // Added default
            fire_sound_effect: None,
        }
    }
}


#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(PartialEq)] // Default is custom
pub enum AttackTypeData {
    StandardProjectile(StandardProjectileParams),
    ReturningProjectile(ReturningProjectileParams),
    ChanneledBeam(ChanneledBeamParams),
    ConeAttack(ConeAttackParams),
    LobbedAoEPool(LobbedAoEPoolParams),
    ChargeUpEnergyShot(ChargeUpEnergyShotParams),
    TrailOfFire(TrailOfFireParams),
    ChainZap(ChainZapParams),
    PointBlankNova(PointBlankNovaParams),
    PersistentAura(PersistentAuraParams),
    DebuffAura(DebuffAuraParams),
    ExpandingEnergyBomb(ExpandingEnergyBombParams),
    HomingDebuffProjectile(HomingDebuffProjectileParams),
    BouncingProjectile(BouncingProjectileParams),
    LifestealProjectile(LifestealProjectileParams),
    GroundTargetedAoE(GroundTargetedAoEParams),
    LineDashAttack(LineDashAttackParams),
    OrbitingPet(OrbitingPetParams),
    RepositioningTether(RepositioningTetherParams),
    BlinkStrikeProjectile(BlinkStrikeProjectileParams),
    LobbedBouncingMagma(LobbedBouncingMagmaParams),
}
// Ensure Default impl for AttackTypeData is still valid with potentially changed StandardProjectileParams
impl Default for AttackTypeData {
    fn default() -> Self {
        AttackTypeData::StandardProjectile(StandardProjectileParams::default())
    }
}


#[derive(Debug, Clone, Reflect, Default, Serialize, Deserialize)] // Added Default, Serialize, Deserialize
#[reflect(Default)] // Added reflect Default
pub struct ItemDefinition {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub effects: Vec<ItemEffect>,
    pub icon_path: String,
}

#[derive(Resource, Default, Reflect, Serialize, Deserialize)] #[reflect(Resource)] // Added Serialize, Deserialize
pub struct ItemLibrary { pub items: Vec<ItemDefinition>, }
impl ItemLibrary { pub fn get_item_definition(&self, id: ItemId) -> Option<&ItemDefinition> { self.items.iter().find(|def| def.id == id) } }

#[derive(Component, Debug, Reflect, Default, Serialize, Deserialize)] // Added Reflect, Default, Serialize, Deserialize
#[reflect(Component, Default)] // Added reflect Component, Default
pub struct ItemDrop { pub item_id: ItemId, }
pub const ITEM_DROP_SIZE: Vec2 = Vec2::new(24.0, 24.0);

#[derive(Component, Reflect, Default, Debug, Serialize, Deserialize)] #[reflect(Component, Default)] // Added Serialize, Deserialize and reflect Default
pub struct ExplosionEffect { pub damage: i32, pub radius_sq: f32, #[serde(skip)] pub timer: Timer, pub already_hit_entities: Vec<Entity>, }
#[derive(Component, Reflect, Default, Debug, Serialize, Deserialize)] #[reflect(Component, Default)] // Added Serialize, Deserialize and reflect Default
pub struct RetaliationNovaEffect { pub damage: i32, pub radius_sq: f32, #[serde(skip)] pub timer: Timer, pub already_hit_entities: Vec<Entity>, }
#[derive(Component, Reflect, Default, Debug, Serialize, Deserialize)] #[reflect(Component, Default)] // Added Serialize, Deserialize and reflect Default
pub struct TemporaryHealthRegenBuff { pub regen_per_second: f32, #[serde(skip)] pub duration_timer: Timer, }


#[derive(Debug, Clone, Reflect, Default, Serialize, Deserialize)] // Added Default, Serialize, Deserialize
#[reflect(Default)] // Added reflect Default
pub struct AutomaticWeaponDefinition {
    pub id: AutomaticWeaponId,
    pub name: String,
    pub attack_data: AttackTypeData,
}

#[derive(Resource, Default, Reflect, Serialize, Deserialize)] // Added Serialize, Deserialize
#[reflect(Resource)]
pub struct AutomaticWeaponLibrary {
    pub weapons: Vec<AutomaticWeaponDefinition>,
}

impl AutomaticWeaponLibrary {
    pub fn get_weapon_definition(&self, id: AutomaticWeaponId) -> Option<&AutomaticWeaponDefinition> {
        self.weapons.iter().find(|def| def.id == id)
    }
}


pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app .register_type::<ItemId>() 
            .register_type::<SurvivorTemporaryBuff>() 
            .register_type::<ItemEffect>() 
            .register_type::<ItemDrop>() // Added ItemDrop registration
            .register_type::<ItemLibrary>() // Already Reflect, Resource, Default, Serialize, Deserialize
            .register_type::<ExplosionEffect>()      // Reflect, Component, Default, Debug, Serialize, Deserialize
            .register_type::<RetaliationNovaEffect>() // Reflect, Component, Default, Debug, Serialize, Deserialize
            .register_type::<TemporaryHealthRegenBuff>() // Reflect, Component, Default, Debug, Serialize, Deserialize
            .register_type::<AutomaticWeaponId>() // Reflect, Default
            .register_type::<StandardProjectileParams>() // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<ReturningProjectileParams>() // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<ChanneledBeamParams>()      // Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<ConeAttackParams>()         // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<LobbedAoEPoolParams>()      // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<ChargeLevelParams>()        // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<ChargeUpEnergyShotParams>() // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<TrailOfFireParams>()        // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<ChainZapParams>()           // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<PointBlankNovaParams>()     // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<PersistentAuraParams>()     // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<AuraDebuffType>()           // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<DebuffAuraParams>()         // Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<ExpandingEnergyBombParams>()// Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<ProjectileDebuffType>()     // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<HomingDebuffProjectileParams>()// Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<BouncingProjectileParams>() // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<LifestealProjectileParams>()// Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<GroundTargetedAoEParams>()  // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<LineDashAttackParams>()     // Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<OrbitingPetParams>()        // Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<RepositioningTetherMode>()  // Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<RepositioningTetherParams>()// Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<BlinkStrikeProjectileParams>()// Reflect, Default, PartialEq, Serialize, Deserialize
            .register_type::<LobbedBouncingMagmaParams>()// Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<AttackTypeData>()           // Reflect, PartialEq, Serialize, Deserialize (custom Default)
            .register_type::<AutomaticWeaponDefinition>() // Reflect, Default, Serialize, Deserialize
            .register_type::<AutomaticWeaponLibrary>()   // Reflect, Resource, Default, Serialize, Deserialize
            .init_resource::<ItemLibrary>()
            .init_resource::<AutomaticWeaponLibrary>()
            .add_systems(Startup, (populate_item_library, populate_automatic_weapon_library) )
            .add_systems(Update, ( apply_collected_item_effects_system.run_if(on_event::<ItemCollectedEvent>()), explosion_effect_system.run_if(in_state(AppState::InGame)), retaliation_nova_effect_system.run_if(in_state(AppState::InGame)), temporary_health_regen_buff_system.run_if(in_state(AppState::InGame)), ));
    }
}

pub fn populate_automatic_weapon_library(mut library: ResMut<AutomaticWeaponLibrary>) {
    let all_definitions = automatic_weapons::get_all_weapon_definitions();
    for def in all_definitions {
        library.weapons.push(def);
    }
}


fn populate_item_library(mut library: ResMut<ItemLibrary>) {
    library.items.push(ItemDefinition { id: ItemId(1), name: "Corrupted Heart".to_string(), description: "Increases Max Health by 25.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: Some(25), speed_multiplier: None, damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_corrupted_heart_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(2), name: "Whispering Idol".to_string(), description: "Increases Movement Speed by 15%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: Some(1.15), damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_whispering_idol_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(3), name: "Shard of Agony".to_string(), description: "Increases automatic weapon damage by 5.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: Some(5), xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_shard_of_agony_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(4), name: "Occult Tome Fragment".to_string(), description: "Increases XP gain by 20%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: None, xp_gain_multiplier: Some(1.20), pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_occult_tome_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(5), name: "Grasping Tentacle (Dried)".to_string(), description: "Increases pickup radius by 25%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: Some(0.25), auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_grasping_tentacle_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(6), name: "Fragmented Sanity".to_string(), description: "Your automatic projectiles have a chance to violently detonate on impact.".to_string(), effects: vec![ItemEffect::OnAutomaticProjectileHitExplode { chance: 0.15, explosion_damage: 20, explosion_radius: 75.0, explosion_color: Color::rgba(1.0, 0.5, 0.2, 0.6), }], icon_path: "sprites/icons/item_fragmented_sanity_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(7), name: "Cloak of VengefulSpirits".to_string(), description: "When struck, has a chance to unleash a damaging psychic nova.".to_string(), effects: vec![ItemEffect::OnSurvivorHitRetaliate { chance: 0.25, retaliation_damage: 30, retaliation_radius: 120.0, retaliation_color: Color::rgba(0.9, 0.1, 0.1, 0.5), }], icon_path: "sprites/icons/item_cloak_vengeful_spirits_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(8), name: "Soul Siphon Shard".to_string(), description: "Defeated foes have a 20% chance to grant brief, rapid health regeneration.".to_string(), effects: vec![ItemEffect::OnHorrorKillTrigger { chance: 0.20, effect: SurvivorTemporaryBuff::HealthRegen { rate: 5.0, duration_secs: 3.0 }, }], icon_path: "sprites/icons/item_soul_siphon_shard_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(9), name: "Tome of Forbidden Rites".to_string(), description: "Grants knowledge of the 'Void Lance' skill.".to_string(), effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(3) }], icon_path: "sprites/icons/item_tome_forbidden_rites_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(10), name: "Glyph-Etched Wardstone".to_string(), description: "Activates a Circle of Warding, damaging nearby foes.".to_string(), effects: vec![ItemEffect::ActivateCircleOfWarding { base_damage: 3, base_radius: 75.0, base_tick_interval: 0.5, }], icon_path: "sprites/icons/item_glyph_wardstone_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(11), name: "Broodmother's Oculus".to_string(), description: "Summons a Swarm of Nightmares to orbit and attack enemies.".to_string(), effects: vec![ItemEffect::ActivateSwarmOfNightmares { num_larvae: 2, base_damage: 5, base_orbit_radius: 80.0, base_rotation_speed: std::f32::consts::PI / 2.0, }], icon_path: "sprites/icons/item_broodmother_oculus_placeholder.png".to_string() });
    library.items.push(ItemDefinition { id: ItemId(12), name: "Crystalline Conduit".to_string(), description: "Increases automatic weapon damage by +3 and projectile speed by +10%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: Some(3), xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: Some(0.10) }], icon_path: "sprites/icons/item_crystalline_conduit_placeholder.png".to_string() });
    library.items.push(ItemDefinition {
        id: ItemId(13),
        name: "Tome of Shattered Thoughts".to_string(),
        description: "Unlocks the 'Mind Shatter' psychic burst skill.".to_string(),
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(2) }],
        icon_path: "sprites/icons/item_tome_mind_shatter_placeholder.png".to_string()
    });
    library.items.push(ItemDefinition {
        id: ItemId(14),
        name: "Tome of the Glacial Heart".to_string(),
        description: "Unlocks the 'Glacial Nova' chilling skill.".to_string(),
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(5) }],
        icon_path: "sprites/icons/item_tome_glacial_nova_placeholder.png".to_string()
    });
    library.items.push(ItemDefinition {
        id: ItemId(15),
        name: "Tome of the Watcher".to_string(),
        description: "Unlocks the 'Psychic Sentry' summoning skill.".to_string(),
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(6) }],
        icon_path: "sprites/icons/item_tome_psychic_sentry_placeholder.png".to_string()
    });
     library.items.push(ItemDefinition {
        id: ItemId(16),
        name: "Tome of Ethereal Defense".to_string(),
        description: "Unlocks the 'Ethereal Ward' defensive skill.".to_string(),
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(7) }],
        icon_path: "sprites/icons/item_tome_ethereal_ward_placeholder.png".to_string()
    });

}

fn apply_collected_item_effects_system(
    mut events: EventReader<ItemCollectedEvent>,
    mut player_query: Query<(&mut Survivor, Option<&mut Health>, Option<&mut CircleOfWarding>, Option<&mut SwarmOfNightmares>)>,
    item_library: Res<ItemLibrary>,
    skill_library: Res<SkillLibrary>,
) {
    if let Ok((mut player, mut opt_health_component, mut opt_circle_aura, mut opt_nightmare_swarm)) = player_query.get_single_mut() {
        for event in events.read() {
            let item_id = event.0;
            let is_new_item = !player.collected_item_ids.contains(&item_id);

            if let Some(item_def) = item_library.get_item_definition(item_id) {
                let mut applied_successfully = true;
                if is_new_item {
                    for effect in &item_def.effects {
                        match effect {
                            ItemEffect::PassiveStatBoost {
                                max_health_increase,
                                speed_multiplier,
                                damage_increase,
                                xp_gain_multiplier,
                                pickup_radius_increase,
                                auto_weapon_projectile_speed_multiplier_increase
                            } => {
                                if let Some(hp_boost) = max_health_increase { player.max_health += *hp_boost; if let Some(ref mut health_comp) = opt_health_component { health_comp.0 += *hp_boost; health_comp.0 = health_comp.0.min(player.max_health); } }
                                if let Some(speed_mult) = speed_multiplier { player.speed *= *speed_mult; }
                                if let Some(dmg_inc) = damage_increase { player.auto_weapon_damage_bonus += *dmg_inc; }
                                if let Some(xp_mult) = xp_gain_multiplier { player.xp_gain_multiplier *= *xp_mult; }
                                if let Some(radius_inc_percent) = pickup_radius_increase { player.pickup_radius_multiplier *= 1.0 + radius_inc_percent; }
                                if let Some(projectile_speed_inc) = auto_weapon_projectile_speed_multiplier_increase { player.auto_weapon_projectile_speed_multiplier *= 1.0 + projectile_speed_inc; }
                            }
                            ItemEffect::GrantSpecificSkill { skill_id } => {
                                if let Some(skill_to_grant_def) = skill_library.get_skill_definition(*skill_id) {
                                    let already_has_skill = player.equipped_skills.iter().any(|s| s.definition_id == *skill_id);
                                    if !already_has_skill { if player.equipped_skills.len() < 5 {
                                        player.equipped_skills.push(ActiveSkillInstance::new(skill_to_grant_def.id ));
                                    } else { applied_successfully = false; }
                                    } else { applied_successfully = false;
                                    }
                                } else { applied_successfully = false; }
                            }
                            ItemEffect::ActivateCircleOfWarding { base_damage, base_radius, base_tick_interval } => {
                                if let Some(ref mut circle_aura) = opt_circle_aura {
                                    if !circle_aura.is_active {
                                        circle_aura.is_active = true;
                                        circle_aura.base_damage_per_tick = *base_damage;
                                        circle_aura.current_radius = *base_radius;
                                        circle_aura.damage_tick_timer = Timer::from_seconds(*base_tick_interval, TimerMode::Repeating);
                                    } else {
                                        circle_aura.base_damage_per_tick += 1;
                                        circle_aura.current_radius *= 1.05;
                                    }
                                } else { applied_successfully = false; }
                            }
                            ItemEffect::ActivateSwarmOfNightmares { num_larvae, base_damage, base_orbit_radius, base_rotation_speed } => {
                                if let Some(ref mut nightmare_swarm) = opt_nightmare_swarm {
                                    if !nightmare_swarm.is_active {
                                        nightmare_swarm.is_active = true;
                                        nightmare_swarm.num_larvae = *num_larvae;
                                        nightmare_swarm.damage_per_hit = *base_damage;
                                        nightmare_swarm.orbit_radius = *base_orbit_radius;
                                        nightmare_swarm.rotation_speed = *base_rotation_speed;
                                    } else {
                                        nightmare_swarm.num_larvae = (nightmare_swarm.num_larvae + 1).min(8);
                                        nightmare_swarm.damage_per_hit += 1;
                                    }
                                } else { applied_successfully = false; }
                            }
                            ItemEffect::OnAutomaticProjectileHitExplode {..} | ItemEffect::OnSurvivorHitRetaliate {..} | ItemEffect::OnHorrorKillTrigger {..} => {
                            }
                        }
                    }
                }
                if is_new_item && applied_successfully {
                     player.collected_item_ids.push(item_id);
                } else if !is_new_item {
                }
            }
        }
    }
}

fn explosion_effect_system( mut commands: Commands, time: Res<Time>, mut explosion_query: Query<(Entity, &mut ExplosionEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (explosion_entity, mut explosion, explosion_g_transform, mut sprite, mut vis_transform) in explosion_query.iter_mut() { explosion.timer.tick(time.delta()); let progress = explosion.timer.percent(); let current_radius = explosion.radius_sq.sqrt(); vis_transform.scale = Vec3::splat(current_radius * 2.0 * progress); sprite.color.set_a(1.0 - progress); if explosion.timer.percent() < 0.5 { let explosion_pos = explosion_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() { if explosion.already_hit_entities.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(explosion_pos) < explosion.radius_sq { horror_health.0 -= explosion.damage; visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), explosion.damage, &time); sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); explosion.already_hit_entities.push(horror_entity); } } } if explosion.timer.finished() { commands.entity(explosion_entity).despawn_recursive(); } } }
fn retaliation_nova_effect_system( mut commands: Commands, time: Res<Time>, mut nova_query: Query<(Entity, &mut RetaliationNovaEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (nova_entity, mut nova, nova_g_transform, mut sprite, mut vis_transform) in nova_query.iter_mut() { nova.timer.tick(time.delta()); let progress = nova.timer.percent(); let current_radius = nova.radius_sq.sqrt(); vis_transform.scale = Vec3::splat(current_radius * 2.0 * progress); sprite.color.set_a(1.0 - progress * progress); if nova.timer.percent() < 0.3 { let nova_pos = nova_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() { if nova.already_hit_entities.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(nova_pos) < nova.radius_sq { horror_health.0 -= nova.damage; visual_effects::spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), nova.damage, &time); sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); nova.already_hit_entities.push(horror_entity); } } } if nova.timer.finished() { commands.entity(nova_entity).despawn_recursive(); } } }
fn temporary_health_regen_buff_system( mut commands: Commands, time: Res<Time>, mut buff_query: Query<(Entity, &mut TemporaryHealthRegenBuff)>, mut player_query: Query<(&Survivor, &mut Health)>) {
    if let Ok((survivor_stats, mut health_component)) = player_query.get_single_mut() {
        for (entity, mut buff) in buff_query.iter_mut() {
            buff.duration_timer.tick(time.delta());
            if buff.duration_timer.finished() {
                commands.entity(entity).remove::<TemporaryHealthRegenBuff>();
            } else {
                let regen_amount = buff.regen_per_second * time.delta().as_secs_f32();
                health_component.0 = (health_component.0 as f32 + regen_amount).round() as i32;
                health_component.0 = health_component.0.min(survivor_stats.max_health);
            }
        }
    }
}