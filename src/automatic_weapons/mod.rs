use crate::items::AutomaticWeaponDefinition;
use crate::upgrades::UpgradeCard;

pub mod primordial_ichor_blast;
pub mod eldritch_gatling;
pub mod void_cannon;
pub mod spectral_blades;
pub mod inferno_bolt;
pub mod chain_lightning;
pub mod arcane_ray;
pub mod shadow_orb;
pub mod holy_lance;
pub mod venom_spit;
pub mod glacial_spike;
pub mod earthshatter_shard;
pub mod sunfire_burst;
pub mod moonbeam_dart;
pub mod spirit_bomb;
pub mod void_tendril;
pub mod crystal_shard;
pub mod magma_ball;
pub mod sand_blast;
pub mod metal_shrapnel;
pub mod natures_wrath;
pub mod chi_bolt;
pub mod psionic_lash;
pub mod aether_bolt;

pub fn get_all_weapon_definitions() -> Vec<AutomaticWeaponDefinition> {
    let mut definitions = Vec::new();
    definitions.push(primordial_ichor_blast::define_primordial_ichor_blast());
    definitions.push(eldritch_gatling::define_eldritch_gatling());
    definitions.push(void_cannon::define_void_cannon());
    definitions.push(spectral_blades::define_spectral_blades());
    definitions.push(inferno_bolt::define_inferno_bolt());
    definitions.push(chain_lightning::define_chain_lightning());
    definitions.push(arcane_ray::define_arcane_ray());
    definitions.push(shadow_orb::define_shadow_orb());
    definitions.push(holy_lance::define_holy_lance());
    definitions.push(venom_spit::define_venom_spit());
    definitions.push(glacial_spike::define_glacial_spike());
    definitions.push(earthshatter_shard::define_earthshatter_shard());
    definitions.push(sunfire_burst::define_sunfire_burst());
    definitions.push(moonbeam_dart::define_moonbeam_dart());
    definitions.push(spirit_bomb::define_spirit_bomb());
    definitions.push(void_tendril::define_void_tendril());
    definitions.push(crystal_shard::define_crystal_shard());
    definitions.push(magma_ball::define_magma_ball());
    definitions.push(sand_blast::define_sand_blast());
    definitions.push(metal_shrapnel::define_metal_shrapnel());
    definitions.push(natures_wrath::define_natures_wrath());
    definitions.push(chi_bolt::define_chi_bolt());
    definitions.push(psionic_lash::define_psionic_lash());
    definitions.push(aether_bolt::define_aether_bolt());
    definitions
}

pub fn get_all_specific_weapon_upgrades() -> Vec<UpgradeCard> {
    let mut specific_upgrades = Vec::new();

    // Call get_specific_upgrades for weapons that have it
    specific_upgrades.extend(primordial_ichor_blast::get_specific_upgrades());
    specific_upgrades.extend(eldritch_gatling::get_specific_upgrades());
    specific_upgrades.extend(spectral_blades::get_specific_upgrades());
    specific_upgrades.extend(venom_spit::get_specific_upgrades());
    specific_upgrades.extend(sunfire_burst::get_specific_upgrades());
    // As more weapons get this function, add their calls here

    specific_upgrades
}
