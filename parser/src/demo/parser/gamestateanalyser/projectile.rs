use crate::demo::data::game_state::{GameState, Handle, PipeType, Projectile, ProjectileType};
use crate::demo::message::{PacketEntity, UpdateType};
use crate::demo::packet::datatable::ServerClassName;
use crate::demo::parser::analyser::Team;
use crate::demo::sendprop::SendPropIdentifier;
use crate::demo::vector::Vector;
use crate::ParserState;

pub fn handle_projectile_entity(
    state: &mut GameState,
    entity: &PacketEntity,
    parser_state: &ParserState,
    class_names: &[ServerClassName],
) {
    let Some(class_name) = class_names.get(usize::from(entity.server_class)) else {
        return;
    };

    const ROCKET_ORIGIN: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFBaseRocket", "m_vecOrigin"); // rockets, arrows, more?
    const GRENADE_ORIGIN: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFWeaponBaseGrenadeProj", "m_vecOrigin");
    // todo: flares?
    const TEAM: SendPropIdentifier = SendPropIdentifier::new("DT_BaseEntity", "m_iTeamNum");
    const INITIAL_SPEED: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFBaseRocket", "m_vInitialVelocity");
    const LAUNCHER: SendPropIdentifier =
        SendPropIdentifier::new("DT_BaseProjectile", "m_hOriginalLauncher");
    const PIPE_TYPE: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFProjectile_Pipebomb", "m_iType");
    const ROCKET_ROTATION: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFBaseRocket", "m_angRotation");
    const GRENADE_ROTATION: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFWeaponBaseGrenadeProj", "m_angRotation");
    const CRITICAL_GRENADE: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFWeaponBaseGrenadeProj", "m_bCritical");
    const CRITICAL_ROCKET: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFProjectile_Rocket", "m_bCritical");
    const CRITICAL_FLARE: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFProjectile_Flare", "m_bCritical");
    const CRITICAL_ARROW: SendPropIdentifier =
        SendPropIdentifier::new("DT_TFProjectile_Arrow", "m_bCritical");

    if entity.update_type == UpdateType::Delete {
        state.projectile_destroy(entity.entity_index);
        return;
    }

    let projectile = state
        .projectiles
        .entry(entity.entity_index)
        .or_insert_with(|| Projectile::new(entity.entity_index, entity.server_class, class_name));

    // todo: bounds for grenades

    for prop in entity.props(parser_state) {
        match prop.identifier {
            ROCKET_ORIGIN | GRENADE_ORIGIN => {
                let pos = Vector::try_from(&prop.value).unwrap_or_default();
                projectile.position = pos
            }
            TEAM => {
                let team = Team::new(i64::try_from(&prop.value).unwrap_or_default());
                projectile.team = team;
            }
            INITIAL_SPEED => {
                let speed = Vector::try_from(&prop.value).unwrap_or_default();
                projectile.initial_speed = speed;
            }
            LAUNCHER => {
                let launcher = Handle(i64::try_from(&prop.value).unwrap_or_default());
                projectile.launcher = launcher;
            }
            PIPE_TYPE => {
                let pipe_type = PipeType::new(i64::try_from(&prop.value).unwrap_or_default());
                if let Some(class_name) = class_names.get(usize::from(entity.server_class)) {
                    let ty = ProjectileType::new(class_name, Some(pipe_type));
                    projectile.ty = ty;
                }
            }
            ROCKET_ROTATION | GRENADE_ROTATION => {
                let rotation = Vector::try_from(&prop.value).unwrap_or_default();
                projectile.rotation = rotation;
            }
            CRITICAL_GRENADE | CRITICAL_ROCKET | CRITICAL_FLARE | CRITICAL_ARROW => {
                let critical = bool::try_from(&prop.value).unwrap_or_default();
                projectile.critical = critical;
            }
            _ => {}
        }
    }
}
