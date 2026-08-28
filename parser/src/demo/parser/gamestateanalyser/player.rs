use crate::demo::data::game_state::{GameState, Handle, PlayerClassData, PlayerState};
use crate::demo::message::{EntityId, PacketEntity};
use crate::demo::parser::analyser::{Class, Team};
use crate::demo::sendprop::SendPropIdentifier;
use crate::demo::vector::{Vector, VectorXY};
use crate::ParserState;
use std::str::FromStr;

pub fn handle_player_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
) {
    let player = state.get_or_create_player(entity.entity_index);

    const OUTER: SendPropIdentifier = SendPropIdentifier::new("DT_AttributeContainer", "m_hOuter");
    const OUTER2: SendPropIdentifier = SendPropIdentifier::new("DT_AttributeManager", "m_hOuter");

    const HEALTH_PROP: SendPropIdentifier = SendPropIdentifier::new("DT_BasePlayer", "m_iHealth");
    const MAX_HEALTH_PROP: SendPropIdentifier =
        SendPropIdentifier::new("DT_BasePlayer", "m_iMaxHealth");
    const LIFE_STATE_PROP: SendPropIdentifier =
        SendPropIdentifier::new("DT_BasePlayer", "m_lifeState");
    const FLAGS_PROP: SendPropIdentifier = SendPropIdentifier::new("DT_BasePlayer", "m_fFlags");
    const VELOCITY_X: SendPropIdentifier =
        SendPropIdentifier::new("DT_LocalPlayerExclusive", "m_vecVelocity[0]");
    const VELOCITY_Y: SendPropIdentifier =
        SendPropIdentifier::new("DT_LocalPlayerExclusive", "m_vecVelocity[1]");
    const VELOCITY_Z: SendPropIdentifier =
        SendPropIdentifier::new("DT_LocalPlayerExclusive", "m_vecVelocity[2]");

    const LOCAL_ORIGIN: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_vecOrigin");
    const NON_LOCAL_ORIGIN: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin");
    const LOCAL_ORIGIN_Z: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_vecOrigin[2]");
    const NON_LOCAL_ORIGIN_Z: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_vecOrigin[2]");
    const LOCAL_EYE_ANGLES: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[1]");
    const NON_LOCAL_EYE_ANGLES: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
    const LOCAL_PITCH_ANGLES: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFLocalPlayerExclusive", "m_angEyeAngles[0]");
    const NON_LOCAL_PITCH_ANGLES: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[0]");

    const SIMTIME_PROP: SendPropIdentifier =
        SendPropIdentifier::new("DT_BaseEntity", "m_flSimulationTime");
    const PROP_BB_MAX: SendPropIdentifier =
        SendPropIdentifier::new("DT_CollisionProperty", "m_vecMaxsPreScaled");
    const PLAYER_COND: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCond");
    const PLAYER_COND_EX1: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCondEx");
    const PLAYER_COND_EX2: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCondEx2");
    const PLAYER_COND_EX3: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCondEx3");
    const PLAYER_COND_EX4: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nPlayerCondEx4");
    const PLAYER_COND_BITS: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerConditionListExclusive", "_condition_bits");

    const WEAPON_0: SendPropIdentifier = SendPropIdentifier::new("m_hMyWeapons", "000");
    const WEAPON_1: SendPropIdentifier = SendPropIdentifier::new("m_hMyWeapons", "001");
    const WEAPON_2: SendPropIdentifier = SendPropIdentifier::new("m_hMyWeapons", "002");

    const DISGUISE_TEAM: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nDisguiseTeam");
    const DISGUISE_CLASS: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_nDisguiseClass");
    const CLOAK_LEVEL: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFPlayerShared", "m_flCloakMeter");

    player.in_pvs = entity.in_pvs;

    for prop in entity.props(parser_state) {
        match prop.identifier {
            OUTER | OUTER2 => {
                player.handle = Handle::try_from(&prop.value).unwrap_or_default();
            }
            HEALTH_PROP => player.health = i64::try_from(&prop.value).unwrap_or_default() as u16,
            MAX_HEALTH_PROP => {
                player.max_health = i64::try_from(&prop.value).unwrap_or_default() as u16
            }
            LIFE_STATE_PROP => {
                player.state = PlayerState::new(i64::try_from(&prop.value).unwrap_or_default())
            }
            FLAGS_PROP => {
                player.flags = i64::try_from(&prop.value).unwrap_or_default() as u32;
                player.flags_known = true;
            }
            VELOCITY_X => {
                player.velocity.x = f32::try_from(&prop.value).unwrap_or_default();
                player.velocity_known = true;
            }
            VELOCITY_Y => {
                player.velocity.y = f32::try_from(&prop.value).unwrap_or_default();
                player.velocity_known = true;
            }
            VELOCITY_Z => {
                player.velocity.z = f32::try_from(&prop.value).unwrap_or_default();
                player.velocity_known = true;
            }
            LOCAL_ORIGIN | NON_LOCAL_ORIGIN => {
                let pos_xy = VectorXY::try_from(&prop.value).unwrap_or_default();
                player.position.x = pos_xy.x;
                player.position.y = pos_xy.y;
            }
            LOCAL_ORIGIN_Z | NON_LOCAL_ORIGIN_Z => {
                player.position.z = f32::try_from(&prop.value).unwrap_or_default()
            }
            LOCAL_EYE_ANGLES | NON_LOCAL_EYE_ANGLES => {
                player.view_angle = f32::try_from(&prop.value).unwrap_or_default()
            }
            LOCAL_PITCH_ANGLES | NON_LOCAL_PITCH_ANGLES => {
                player.pitch_angle = f32::try_from(&prop.value).unwrap_or_default()
            }
            SIMTIME_PROP => {
                player.simulation_time = i64::try_from(&prop.value).unwrap_or_default() as u16
            }
            PROP_BB_MAX => {
                let max = Vector::try_from(&prop.value).unwrap_or_default();
                player.bounds.max = max;
            }
            WEAPON_0 => {
                let handle = Handle::try_from(&prop.value).unwrap_or_default();
                player.weapons[0] = handle;
            }
            WEAPON_1 => {
                let handle = Handle::try_from(&prop.value).unwrap_or_default();
                player.weapons[1] = handle;
            }
            WEAPON_2 => {
                let handle = Handle::try_from(&prop.value).unwrap_or_default();
                player.weapons[2] = handle;
            }
            PLAYER_COND | PLAYER_COND_BITS => {
                player.conditions[0..4].copy_from_slice(
                    &i64::try_from(&prop.value).unwrap_or_default().to_le_bytes()[0..4],
                );
            }
            PLAYER_COND_EX1 => {
                player.conditions[4..8].copy_from_slice(
                    &i64::try_from(&prop.value).unwrap_or_default().to_le_bytes()[0..4],
                );
            }
            PLAYER_COND_EX2 => {
                player.conditions[8..12].copy_from_slice(
                    &i64::try_from(&prop.value).unwrap_or_default().to_le_bytes()[0..4],
                );
            }
            PLAYER_COND_EX3 => {
                player.conditions[12..16].copy_from_slice(
                    &i64::try_from(&prop.value).unwrap_or_default().to_le_bytes()[0..4],
                );
            }
            PLAYER_COND_EX4 => {
                player.conditions[16..20].copy_from_slice(
                    &i64::try_from(&prop.value).unwrap_or_default().to_le_bytes()[0..4],
                );
            }
            DISGUISE_TEAM => {
                if let PlayerClassData::Spy { disguise_team, .. } = &mut player.class_data {
                    *disguise_team = Team::new(i64::try_from(&prop.value).unwrap_or_default())
                }
            }
            DISGUISE_CLASS => {
                if let PlayerClassData::Spy { disguise_class, .. } = &mut player.class_data {
                    *disguise_class = Class::new(i64::try_from(&prop.value).unwrap_or_default());
                }
            }
            CLOAK_LEVEL => {
                if let PlayerClassData::Spy { cloak, .. } = &mut player.class_data {
                    *cloak = f32::try_from(&prop.value).unwrap_or_default();
                }
            }
            _ => {}
        }
    }
}

