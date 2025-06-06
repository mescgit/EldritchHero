use bevy::prelude::*;
use crate::items::{AutomaticWeaponDefinition, AutomaticWeaponId, AttackTypeData, ChainZapParams};

pub fn define_chain_lightning() -> AutomaticWeaponDefinition {
    AutomaticWeaponDefinition {
        id: AutomaticWeaponId(5),
        name: "Chain Lightning".to_string(),
        attack_data: AttackTypeData::ChainZap(ChainZapParams {
            base_fire_rate_secs: 1.2,
            initial_target_range: 300.0,
            max_chains: 3,
            chain_search_radius: 150.0,
            base_damage_per_zap: 15,
            damage_falloff_per_chain: 0.8,
            zap_color: Color::rgb(0.5, 0.8, 1.0),
            zap_width: 5.0,
            zap_duration_secs: 0.15,
        }),
    }
}
