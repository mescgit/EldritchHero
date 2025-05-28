// src/items.rs
use bevy::prelude::*;
use crate::{
    survivor::Survivor,
    components::{Health as ComponentHealth, Health},
    game::{AppState, ItemCollectedEvent},
    horror::Horror,
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
    skills::{SkillId, SkillLibrary, ActiveSkillInstance},
    weapons::{CircleOfWarding, SwarmOfNightmares},
};

// --- Standard Items (Relics) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SurvivorTemporaryBuff { HealthRegen { rate: f32, duration_secs: f32 }, }

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ItemEffect {
    PassiveStatBoost {
        max_health_increase: Option<i32>,
        speed_multiplier: Option<f32>,
        damage_increase: Option<i32>,
        xp_gain_multiplier: Option<f32>,
        pickup_radius_increase: Option<f32>,
        auto_weapon_projectile_speed_multiplier_increase: Option<f32>,
    },
    OnAutomaticProjectileHitExplode { chance: f32, explosion_damage: i32, explosion_radius: f32, explosion_color: Color, },
    OnSurvivorHitRetaliate { chance: f32, retaliation_damage: i32, retaliation_radius: f32, retaliation_color: Color, },
    OnHorrorKillTrigger { chance: f32, effect: SurvivorTemporaryBuff, },
    GrantSpecificSkill { skill_id: SkillId, },
    ActivateCircleOfWarding { base_damage: i32, base_radius: f32, base_tick_interval: f32 },
    ActivateSwarmOfNightmares { num_larvae: u32, base_damage: i32, base_orbit_radius: f32, base_rotation_speed: f32 },
}

#[derive(Debug, Clone, Reflect)]
pub struct ItemDefinition {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub effects: Vec<ItemEffect>,
    pub icon_path: &'static str, 
}

#[derive(Resource, Default, Reflect)] #[reflect(Resource)]
pub struct ItemLibrary { pub items: Vec<ItemDefinition>, }
impl ItemLibrary { pub fn get_item_definition(&self, id: ItemId) -> Option<&ItemDefinition> { self.items.iter().find(|def| def.id == id) } }

#[derive(Component, Debug)] pub struct ItemDrop { pub item_id: ItemId, }
pub const ITEM_DROP_SIZE: Vec2 = Vec2::new(24.0, 24.0);

#[derive(Component, Reflect, Default, Debug)] #[reflect(Component)]
pub struct ExplosionEffect { pub damage: i32, pub radius_sq: f32, pub timer: Timer, pub already_hit_entities: Vec<Entity>, }
#[derive(Component, Reflect, Default, Debug)] #[reflect(Component)]
pub struct RetaliationNovaEffect { pub damage: i32, pub radius_sq: f32, pub timer: Timer, pub already_hit_entities: Vec<Entity>, }
#[derive(Component, Reflect, Default, Debug)] #[reflect(Component)]
pub struct TemporaryHealthRegenBuff { pub regen_per_second: f32, pub duration_timer: Timer, }

// --- Automatic Weapons ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub struct AutomaticWeaponId(pub u32);

// --- New Structs and Enum for Attack Types ---

#[derive(Debug, Clone, Reflect)]
pub struct StandardProjectileParams {
    pub base_damage: i32,
    pub base_fire_rate_secs: f32,
    pub base_projectile_speed: f32,
    pub base_piercing: u32,
    pub additional_projectiles: u32, // Number of projectiles fired at once
    pub projectile_sprite_path: &'static str,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_lifetime_secs: f32,
}

#[derive(Debug, Clone, Reflect)]
pub struct ReturningProjectileParams {
    pub base_damage: i32,
    pub base_fire_rate_secs: f32,
    pub projectile_sprite_path: &'static str,
    pub projectile_size: Vec2,
    pub projectile_color: Color,
    pub projectile_speed: f32,
    pub travel_distance: f32, // Max distance before returning
    pub piercing: u32, // Piercing on outgoing and return
    // Add other specific fields as needed, e.g., hover_duration_secs
}

