use crate::demo::data::attributes::{has_attribute, Attribute};
use crate::demo::data::game_state::{GameState, Handle, MedigunType, PlayerClassData};
use crate::demo::message::{EntityId, PacketEntity, UpdateType};
use crate::demo::sendprop::SendPropIdentifier;
use std::collections::HashMap;

pub fn handle_medigun_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    outer_map_rev: &HashMap<EntityId, Handle>,
) {
    const OUTER: SendPropIdentifier = SendPropIdentifier::new("DT_AttributeContainer", "m_hOuter");
    const TARGET: SendPropIdentifier =
        SendPropIdentifier::new("DT_WeaponMedigun", "m_hHealingTarget");

    if entity.update_type == UpdateType::Enter {
        let mut ty = MedigunType::Uber;
        if has_attribute(&entity.props, Attribute::MedigunChargeIsCritBoost) {
            ty = MedigunType::Kritzkrieg;
        }
        if has_attribute(&entity.props, Attribute::MedigunChargeIsMegaHeal) {
            ty = MedigunType::Quickfix;
        }
        if has_attribute(&entity.props, Attribute::MedigunChargeIsResists) {
            ty = MedigunType::Vaccinator;
        }

        if let Some(handle) = entity.get_own_prop_value_by_identifier::<Handle>(OUTER) {
            if let Some(player) = state.get_player_by_weapon_handle(handle) {
                if let PlayerClassData::Medic { medigun, .. } = &mut player.class_data {
                    *medigun = ty;
                }
            }
        }
    }

    if let Some(target_handle) = entity.get_own_prop_value_by_identifier::<Handle>(TARGET) {
        let target_id = state
            .get_player_by_handle(target_handle)
            .map(|target| target.entity);
        let medic = outer_map_rev
            .get(&entity.entity_index)
            .copied()
            .and_then(|self_handle| state.get_player_by_weapon_handle(self_handle));

        if let Some(medic) = medic {
            if let PlayerClassData::Medic {
                target,
                last_target,
                ..
            } = &mut medic.class_data
            {
                *target = target_id;
                if target_id.is_some() {
                    *last_target = target_id;
                }
            }
        }
    }
}
