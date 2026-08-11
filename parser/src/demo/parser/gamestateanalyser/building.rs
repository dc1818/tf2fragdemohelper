use crate::demo::data::game_state::{
    Building, BuildingClass, Dispenser, GameState, Handle, Sentry, Teleporter,
};
use crate::demo::message::{EntityId, PacketEntity, UpdateType};
use crate::demo::parser::analyser::{Team, UserId};
use crate::demo::sendprop::{SendPropIdentifier, SendPropValue};
use crate::demo::vector::Vector;
use crate::ParserState;

pub fn handle_sentry_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
) {
    const ROTATION: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_angRotation");
    const ANGLE: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFNonLocalPlayerExclusive", "m_angEyeAngles[1]");
    const MINI: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_bMiniBuilding");
    const CONTROLLED: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectSentrygun", "m_bPlayerControlled");
    const TARGET: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectSentrygun", "m_hAutoAimTarget");
    const SHELLS: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectSentrygun", "m_iAmmoShells");
    const ROCKETS: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectSentrygun", "m_iAmmoRockets");
    const SHIELD: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectSentrygun", "m_nShieldLevel");
    #[allow(dead_code)]
    const POSE_PARAMETER_PITCH: SendPropIdentifier =
        SendPropIdentifier::new("m_flPoseParameter", "000");
    const POSE_PARAMETER_YAW: SendPropIdentifier =
        SendPropIdentifier::new("m_flPoseParameter", "001");

    if entity.update_type == UpdateType::Delete {
        state.remove_building(entity.entity_index);
        return;
    }

    handle_building(state, entity, parser_state, BuildingClass::Sentry);

    let building = state.get_or_create_building(entity.entity_index, BuildingClass::Sentry);

    if let Building::Sentry(sentry) = building {
        for prop in entity.props(parser_state) {
            match prop.identifier {
                ROTATION => sentry.angle = Vector::try_from(&prop.value).unwrap_or_default().y,
                ANGLE => sentry.angle = f32::try_from(&prop.value).unwrap_or_default(),
                MINI => sentry.is_mini = i64::try_from(&prop.value).unwrap_or_default() > 0,
                CONTROLLED => {
                    sentry.player_controlled = i64::try_from(&prop.value).unwrap_or_default() > 0
                }
                TARGET => {
                    sentry.auto_aim_target = Handle::try_from(&prop.value).unwrap_or_default()
                }
                SHELLS => sentry.shells = i64::try_from(&prop.value).unwrap_or_default() as u16,
                ROCKETS => sentry.rockets = i64::try_from(&prop.value).unwrap_or_default() as u16,
                SHIELD => sentry.shield = bool::try_from(&prop.value).unwrap_or_default(),
                POSE_PARAMETER_YAW => sentry.yaw = f32::try_from(&prop.value).unwrap_or_default(),
                _ => {}
            }
        }
    }
}

pub fn handle_teleporter_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
) {
    const RECHARGE_TIME: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectTeleporter", "m_flRechargeTime");
    const RECHARGE_DURATION: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectTeleporter", "m_flCurrentRechargeDuration");
    const TIMES_USED: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectTeleporter", "m_iTimesUsed");
    const OTHER_END: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectTeleporter", "m_bMatchBuilding");
    const YAW_TO_EXIT: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectTeleporter", "m_flYawToExit");
    const IS_ENTRANCE: SendPropIdentifier =
        SendPropIdentifier::new("DT_BaseObject", "m_iObjectMode");

    if entity.update_type == UpdateType::Delete {
        state.remove_building(entity.entity_index);
        return;
    }

    handle_building(state, entity, parser_state, BuildingClass::Teleporter);

    let building = state.get_or_create_building(entity.entity_index, BuildingClass::Teleporter);

    if let Building::Teleporter(teleporter) = building {
        for prop in entity.props(parser_state) {
            match prop.identifier {
                RECHARGE_TIME => {
                    teleporter.recharge_time = f32::try_from(&prop.value).unwrap_or_default()
                }
                RECHARGE_DURATION => {
                    teleporter.recharge_duration = f32::try_from(&prop.value).unwrap_or_default()
                }
                TIMES_USED => {
                    teleporter.times_used = i64::try_from(&prop.value).unwrap_or_default() as u16
                }
                OTHER_END => {
                    teleporter.other_end =
                        EntityId::from(i64::try_from(&prop.value).unwrap_or_default() as u32)
                }
                YAW_TO_EXIT => {
                    teleporter.yaw_to_exit = f32::try_from(&prop.value).unwrap_or_default()
                }
                IS_ENTRANCE => {
                    teleporter.is_entrance = i64::try_from(&prop.value).unwrap_or_default() == 0
                }
                _ => {}
            }
        }
    }
}

