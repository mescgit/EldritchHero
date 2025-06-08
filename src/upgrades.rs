// src/upgrades.rs
use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::{
    skills::SkillId,
    automatic_weapons, // Added this line
};

use crate::items::AutomaticWeaponId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default)]
pub enum LobbedAoEPoolField {
    BaseDamageOnImpact,
    PoolDamagePerTick,
    BaseFireRateSecs,
    ProjectileSpeed,
    ProjectileArcHeight,
    PoolRadius,
    PoolDurationSecs,
    PoolTickIntervalSecs,
    MaxActivePools,
}
impl Default for LobbedAoEPoolField { fn default() -> Self { LobbedAoEPoolField::PoolRadius } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default)]
pub enum ChanneledBeamField {
    BaseDamagePerTick,
    TickRateSecs,
    Range,
    BeamWidth,
    MovementPenaltyMultiplier,
    MaxDurationSecs,
    CooldownSecs,
}
impl Default for ChanneledBeamField { fn default() -> Self { ChanneledBeamField::Range } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default)]
pub enum ReturningProjectileField {
    BaseDamage,
    BaseFireRateSecs,
    ProjectileSpeed,
    TravelDistance,
    Piercing,
}
impl Default for ReturningProjectileField { fn default() -> Self { ReturningProjectileField::TravelDistance } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default)]
pub enum StandardProjectileField {
    BaseDamage,
    BaseFireRateSecs,
    BaseProjectileSpeed,
    BasePiercing,
    AdditionalProjectiles,
    ProjectileLifetimeSecs,
}
impl Default for StandardProjectileField { fn default() -> Self { StandardProjectileField::BaseDamage } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Default)]
pub enum ConeAttackField {
    BaseDamage,
    BaseFireRateSecs,
    ConeAngleDegrees,
    ConeRadius,
}
impl Default for ConeAttackField { fn default() -> Self { ConeAttackField::ConeRadius } }


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
#[reflect(Default)] // Added Default derive
pub enum UpgradeRarity {
    #[default]
    Regular,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, PartialEq, Reflect)] // Added Reflect here
pub enum UpgradeType {
    // Survivor Stats
    SurvivorSpeed(u32), 
    MaxEndurance(i32), 
    EnduranceRegeneration(f32),
    EchoesGainMultiplier(u32), 
    SoulAttractionRadius(u32), 

    // Automatic Weapon Upgrades (Now for Inherent Weapon) - Flat increases
    IncreaseAutoWeaponDamage(i32),          
    IncreaseAutoWeaponFireRate(u32), // This is likely a percentage for fire rate cooldown reduction
    IncreaseAutoWeaponProjectileSpeed(u32), // This is likely a percentage
    IncreaseAutoWeaponPiercing(u32),      
    IncreaseAutoWeaponProjectiles(u32),   

    // New Auto-Attack Upgrades (Percentage based or distinct mechanics)
    AutoAttackDamagePercent(f32),
    AutoAttackSpeedPercent(f32),
    AutoAttackFireRatePercent(f32), // Percentage increase to fire rate (means reduce cooldown)
    AutoAttackAddProjectiles(u32), // Flat number of additional projectiles (distinct from IncreaseAutoWeaponProjectiles for potentially different scaling/source)
    AutoAttackAddPiercing(u32), // Flat increase to piercing count (distinct from IncreaseAutoWeaponPiercing)

    // Circle of Warding
    InscribeCircleOfWarding,
    IncreaseCircleRadius(u32), 
    IncreaseCircleDamage(i32), 
    DecreaseCircleTickRate(u32), 
    
    // Swarm of Nightmares
    ManifestSwarmOfNightmares, 
    IncreaseNightmareCount(u32), 
    IncreaseNightmareDamage(i32), 
    IncreaseNightmareRadius(f32), 
    IncreaseNightmareRotationSpeed(f32),
    
    // Active Skill Upgrades
    IncreaseSkillDamage { slot_index: usize, amount: i32 }, 
    ReduceSkillCooldown { slot_index: usize, percent_reduction: f32 }, 
    IncreaseSkillAoERadius { slot_index: usize, percent_increase: f32 },
    
    // Utility/Granting
    GrantRandomRelic, 
    GrantSkill(SkillId),

    // --- Auto-Attack Focused (New Batch) ---
    AutoAttackAddFireDamage(u32),
    AutoAttackAddColdDamage(u32),
    AutoAttackAddLightningDamage(u32),
    AutoAttackAddPoisonDamage(u32), // Base DPS
    AutoAttackCritChance(f32), // Percent
    AutoAttackCritDamage(f32), // Percent bonus
    AutoAttackExecuteLowHealth(f32), // Percent health threshold
    AutoAttackLifeSteal(f32), // Percent of damage
    AutoAttackChainChance(f32), // Percent chance
    AutoAttackForkChance(f32), // Percent chance
    AutoAttackChillChance(f32), // Percent chance
    AutoAttackStunChance(f32), // Percent chance
    AutoAttackBurnChance(f32), // Percent chance (distinct from flat fire)
    AutoAttackReduceHealingChance(f32), // Percent chance
    AutoAttackAreaDamageOnHitChance(f32), // Percent chance, value is flat damage for the AoE
    AutoAttackIncreaseDuration(f32), // Percent increase for projectile lifetime
    AutoAttackHomingStrength(f32), // Flat increase to homing factor/strength
    AutoAttackRicochetChance(f32), // Percent chance
    AutoAttackShieldPenetration(f32), // Percent shield penetration
    AutoAttackCullStrikeChance(f32), // Percent chance for massive damage on low health non-elites

    // --- Survivor Defensive Stats (New Batch) ---
    IncreaseArmor(u32),
    IncreaseEvasionChance(f32), // Percent
    IncreaseBlockChance(f32), // Percent
    IncreaseDamageReduction(f32), // Flat Percent reduction
    IncreaseTenacity(f32), // Percent reduction to CC duration
    IncreaseStatusEffectResistance(f32), // Percent chance to resist
    IncreaseHealingEffectiveness(f32), // Percent bonus
    OnHitGainTemporaryArmor(u32), // Flat armor for X seconds (duration fixed or separate upgrade)
    OnHitGainTemporarySpeed(f32), // Percent speed for X seconds (duration fixed or separate upgrade)
    AfterBeingHitSpawnRetaliationNova(i32), // Flat damage

    // --- Survivor Utility/Mobility (New Batch) ---
    IncreaseDashCharges(u32),
    ReduceDashCooldown(f32), // Percent
    IncreaseDashRange(f32), // Percent
    DashGrantsInvulnerability(f32), // Duration in seconds
    IncreaseMovementOutOfCombat(f32), // Percent
    ReduceSlowEffectiveness(f32), // Percent reduction on slows
    GainShieldOnKill(u32), // Flat shield amount
    IncreaseEchoesDropRate(f32), // Percent more echoes orbs
    IncreaseRelicDropRate(f32), // Percent higher chance for relics
    ChanceForFreeSkillUse(f32), // Percent chance