#[derive(Debug, Clone, Reflect)]
pub struct ChanneledBeamParams {
    pub base_damage_per_tick: i32,
    pub tick_rate_secs: f32, // How often damage is applied
    pub range: f32,
    pub beam_width: f32, // For visual representation and collision
    pub beam_color: Color,
    pub movement_penalty_multiplier: f32, // e.g., 0.5 for 50% speed
    // Add other specific fields, e.g., ramp_up_stats
}

#[derive(Debug, Clone, Reflect)]
pub struct ConeAttackParams {
    pub base_damage: i32,
    pub base_fire_rate_secs: f32,
    pub cone_angle_degrees: f32,
    pub cone_radius: f32,
    pub color: Color, // For visual effect
    // Add other specific fields, e.g., knockback_strength
}

#[derive(Debug, Clone, Reflect)]
pub enum AttackTypeData {
    StandardProjectile(StandardProjectileParams),
    ReturningProjectile(ReturningProjectileParams),
    ChanneledBeam(ChanneledBeamParams),
    ConeAttack(ConeAttackParams),
    // We will add more variants here as we implement more attack types
}

#[derive(Debug, Clone, Reflect)]
pub struct AutomaticWeaponDefinition {
    pub id: AutomaticWeaponId,
    pub name: String,
    pub attack_data: AttackTypeData,
}

#[derive(Resource, Default, Reflect)]
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
        app .register_type::<ItemId>() .register_type::<SurvivorTemporaryBuff>() .register_type::<ItemEffect>() .register_type::<ItemLibrary>()
            .register_type::<ExplosionEffect>() .register_type::<RetaliationNovaEffect>() .register_type::<TemporaryHealthRegenBuff>()
            .register_type::<AutomaticWeaponId>() 
            .register_type::<StandardProjectileParams>() .register_type::<ReturningProjectileParams>() .register_type::<ChanneledBeamParams>() .register_type::<ConeAttackParams>() .register_type::<AttackTypeData>()
            .register_type::<AutomaticWeaponDefinition>() .register_type::<AutomaticWeaponLibrary>()
            .init_resource::<ItemLibrary>()
            .init_resource::<AutomaticWeaponLibrary>()
            .add_systems(Startup, (populate_item_library, populate_automatic_weapon_library) )
            .add_systems(Update, ( apply_collected_item_effects_system.run_if(on_event::<ItemCollectedEvent>()), explosion_effect_system.run_if(in_state(AppState::InGame)), retaliation_nova_effect_system.run_if(in_state(AppState::InGame)), temporary_health_regen_buff_system.run_if(in_state(AppState::InGame)), ));
    }
}

