mod building;
mod player;
mod projectile;
mod weapon;

pub use crate::demo::data::game_state::{
    Building, BuildingClass, Dispenser, GameState, Kill, PlayerState, Sentry, Teleporter, World,
};
use crate::demo::data::game_state::{Cart, Handle, Objective};
use crate::demo::data::DemoTick;
use crate::demo::gameevent_gen::ObjectDestroyedEvent;
use crate::demo::gamevent::GameEvent;
use crate::demo::message::gameevent::GameEventMessage;
use crate::demo::message::packetentities::{EntityId, PacketEntity};
use crate::demo::message::{Message, UpdateType};
use crate::demo::packet::datatable::{ParseSendTable, ServerClass, ServerClassName};
use crate::demo::packet::message::MessagePacketMeta;
use crate::demo::packet::stringtable::StringTableEntry;
pub use crate::demo::parser::analyser::{Class, Team, UserId};
use crate::demo::parser::gamestateanalyser::building::{
    handle_dispenser_entity, handle_sentry_entity, handle_teleporter_entity,
};
use crate::demo::parser::gamestateanalyser::player::{
    handle_player_entity, handle_player_resource,
};
use crate::demo::parser::gamestateanalyser::projectile::handle_projectile_entity;
use crate::demo::parser::gamestateanalyser::weapon::handle_medigun_entity;
use crate::demo::parser::handler::BorrowMessageHandler;
use crate::demo::parser::MessageHandler;
use crate::demo::sendprop::{SendProp, SendPropIdentifier, SendPropValue};
use crate::demo::vector::Vector;
use crate::{MessageType, ParserState, ReadResult, Stream};
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct CachedEntities {}

#[derive(Default, Debug)]
pub struct GameStateAnalyser {
    pub state: GameState,
    tick: DemoTick,
    class_names: Vec<ServerClassName>, // indexed by ClassId
    outer_map: HashMap<Handle, EntityId>,
    outer_map_rev: HashMap<EntityId, Handle>,
}

impl MessageHandler for GameStateAnalyser {
    type Output = GameState;

    fn does_handle(message_type: MessageType) -> bool {
        matches!(
            message_type,
            MessageType::PacketEntities | MessageType::GameEvent | MessageType::ServerInfo
        )
    }