    // --- Weapon-Specific (Aura/Orbiter - Circle of Warding / Swarm of Nightmares) (New Batch) ---
    AuraIncreaseSizePerKill(f32), // Percent size increase stack
    OrbiterIncreaseSpeedPerKill(f32), // Percent speed increase stack
    AuraPullEnemiesChance(f32), // Percent chance per tick
    OrbiterExplodeOnKillChance(f32), // Percent chance, value is flat damage for explosion
    AuraDebuffEnemies(f32), // Percent increased damage taken for enemies in aura

    // Weapon-Specific Parameter Modifications
    ModifyStandardProjectile { weapon_id: AutomaticWeaponId, field: StandardProjectileField, change_value: f32, is_percentage: bool },
    ModifyReturningProjectile { weapon_id: AutomaticWeaponId, field: ReturningProjectileField, change_value: f32, is_percentage: bool },
    ModifyChanneledBeam { weapon_id: AutomaticWeaponId, field: ChanneledBeamField, change_value: f32, is_percentage: bool },
    ModifyConeAttack { weapon_id: AutomaticWeaponId, field: ConeAttackField, change_value: f32, is_percentage: bool },
    ModifyLobbedAoEPool { weapon_id: AutomaticWeaponId, field: LobbedAoEPoolField, change_value: f32, is_percentage: bool },
    ModifyOrbitingPet { weapon_id: AutomaticWeaponId, field: OrbitingPetField, change_value: f32, is_percentage: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
#[reflect(Default)]
pub enum OrbitingPetField {
    #[default]
    MaxActiveOrbs,
    OrbDurationSecs,
    OrbitRadius,
    OrbitSpeedRadPerSec,
    // Deployment related
    CanBeDeployedAtLocation, // Might be a boolean toggle rather than numerical.
    DeploymentRange,
    // Pulse Attack related
    PulseDamage,
    PulseRadius,
    PulseIntervalSecs,
    // Bolt Attack related
    BoltDamage,
    BoltSpeed,
    BoltFireIntervalSecs,
    BoltLifetimeSecs,
    BoltHomingStrength,
}

#[derive(Debug, Clone, Reflect, PartialEq)] // Added PartialEq
pub struct UpgradeCard {
    pub id: UpgradeId,
    pub name: String,
    pub description: String,
    pub upgrade_type: UpgradeType,
    pub rarity: UpgradeRarity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)] // Added Reflect and Default
pub struct UpgradeId(pub u32);

#[derive(Resource, Default, Reflect)] // Added Reflect
#[reflect(Resource)] // Added reflect attribute
pub struct UpgradePool {
    pub available_upgrades: Vec<UpgradeCard>,
}

impl UpgradePool {
    pub fn initialize(&mut self) {
        self.available_upgrades = vec![
            // Survivor Stats
            UpgradeCard {id: UpgradeId(0), name: "Borrowed Swiftness".to_string(), description: "Your limbs move with uncanny swiftness borrowed from beyond. +10% speed.".to_string(), upgrade_type: UpgradeType::SurvivorSpeed(10), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(1), name: "Flesh-Bound Pact".to_string(), description: "A pact seals your flesh against oblivion. +20 Max Endurance.".to_string(), upgrade_type: UpgradeType::MaxEndurance(20), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(5), name: "Otherworldly Agility".to_string(), description: "You glide like a creature not of this realm. +15% speed.".to_string(), upgrade_type: UpgradeType::SurvivorSpeed(15), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(6), name: "Resilient Corpus".to_string(), description: "Your form knits itself against harsher realities. +30 Max Endurance.".to_string(), upgrade_type: UpgradeType::MaxEndurance(30), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(300), name: "Unnatural Vigor".to_string(), description: "Reality warps to mend your wounds. Regenerate 0.5 Endurance/sec.".to_string(), upgrade_type: UpgradeType::EnduranceRegeneration(0.5), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(301), name: "Bound by Ichor".to_string(), description: "Strange energies sustain your form. Regenerate 1.0 Endurance/sec.".to_string(), upgrade_type: UpgradeType::EnduranceRegeneration(1.0), rarity: UpgradeRarity::Regular,},

            // Automatic Weapon (Main Attack)
            UpgradeCard {id: UpgradeId(2), name: "Maddening Focus".to_string(), description: "Your automatic attacks strike with greater force. +5 Damage.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponDamage(5), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(3), name: "Rapid Sanity Strain".to_string(), description: "Your mind strains faster, casting automatic attacks more quickly. +15% fire rate.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponFireRate(15), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(4), name: "Swift Projectiles".to_string(), description: "Your automatic projectiles travel faster. +20% velocity.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponProjectileSpeed(20), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(7), name: "Piercing Thoughts".to_string(), description: "Your automatic attacks carry deeper malevolence. +8 Damage.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponDamage(8), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(8), name: "Hyper Reflex".to_string(), description: "Your mind strains with startling alacrity, casting automatic attacks faster. +20% fire rate.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponFireRate(20), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(9), name: "Unraveling Force".to_string(), description: "Your automatic projectiles tear through more horrors. Pierce +1 horror.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponPiercing(1), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(12), name: "Persistent Dread".to_string(), description: "Your automatic projectiles linger longer in reality. Pierce +2 horrors.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponPiercing(2), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(200), name: "Fractured Consciousness".to_string(), description: "Your mind splinters, projecting an additional automatic projectile. +1 Projectile.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponProjectiles(1), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(201), name: "Projectile Barrage".to_string(), description: "Your consciousness erupts, projecting two additional automatic projectiles. +2 Projectiles.".to_string(), upgrade_type: UpgradeType::IncreaseAutoWeaponProjectiles(2), rarity: UpgradeRarity::Regular,},

            // Echoes (XP) & Pickups
            UpgradeCard {id: UpgradeId(10), name: "Glimpse Beyond The Veil".to_string(), description: "Glimpses of the abyss accelerate your horrific understanding. +20% Echoes gain.".to_string(), upgrade_type: UpgradeType::EchoesGainMultiplier(20), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(11), name: "Soul Grasp".to_string(), description: "The echoes of fallen horrors are drawn to you. +25% Echoing Soul attraction radius.".to_string(), upgrade_type: UpgradeType::SoulAttractionRadius(25), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(13), name: "Abyssal Understanding".to_string(), description: "You perceive deeper truths, hastening your evolution. +30% Echoes gain.".to_string(), upgrade_type: UpgradeType::EchoesGainMultiplier(30), rarity: UpgradeRarity::Regular,},
            
            // Circle of Warding (Aura Weapon)
            UpgradeCard {id: UpgradeId(100), name: "Inscribe Circle of Warding".to_string(), description: "Manifest an aura of protective, damaging glyphs.".to_string(), upgrade_type: UpgradeType::InscribeCircleOfWarding, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(101), name: "Echoing Wards".to_string(), description: "Your protective circle extends further. +20% circle radius.".to_string(), upgrade_type: UpgradeType::IncreaseCircleRadius(20), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(102), name: "Maddening Wards".to_string(), description: "Your circle inflicts greater mental anguish. +2 circle damage.".to_string(), upgrade_type: UpgradeType::IncreaseCircleDamage(2), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(103), name: "Frenzied Wards".to_string(), description: "Your circle pulses with greater frequency. Circle damages 15% faster.".to_string(), upgrade_type: UpgradeType::DecreaseCircleTickRate(15), rarity: UpgradeRarity::Regular,},

            // Swarm of Nightmares (Orbiter Weapon)
            UpgradeCard {id: UpgradeId(400), name: "Manifest Swarm of Nightmares".to_string(), description: "Conjure 2 nightmare larva that orbit and attack foes.".to_string(), upgrade_type: UpgradeType::ManifestSwarmOfNightmares, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(401), name: "Grow the Nightmare Swarm".to_string(), description: "Add another Nightmare Larva to your psychic defenses. +1 nightmare.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareCount(1), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(402), name: "Venomous Nightmares".to_string(), description: "Your Nightmare Larva inflict deeper wounds. +3 nightmare damage.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareDamage(3), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(403), name: "Extended Nightmare Patrol".to_string(), description: "Your Nightmare Larva patrol a wider area. +15 orbit radius.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareRadius(15.0), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(404), name: "Swifter Nightmares".to_string(), description: "Your Nightmare Larva move with increased speed. +0.5 rad/s orbit speed.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareRotationSpeed(0.5), rarity: UpgradeRarity::Regular,},
            
            // Skill Specific Upgrades
            UpgradeCard {id: UpgradeId(500), name: "Empower Eldritch Bolt".to_string(), description: "Increase Eldritch Bolt damage by 10.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 0, amount: 10 }, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(501), name: "Intensify Mind Shatter".to_string(), description: "Mind Shatter fragments each deal +3 damage.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 1, amount: 3 }, rarity: UpgradeRarity::Regular,}, 
            UpgradeCard {id: UpgradeId(502), name: "Sharpen Void Lance".to_string(), description: "Increase Void Lance damage by 20.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 2, amount: 20 }, rarity: UpgradeRarity::Regular,},
            
            // General/Utility
            UpgradeCard {id: UpgradeId(600), name: "Mysterious Relic".to_string(), description: "The abyss grants you a random relic.".to_string(), upgrade_type: UpgradeType::GrantRandomRelic, rarity: UpgradeRarity::Regular,},

            // Grant Skills
            UpgradeCard {id: UpgradeId(700), name: "Learn: Mind Shatter".to_string(), description: "Unlock the Mind Shatter psychic burst skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(2)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(701), name: "Learn: Void Lance".to_string(), description: "Unlock the Void Lance piercing projectile skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(3)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(702), name: "Learn: Fleeting Agility".to_string(), description: "Unlock the Fleeting Agility self-buff skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(4)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(703), name: "Learn: Glacial Nova".to_string(), description: "Unlock the Glacial Nova chilling skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(5)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(704), name: "Learn: Psychic Sentry".to_string(), description: "Unlock the Psychic Sentry summon skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(6)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(705), name: "Learn: Ethereal Ward".to_string(), description: "Unlock the Ethereal Ward defensive skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(7)), rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(706), name: "Learn: Gaze of the Abyss".to_string(), description: "Unlock the Gaze of the Abyss channeled beam skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(8)), rarity: UpgradeRarity::Regular,},


            // Skill Meta Upgrades
            UpgradeCard {id: UpgradeId(800), name: "Echoing Bolt".to_string(), description: "Eldritch Bolt recharges 15% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 0, percent_reduction: 0.15 }, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(801), name: "Focused Mind Shatter".to_string(), description: "Mind Shatter recharges 15% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 1, percent_reduction: 0.15 }, rarity: UpgradeRarity::Regular,}, 
            UpgradeCard {id: UpgradeId(802), name: "Accelerated Void".to_string(), description: "Void Lance recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 2, percent_reduction: 0.10 }, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(803), name: "Heightened Reflexes".to_string(), description: "Fleeting Agility recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 3, percent_reduction: 0.10 }, rarity: UpgradeRarity::Regular,},
            UpgradeCard {id: UpgradeId(804), name: "Cryo-Resonance".to_string(), description: "Glacial Nova recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 4, percent_reduction: 0.10 }, rarity: UpgradeRarity::Regular,}, 
            UpgradeCard {id: UpgradeId(805), name: "Expanded Chill".to_string(), description: "Glacial Nova's area of effect expands by 15%.".to_string(), upgrade_type: UpgradeType::IncreaseSkillAoERadius { slot_index: 4, percent_increase: 0.15 }, rarity: UpgradeRarity::Regular,},

            // --- New Auto-Attack Upgrades with Rarity ---

            // AutoAttackDamagePercent (Base: 10.0)
            UpgradeCard {
                id: UpgradeId(900),
                name: "Faint Malevolence".to_string(),
                description: "Auto-attacks whisper of greater pain. +10 base damage bonus.".to_string(),
                upgrade_type: UpgradeType::AutoAttackDamagePercent(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(901),
                name: "Resonant Malice".to_string(),
                description: "Auto-attacks echo with amplified force. +20 base damage bonus.".to_string(),
                upgrade_type: UpgradeType::AutoAttackDamagePercent(10.0), // Base value remains 10.0
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(902),
                name: "Screaming Hatred".to_string(),
                description: "Auto-attacks become conduits of pure agony. +30 base damage bonus.".to_string(),
                upgrade_type: UpgradeType::AutoAttackDamagePercent(10.0), // Base value remains 10.0
                rarity: UpgradeRarity::Legendary,
            },

            // AutoAttackSpeedPercent (Base: 10.0) -> Projectile Speed
            UpgradeCard {
                id: UpgradeId(903),
                name: "Erratic Impulses".to_string(),
                description: "Auto-attack projectiles gain a slight surge in speed. +10% projectile speed.".to_string(),
                upgrade_type: UpgradeType::AutoAttackSpeedPercent(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(904),
                name: "Accelerated Trajectory".to_string(),
                description: "Auto-attack projectiles are noticeably swifter. +20% projectile speed.".to_string(),
                upgrade_type: UpgradeType::AutoAttackSpeedPercent(10.0), // Base value
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(905),
                name: "Warp-Speed Projectiles".to_string(),
                description: "Auto-attack projectiles tear through reality. +30% projectile speed.".to_string(),
                upgrade_type: UpgradeType::AutoAttackSpeedPercent(10.0), // Base value
                rarity: UpgradeRarity::Legendary,
            },

            // AutoAttackFireRatePercent (Base: 10.0)
            UpgradeCard {
                id: UpgradeId(906),
                name: "Hastened Thoughts".to_string(),
                description: "The rhythm of your auto-attacks quickens slightly. +10% fire rate.".to_string(),
                upgrade_type: UpgradeType::AutoAttackFireRatePercent(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(907),
                name: "Frenzied Volley".to_string(),
                description: "Your auto-attacks unleash in a rapid succession. +20% fire rate.".to_string(),
                upgrade_type: UpgradeType::AutoAttackFireRatePercent(10.0), // Base value
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(908),
                name: "Ceaseless Barrage".to_string(),
                description: "Auto-attacks become a relentless storm. +30% fire rate.".to_string(),
                upgrade_type: UpgradeType::AutoAttackFireRatePercent(10.0), // Base value
                rarity: UpgradeRarity::Legendary,
            },

            // AutoAttackAddProjectiles (Base: 1)
            UpgradeCard {
                id: UpgradeId(909),
                name: "Echoing Shot".to_string(),
                description: "A faint echo follows your auto-attack. +1 projectile.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddProjectiles(1),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(910),
                name: "Splintering Manifestation".to_string(),
                description: "Your auto-attack splits into multiple vectors. +2 projectiles.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddProjectiles(1), // Base value
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(911),
                name: "Myriad Assault".to_string(),
                description: "Your auto-attack shatters into a cascade of force. +3 projectiles.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddProjectiles(1), // Base value
                rarity: UpgradeRarity::Legendary,
            },

            // AutoAttackAddPiercing (Base: 1)
            UpgradeCard {
                id: UpgradeId(912),
                name: "Penetrating Whisper".to_string(),
                description: "Your auto-attacks pierce an additional foe.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPiercing(1),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(913),
                name: "Unseen Impalement".to_string(),
                description: "Your auto-attacks pass through multiple horrors. +2 piercing.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPiercing(1), // Base value
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(914),
                name: "Reality Rending Force".to_string(),
                description: "Your auto-attacks ignore the forms of many foes. +3 piercing.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPiercing(1), // Base value
                rarity: UpgradeRarity::Legendary,
            },

            // --- Batch 1: New Auto-Attack Focused Upgrades ---

            // 1. AutoAttackAddFireDamage (Base: 5)
            UpgradeCard {
                id: UpgradeId(915),
                name: "Singeing Strike".to_string(),
                description: "Auto-attacks inflict an additional 5 fire damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddFireDamage(5),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(916),
                name: "Burning Lash".to_string(),
                description: "Auto-attacks inflict an additional 10 fire damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddFireDamage(5),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(917),
                name: "Inferno Brand".to_string(),
                description: "Auto-attacks inflict an additional 15 fire damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddFireDamage(5),
                rarity: UpgradeRarity::Legendary,
            },

            // 2. AutoAttackAddColdDamage (Base: 5)
            UpgradeCard {
                id: UpgradeId(918),
                name: "Chilling Touch".to_string(),
                description: "Auto-attacks inflict an additional 5 cold damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddColdDamage(5),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(919),
                name: "Freezing Blow".to_string(),
                description: "Auto-attacks inflict an additional 10 cold damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddColdDamage(5),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(920),
                name: "Glacial Spike".to_string(), // Note: name collision with a weapon, might need rename
                description: "Auto-attacks inflict an additional 15 cold damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddColdDamage(5),
                rarity: UpgradeRarity::Legendary,
            },

            // 3. AutoAttackAddLightningDamage (Base: 5)
            UpgradeCard {
                id: UpgradeId(921),
                name: "Sparking Hit".to_string(),
                description: "Auto-attacks inflict an additional 5 lightning damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddLightningDamage(5),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(922),
                name: "Shocking Jolt".to_string(),
                description: "Auto-attacks inflict an additional 10 lightning damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddLightningDamage(5),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(923),
                name: "Thunderclap Strike".to_string(),
                description: "Auto-attacks inflict an additional 15 lightning damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddLightningDamage(5),
                rarity: UpgradeRarity::Legendary,
            },

            // 4. AutoAttackAddPoisonDamage (Base: 3 DPS)
            UpgradeCard {
                id: UpgradeId(924),
                name: "Venomous Barb".to_string(),
                description: "Auto-attacks apply poison, dealing 3 damage per second.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPoisonDamage(3),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(925),
                name: "Toxic Sting".to_string(),
                description: "Auto-attacks apply potent poison, dealing 6 damage per second.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPoisonDamage(3),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(926),
                name: "Withering Contagion".to_string(),
                description: "Auto-attacks apply virulent poison, dealing 9 damage per second.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAddPoisonDamage(3),
                rarity: UpgradeRarity::Legendary,
            },

            // 5. AutoAttackCritChance (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(927),
                name: "Precise Incision".to_string(),
                description: "Increases auto-attack critical strike chance by 5%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritChance(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(928),
                name: "Keen Edge".to_string(),
                description: "Increases auto-attack critical strike chance by 10%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritChance(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(929),
                name: "Fatal Flaw".to_string(),
                description: "Increases auto-attack critical strike chance by 15%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritChance(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 6. AutoAttackCritDamage (Base: 20.0%)
            UpgradeCard {
                id: UpgradeId(930),
                name: "Forceful Blow".to_string(),
                description: "Increases auto-attack critical strike damage by 20%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritDamage(20.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(931),
                name: "Devastating Impact".to_string(),
                description: "Increases auto-attack critical strike damage by 40%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritDamage(20.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(932),
                name: "Obliterating Force".to_string(),
                description: "Increases auto-attack critical strike damage by 60%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCritDamage(20.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 7. AutoAttackExecuteLowHealth (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(933),
                name: "Finisher's Touch".to_string(),
                description: "Auto-attacks have a chance to instantly kill enemies below 10% health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackExecuteLowHealth(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(934),
                name: "Executioner's Precision".to_string(),
                description: "Auto-attacks have a higher chance to instantly kill enemies below 20% health.".to_string(), // Description implies higher chance and threshold
                upgrade_type: UpgradeType::AutoAttackExecuteLowHealth(10.0), // Base threshold
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(935),
                name: "Reaper's Scythe".to_string(),
                description: "Auto-attacks reliably instantly kill enemies below 30% health.".to_string(), // Description implies even higher chance and threshold
                upgrade_type: UpgradeType::AutoAttackExecuteLowHealth(10.0), // Base threshold
                rarity: UpgradeRarity::Legendary,
            },

            // 8. AutoAttackLifeSteal (Base: 2.0%)
            UpgradeCard {
                id: UpgradeId(936),
                name: "Siphoning Strike".to_string(),
                description: "Auto-attacks steal 2% of damage dealt as health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackLifeSteal(2.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(937),
                name: "Vampiric Touch".to_string(),
                description: "Auto-attacks steal 4% of damage dealt as health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackLifeSteal(2.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(938),
                name: "Essence Drain".to_string(),
                description: "Auto-attacks steal 6% of damage dealt as health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackLifeSteal(2.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 9. AutoAttackChainChance (Base: 15.0%)
            UpgradeCard {
                id: UpgradeId(939),
                name: "Ricocheting Shard".to_string(),
                description: "Auto-attacks have a 15% chance to chain to one additional enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChainChance(15.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(940),
                name: "Chaining Bolts".to_string(),
                description: "Auto-attacks have a 30% chance to chain to one additional enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChainChance(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(941),
                name: "Arc Lightning".to_string(),
                description: "Auto-attacks have a 45% chance to chain to one additional enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChainChance(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 10. AutoAttackForkChance (Base: 15.0%)
            UpgradeCard {
                id: UpgradeId(942),
                name: "Splitting Missile".to_string(),
                description: "Auto-attacks have a 15% chance to fork into two projectiles.".to_string(),
                upgrade_type: UpgradeType::AutoAttackForkChance(15.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(943),
                name: "Diverging Force".to_string(),
                description: "Auto-attacks have a 30% chance to fork into two projectiles.".to_string(),
                upgrade_type: UpgradeType::AutoAttackForkChance(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(944),
                name: "Fracturing Barrage".to_string(),
                description: "Auto-attacks have a 45% chance to fork into two projectiles.".to_string(),
                upgrade_type: UpgradeType::AutoAttackForkChance(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 11. AutoAttackChillChance (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(945),
                name: "Numbing Grasp".to_string(),
                description: "Auto-attacks have a 10% chance to chill enemies, slowing them.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChillChance(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(946),
                name: "Frostbite".to_string(),
                description: "Auto-attacks have a 20% chance to chill enemies, slowing them.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChillChance(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(947),
                name: "Absolute Zero".to_string(),
                description: "Auto-attacks have a 30% chance to chill enemies, slowing them.".to_string(),
                upgrade_type: UpgradeType::AutoAttackChillChance(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 12. AutoAttackStunChance (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(948),
                name: "Concussive Force".to_string(),
                description: "Auto-attacks have a 5% chance to briefly stun enemies.".to_string(),
                upgrade_type: UpgradeType::AutoAttackStunChance(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(949),
                name: "Impact Trauma".to_string(),
                description: "Auto-attacks have a 10% chance to briefly stun enemies.".to_string(),
                upgrade_type: UpgradeType::AutoAttackStunChance(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(950),
                name: "Paralyzing Blow".to_string(),
                description: "Auto-attacks have a 15% chance to briefly stun enemies.".to_string(),
                upgrade_type: UpgradeType::AutoAttackStunChance(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 13. AutoAttackBurnChance (Base: 10.0%) - Distinct from flat fire, implies DoT
            UpgradeCard {
                id: UpgradeId(951),
                name: "Igniting Spark".to_string(),
                description: "Auto-attacks have a 10% chance to ignite enemies, dealing fire damage over time.".to_string(),
                upgrade_type: UpgradeType::AutoAttackBurnChance(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(952),
                name: "Wildfire Spread".to_string(),
                description: "Auto-attacks have a 20% chance to ignite enemies, dealing fire damage over time.".to_string(),
                upgrade_type: UpgradeType::AutoAttackBurnChance(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(953),
                name: "Immolation".to_string(),
                description: "Auto-attacks have a 30% chance to ignite enemies, dealing fire damage over time.".to_string(),
                upgrade_type: UpgradeType::AutoAttackBurnChance(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 14. AutoAttackReduceHealingChance (Base: 25.0%)
            UpgradeCard {
                id: UpgradeId(954),
                name: "Mortal Wound".to_string(),
                description: "Auto-attacks have a 25% chance to reduce enemy healing effectiveness.".to_string(),
                upgrade_type: UpgradeType::AutoAttackReduceHealingChance(25.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(955),
                name: "Grievous Affliction".to_string(),
                description: "Auto-attacks have a 50% chance to reduce enemy healing effectiveness.".to_string(),
                upgrade_type: UpgradeType::AutoAttackReduceHealingChance(25.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(956),
                name: "Curse of Frailty".to_string(),
                description: "Auto-attacks have a 75% chance to reduce enemy healing effectiveness.".to_string(),
                upgrade_type: UpgradeType::AutoAttackReduceHealingChance(25.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 15. AutoAttackAreaDamageOnHitChance (Base: 10.0% chance, 8 flat damage for AoE)
            UpgradeCard {
                id: UpgradeId(957),
                name: "Shockwave Impact".to_string(),
                description: "Auto-attacks have a 10% chance to create a small shockwave, dealing 8 area damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAreaDamageOnHitChance(8.0), // Storing AoE damage as base_val
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(958),
                name: "Explosive Burst".to_string(),
                description: "Auto-attacks have a 20% chance to create a small shockwave, dealing 16 area damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAreaDamageOnHitChance(8.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(959),
                name: "Cataclysmic Eruption".to_string(),
                description: "Auto-attacks have a 30% chance to create a small shockwave, dealing 24 area damage.".to_string(),
                upgrade_type: UpgradeType::AutoAttackAreaDamageOnHitChance(8.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 16. AutoAttackIncreaseDuration (Base: 15.0%)
            UpgradeCard {
                id: UpgradeId(960),
                name: "Lingering Presence".to_string(),
                description: "Increases auto-attack projectile lifetime by 15%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackIncreaseDuration(15.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(961),
                name: "Extended Reach".to_string(),
                description: "Increases auto-attack projectile lifetime by 30%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackIncreaseDuration(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(962),
                name: "Eternal Pursuit".to_string(),
                description: "Increases auto-attack projectile lifetime by 45%.".to_string(),
                upgrade_type: UpgradeType::AutoAttackIncreaseDuration(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 17. AutoAttackHomingStrength (Base: 0.5)
            UpgradeCard {
                id: UpgradeId(963),
                name: "Seeking Whispers".to_string(),
                description: "Auto-attack projectiles gain a slight homing effect (strength 0.5).".to_string(),
                upgrade_type: UpgradeType::AutoAttackHomingStrength(0.5),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(964),
                name: "Guided Path".to_string(),
                description: "Auto-attack projectiles gain a noticeable homing effect (strength 1.0).".to_string(),
                upgrade_type: UpgradeType::AutoAttackHomingStrength(0.5),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(965),
                name: "Inevitable Impact".to_string(),
                description: "Auto-attack projectiles strongly home in on targets (strength 1.5).".to_string(),
                upgrade_type: UpgradeType::AutoAttackHomingStrength(0.5),
                rarity: UpgradeRarity::Legendary,
            },

            // 18. AutoAttackRicochetChance (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(966),
                name: "Bouncing Shard".to_string(),
                description: "Auto-attacks have a 10% chance to ricochet to a nearby enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackRicochetChance(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(967),
                name: "Rebounding Force".to_string(),
                description: "Auto-attacks have a 20% chance to ricochet to a nearby enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackRicochetChance(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(968),
                name: "Unstoppable Deflection".to_string(),
                description: "Auto-attacks have a 30% chance to ricochet to a nearby enemy.".to_string(),
                upgrade_type: UpgradeType::AutoAttackRicochetChance(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 19. AutoAttackShieldPenetration (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(969),
                name: "Armor Piercer".to_string(),
                description: "Auto-attacks ignore 10% of enemy shields/armor.".to_string(),
                upgrade_type: UpgradeType::AutoAttackShieldPenetration(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(970),
                name: "Shield Breaker".to_string(),
                description: "Auto-attacks ignore 20% of enemy shields/armor.".to_string(),
                upgrade_type: UpgradeType::AutoAttackShieldPenetration(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(971),
                name: "Defense Shatterer".to_string(),
                description: "Auto-attacks ignore 30% of enemy shields/armor.".to_string(),
                upgrade_type: UpgradeType::AutoAttackShieldPenetration(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 20. AutoAttackCullStrikeChance (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(972),
                name: "Mercy Stroke".to_string(),
                description: "Auto-attacks have a 5% chance to deal massive damage to non-elite enemies below 15% health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCullStrikeChance(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(973),
                name: "Decimator".to_string(),
                description: "Auto-attacks have a 10% chance to deal massive damage to non-elite enemies below 15% health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCullStrikeChance(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(974),
                name: "Annihilator".to_string(),
                description: "Auto-attacks have a 15% chance to deal massive damage to non-elite enemies below 15% health.".to_string(),
                upgrade_type: UpgradeType::AutoAttackCullStrikeChance(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // --- Batch 2: Survivor Defensive Stats ---

            // 21. IncreaseArmor (Base: 10)
            UpgradeCard {
                id: UpgradeId(975),
                name: "Reinforced Hide".to_string(),
                description: "Your form becomes tougher, granting +10 armor.".to_string(),
                upgrade_type: UpgradeType::IncreaseArmor(10),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(976),
                name: "Ironclad Form".to_string(),
                description: "Your resilience is greatly enhanced, granting +20 armor.".to_string(),
                upgrade_type: UpgradeType::IncreaseArmor(10),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(977),
                name: "Adamantine Body".to_string(),
                description: "You become nearly impervious to harm, granting +30 armor.".to_string(),
                upgrade_type: UpgradeType::IncreaseArmor(10),
                rarity: UpgradeRarity::Legendary,
            },

            // 22. IncreaseEvasionChance (Base: 3.0%)
            UpgradeCard {
                id: UpgradeId(978),
                name: "Slight Shift".to_string(),
                description: "You become slightly harder to hit. +3% evasion chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseEvasionChance(3.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(979),
                name: "Blurred Form".to_string(),
                description: "Attacks sometimes pass through you. +6% evasion chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseEvasionChance(3.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(980),
                name: "Phase Walker".to_string(),
                description: "You are exceptionally elusive. +9% evasion chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseEvasionChance(3.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 23. IncreaseBlockChance (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(981),
                name: "Guarded Stance".to_string(),
                description: "You are more adept at blocking incoming attacks. +5% block chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseBlockChance(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(982),
                name: "Reactive Barrier".to_string(),
                description: "You instinctively deflect attacks. +10% block chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseBlockChance(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(983),
                name: "Impenetrable Defense".to_string(),
                description: "Few attacks can find their mark. +15% block chance.".to_string(),
                upgrade_type: UpgradeType::IncreaseBlockChance(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 24. IncreaseDamageReduction (Base: 2.0%)
            UpgradeCard {
                id: UpgradeId(984),
                name: "Thick Skin".to_string(),
                description: "Reduces all incoming damage by 2%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDamageReduction(2.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(985),
                name: "Resilient Soul".to_string(),
                description: "Reduces all incoming damage by 4%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDamageReduction(2.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(986),
                name: "Aegis of Survival".to_string(),
                description: "Reduces all incoming damage by 6%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDamageReduction(2.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 25. IncreaseTenacity (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(987),
                name: "Steadfast Will".to_string(),
                description: "Reduces the duration of crowd control effects on you by 10%.".to_string(),
                upgrade_type: UpgradeType::IncreaseTenacity(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(988),
                name: "Unbreakable Mind".to_string(),
                description: "Reduces the duration of crowd control effects on you by 20%.".to_string(),
                upgrade_type: UpgradeType::IncreaseTenacity(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(989),
                name: "Indomitable Spirit".to_string(),
                description: "Reduces the duration of crowd control effects on you by 30%.".to_string(),
                upgrade_type: UpgradeType::IncreaseTenacity(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 26. IncreaseStatusEffectResistance (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(990),
                name: "Purified Blood".to_string(),
                description: "Increases resistance to harmful status effects by 10%.".to_string(),
                upgrade_type: UpgradeType::IncreaseStatusEffectResistance(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(991),
                name: "Warding Sigils".to_string(),
                description: "Increases resistance to harmful status effects by 20%.".to_string(),
                upgrade_type: UpgradeType::IncreaseStatusEffectResistance(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(992),
                name: "Aura of Purity".to_string(),
                description: "Increases resistance to harmful status effects by 30%.".to_string(),
                upgrade_type: UpgradeType::IncreaseStatusEffectResistance(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 27. IncreaseHealingEffectiveness (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(993),
                name: "Vitality Boost".to_string(),
                description: "Increases effectiveness of all healing received by 10%.".to_string(),
                upgrade_type: UpgradeType::IncreaseHealingEffectiveness(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(994),
                name: "Rejuvenating Flow".to_string(),
                description: "Increases effectiveness of all healing received by 20%.".to_string(),
                upgrade_type: UpgradeType::IncreaseHealingEffectiveness(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(995),
                name: "Blessed Constitution".to_string(),
                description: "Increases effectiveness of all healing received by 30%.".to_string(),
                upgrade_type: UpgradeType::IncreaseHealingEffectiveness(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 28. OnHitGainTemporaryArmor (Base: 15)
            UpgradeCard {
                id: UpgradeId(996),
                name: "Reactive Plating".to_string(),
                description: "Gain +15 armor for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporaryArmor(15),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(997),
                name: "Adaptive Carapace".to_string(),
                description: "Gain +30 armor for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporaryArmor(15),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(998),
                name: "Instant Fortress".to_string(),
                description: "Gain +45 armor for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporaryArmor(15),
                rarity: UpgradeRarity::Legendary,
            },

            // 29. OnHitGainTemporarySpeed (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(999),
                name: "Adrenaline Rush".to_string(),
                description: "Gain +10% movement speed for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporarySpeed(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1000),
                name: "Escape Reflex".to_string(),
                description: "Gain +20% movement speed for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporarySpeed(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1001),
                name: "Phantom Step".to_string(),
                description: "Gain +30% movement speed for a short duration when hit.".to_string(),
                upgrade_type: UpgradeType::OnHitGainTemporarySpeed(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 30. AfterBeingHitSpawnRetaliationNova (Base: 20 damage)
            UpgradeCard {
                id: UpgradeId(1002),
                name: "Painful Retort".to_string(),
                description: "After being hit, release a nova dealing 20 damage to nearby enemies.".to_string(),
                upgrade_type: UpgradeType::AfterBeingHitSpawnRetaliationNova(20),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1003),
                name: "Vengeful Burst".to_string(),
                description: "After being hit, release a powerful nova dealing 40 damage to nearby enemies.".to_string(),
                upgrade_type: UpgradeType::AfterBeingHitSpawnRetaliationNova(20),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1004),
                name: "Wrathful Detonation".to_string(),
                description: "After being hit, release a devastating nova dealing 60 damage to nearby enemies.".to_string(),
                upgrade_type: UpgradeType::AfterBeingHitSpawnRetaliationNova(20),
                rarity: UpgradeRarity::Legendary,
            },
            
            // --- Batch 3: Survivor Utility/Mobility ---

            // 31. IncreaseDashCharges (Base: 1)
            UpgradeCard {
                id: UpgradeId(1005),
                name: "Evasive Maneuver".to_string(),
                description: "Gain an additional dash charge.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashCharges(1),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1006),
                name: "Repeated Shift".to_string(),
                description: "Gain 2 additional dash charges.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashCharges(1),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1007),
                name: "Blink Master".to_string(),
                description: "Gain 3 additional dash charges.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashCharges(1),
                rarity: UpgradeRarity::Legendary,
            },

            // 32. ReduceDashCooldown (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(1008),
                name: "Quick Step".to_string(),
                description: "Reduces dash cooldown by 10%.".to_string(),
                upgrade_type: UpgradeType::ReduceDashCooldown(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1009),
                name: "Rapid Evasion".to_string(),
                description: "Reduces dash cooldown by 20%.".to_string(),
                upgrade_type: UpgradeType::ReduceDashCooldown(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1010),
                name: "Continuous Motion".to_string(),
                description: "Reduces dash cooldown by 30%.".to_string(),
                upgrade_type: UpgradeType::ReduceDashCooldown(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 33. IncreaseDashRange (Base: 15.0%)
            UpgradeCard {
                id: UpgradeId(1011),
                name: "Extended Lunge".to_string(),
                description: "Increases dash range by 15%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashRange(15.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1012),
                name: "Far Reach".to_string(),
                description: "Increases dash range by 30%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashRange(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1013),
                name: "Dimension Strider".to_string(),
                description: "Increases dash range by 45%.".to_string(),
                upgrade_type: UpgradeType::IncreaseDashRange(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 34. DashGrantsInvulnerability (Base: 0.1s)
            UpgradeCard {
                id: UpgradeId(1014),
                name: "Fleeting Invincibility".to_string(),
                description: "Dashing grants 0.1 seconds of invulnerability.".to_string(),
                upgrade_type: UpgradeType::DashGrantsInvulnerability(0.1),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1015),
                name: "Phasing Dash".to_string(),
                description: "Dashing grants 0.2 seconds of invulnerability.".to_string(),
                upgrade_type: UpgradeType::DashGrantsInvulnerability(0.1),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1016),
                name: "Ghostly Evasion".to_string(),
                description: "Dashing grants 0.3 seconds of invulnerability.".to_string(),
                upgrade_type: UpgradeType::DashGrantsInvulnerability(0.1),
                rarity: UpgradeRarity::Legendary,
            },

            // 35. IncreaseMovementOutOfCombat (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(1017),
                name: "Swift Explorer".to_string(),
                description: "Increases movement speed by 10% when out of combat.".to_string(),
                upgrade_type: UpgradeType::IncreaseMovementOutOfCombat(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1018),
                name: "Pathfinder's Pace".to_string(),
                description: "Increases movement speed by 20% when out of combat.".to_string(),
                upgrade_type: UpgradeType::IncreaseMovementOutOfCombat(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1019),
                name: "Wind Runner".to_string(),
                description: "Increases movement speed by 30% when out of combat.".to_string(),
                upgrade_type: UpgradeType::IncreaseMovementOutOfCombat(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 36. ReduceSlowEffectiveness (Base: 15.0%)
            UpgradeCard {
                id: UpgradeId(1020),
                name: "Surefooted".to_string(),
                description: "Reduces effectiveness of slows on you by 15%.".to_string(),
                upgrade_type: UpgradeType::ReduceSlowEffectiveness(15.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1021),
                name: "Unwavering Stride".to_string(),
                description: "Reduces effectiveness of slows on you by 30%.".to_string(),
                upgrade_type: UpgradeType::ReduceSlowEffectiveness(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1022),
                name: "Freedom of Movement".to_string(),
                description: "Reduces effectiveness of slows on you by 45%.".to_string(),
                upgrade_type: UpgradeType::ReduceSlowEffectiveness(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 37. GainShieldOnKill (Base: 5)
            UpgradeCard {
                id: UpgradeId(1023),
                name: "Ephemeral Ward".to_string(),
                description: "Gain a 5 point shield for a short duration on killing an enemy.".to_string(),
                upgrade_type: UpgradeType::GainShieldOnKill(5),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1024),
                name: "Soul Barrier".to_string(),
                description: "Gain a 10 point shield for a short duration on killing an enemy.".to_string(),
                upgrade_type: UpgradeType::GainShieldOnKill(5),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1025),
                name: "Spirit Fortress".to_string(),
                description: "Gain a 15 point shield for a short duration on killing an enemy.".to_string(),
                upgrade_type: UpgradeType::GainShieldOnKill(5),
                rarity: UpgradeRarity::Legendary,
            },

            // 38. IncreaseEchoesDropRate (Base: 10.0%)
            UpgradeCard {
                id: UpgradeId(1026),
                name: "Echo Collector".to_string(),
                description: "Increases the drop rate of Echoes by 10%.".to_string(),
                upgrade_type: UpgradeType::IncreaseEchoesDropRate(10.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1027),
                name: "Abyssal Magnet".to_string(),
                description: "Increases the drop rate of Echoes by 20%.".to_string(),
                upgrade_type: UpgradeType::IncreaseEchoesDropRate(10.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1028),
                name: "Soul Harvester".to_string(),
                description: "Increases the drop rate of Echoes by 30%.".to_string(),
                upgrade_type: UpgradeType::IncreaseEchoesDropRate(10.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 39. IncreaseRelicDropRate (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(1029),
                name: "Fortune Seeker".to_string(),
                description: "Slightly increases the chance of finding Relics by 5%.".to_string(),
                upgrade_type: UpgradeType::IncreaseRelicDropRate(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1030),
                name: "Relic Hunter".to_string(),
                description: "Moderately increases the chance of finding Relics by 10%.".to_string(),
                upgrade_type: UpgradeType::IncreaseRelicDropRate(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1031),
                name: "Treasure Master".to_string(),
                description: "Significantly increases the chance of finding Relics by 15%.".to_string(),
                upgrade_type: UpgradeType::IncreaseRelicDropRate(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 40. ChanceForFreeSkillUse (Base: 3.0%)
            UpgradeCard {
                id: UpgradeId(1032),
                name: "Moment of Clarity".to_string(),
                description: "Skills have a 3% chance to not consume resources or cooldown.".to_string(),
                upgrade_type: UpgradeType::ChanceForFreeSkillUse(3.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1033),
                name: "Inspired Casting".to_string(),
                description: "Skills have a 6% chance to not consume resources or cooldown.".to_string(),
                upgrade_type: UpgradeType::ChanceForFreeSkillUse(3.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1034),
                name: "Transcendent Power".to_string(),
                description: "Skills have a 9% chance to not consume resources or cooldown.".to_string(),
                upgrade_type: UpgradeType::ChanceForFreeSkillUse(3.0),
                rarity: UpgradeRarity::Legendary,
            },

            // --- Batch 4: Weapon-Specific (Aura/Orbiter) ---

            // 41. AuraIncreaseSizePerKill (Base: 1.0%)
            UpgradeCard {
                id: UpgradeId(1035),
                name: "Consuming Aura".to_string(),
                description: "Aura grows by 1% per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::AuraIncreaseSizePerKill(1.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1036),
                name: "Devouring Expanse".to_string(),
                description: "Aura grows by 2% per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::AuraIncreaseSizePerKill(1.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1037),
                name: "All-Encompassing Void".to_string(),
                description: "Aura grows by 3% per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::AuraIncreaseSizePerKill(1.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 42. OrbiterIncreaseSpeedPerKill (Base: 1.0%)
            UpgradeCard {
                id: UpgradeId(1038),
                name: "Frenzied Orbiters".to_string(),
                description: "Orbiters gain 1% speed per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::OrbiterIncreaseSpeedPerKill(1.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1039),
                name: "Accelerated Swarm".to_string(),
                description: "Orbiters gain 2% speed per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::OrbiterIncreaseSpeedPerKill(1.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1040),
                name: "Blinding Whirlwind".to_string(),
                description: "Orbiters gain 3% speed per kill for a short duration (stacks).".to_string(),
                upgrade_type: UpgradeType::OrbiterIncreaseSpeedPerKill(1.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 43. AuraPullEnemiesChance (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(1041),
                name: "Weakening Grasp".to_string(),
                description: "Aura has a 5% chance each tick to pull weak enemies closer.".to_string(),
                upgrade_type: UpgradeType::AuraPullEnemiesChance(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1042),
                name: "Binding Field".to_string(),
                description: "Aura has a 10% chance each tick to pull weak enemies closer.".to_string(),
                upgrade_type: UpgradeType::AuraPullEnemiesChance(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1043),
                name: "Singularity Effect".to_string(),
                description: "Aura has a 15% chance each tick to pull weak enemies closer.".to_string(),
                upgrade_type: UpgradeType::AuraPullEnemiesChance(5.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 44. OrbiterExplodeOnKillChance (Base: 10.0% chance, 15 damage)
            UpgradeCard {
                id: UpgradeId(1044),
                name: "Unstable Orbiters".to_string(),
                description: "Orbiters have a 10% chance to explode on killing an enemy, dealing 15 damage.".to_string(),
                upgrade_type: UpgradeType::OrbiterExplodeOnKillChance(15.0), // Storing explosion damage
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1045),
                name: "Volatile Swarm".to_string(),
                description: "Orbiters have a 20% chance to explode on killing an enemy, dealing 30 damage.".to_string(),
                upgrade_type: UpgradeType::OrbiterExplodeOnKillChance(15.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1046),
                name: "Chain Reaction Nightmares".to_string(),
                description: "Orbiters have a 30% chance to explode on killing an enemy, dealing 45 damage.".to_string(),
                upgrade_type: UpgradeType::OrbiterExplodeOnKillChance(15.0),
                rarity: UpgradeRarity::Legendary,
            },

            // 45. AuraDebuffEnemies (Base: 5.0%)
            UpgradeCard {
                id: UpgradeId(1047),
                name: "Weakening Presence".to_string(),
                description: "Enemies within your aura take 5% increased damage.".to_string(),
                upgrade_type: UpgradeType::AuraDebuffEnemies(5.0),
                rarity: UpgradeRarity::Regular,
            },
            UpgradeCard {
                id: UpgradeId(1048),
                name: "Curse of Vulnerability".to_string(),
                description: "Enemies within your aura take 10% increased damage.".to_string(),
                upgrade_type: UpgradeType::AuraDebuffEnemies(5.0),
                rarity: UpgradeRarity::Rare,
            },
            UpgradeCard {
                id: UpgradeId(1049),
                name: "Mark of the Abyss".to_string(),
                description: "Enemies within your aura take 15% increased damage.".to_string(),
                upgrade_type: UpgradeType::AuraDebuffEnemies(5.0),
                rarity: UpgradeRarity::Legendary,
            },

        ];
        let specific_weapon_upgrades = automatic_weapons::get_all_specific_weapon_upgrades();
        self.available_upgrades.extend(specific_weapon_upgrades);
    }
    pub fn get_random_upgrades(&self, count: usize) -> Vec<UpgradeCard> { let mut rng = rand::thread_rng(); self.available_upgrades.choose_multiple(&mut rng, count).cloned().collect() }
}

#[derive(Component, Debug, Clone, Reflect, Default)] // Added Default
#[reflect(Component)] // Added reflect attribute
pub struct OfferedUpgrades { pub choices: Vec<UpgradeCard>, }

pub struct UpgradePlugin;
impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        let mut upgrade_pool = UpgradePool::default();
        upgrade_pool.initialize();
        app.insert_resource(upgrade_pool)
            .register_type::<UpgradeRarity>()
            .register_type::<UpgradeType>()
            .register_type::<UpgradeCard>()
            .register_type::<UpgradeId>()
            .register_type::<UpgradePool>()
            .register_type::<OfferedUpgrades>()
            .register_type::<Vec<UpgradeCard>>() // Also register Vec<UpgradeCard> if used in reflected components like OfferedUpgrades
            .register_type::<LobbedAoEPoolField>()
            .register_type::<ChanneledBeamField>()
            .register_type::<ReturningProjectileField>()
            .register_type::<StandardProjectileField>()
            .register_type::<ConeAttackField>()
            .register_type::<OrbitingPetField>();
    }
}