fn populate_automatic_weapon_library(mut library: ResMut<AutomaticWeaponLibrary>) {
    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(0),
        name: "Primordial Ichor Blast".to_string(),
        base_damage: 10,
        base_fire_rate_secs: 0.5,
        base_projectile_speed: 600.0,
        base_piercing: 0,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/ichor_blast_placeholder.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.7, 0.5, 1.0),
        projectile_lifetime_secs: 2.0,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(1),
        name: "Eldritch Gatling".to_string(),
        base_damage: 5,
        base_fire_rate_secs: 0.15,
        base_projectile_speed: 550.0,
        base_piercing: 0,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/eldritch_gatling_projectile_placeholder.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.3, 0.9, 0.4),
        projectile_lifetime_secs: 1.5,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(2),
        name: "Void Cannon".to_string(),
        base_damage: 30,
        base_fire_rate_secs: 1.25,
        base_projectile_speed: 450.0,
        base_piercing: 1,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/void_cannon_projectile_placeholder.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.4, 0.1, 0.7),
        projectile_lifetime_secs: 2.5,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(3), 
        name: "Spectral Blades".to_string(),
        base_damage: 12,
        base_fire_rate_secs: 0.75, 
        base_projectile_speed: 400.0,
        base_piercing: 0,
        additional_projectiles: 2, 
        projectile_sprite_path: "sprites/spectral_blade_placeholder.png", 
        projectile_size: Vec2::new(50.0, 50.0), 
        projectile_color: Color::rgb(0.6, 0.9, 1.0), 
        projectile_lifetime_secs: 0.4, 
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(4),
        name: "Inferno Bolt".to_string(),
        base_damage: 25,
        base_fire_rate_secs: 0.8,
        base_projectile_speed: 700.0,
        base_piercing: 1,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_inferno_bolt.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(1.0, 0.3, 0.0),
        projectile_lifetime_secs: 1.5,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(5),
        name: "Chain Lightning".to_string(),
        base_damage: 15,
        base_fire_rate_secs: 1.2,
        base_projectile_speed: 800.0,
        base_piercing: 2, // Can hit multiple targets
        additional_projectiles: 1, // Represents the chaining idea
        projectile_sprite_path: "sprites/auto_chain_lightning.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.5, 0.8, 1.0),
        projectile_lifetime_secs: 1.0,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(6),
        name: "Arcane Ray".to_string(),
        base_damage: 40,
        base_fire_rate_secs: 1.5,
        base_projectile_speed: 900.0,
        base_piercing: 0, // Concentrated beam
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_arcane_ray.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.7, 0.2, 0.9),
        projectile_lifetime_secs: 0.8,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(7),
        name: "Shadow Orb".to_string(),
        base_damage: 18,
        base_fire_rate_secs: 0.6,
        base_projectile_speed: 400.0, // Slower, menacing
        base_piercing: 1,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_shadow_orb.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.2, 0.1, 0.3),
        projectile_lifetime_secs: 2.0,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(8),
        name: "Holy Lance".to_string(),
        base_damage: 30,
        base_fire_rate_secs: 1.0,
        base_projectile_speed: 750.0,
        base_piercing: 3, // Piercing holy power
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_holy_lance.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(1.0, 1.0, 0.5),
        projectile_lifetime_secs: 1.2,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(9),
        name: "Venom Spit".to_string(),
        base_damage: 10, // Damage over time implied elsewhere
        base_fire_rate_secs: 0.4,
        base_projectile_speed: 500.0,
        base_piercing: 0,
        additional_projectiles: 2, // Multiple globs
        projectile_sprite_path: "sprites/auto_venom_spit.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.2, 0.8, 0.1),
        projectile_lifetime_secs: 1.8,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(10),
        name: "Glacial Spike".to_string(),
        base_damage: 22,
        base_fire_rate_secs: 0.9,
        base_projectile_speed: 600.0,
        base_piercing: 1, // Chilling effect implied
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_glacial_spike.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.4, 0.7, 1.0),
        projectile_lifetime_secs: 1.3,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(11),
        name: "EarthShatter Shard".to_string(),
        base_damage: 35,
        base_fire_rate_secs: 1.8, // Slow, impactful
        base_projectile_speed: 300.0, // Slow moving earth
        base_piercing: 0, // Area effect implied
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_earthshatter_shard.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.6, 0.4, 0.2),
        projectile_lifetime_secs: 2.2,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(12),
        name: "Sunfire Burst".to_string(),
        base_damage: 28,
        base_fire_rate_secs: 0.7,
        base_projectile_speed: 850.0,
        base_piercing: 0,
        additional_projectiles: 3, // Burst effect
        projectile_sprite_path: "sprites/auto_sunfire_burst.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(1.0, 0.8, 0.2),
        projectile_lifetime_secs: 0.5, // Short burst
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(13),
        name: "Moonbeam Dart".to_string(),
        base_damage: 12,
        base_fire_rate_secs: 0.3, // Fast darts
        base_projectile_speed: 950.0,
        base_piercing: 1,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_moonbeam_dart.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.7, 0.7, 0.9),
        projectile_lifetime_secs: 1.0,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(14),
        name: "Spirit Bomb".to_string(),
        base_damage: 50, // High damage, slow fire rate
        base_fire_rate_secs: 2.0,
        base_projectile_speed: 250.0, // Slow moving orb
        base_piercing: 0, // Large AoE implied
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_spirit_bomb.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.6, 1.0, 0.9),
        projectile_lifetime_secs: 3.0,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(15),
        name: "Void Tendril".to_string(),
        base_damage: 18,
        base_fire_rate_secs: 0.65,
        base_projectile_speed: 550.0,
        base_piercing: 2, // Lashing out
        additional_projectiles: 1, // Could be multiple tendrils
        projectile_sprite_path: "sprites/auto_void_tendril.png",
        projectile_size: Vec2::new(50.0, 50.0), // Long and thin
        projectile_color: Color::rgb(0.3, 0.0, 0.5),
        projectile_lifetime_secs: 0.7,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(16),
        name: "Crystal Shard".to_string(),
        base_damage: 10,
        base_fire_rate_secs: 0.2, // Very fast firing
        base_projectile_speed: 700.0,
        base_piercing: 0,
        additional_projectiles: 4, // Shotgun-like spread
        projectile_sprite_path: "sprites/auto_crystal_shard.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.8, 0.6, 1.0),
        projectile_lifetime_secs: 0.4,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(17),
        name: "Magma Ball".to_string(),
        base_damage: 32,
        base_fire_rate_secs: 1.3,
        base_projectile_speed: 400.0,
        base_piercing: 0, // Splash damage implied
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_magma_ball.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.9, 0.2, 0.0),
        projectile_lifetime_secs: 2.5,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(18),
        name: "Sand Blast".to_string(),
        base_damage: 8,
        base_fire_rate_secs: 0.1, // Extremely fast, low damage
        base_projectile_speed: 600.0,
        base_piercing: 0,
        additional_projectiles: 2, // Wide cone
        projectile_sprite_path: "sprites/auto_sand_blast.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.9, 0.8, 0.5),
        projectile_lifetime_secs: 0.3, // Short range
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(19),
        name: "Metal Shrapnel".to_string(),
        base_damage: 12,
        base_fire_rate_secs: 0.5,
        base_projectile_speed: 650.0,
        base_piercing: 3, // Piercing metal
        additional_projectiles: 3, // Multiple pieces
        projectile_sprite_path: "sprites/auto_metal_shrapnel.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.6, 0.6, 0.6),
        projectile_lifetime_secs: 0.9,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(20),
        name: "Nature's Wrath".to_string(),
        base_damage: 20,
        base_fire_rate_secs: 1.1,
        base_projectile_speed: 500.0,
        base_piercing: 1,
        additional_projectiles: 1, // Thorny vine or similar
        projectile_sprite_path: "sprites/auto_natures_wrath.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.1, 0.6, 0.2),
        projectile_lifetime_secs: 1.6,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(21),
        name: "Chi Bolt".to_string(),
        base_damage: 18,
        base_fire_rate_secs: 0.45,
        base_projectile_speed: 750.0,
        base_piercing: 0,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_chi_bolt.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.5, 0.9, 0.8),
        projectile_lifetime_secs: 1.1,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(22),
        name: "Psionic Lash".to_string(),
        base_damage: 26,
        base_fire_rate_secs: 0.85,
        base_projectile_speed: 600.0, // Mental force, not super fast but accurate
        base_piercing: 2,
        additional_projectiles: 0,
        projectile_sprite_path: "sprites/auto_psionic_lash.png",
        projectile_size: Vec2::new(50.0, 50.0), // Whip-like
        projectile_color: Color::rgb(0.8, 0.4, 0.9),
        projectile_lifetime_secs: 0.6,
    });

    library.weapons.push(AutomaticWeaponDefinition {
        id: AutomaticWeaponId(23),
        name: "Aether Bolt".to_string(),
        base_damage: 14,
        base_fire_rate_secs: 0.25,
        base_projectile_speed: 1000.0, // Very fast
        base_piercing: 1,
        additional_projectiles: 1,
        projectile_sprite_path: "sprites/auto_aether_bolt.png",
        projectile_size: Vec2::new(50.0, 50.0),
        projectile_color: Color::rgb(0.9, 0.9, 0.9),
        projectile_lifetime_secs: 1.4,
    });
}