    fn handle_message(&mut self, message: &Message, _tick: DemoTick, parser_state: &ParserState) {
        match message {
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity, parser_state);
                }
                for id in &message.removed_entities {
                    self.state.projectile_destroy(*id);
                    self.state.remove_building(*id);
                }
            }
            Message::ServerInfo(message) => {
                self.state.interval_per_tick = message.interval_per_tick
            }
            Message::GameEvent(GameEventMessage { event, .. }) => {
                self.state.events.push((self.tick, event.clone()));
                match event {
                    GameEvent::PlayerDeath(death) => {
                        self.state.kills.push(Kill::new(self.tick, death.as_ref()))
                    }
                    GameEvent::RoundStart(_) => {
                        self.state.buildings.clear();
                        self.state.projectiles.clear();
                    }
                    GameEvent::TeamPlayRoundStart(_) => {
                        self.state.buildings.clear();
                        self.state.projectiles.clear();
                    }
                    GameEvent::ObjectDestroyed(ObjectDestroyedEvent { index, .. }) => {
                        self.state.remove_building((*index as u32).into());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_string_entry(
        &mut self,
        table: &str,
        index: usize,
        entry: &StringTableEntry,
        _parser_state: &ParserState,
    ) {
        if table == "userinfo" {
            let _ = self.parse_user_info(
                index,
                entry.text.as_ref().map(|s| s.as_ref()),
                entry.extra_data.as_ref().map(|data| data.data.clone()),
            );
        }
    }

    fn handle_data_tables(
        &mut self,
        _parse_tables: &[ParseSendTable],
        server_classes: &[ServerClass],
        _parser_state: &ParserState,
    ) {
        self.class_names = server_classes
            .iter()
            .map(|class| &class.name)
            .cloned()
            .collect();
    }

    fn handle_packet_meta(
        &mut self,
        tick: DemoTick,
        _meta: &MessagePacketMeta,
        _parser_state: &ParserState,
    ) {
        self.state.tick = tick;
        self.tick = tick;
    }

    fn into_output(mut self, state: &ParserState) -> Self::Output {
        self.state.server_classes = state.server_classes.clone();
        self.state
    }
}

impl BorrowMessageHandler for GameStateAnalyser {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.state
    }
}

impl GameStateAnalyser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const OUTER: SendPropIdentifier =
            SendPropIdentifier::new("DT_AttributeContainer", "m_hOuter");
        const OUTER2: SendPropIdentifier =
            SendPropIdentifier::new("DT_AttributeManager", "m_hOuter");

        let Some(class_name) = self.class_names.get(usize::from(entity.server_class)) else {
            return;
        };

        if entity.update_type == UpdateType::Enter {
            if let Some(prop) = entity
                .get_prop_by_identifier(&OUTER, parser_state)
                .or_else(|| entity.get_prop_by_identifier(&OUTER2, parser_state))
            {
                if let Ok(outer) = Handle::try_from(&prop.value) {
                    self.outer_map.insert(outer, entity.entity_index);
                    self.outer_map_rev.insert(entity.entity_index, outer);
                }
            }
        }

        match class_name.as_str() {
            "CTFPlayer" => handle_player_entity(&mut self.state, entity, parser_state),
            "CTFPlayerResource" => handle_player_resource(&mut self.state, entity, parser_state),
            "CWorld" => self.handle_world_entity(entity, parser_state),
            "CObjectSentrygun" => handle_sentry_entity(&mut self.state, entity, parser_state),
            "CObjectDispenser" => handle_dispenser_entity(&mut self.state, entity, parser_state),
            "CObjectTeleporter" => handle_teleporter_entity(&mut self.state, entity, parser_state),
            "CFuncTrackTrain" => self.handle_train_entity(entity, parser_state),
            "CWeaponMedigun" => handle_medigun_entity(&mut self.state, entity, &self.outer_map_rev),
            _ if class_name.starts_with("CTFProjectile_")
                || class_name.as_str() == "CTFGrenadePipebombProjectile" =>
            {
                handle_projectile_entity(&mut self.state, entity, parser_state, &self.class_names)
            }
            _ => {}
        }
    }

    pub fn handle_world_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const MINS: SendPropIdentifier = SendPropIdentifier::new("DT_WORLD", "m_WorldMins");
        const MAXS: SendPropIdentifier = SendPropIdentifier::new("DT_WORLD", "m_WorldMaxs");

        if let (
            Some(SendProp {
                value: SendPropValue::Vector(boundary_min),
                ..
            }),
            Some(SendProp {
                value: SendPropValue::Vector(boundary_max),
                ..
            }),
        ) = (
            entity.get_prop_by_identifier(&MINS, parser_state),
            entity.get_prop_by_identifier(&MAXS, parser_state),
        ) {
            self.state.world = Some(World {
                boundary_min,
                boundary_max,
            })
        }
    }

    pub fn handle_train_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const POSITION: SendPropIdentifier =
            SendPropIdentifier::new("DT_BaseEntity", "m_vecOrigin");

        let objective = self
            .state
            .objectives
            .entry(entity.entity_index)
            .or_insert_with(|| Objective::Cart(Cart::default()));

        #[allow(irrefutable_let_patterns)]
        if let Objective::Cart(cart) = objective {
            for prop in entity.props(parser_state) {
                if prop.identifier == POSITION {
                    let pos = Vector::try_from(&prop.value).unwrap_or_default();
                    cart.position = pos
                }
            }
        }
    }

    #[allow(dead_code, unused_variables)]
    pub fn handle_cp_entity(&mut self, entity: &PacketEntity, parser_state: &ParserState) {
        const OWNERS: [SendPropIdentifier; 5] = [
            SendPropIdentifier::new("m_iOwner", "000"),
            SendPropIdentifier::new("m_iOwner", "001"),
            SendPropIdentifier::new("m_iOwner", "002"),
            SendPropIdentifier::new("m_iOwner", "003"),
            SendPropIdentifier::new("m_iOwner", "004"),
        ];
        const CAP_PERCENTAGE: [SendPropIdentifier; 5] = [
            SendPropIdentifier::new("m_flLazyCapPerc", "000"),
            SendPropIdentifier::new("m_flLazyCapPerc", "001"),
            SendPropIdentifier::new("m_flLazyCapPerc", "002"),
            SendPropIdentifier::new("m_flLazyCapPerc", "003"),
            SendPropIdentifier::new("m_flLazyCapPerc", "004"),
        ];

        let objective = self
            .state
            .objectives
            .entry(entity.entity_index)
            .or_insert_with(|| Objective::Cart(Cart::default()));

        todo!()
    }

    fn parse_user_info(
        &mut self,
        index: usize,
        text: Option<&str>,
        data: Option<Stream>,
    ) -> ReadResult<()> {
        if let Some(user_info) =
            crate::demo::data::UserInfo::parse_from_string_table(index as u16, text, data)?
        {
            let id = user_info.entity_id;
            self.state.get_or_create_player(id).info = Some(user_info.into());
        }

        Ok(())
    }
}