pub fn handle_dispenser_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
) {
    const AMMO: SendPropIdentifier = SendPropIdentifier::new("DT_ObjectDispenser", "m_iAmmoMetal");
    const HEALING: SendPropIdentifier =
        SendPropIdentifier::new("DT_ObjectDispenser", "healing_array");

    if entity.update_type == UpdateType::Delete {
        state.remove_building(entity.entity_index);
        return;
    }

    handle_building(state, entity, parser_state, BuildingClass::Dispenser);

    let building = state.get_or_create_building(entity.entity_index, BuildingClass::Dispenser);

    if let Building::Dispenser(dispenser) = building {
        for prop in entity.props(parser_state) {
            match prop.identifier {
                AMMO => dispenser.metal = i64::try_from(&prop.value).unwrap_or_default() as u16,
                HEALING => {
                    let values = match &prop.value {
                        SendPropValue::Array(vec) => vec.as_slice(),
                        _ => Default::default(),
                    };

                    dispenser.healing = values
                        .iter()
                        .map(|val| UserId::from(i64::try_from(val).unwrap_or_default() as u16))
                        .collect()
                }
                _ => {}
            }
        }
    }
}

fn handle_building(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
    class: BuildingClass,
) {
    let building = state.get_or_create_building(entity.entity_index, class);

    const LOCAL_ORIGIN: SendPropIdentifier =
        SendPropIdentifier::new("DT_BaseEntity", "m_vecOrigin");
    const TEAM: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");
    const ANGLE: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_angRotation");
    const SAPPED: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_bHasSapper");
    const BUILDING: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_bBuilding");
    const LEVEL: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_iUpgradeLevel");
    const BUILDER: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_hBuilder");
    const MAX_HEALTH: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_iMaxHealth");
    const HEALTH: SendPropIdentifier = SendPropIdentifier::new("DT_BaseObject", "m_iHealth");
    const PROGRESS: SendPropIdentifier =
        SendPropIdentifier::new("DT_BaseObject", "m_flPercentageConstructed");

    match building {
        Building::Sentry(Sentry {
            position,
            team,
            angle,
            sapped,
            builder,
            level,
            building,
            max_health,
            health,
            construction_progress,
            ..
        })
        | Building::Dispenser(Dispenser {
            position,
            team,
            angle,
            sapped,
            builder,
            level,
            building,
            max_health,
            health,
            construction_progress,
            ..
        })
        | Building::Teleporter(Teleporter {
            position,
            team,
            angle,
            sapped,
            builder,
            level,
            building,
            max_health,
            health,
            construction_progress,
            ..
        }) => {
            // picked up
            if entity.update_type == UpdateType::Leave {
                *health = 0;
            }
            for prop in entity.props(parser_state) {
                match prop.identifier {
                    LOCAL_ORIGIN => *position = Vector::try_from(&prop.value).unwrap_or_default(),
                    TEAM => *team = Team::new(i64::try_from(&prop.value).unwrap_or_default()),
                    ANGLE => {
                        *angle = Vector::try_from(&prop.value)
                            .map(|v| v.y)
                            .unwrap_or_default()
                    }
                    SAPPED => *sapped = i64::try_from(&prop.value).unwrap_or_default() > 0,
                    BUILDING => *building = i64::try_from(&prop.value).unwrap_or_default() > 0,
                    LEVEL => *level = i64::try_from(&prop.value).unwrap_or_default() as u8,
                    BUILDER => {
                        *builder =
                            UserId::from(i64::try_from(&prop.value).unwrap_or_default() as u16)
                    }
                    MAX_HEALTH => {
                        *max_health = i64::try_from(&prop.value).unwrap_or_default() as u16
                    }
                    HEALTH => *health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    PROGRESS => {
                        *construction_progress = f32::try_from(&prop.value).unwrap_or_default()
                    }
                    _ => {}
                }
            }
        }
    }
}