fn populate_item_library(mut library: ResMut<ItemLibrary>) {
    library.items.push(ItemDefinition { id: ItemId(1), name: "Corrupted Heart".to_string(), description: "Increases Max Health by 25.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: Some(25), speed_multiplier: None, damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_corrupted_heart_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(2), name: "Whispering Idol".to_string(), description: "Increases Movement Speed by 15%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: Some(1.15), damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_whispering_idol_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(3), name: "Shard of Agony".to_string(), description: "Increases automatic weapon damage by 5.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: Some(5), xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_shard_of_agony_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(4), name: "Occult Tome Fragment".to_string(), description: "Increases XP gain by 20%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: None, xp_gain_multiplier: Some(1.20), pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_occult_tome_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(5), name: "Grasping Tentacle (Dried)".to_string(), description: "Increases pickup radius by 25%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: None, xp_gain_multiplier: None, pickup_radius_increase: Some(0.25), auto_weapon_projectile_speed_multiplier_increase: None }], icon_path: "sprites/icons/item_grasping_tentacle_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(6), name: "Fragmented Sanity".to_string(), description: "Your automatic projectiles have a chance to violently detonate on impact.".to_string(), effects: vec![ItemEffect::OnAutomaticProjectileHitExplode { chance: 0.15, explosion_damage: 20, explosion_radius: 75.0, explosion_color: Color::rgba(1.0, 0.5, 0.2, 0.6), }], icon_path: "sprites/icons/item_fragmented_sanity_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(7), name: "Cloak of VengefulSpirits".to_string(), description: "When struck, has a chance to unleash a damaging psychic nova.".to_string(), effects: vec![ItemEffect::OnSurvivorHitRetaliate { chance: 0.25, retaliation_damage: 30, retaliation_radius: 120.0, retaliation_color: Color::rgba(0.9, 0.1, 0.1, 0.5), }], icon_path: "sprites/icons/item_cloak_vengeful_spirits_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(8), name: "Soul Siphon Shard".to_string(), description: "Defeated foes have a 20% chance to grant brief, rapid health regeneration.".to_string(), effects: vec![ItemEffect::OnHorrorKillTrigger { chance: 0.20, effect: SurvivorTemporaryBuff::HealthRegen { rate: 5.0, duration_secs: 3.0 }, }], icon_path: "sprites/icons/item_soul_siphon_shard_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(9), name: "Tome of Forbidden Rites".to_string(), description: "Grants knowledge of the 'Void Lance' skill.".to_string(), effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(3) }], icon_path: "sprites/icons/item_tome_forbidden_rites_placeholder.png" }); // Existing Tome
    library.items.push(ItemDefinition { id: ItemId(10), name: "Glyph-Etched Wardstone".to_string(), description: "Activates a Circle of Warding, damaging nearby foes.".to_string(), effects: vec![ItemEffect::ActivateCircleOfWarding { base_damage: 3, base_radius: 75.0, base_tick_interval: 0.5, }], icon_path: "sprites/icons/item_glyph_wardstone_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(11), name: "Broodmother's Oculus".to_string(), description: "Summons a Swarm of Nightmares to orbit and attack enemies.".to_string(), effects: vec![ItemEffect::ActivateSwarmOfNightmares { num_larvae: 2, base_damage: 5, base_orbit_radius: 80.0, base_rotation_speed: std::f32::consts::PI / 2.0, }], icon_path: "sprites/icons/item_broodmother_oculus_placeholder.png" });
    library.items.push(ItemDefinition { id: ItemId(12), name: "Crystalline Conduit".to_string(), description: "Increases automatic weapon damage by +3 and projectile speed by +10%.".to_string(), effects: vec![ItemEffect::PassiveStatBoost { max_health_increase: None, speed_multiplier: None, damage_increase: Some(3), xp_gain_multiplier: None, pickup_radius_increase: None, auto_weapon_projectile_speed_multiplier_increase: Some(0.10) }], icon_path: "sprites/icons/item_crystalline_conduit_placeholder.png" });

    // New Tomes for Granting Skills
    library.items.push(ItemDefinition { 
        id: ItemId(13), 
        name: "Tome of Shattered Thoughts".to_string(), 
        description: "Unlocks the 'Mind Shatter' psychic burst skill.".to_string(), 
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(2) }], 
        icon_path: "sprites/icons/item_tome_mind_shatter_placeholder.png" 
    });
    library.items.push(ItemDefinition { 
        id: ItemId(14), 
        name: "Tome of the Glacial Heart".to_string(), 
        description: "Unlocks the 'Glacial Nova' chilling skill.".to_string(), 
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(5) }], 
        icon_path: "sprites/icons/item_tome_glacial_nova_placeholder.png" 
    });
    library.items.push(ItemDefinition { 
        id: ItemId(15), 
        name: "Tome of the Watcher".to_string(), 
        description: "Unlocks the 'Psychic Sentry' summoning skill.".to_string(), 
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(6) }], 
        icon_path: "sprites/icons/item_tome_psychic_sentry_placeholder.png" 
    });
     library.items.push(ItemDefinition { 
        id: ItemId(16), 
        name: "Tome of Ethereal Defense".to_string(), 
        description: "Unlocks the 'Ethereal Ward' defensive skill.".to_string(), 
        effects: vec![ItemEffect::GrantSpecificSkill { skill_id: SkillId(7) }], 
        icon_path: "sprites/icons/item_tome_ethereal_ward_placeholder.png" 
    });

}