pub fn handle_player_resource(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
) {
    for prop in entity.props(parser_state) {
        if let Some((table_name, prop_name)) = prop.identifier.names() {
            if let Ok(player_id) = u32::from_str(prop_name.as_str()) {
                let entity_id = EntityId::from(player_id);
                if let Some(player) = state
                    .players
                    .iter_mut()
                    .find(|player| player.entity == entity_id)
                {
                    match table_name.as_str() {
                        "m_iTeam" => {
                            player.team = Team::new(i64::try_from(&prop.value).unwrap_or_default())
                        }
                        "m_iMaxHealth" => {
                            player.max_health =
                                i64::try_from(&prop.value).unwrap_or_default() as u16
                        }
                        "m_iPlayerClass" => {
                            let class = Class::new(i64::try_from(&prop.value).unwrap_or_default());
                            if player.class != class {
                                player.class = class;
                                player.class_data = PlayerClassData::default_for_class(class);
                            }
                        }
                        "m_iChargeLevel" => {
                            if let PlayerClassData::Medic { charge, .. } = &mut player.class_data {
                                *charge = i64::try_from(&prop.value).unwrap_or_default() as u8
                            }
                        }
                        "m_iPing" => {
                            player.ping = i64::try_from(&prop.value).unwrap_or_default() as u16
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