fn apply_collected_item_effects_system(
    mut events: EventReader<ItemCollectedEvent>,
    mut player_query: Query<(&mut Survivor, Option<&mut ComponentHealth>, Option<&mut CircleOfWarding>, Option<&mut SwarmOfNightmares>)>,
    item_library: Res<ItemLibrary>,
    skill_library: Res<SkillLibrary>,
) {
    if let Ok((mut player, mut opt_health_component, mut opt_circle_aura, mut opt_nightmare_swarm)) = player_query.get_single_mut() {
        for event in events.read() {
            let item_id = event.0;
            let is_new_item = !player.collected_item_ids.contains(&item_id);
            
            if let Some(item_def) = item_library.get_item_definition(item_id) {
                let mut applied_successfully = true; 
                if is_new_item { // Most effects only apply once, when the item is first collected.
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
                                if let Some(_skill_to_grant_def) = skill_library.get_skill_definition(*skill_id) {
                                    let already_has_skill = player.equipped_skills.iter().any(|s| s.definition_id == *skill_id);
                                    if !already_has_skill { if player.equipped_skills.len() < 5 { 
                                        player.equipped_skills.push(ActiveSkillInstance::new(*skill_id ));
                                    } else { applied_successfully = false; }
                                    } else { applied_successfully = false; /* Already has skill, don't mark as new for collection if this is the only effect */ }
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
                                        circle_aura.current_radius *= 1.05; // Slight radius increase for stacking
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
                                        nightmare_swarm.num_larvae = (nightmare_swarm.num_larvae + 1).min(8); // Max 8 larvae
                                        nightmare_swarm.damage_per_hit += 1;
                                    }
                                } else { applied_successfully = false; }
                            }
                             // OnHit/OnKill effects are passive listeners, no direct application here, just need the item in collected_item_ids
                            ItemEffect::OnAutomaticProjectileHitExplode {..} | ItemEffect::OnSurvivorHitRetaliate {..} | ItemEffect::OnHorrorKillTrigger {..} => {
                                // These effects are checked elsewhere, just need the item ID to be in the list.
                                // If an item *only* had these, it would still be "applied_successfully".
                            }
                        }
                    }
                }
                // Add to collected_item_ids only if it's genuinely new and all primary effects applied
                // or if the item is meant to be stackable for some effects even if not "new" in terms of granting.
                // For simplicity now, if it's new and any core granting effect was attempted, we add it.
                // Passive stat boosts are always "new" in their effect if the item is new.
                if is_new_item && applied_successfully {
                     player.collected_item_ids.push(item_id);
                } else if !is_new_item {
                    // Handle effects for already owned items that might stack (like Circle of Warding / Swarm in current logic)
                    // This part might need refinement if some effects are one-time and others stackable.
                    // The current logic in ActivateCircle/Swarm handles basic stacking.
                    // PassiveStatBoosts are currently only applied if is_new_item.
                }


            }
        }
    }
}

fn explosion_effect_system( mut commands: Commands, time: Res<Time>, mut explosion_query: Query<(Entity, &mut ExplosionEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (explosion_entity, mut explosion, explosion_g_transform, mut sprite, mut vis_transform) in explosion_query.iter_mut() { explosion.timer.tick(time.delta()); let progress = explosion.timer.fraction(); let current_radius = explosion.radius_sq.sqrt(); vis_transform.scale = Vec3::splat(current_radius * 2.0 * progress); sprite.color.set_a(1.0 - progress); if explosion.timer.fraction() < 0.5 { let explosion_pos = explosion_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() { if explosion.already_hit_entities.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(explosion_pos) < explosion.radius_sq { horror_health.0 -= explosion.damage; spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), explosion.damage, &time); sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); explosion.already_hit_entities.push(horror_entity); } } } if explosion.timer.finished() { commands.entity(explosion_entity).despawn_recursive(); } } }
fn retaliation_nova_effect_system( mut commands: Commands, time: Res<Time>, mut nova_query: Query<(Entity, &mut RetaliationNovaEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (nova_entity, mut nova, nova_g_transform, mut sprite, mut vis_transform) in nova_query.iter_mut() { nova.timer.tick(time.delta()); let progress = nova.timer.fraction(); let current_radius = nova.radius_sq.sqrt(); vis_transform.scale = Vec3::splat(current_radius * 2.0 * progress); sprite.color.set_a(1.0 - progress * progress); if nova.timer.fraction() < 0.3 { let nova_pos = nova_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() { if nova.already_hit_entities.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(nova_pos) < nova.radius_sq { horror_health.0 -= nova.damage; spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), nova.damage, &time); sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); nova.already_hit_entities.push(horror_entity); } } } if nova.timer.finished() { commands.entity(nova_entity).despawn_recursive(); } } }
fn temporary_health_regen_buff_system( mut commands: Commands, time: Res<Time>, mut buff_query: Query<(Entity, &mut TemporaryHealthRegenBuff, &Survivor, &mut ComponentHealth)>,) { for (entity, mut buff, survivor_stats, mut health_component) in buff_query.iter_mut() { buff.duration_timer.tick(time.delta()); if buff.duration_timer.finished() { commands.entity(entity).remove::<TemporaryHealthRegenBuff>(); } else { let regen_amount = buff.regen_per_second * time.delta().as_secs_f32(); health_component.0 = (health_component.0 as f32 + regen_amount).round() as i32; health_component.0 = health_component.0.min(survivor_stats.max_health); } } }