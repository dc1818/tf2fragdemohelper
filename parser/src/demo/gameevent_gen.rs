use super::gamevent::{
    EventValue, GameEventDefinition, GameEventEntry, GameEventValue, RawGameEvent,
};
use crate::demo::data::MaybeUtf8String;
use crate::demo::Stream;
use crate::{ParseError, Result};
use bitbuffer::{BitRead, BitWrite, BitWriteStream, LittleEndian};
use serde::{Deserialize, Serialize};
use std::mem::size_of;
fn read_value<'a, T: EventValue + BitRead<'a, LittleEndian> + Default>(
    stream: &mut Stream<'a>,
    entry: Option<&GameEventEntry>,
    name: &'static str,
) -> Result<T> {
    let entry = match entry {
        Some(entry) => entry,
        None => {
            return Ok(T::default());
        }
    };
    if T::value_type() != entry.kind {
        return Err(ParseError::InvalidGameEvent {
            expected_type: T::value_type(),
            name,
            found_type: entry.kind,
        });
    }
    Ok(T::read(stream)?)
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerSpawnEvent {
    pub hostname: MaybeUtf8String,
    pub address: MaybeUtf8String,
    pub ip: u32,
    pub port: u16,
    pub game: MaybeUtf8String,
    pub map_name: MaybeUtf8String,
    pub max_players: u32,
    pub os: MaybeUtf8String,
    pub dedicated: bool,
    pub password: bool,
}
impl ServerSpawnEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerSpawnEvent {
            hostname: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7992289610851289516u64),
                "hostname",
            )?,
            address: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(1673076945917317811u64),
                "address",
            )?,
            ip: read_value::<u32>(stream, definition.get_entry(628043273916406972u64), "ip")?,
            port: read_value::<u16>(
                stream,
                definition.get_entry(10100688915994460070u64),
                "port",
            )?,
            game: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(10005491431272162599u64),
                "game",
            )?,
            map_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18037678950216614794u64),
                "map_name",
            )?,
            max_players: read_value::<u32>(
                stream,
                definition.get_entry(6820574247554962453u64),
                "max_players",
            )?,
            os: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(626093839799967319u64),
                "os",
            )?,
            dedicated: read_value::<bool>(
                stream,
                definition.get_entry(17181338330120084322u64),
                "dedicated",
            )?,
            password: read_value::<bool>(
                stream,
                definition.get_entry(5411718394350379800u64),
                "password",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7992289610851289516u64 => Ok(self.hostname.clone().into()),
            1673076945917317811u64 => Ok(self.address.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            10100688915994460070u64 => Ok(self.port.clone().into()),
            10005491431272162599u64 => Ok(self.game.clone().into()),
            18037678950216614794u64 => Ok(self.map_name.clone().into()),
            6820574247554962453u64 => Ok(self.max_players.clone().into()),
            626093839799967319u64 => Ok(self.os.clone().into()),
            17181338330120084322u64 => Ok(self.dedicated.clone().into()),
            5411718394350379800u64 => Ok(self.password.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerSpawn",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerChangeLevelFailedEvent {
    pub level_name: MaybeUtf8String,
}
impl ServerChangeLevelFailedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerChangeLevelFailedEvent {
            level_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8103714013497669086u64),
                "level_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8103714013497669086u64 => Ok(self.level_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerChangeLevelFailed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerShutdownEvent {
    pub reason: MaybeUtf8String,
}
impl ServerShutdownEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerShutdownEvent {
            reason: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerShutdown",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerCvarEvent {
    pub cvar_name: MaybeUtf8String,
    pub cvar_value: MaybeUtf8String,
}
impl ServerCvarEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerCvarEvent {
            cvar_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8822721269188576188u64),
                "cvar_name",
            )?,
            cvar_value: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(9254320334284503348u64),
                "cvar_value",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8822721269188576188u64 => Ok(self.cvar_name.clone().into()),
            9254320334284503348u64 => Ok(self.cvar_value.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerCvar",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerMessageEvent {
    pub text: MaybeUtf8String,
}
impl ServerMessageEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerMessageEvent {
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerMessage",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerAddBanEvent {
    pub name: MaybeUtf8String,
    pub user_id: u16,
    pub network_id: MaybeUtf8String,
    pub ip: MaybeUtf8String,
    pub duration: MaybeUtf8String,
    pub by: MaybeUtf8String,
    pub kicked: bool,
}
impl ServerAddBanEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerAddBanEvent {
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            ip: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(628043273916406972u64),
                "ip",
            )?,
            duration: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(10012068961515151501u64),
                "duration",
            )?,
            by: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(623268094916032724u64),
                "by",
            )?,
            kicked: read_value::<bool>(
                stream,
                definition.get_entry(11906843383782741950u64),
                "kicked",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14176396743819860870u64 => Ok(self.name.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            10012068961515151501u64 => Ok(self.duration.clone().into()),
            623268094916032724u64 => Ok(self.by.clone().into()),
            11906843383782741950u64 => Ok(self.kicked.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerAddBan",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServerRemoveBanEvent {
    pub network_id: MaybeUtf8String,
    pub ip: MaybeUtf8String,
    pub by: MaybeUtf8String,
}
impl ServerRemoveBanEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ServerRemoveBanEvent {
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            ip: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(628043273916406972u64),
                "ip",
            )?,
            by: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(623268094916032724u64),
                "by",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            623268094916032724u64 => Ok(self.by.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ServerRemoveBan",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerConnectEvent {
    pub name: MaybeUtf8String,
    pub index: u8,
    pub user_id: u16,
    pub network_id: MaybeUtf8String,
    pub address: MaybeUtf8String,
    pub bot: u16,
}
impl PlayerConnectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerConnectEvent {
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
            index: read_value::<u8>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            address: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(1673076945917317811u64),
                "address",
            )?,
            bot: read_value::<u16>(stream, definition.get_entry(21728656485903294u64), "bot")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14176396743819860870u64 => Ok(self.name.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            1673076945917317811u64 => Ok(self.address.clone().into()),
            21728656485903294u64 => Ok(self.bot.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerConnect",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerConnectClientEvent {
    pub name: MaybeUtf8String,
    pub index: u8,
    pub user_id: u16,
    pub network_id: MaybeUtf8String,
    pub bot: u16,
}
impl PlayerConnectClientEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerConnectClientEvent {
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
            index: read_value::<u8>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            bot: read_value::<u16>(stream, definition.get_entry(21728656485903294u64), "bot")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14176396743819860870u64 => Ok(self.name.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            21728656485903294u64 => Ok(self.bot.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerConnectClient",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerInfoEvent {
    pub name: MaybeUtf8String,
    pub index: u8,
    pub user_id: u16,
    pub network_id: MaybeUtf8String,
    pub bot: bool,
}
impl PlayerInfoEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerInfoEvent {
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
            index: read_value::<u8>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            bot: read_value::<bool>(stream, definition.get_entry(21728656485903294u64), "bot")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14176396743819860870u64 => Ok(self.name.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            21728656485903294u64 => Ok(self.bot.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerInfo",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDisconnectEvent {
    pub user_id: u16,
    pub reason: MaybeUtf8String,
    pub name: MaybeUtf8String,
    pub network_id: MaybeUtf8String,
    pub bot: u16,
}
impl PlayerDisconnectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDisconnectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            reason: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
            network_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2293149186490744864u64),
                "network_id",
            )?,
            bot: read_value::<u16>(stream, definition.get_entry(21728656485903294u64), "bot")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            14176396743819860870u64 => Ok(self.name.clone().into()),
            2293149186490744864u64 => Ok(self.network_id.clone().into()),
            21728656485903294u64 => Ok(self.bot.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDisconnect",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerActivateEvent {
    pub user_id: u16,
}
impl PlayerActivateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerActivateEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerActivate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerSayEvent {
    pub user_id: u16,
    pub text: MaybeUtf8String,
}
impl PlayerSayEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerSayEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerSay",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClientDisconnectEvent {
    pub message: MaybeUtf8String,
}
impl ClientDisconnectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ClientDisconnectEvent {
            message: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(6080987277291999908u64),
                "message",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6080987277291999908u64 => Ok(self.message.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ClientDisconnect",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClientBeginConnectEvent {
    pub address: MaybeUtf8String,
    pub ip: u32,
    pub port: u16,
    pub source: MaybeUtf8String,
}
impl ClientBeginConnectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ClientBeginConnectEvent {
            address: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(1673076945917317811u64),
                "address",
            )?,
            ip: read_value::<u32>(stream, definition.get_entry(628043273916406972u64), "ip")?,
            port: read_value::<u16>(
                stream,
                definition.get_entry(10100688915994460070u64),
                "port",
            )?,
            source: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8564681157369146808u64),
                "source",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1673076945917317811u64 => Ok(self.address.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            10100688915994460070u64 => Ok(self.port.clone().into()),
            8564681157369146808u64 => Ok(self.source.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ClientBeginConnect",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClientConnectedEvent {
    pub address: MaybeUtf8String,
    pub ip: u32,
    pub port: u16,
}
impl ClientConnectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ClientConnectedEvent {
            address: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(1673076945917317811u64),
                "address",
            )?,
            ip: read_value::<u32>(stream, definition.get_entry(628043273916406972u64), "ip")?,
            port: read_value::<u16>(
                stream,
                definition.get_entry(10100688915994460070u64),
                "port",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1673076945917317811u64 => Ok(self.address.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            10100688915994460070u64 => Ok(self.port.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ClientConnected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClientFullConnectEvent {
    pub address: MaybeUtf8String,
    pub ip: u32,
    pub port: u16,
}
impl ClientFullConnectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ClientFullConnectEvent {
            address: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(1673076945917317811u64),
                "address",
            )?,
            ip: read_value::<u32>(stream, definition.get_entry(628043273916406972u64), "ip")?,
            port: read_value::<u16>(
                stream,
                definition.get_entry(10100688915994460070u64),
                "port",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1673076945917317811u64 => Ok(self.address.clone().into()),
            628043273916406972u64 => Ok(self.ip.clone().into()),
            10100688915994460070u64 => Ok(self.port.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ClientFullConnect",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HostQuitEvent {}
impl HostQuitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HostQuitEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HostQuit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamInfoEvent {
    pub team_id: u8,
    pub team_name: MaybeUtf8String,
}
impl TeamInfoEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamInfoEvent {
            team_id: read_value::<u8>(
                stream,
                definition.get_entry(16102541790268531873u64),
                "team_id",
            )?,
            team_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2516817673391228199u64),
                "team_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16102541790268531873u64 => Ok(self.team_id.clone().into()),
            2516817673391228199u64 => Ok(self.team_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamInfo",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamScoreEvent {
    pub team_id: u8,
    pub score: u16,
}
impl TeamScoreEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamScoreEvent {
            team_id: read_value::<u8>(
                stream,
                definition.get_entry(16102541790268531873u64),
                "team_id",
            )?,
            score: read_value::<u16>(
                stream,
                definition.get_entry(13911166232573650165u64),
                "score",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16102541790268531873u64 => Ok(self.team_id.clone().into()),
            13911166232573650165u64 => Ok(self.score.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamScore",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayBroadcastAudioEvent {
    pub team: u8,
    pub sound: MaybeUtf8String,
    pub additional_flags: u16,
    pub player: u16,
}
impl TeamPlayBroadcastAudioEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayBroadcastAudioEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            sound: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7337464818993397268u64),
                "sound",
            )?,
            additional_flags: read_value::<u16>(
                stream,
                definition.get_entry(10653216584182196624u64),
                "additional_flags",
            )?,
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            7337464818993397268u64 => Ok(self.sound.clone().into()),
            10653216584182196624u64 => Ok(self.additional_flags.clone().into()),
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayBroadcastAudio",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerTeamEvent {
    pub user_id: u16,
    pub team: u8,
    pub old_team: u8,
    pub disconnect: bool,
    pub auto_team: bool,
    pub silent: bool,
    pub name: MaybeUtf8String,
}
impl PlayerTeamEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerTeamEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            old_team: read_value::<u8>(
                stream,
                definition.get_entry(11079076405359550719u64),
                "old_team",
            )?,
            disconnect: read_value::<bool>(
                stream,
                definition.get_entry(6424045679635350635u64),
                "disconnect",
            )?,
            auto_team: read_value::<bool>(
                stream,
                definition.get_entry(9165371035831628223u64),
                "auto_team",
            )?,
            silent: read_value::<bool>(
                stream,
                definition.get_entry(6452236368899434340u64),
                "silent",
            )?,
            name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14176396743819860870u64),
                "name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            11079076405359550719u64 => Ok(self.old_team.clone().into()),
            6424045679635350635u64 => Ok(self.disconnect.clone().into()),
            9165371035831628223u64 => Ok(self.auto_team.clone().into()),
            6452236368899434340u64 => Ok(self.silent.clone().into()),
            14176396743819860870u64 => Ok(self.name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerTeam",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerClassEvent {
    pub user_id: u16,
    pub class: MaybeUtf8String,
}
impl PlayerClassEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerClassEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            class: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(15066323702654938015u64),
                "class",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            15066323702654938015u64 => Ok(self.class.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerClass",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDeathEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub assister: u16,
    pub weapon_log_class_name: MaybeUtf8String,
    pub stun_flags: u16,
    pub death_flags: u16,
    pub silent_kill: bool,
    pub player_penetrate_count: u16,
    pub assister_fallback: MaybeUtf8String,
    pub kill_streak_total: u16,
    pub kill_streak_wep: u16,
    pub kill_streak_assist: u16,
    pub kill_streak_victim: u16,
    pub ducks_streaked: u16,
    pub duck_streak_total: u16,
    pub duck_streak_assist: u16,
    pub duck_streak_victim: u16,
    pub rocket_jump: bool,
    pub weapon_def_index: u32,
    pub crit_type: u16,
}
impl PlayerDeathEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDeathEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
            stun_flags: read_value::<u16>(
                stream,
                definition.get_entry(16746745151415897845u64),
                "stun_flags",
            )?,
            death_flags: read_value::<u16>(
                stream,
                definition.get_entry(210841622282264177u64),
                "death_flags",
            )?,
            silent_kill: read_value::<bool>(
                stream,
                definition.get_entry(5449831253309542421u64),
                "silent_kill",
            )?,
            player_penetrate_count: read_value::<u16>(
                stream,
                definition.get_entry(6165847213797285919u64),
                "player_penetrate_count",
            )?,
            assister_fallback: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2624120833319605424u64),
                "assister_fallback",
            )?,
            kill_streak_total: read_value::<u16>(
                stream,
                definition.get_entry(10219443329572148957u64),
                "kill_streak_total",
            )?,
            kill_streak_wep: read_value::<u16>(
                stream,
                definition.get_entry(14151704064294986651u64),
                "kill_streak_wep",
            )?,
            kill_streak_assist: read_value::<u16>(
                stream,
                definition.get_entry(3408761288007698574u64),
                "kill_streak_assist",
            )?,
            kill_streak_victim: read_value::<u16>(
                stream,
                definition.get_entry(14613767699666342005u64),
                "kill_streak_victim",
            )?,
            ducks_streaked: read_value::<u16>(
                stream,
                definition.get_entry(8814124002674372577u64),
                "ducks_streaked",
            )?,
            duck_streak_total: read_value::<u16>(
                stream,
                definition.get_entry(2758270581670703974u64),
                "duck_streak_total",
            )?,
            duck_streak_assist: read_value::<u16>(
                stream,
                definition.get_entry(10967369523768500963u64),
                "duck_streak_assist",
            )?,
            duck_streak_victim: read_value::<u16>(
                stream,
                definition.get_entry(620103205137188524u64),
                "duck_streak_victim",
            )?,
            rocket_jump: read_value::<bool>(
                stream,
                definition.get_entry(16207427969859362406u64),
                "rocket_jump",
            )?,
            weapon_def_index: read_value::<u32>(
                stream,
                definition.get_entry(4132306594868589054u64),
                "weapon_def_index",
            )?,
            crit_type: read_value::<u16>(
                stream,
                definition.get_entry(7263029362109349446u64),
                "crit_type",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            16746745151415897845u64 => Ok(self.stun_flags.clone().into()),
            210841622282264177u64 => Ok(self.death_flags.clone().into()),
            5449831253309542421u64 => Ok(self.silent_kill.clone().into()),
            6165847213797285919u64 => Ok(self.player_penetrate_count.clone().into()),
            2624120833319605424u64 => Ok(self.assister_fallback.clone().into()),
            10219443329572148957u64 => Ok(self.kill_streak_total.clone().into()),
            14151704064294986651u64 => Ok(self.kill_streak_wep.clone().into()),
            3408761288007698574u64 => Ok(self.kill_streak_assist.clone().into()),
            14613767699666342005u64 => Ok(self.kill_streak_victim.clone().into()),
            8814124002674372577u64 => Ok(self.ducks_streaked.clone().into()),
            2758270581670703974u64 => Ok(self.duck_streak_total.clone().into()),
            10967369523768500963u64 => Ok(self.duck_streak_assist.clone().into()),
            620103205137188524u64 => Ok(self.duck_streak_victim.clone().into()),
            16207427969859362406u64 => Ok(self.rocket_jump.clone().into()),
            4132306594868589054u64 => Ok(self.weapon_def_index.clone().into()),
            7263029362109349446u64 => Ok(self.crit_type.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDeath",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHurtEvent {
    pub user_id: u16,
    pub health: u16,
    pub attacker: u16,
    pub damage_amount: u16,
    pub custom: u16,
    pub show_disguised_crit: bool,
    pub crit: bool,
    pub mini_crit: bool,
    pub all_see_crit: bool,
    pub weapon_id: u16,
    pub bonus_effect: u8,
}
impl PlayerHurtEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHurtEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            health: read_value::<u16>(
                stream,
                definition.get_entry(9181103189905877455u64),
                "health",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            damage_amount: read_value::<u16>(
                stream,
                definition.get_entry(7439038394412279612u64),
                "damage_amount",
            )?,
            custom: read_value::<u16>(
                stream,
                definition.get_entry(604290716149806926u64),
                "custom",
            )?,
            show_disguised_crit: read_value::<bool>(
                stream,
                definition.get_entry(14301803044080296297u64),
                "show_disguised_crit",
            )?,
            crit: read_value::<bool>(stream, definition.get_entry(1324453635955533101u64), "crit")?,
            mini_crit: read_value::<bool>(
                stream,
                definition.get_entry(18286698110279670006u64),
                "mini_crit",
            )?,
            all_see_crit: read_value::<bool>(
                stream,
                definition.get_entry(3290419718563846047u64),
                "all_see_crit",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            bonus_effect: read_value::<u8>(
                stream,
                definition.get_entry(4613275483771643085u64),
                "bonus_effect",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            9181103189905877455u64 => Ok(self.health.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            7439038394412279612u64 => Ok(self.damage_amount.clone().into()),
            604290716149806926u64 => Ok(self.custom.clone().into()),
            14301803044080296297u64 => Ok(self.show_disguised_crit.clone().into()),
            1324453635955533101u64 => Ok(self.crit.clone().into()),
            18286698110279670006u64 => Ok(self.mini_crit.clone().into()),
            3290419718563846047u64 => Ok(self.all_see_crit.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            4613275483771643085u64 => Ok(self.bonus_effect.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHurt",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerChatEvent {
    pub team_only: bool,
    pub user_id: u16,
    pub text: MaybeUtf8String,
}
impl PlayerChatEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerChatEvent {
            team_only: read_value::<bool>(
                stream,
                definition.get_entry(8997360128490965478u64),
                "team_only",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8997360128490965478u64 => Ok(self.team_only.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerChat",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerScoreEvent {
    pub user_id: u16,
    pub kills: u16,
    pub deaths: u16,
    pub score: u16,
}
impl PlayerScoreEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerScoreEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            kills: read_value::<u16>(
                stream,
                definition.get_entry(8934927864608526494u64),
                "kills",
            )?,
            deaths: read_value::<u16>(
                stream,
                definition.get_entry(15487195249286514682u64),
                "deaths",
            )?,
            score: read_value::<u16>(
                stream,
                definition.get_entry(13911166232573650165u64),
                "score",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            8934927864608526494u64 => Ok(self.kills.clone().into()),
            15487195249286514682u64 => Ok(self.deaths.clone().into()),
            13911166232573650165u64 => Ok(self.score.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerScore",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerSpawnEvent {
    pub user_id: u16,
    pub team: u16,
    pub class: u16,
}
impl PlayerSpawnEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerSpawnEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            team: read_value::<u16>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            class: read_value::<u16>(
                stream,
                definition.get_entry(15066323702654938015u64),
                "class",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            15066323702654938015u64 => Ok(self.class.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerSpawn",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerShootEvent {
    pub user_id: u16,
    pub weapon: u8,
    pub mode: u8,
}
impl PlayerShootEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerShootEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            weapon: read_value::<u8>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            mode: read_value::<u8>(stream, definition.get_entry(954177780379921842u64), "mode")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            954177780379921842u64 => Ok(self.mode.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerShoot",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerUseEvent {
    pub user_id: u16,
    pub entity: u16,
}
impl PlayerUseEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerUseEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            entity: read_value::<u16>(
                stream,
                definition.get_entry(10409922166629367034u64),
                "entity",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10409922166629367034u64 => Ok(self.entity.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerUse",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerChangeNameEvent {
    pub user_id: u16,
    pub old_name: MaybeUtf8String,
    pub new_name: MaybeUtf8String,
}
impl PlayerChangeNameEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerChangeNameEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            old_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11919108480345551253u64),
                "old_name",
            )?,
            new_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8904377156710117674u64),
                "new_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            11919108480345551253u64 => Ok(self.old_name.clone().into()),
            8904377156710117674u64 => Ok(self.new_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerChangeName",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHintMessageEvent {
    pub hint_message: MaybeUtf8String,
}
impl PlayerHintMessageEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHintMessageEvent {
            hint_message: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(13996716249204415567u64),
                "hint_message",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            13996716249204415567u64 => Ok(self.hint_message.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHintMessage",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BasePlayerTeleportedEvent {
    pub ent_index: u16,
}
impl BasePlayerTeleportedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BasePlayerTeleportedEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BasePlayerTeleported",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameInitEvent {}
impl GameInitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameInitEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameInit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameNewMapEvent {
    pub map_name: MaybeUtf8String,
}
impl GameNewMapEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameNewMapEvent {
            map_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18037678950216614794u64),
                "map_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18037678950216614794u64 => Ok(self.map_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameNewMap",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameStartEvent {
    pub rounds_limit: u32,
    pub time_limit: u32,
    pub frag_limit: u32,
    pub objective: MaybeUtf8String,
}
impl GameStartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameStartEvent {
            rounds_limit: read_value::<u32>(
                stream,
                definition.get_entry(6594118856211890507u64),
                "rounds_limit",
            )?,
            time_limit: read_value::<u32>(
                stream,
                definition.get_entry(8925605756456439511u64),
                "time_limit",
            )?,
            frag_limit: read_value::<u32>(
                stream,
                definition.get_entry(9937264313491586980u64),
                "frag_limit",
            )?,
            objective: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8095747183904291896u64),
                "objective",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6594118856211890507u64 => Ok(self.rounds_limit.clone().into()),
            8925605756456439511u64 => Ok(self.time_limit.clone().into()),
            9937264313491586980u64 => Ok(self.frag_limit.clone().into()),
            8095747183904291896u64 => Ok(self.objective.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameStart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameEndEvent {
    pub winner: u8,
}
impl GameEndEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameEndEvent {
            winner: read_value::<u8>(
                stream,
                definition.get_entry(4337804175666422150u64),
                "winner",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            4337804175666422150u64 => Ok(self.winner.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameEnd",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RoundStartEvent {
    pub time_limit: u32,
    pub frag_limit: u32,
    pub objective: MaybeUtf8String,
}
impl RoundStartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RoundStartEvent {
            time_limit: read_value::<u32>(
                stream,
                definition.get_entry(8925605756456439511u64),
                "time_limit",
            )?,
            frag_limit: read_value::<u32>(
                stream,
                definition.get_entry(9937264313491586980u64),
                "frag_limit",
            )?,
            objective: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8095747183904291896u64),
                "objective",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8925605756456439511u64 => Ok(self.time_limit.clone().into()),
            9937264313491586980u64 => Ok(self.frag_limit.clone().into()),
            8095747183904291896u64 => Ok(self.objective.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RoundStart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RoundEndEvent {
    pub winner: u8,
    pub reason: u8,
    pub message: MaybeUtf8String,
}
impl RoundEndEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RoundEndEvent {
            winner: read_value::<u8>(
                stream,
                definition.get_entry(4337804175666422150u64),
                "winner",
            )?,
            reason: read_value::<u8>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
            message: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(6080987277291999908u64),
                "message",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            4337804175666422150u64 => Ok(self.winner.clone().into()),
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            6080987277291999908u64 => Ok(self.message.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RoundEnd",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameMessageEvent {
    pub target: u8,
    pub text: MaybeUtf8String,
}
impl GameMessageEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameMessageEvent {
            target: read_value::<u8>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1653916590517707752u64 => Ok(self.target.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameMessage",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BreakBreakableEvent {
    pub ent_index: u32,
    pub user_id: u16,
    pub material: u8,
}
impl BreakBreakableEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BreakBreakableEvent {
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            material: read_value::<u8>(
                stream,
                definition.get_entry(175488002581160416u64),
                "material",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            175488002581160416u64 => Ok(self.material.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BreakBreakable",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BreakPropEvent {
    pub ent_index: u32,
    pub user_id: u16,
}
impl BreakPropEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BreakPropEvent {
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BreakProp",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EntityKilledEvent {
    pub ent_index_killed: u32,
    pub ent_index_attacker: u32,
    pub ent_index_inflictor: u32,
    pub damage_bits: u32,
}
impl EntityKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EntityKilledEvent {
            ent_index_killed: read_value::<u32>(
                stream,
                definition.get_entry(9772342216534838146u64),
                "ent_index_killed",
            )?,
            ent_index_attacker: read_value::<u32>(
                stream,
                definition.get_entry(15130955426090253880u64),
                "ent_index_attacker",
            )?,
            ent_index_inflictor: read_value::<u32>(
                stream,
                definition.get_entry(12707416538007474931u64),
                "ent_index_inflictor",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9772342216534838146u64 => Ok(self.ent_index_killed.clone().into()),
            15130955426090253880u64 => Ok(self.ent_index_attacker.clone().into()),
            12707416538007474931u64 => Ok(self.ent_index_inflictor.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EntityKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BonusUpdatedEvent {
    pub num_advanced: u16,
    pub num_bronze: u16,
    pub num_silver: u16,
    pub num_gold: u16,
}
impl BonusUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BonusUpdatedEvent {
            num_advanced: read_value::<u16>(
                stream,
                definition.get_entry(18281866615588036317u64),
                "num_advanced",
            )?,
            num_bronze: read_value::<u16>(
                stream,
                definition.get_entry(17784477894966475211u64),
                "num_bronze",
            )?,
            num_silver: read_value::<u16>(
                stream,
                definition.get_entry(654857209225446882u64),
                "num_silver",
            )?,
            num_gold: read_value::<u16>(
                stream,
                definition.get_entry(4800941608394754913u64),
                "num_gold",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18281866615588036317u64 => Ok(self.num_advanced.clone().into()),
            17784477894966475211u64 => Ok(self.num_bronze.clone().into()),
            654857209225446882u64 => Ok(self.num_silver.clone().into()),
            4800941608394754913u64 => Ok(self.num_gold.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BonusUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AchievementEventEvent {
    pub achievement_name: MaybeUtf8String,
    pub cur_val: u16,
    pub max_val: u16,
}
impl AchievementEventEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(AchievementEventEvent {
            achievement_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(15691172238087995014u64),
                "achievement_name",
            )?,
            cur_val: read_value::<u16>(
                stream,
                definition.get_entry(5486189633889604213u64),
                "cur_val",
            )?,
            max_val: read_value::<u16>(
                stream,
                definition.get_entry(15860362688261047681u64),
                "max_val",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            15691172238087995014u64 => Ok(self.achievement_name.clone().into()),
            5486189633889604213u64 => Ok(self.cur_val.clone().into()),
            15860362688261047681u64 => Ok(self.max_val.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "AchievementEvent",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AchievementIncrementEvent {
    pub achievement_id: u32,
    pub cur_val: u16,
    pub max_val: u16,
}
impl AchievementIncrementEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(AchievementIncrementEvent {
            achievement_id: read_value::<u32>(
                stream,
                definition.get_entry(17475110908491474368u64),
                "achievement_id",
            )?,
            cur_val: read_value::<u16>(
                stream,
                definition.get_entry(5486189633889604213u64),
                "cur_val",
            )?,
            max_val: read_value::<u16>(
                stream,
                definition.get_entry(15860362688261047681u64),
                "max_val",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17475110908491474368u64 => Ok(self.achievement_id.clone().into()),
            5486189633889604213u64 => Ok(self.cur_val.clone().into()),
            15860362688261047681u64 => Ok(self.max_val.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "AchievementIncrement",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PhysgunPickupEvent {
    pub ent_index: u32,
}
impl PhysgunPickupEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PhysgunPickupEvent {
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PhysgunPickup",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FlareIgniteNpcEvent {
    pub ent_index: u32,
}
impl FlareIgniteNpcEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FlareIgniteNpcEvent {
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FlareIgniteNpc",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HelicopterGrenadePuntMissEvent {}
impl HelicopterGrenadePuntMissEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HelicopterGrenadePuntMissEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HelicopterGrenadePuntMiss",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserDataDownloadedEvent {}
impl UserDataDownloadedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(UserDataDownloadedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "UserDataDownloaded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RagdollDissolvedEvent {
    pub ent_index: u32,
}
impl RagdollDissolvedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RagdollDissolvedEvent {
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RagdollDissolved",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVChangedModeEvent {
    pub old_mode: u16,
    pub new_mode: u16,
    pub obs_target: u16,
}
impl HLTVChangedModeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVChangedModeEvent {
            old_mode: read_value::<u16>(
                stream,
                definition.get_entry(13993189934714533949u64),
                "old_mode",
            )?,
            new_mode: read_value::<u16>(
                stream,
                definition.get_entry(874641438558876942u64),
                "new_mode",
            )?,
            obs_target: read_value::<u16>(
                stream,
                definition.get_entry(14360750886734159999u64),
                "obs_target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            13993189934714533949u64 => Ok(self.old_mode.clone().into()),
            874641438558876942u64 => Ok(self.new_mode.clone().into()),
            14360750886734159999u64 => Ok(self.obs_target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVChangedMode",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVChangedTargetEvent {
    pub mode: u16,
    pub old_target: u16,
    pub obs_target: u16,
}
impl HLTVChangedTargetEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVChangedTargetEvent {
            mode: read_value::<u16>(stream, definition.get_entry(954177780379921842u64), "mode")?,
            old_target: read_value::<u16>(
                stream,
                definition.get_entry(16423341895021030510u64),
                "old_target",
            )?,
            obs_target: read_value::<u16>(
                stream,
                definition.get_entry(14360750886734159999u64),
                "obs_target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            954177780379921842u64 => Ok(self.mode.clone().into()),
            16423341895021030510u64 => Ok(self.old_target.clone().into()),
            14360750886734159999u64 => Ok(self.obs_target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVChangedTarget",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteEndedEvent {}
impl VoteEndedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteEndedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteEnded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteStartedEvent {
    pub issue: MaybeUtf8String,
    pub param_1: MaybeUtf8String,
    pub team: u8,
    pub initiator: u32,
    pub voteidx: u32,
}
impl VoteStartedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteStartedEvent {
            issue: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2041490703516169504u64),
                "issue",
            )?,
            param_1: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(4990490691588242105u64),
                "param_1",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            initiator: read_value::<u32>(
                stream,
                definition.get_entry(7196121162372295066u64),
                "initiator",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2041490703516169504u64 => Ok(self.issue.clone().into()),
            4990490691588242105u64 => Ok(self.param_1.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            7196121162372295066u64 => Ok(self.initiator.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteStarted",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteChangedEvent {
    pub vote_option_1: u8,
    pub vote_option_2: u8,
    pub vote_option_3: u8,
    pub vote_option_4: u8,
    pub vote_option_5: u8,
    pub potential_votes: u8,
    pub voteidx: u32,
}
impl VoteChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteChangedEvent {
            vote_option_1: read_value::<u8>(
                stream,
                definition.get_entry(8259566198638570134u64),
                "vote_option_1",
            )?,
            vote_option_2: read_value::<u8>(
                stream,
                definition.get_entry(8259565099126941923u64),
                "vote_option_2",
            )?,
            vote_option_3: read_value::<u8>(
                stream,
                definition.get_entry(8259563999615313712u64),
                "vote_option_3",
            )?,
            vote_option_4: read_value::<u8>(
                stream,
                definition.get_entry(8259571696196711189u64),
                "vote_option_4",
            )?,
            vote_option_5: read_value::<u8>(
                stream,
                definition.get_entry(8259570596685082978u64),
                "vote_option_5",
            )?,
            potential_votes: read_value::<u8>(
                stream,
                definition.get_entry(18034020270891649474u64),
                "potential_votes",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8259566198638570134u64 => Ok(self.vote_option_1.clone().into()),
            8259565099126941923u64 => Ok(self.vote_option_2.clone().into()),
            8259563999615313712u64 => Ok(self.vote_option_3.clone().into()),
            8259571696196711189u64 => Ok(self.vote_option_4.clone().into()),
            8259570596685082978u64 => Ok(self.vote_option_5.clone().into()),
            18034020270891649474u64 => Ok(self.potential_votes.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VotePassedEvent {
    pub details: MaybeUtf8String,
    pub param_1: MaybeUtf8String,
    pub team: u8,
    pub voteidx: u32,
}
impl VotePassedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VotePassedEvent {
            details: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(13353550034922503269u64),
                "details",
            )?,
            param_1: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(4990490691588242105u64),
                "param_1",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            13353550034922503269u64 => Ok(self.details.clone().into()),
            4990490691588242105u64 => Ok(self.param_1.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VotePassed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteFailedEvent {
    pub team: u8,
    pub voteidx: u32,
}
impl VoteFailedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteFailedEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteFailed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteCastEvent {
    pub vote_option: u8,
    pub team: u16,
    pub entity_id: u32,
    pub voteidx: u32,
}
impl VoteCastEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteCastEvent {
            vote_option: read_value::<u8>(
                stream,
                definition.get_entry(17670279370117350435u64),
                "vote_option",
            )?,
            team: read_value::<u16>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            entity_id: read_value::<u32>(
                stream,
                definition.get_entry(2085882069322833143u64),
                "entity_id",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17670279370117350435u64 => Ok(self.vote_option.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            2085882069322833143u64 => Ok(self.entity_id.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteCast",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteOptionsEvent {
    pub count: u8,
    pub option_1: MaybeUtf8String,
    pub option_2: MaybeUtf8String,
    pub option_3: MaybeUtf8String,
    pub option_4: MaybeUtf8String,
    pub option_5: MaybeUtf8String,
    pub voteidx: u32,
}
impl VoteOptionsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteOptionsEvent {
            count: read_value::<u8>(
                stream,
                definition.get_entry(12818901015042040436u64),
                "count",
            )?,
            option_1: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14575392658851491999u64),
                "option_1",
            )?,
            option_2: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14575393758363120210u64),
                "option_2",
            )?,
            option_3: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14575394857874748421u64),
                "option_3",
            )?,
            option_4: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14575387161293350944u64),
                "option_4",
            )?,
            option_5: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(14575388260804979155u64),
                "option_5",
            )?,
            voteidx: read_value::<u32>(
                stream,
                definition.get_entry(5777630607239142584u64),
                "voteidx",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12818901015042040436u64 => Ok(self.count.clone().into()),
            14575392658851491999u64 => Ok(self.option_1.clone().into()),
            14575393758363120210u64 => Ok(self.option_2.clone().into()),
            14575394857874748421u64 => Ok(self.option_3.clone().into()),
            14575387161293350944u64 => Ok(self.option_4.clone().into()),
            14575388260804979155u64 => Ok(self.option_5.clone().into()),
            5777630607239142584u64 => Ok(self.voteidx.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteOptions",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplaySavedEvent {}
impl ReplaySavedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplaySavedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplaySaved",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EnteredPerformanceModeEvent {}
impl EnteredPerformanceModeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EnteredPerformanceModeEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EnteredPerformanceMode",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BrowseReplaysEvent {}
impl BrowseReplaysEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BrowseReplaysEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BrowseReplays",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplayYoutubeStatsEvent {
    pub views: u32,
    pub likes: u32,
    pub favorited: u32,
}
impl ReplayYoutubeStatsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplayYoutubeStatsEvent {
            views: read_value::<u32>(
                stream,
                definition.get_entry(14625097093024684817u64),
                "views",
            )?,
            likes: read_value::<u32>(
                stream,
                definition.get_entry(9804554822404214111u64),
                "likes",
            )?,
            favorited: read_value::<u32>(
                stream,
                definition.get_entry(2653817720246189003u64),
                "favorited",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14625097093024684817u64 => Ok(self.views.clone().into()),
            9804554822404214111u64 => Ok(self.likes.clone().into()),
            2653817720246189003u64 => Ok(self.favorited.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplayYoutubeStats",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct InventoryUpdatedEvent {}
impl InventoryUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(InventoryUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "InventoryUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CartUpdatedEvent {}
impl CartUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CartUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CartUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StorePriceSheetUpdatedEvent {}
impl StorePriceSheetUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(StorePriceSheetUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "StorePriceSheetUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EconInventoryConnectedEvent {}
impl EconInventoryConnectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EconInventoryConnectedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EconInventoryConnected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ItemSchemaInitializedEvent {}
impl ItemSchemaInitializedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ItemSchemaInitializedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ItemSchemaInitialized",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GcNewSessionEvent {}
impl GcNewSessionEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GcNewSessionEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GcNewSession",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GcLostSessionEvent {}
impl GcLostSessionEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GcLostSessionEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GcLostSession",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IntroFinishEvent {
    pub player: u16,
}
impl IntroFinishEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(IntroFinishEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "IntroFinish",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct IntroNextCameraEvent {
    pub player: u16,
}
impl IntroNextCameraEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(IntroNextCameraEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "IntroNextCamera",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerChangeClassEvent {
    pub user_id: u16,
    pub class: u16,
}
impl PlayerChangeClassEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerChangeClassEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            class: read_value::<u16>(
                stream,
                definition.get_entry(15066323702654938015u64),
                "class",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            15066323702654938015u64 => Ok(self.class.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerChangeClass",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TfMapTimeRemainingEvent {
    pub seconds: u32,
}
impl TfMapTimeRemainingEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TfMapTimeRemainingEvent {
            seconds: read_value::<u32>(
                stream,
                definition.get_entry(11456985514702388746u64),
                "seconds",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            11456985514702388746u64 => Ok(self.seconds.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TfMapTimeRemaining",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TfGameOverEvent {
    pub reason: MaybeUtf8String,
}
impl TfGameOverEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TfGameOverEvent {
            reason: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TfGameOver",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CtfFlagCapturedEvent {
    pub capping_team: u16,
    pub capping_team_score: u16,
}
impl CtfFlagCapturedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CtfFlagCapturedEvent {
            capping_team: read_value::<u16>(
                stream,
                definition.get_entry(14568126206963925545u64),
                "capping_team",
            )?,
            capping_team_score: read_value::<u16>(
                stream,
                definition.get_entry(4559517251391003144u64),
                "capping_team_score",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14568126206963925545u64 => Ok(self.capping_team.clone().into()),
            4559517251391003144u64 => Ok(self.capping_team_score.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CtfFlagCaptured",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointInitializedEvent {}
impl ControlPointInitializedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointInitializedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointInitialized",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointUpdateImagesEvent {
    pub index: u16,
}
impl ControlPointUpdateImagesEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointUpdateImagesEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointUpdateImages",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointUpdateLayoutEvent {
    pub index: u16,
}
impl ControlPointUpdateLayoutEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointUpdateLayoutEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointUpdateLayout",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointUpdateCappingEvent {
    pub index: u16,
}
impl ControlPointUpdateCappingEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointUpdateCappingEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointUpdateCapping",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointUpdateOwnerEvent {
    pub index: u16,
}
impl ControlPointUpdateOwnerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointUpdateOwnerEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointUpdateOwner",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointStartTouchEvent {
    pub player: u16,
    pub area: u16,
}
impl ControlPointStartTouchEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointStartTouchEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            area: read_value::<u16>(stream, definition.get_entry(9894459526856489156u64), "area")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            9894459526856489156u64 => Ok(self.area.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointStartTouch",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointEndTouchEvent {
    pub player: u16,
    pub area: u16,
}
impl ControlPointEndTouchEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointEndTouchEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            area: read_value::<u16>(stream, definition.get_entry(9894459526856489156u64), "area")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            9894459526856489156u64 => Ok(self.area.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointEndTouch",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointPulseElementEvent {
    pub player: u16,
}
impl ControlPointPulseElementEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointPulseElementEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointPulseElement",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointFakeCaptureEvent {
    pub player: u16,
    pub int_data: u16,
}
impl ControlPointFakeCaptureEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointFakeCaptureEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            int_data: read_value::<u16>(
                stream,
                definition.get_entry(17655270944800390939u64),
                "int_data",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            17655270944800390939u64 => Ok(self.int_data.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointFakeCapture",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointFakeCaptureMultiplierEvent {
    pub player: u16,
    pub int_data: u16,
}
impl ControlPointFakeCaptureMultiplierEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointFakeCaptureMultiplierEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            int_data: read_value::<u16>(
                stream,
                definition.get_entry(17655270944800390939u64),
                "int_data",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            17655270944800390939u64 => Ok(self.int_data.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointFakeCaptureMultiplier",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundSelectedEvent {
    pub round: MaybeUtf8String,
}
impl TeamPlayRoundSelectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundSelectedEvent {
            round: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(3536478298987656219u64),
                "round",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3536478298987656219u64 => Ok(self.round.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundSelected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundStartEvent {
    pub full_reset: bool,
}
impl TeamPlayRoundStartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundStartEvent {
            full_reset: read_value::<bool>(
                stream,
                definition.get_entry(5520647792095461940u64),
                "full_reset",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5520647792095461940u64 => Ok(self.full_reset.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundStart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundActiveEvent {}
impl TeamPlayRoundActiveEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundActiveEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundActive",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayWaitingBeginsEvent {}
impl TeamPlayWaitingBeginsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayWaitingBeginsEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayWaitingBegins",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayWaitingEndsEvent {}
impl TeamPlayWaitingEndsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayWaitingEndsEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayWaitingEnds",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayWaitingAboutToEndEvent {}
impl TeamPlayWaitingAboutToEndEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayWaitingAboutToEndEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayWaitingAboutToEnd",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRestartRoundEvent {}
impl TeamPlayRestartRoundEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRestartRoundEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRestartRound",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayReadyRestartEvent {}
impl TeamPlayReadyRestartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayReadyRestartEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayReadyRestart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundRestartSecondsEvent {
    pub seconds: u16,
}
impl TeamPlayRoundRestartSecondsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundRestartSecondsEvent {
            seconds: read_value::<u16>(
                stream,
                definition.get_entry(11456985514702388746u64),
                "seconds",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            11456985514702388746u64 => Ok(self.seconds.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundRestartSeconds",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayTeamReadyEvent {
    pub team: u8,
}
impl TeamPlayTeamReadyEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayTeamReadyEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayTeamReady",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundWinEvent {
    pub team: u8,
    pub win_reason: u8,
    pub flag_cap_limit: u16,
    pub full_round: u16,
    pub round_time: f32,
    pub losing_team_num_caps: u16,
    pub was_sudden_death: u8,
}
impl TeamPlayRoundWinEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundWinEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            win_reason: read_value::<u8>(
                stream,
                definition.get_entry(3803421146477351589u64),
                "win_reason",
            )?,
            flag_cap_limit: read_value::<u16>(
                stream,
                definition.get_entry(8774624256288798788u64),
                "flag_cap_limit",
            )?,
            full_round: read_value::<u16>(
                stream,
                definition.get_entry(11360866888973275703u64),
                "full_round",
            )?,
            round_time: read_value::<f32>(
                stream,
                definition.get_entry(17889722153966279533u64),
                "round_time",
            )?,
            losing_team_num_caps: read_value::<u16>(
                stream,
                definition.get_entry(1136537408317314454u64),
                "losing_team_num_caps",
            )?,
            was_sudden_death: read_value::<u8>(
                stream,
                definition.get_entry(16618607837222165313u64),
                "was_sudden_death",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            3803421146477351589u64 => Ok(self.win_reason.clone().into()),
            8774624256288798788u64 => Ok(self.flag_cap_limit.clone().into()),
            11360866888973275703u64 => Ok(self.full_round.clone().into()),
            17889722153966279533u64 => Ok(self.round_time.clone().into()),
            1136537408317314454u64 => Ok(self.losing_team_num_caps.clone().into()),
            16618607837222165313u64 => Ok(self.was_sudden_death.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundWin",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayUpdateTimerEvent {}
impl TeamPlayUpdateTimerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayUpdateTimerEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayUpdateTimer",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayRoundStalemateEvent {
    pub reason: u8,
}
impl TeamPlayRoundStalemateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayRoundStalemateEvent {
            reason: read_value::<u8>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayRoundStalemate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayOvertimeBeginEvent {}
impl TeamPlayOvertimeBeginEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayOvertimeBeginEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayOvertimeBegin",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayOvertimeEndEvent {}
impl TeamPlayOvertimeEndEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayOvertimeEndEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayOvertimeEnd",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlaySuddenDeathBeginEvent {}
impl TeamPlaySuddenDeathBeginEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlaySuddenDeathBeginEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlaySuddenDeathBegin",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlaySuddenDeathEndEvent {}
impl TeamPlaySuddenDeathEndEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlaySuddenDeathEndEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlaySuddenDeathEnd",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayGameOverEvent {
    pub reason: MaybeUtf8String,
}
impl TeamPlayGameOverEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayGameOverEvent {
            reason: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(7343356632300987961u64),
                "reason",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7343356632300987961u64 => Ok(self.reason.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayGameOver",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayMapTimeRemainingEvent {
    pub seconds: u16,
}
impl TeamPlayMapTimeRemainingEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayMapTimeRemainingEvent {
            seconds: read_value::<u16>(
                stream,
                definition.get_entry(11456985514702388746u64),
                "seconds",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            11456985514702388746u64 => Ok(self.seconds.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayMapTimeRemaining",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayTimerFlashEvent {
    pub time_remaining: u16,
}
impl TeamPlayTimerFlashEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayTimerFlashEvent {
            time_remaining: read_value::<u16>(
                stream,
                definition.get_entry(4322130481848759231u64),
                "time_remaining",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            4322130481848759231u64 => Ok(self.time_remaining.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayTimerFlash",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayTimerTimeAddedEvent {
    pub timer: u16,
    pub seconds_added: u16,
}
impl TeamPlayTimerTimeAddedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayTimerTimeAddedEvent {
            timer: read_value::<u16>(
                stream,
                definition.get_entry(2968869876298967810u64),
                "timer",
            )?,
            seconds_added: read_value::<u16>(
                stream,
                definition.get_entry(5796598285704248091u64),
                "seconds_added",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2968869876298967810u64 => Ok(self.timer.clone().into()),
            5796598285704248091u64 => Ok(self.seconds_added.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayTimerTimeAdded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayPointStartCaptureEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub team: u8,
    pub cap_team: u8,
    pub cappers: MaybeUtf8String,
    pub cap_time: f32,
}
impl TeamPlayPointStartCaptureEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayPointStartCaptureEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            cap_team: read_value::<u8>(
                stream,
                definition.get_entry(9316766665943420626u64),
                "cap_team",
            )?,
            cappers: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11245576010855792123u64),
                "cappers",
            )?,
            cap_time: read_value::<f32>(
                stream,
                definition.get_entry(6747273962184890386u64),
                "cap_time",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            9316766665943420626u64 => Ok(self.cap_team.clone().into()),
            11245576010855792123u64 => Ok(self.cappers.clone().into()),
            6747273962184890386u64 => Ok(self.cap_time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayPointStartCapture",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayPointCapturedEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub team: u8,
    pub cappers: MaybeUtf8String,
}
impl TeamPlayPointCapturedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayPointCapturedEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            cappers: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11245576010855792123u64),
                "cappers",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            11245576010855792123u64 => Ok(self.cappers.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayPointCaptured",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayPointLockedEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub team: u8,
}
impl TeamPlayPointLockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayPointLockedEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayPointLocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayPointUnlockedEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub team: u8,
}
impl TeamPlayPointUnlockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayPointUnlockedEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayPointUnlocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayCaptureBrokenEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub time_remaining: f32,
}
impl TeamPlayCaptureBrokenEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayCaptureBrokenEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            time_remaining: read_value::<f32>(
                stream,
                definition.get_entry(4322130481848759231u64),
                "time_remaining",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            4322130481848759231u64 => Ok(self.time_remaining.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayCaptureBroken",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayCaptureBlockedEvent {
    pub cp: u8,
    pub cp_name: MaybeUtf8String,
    pub blocker: u8,
    pub victim: u8,
}
impl TeamPlayCaptureBlockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayCaptureBlockedEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            cp_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(5054899044858273797u64),
                "cp_name",
            )?,
            blocker: read_value::<u8>(
                stream,
                definition.get_entry(9150196623075249301u64),
                "blocker",
            )?,
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            5054899044858273797u64 => Ok(self.cp_name.clone().into()),
            9150196623075249301u64 => Ok(self.blocker.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayCaptureBlocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayFlagEventEvent {
    pub player: u16,
    pub carrier: u16,
    pub event_type: u16,
    pub home: u8,
    pub team: u8,
}
impl TeamPlayFlagEventEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayFlagEventEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            carrier: read_value::<u16>(
                stream,
                definition.get_entry(6986802915220291447u64),
                "carrier",
            )?,
            event_type: read_value::<u16>(
                stream,
                definition.get_entry(5234087001556401511u64),
                "event_type",
            )?,
            home: read_value::<u8>(stream, definition.get_entry(4624382957487889774u64), "home")?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            6986802915220291447u64 => Ok(self.carrier.clone().into()),
            5234087001556401511u64 => Ok(self.event_type.clone().into()),
            4624382957487889774u64 => Ok(self.home.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayFlagEvent",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayWinPanelEvent {
    pub panel_style: u8,
    pub winning_team: u8,
    pub win_reason: u8,
    pub cappers: MaybeUtf8String,
    pub flag_cap_limit: u16,
    pub blue_score: u16,
    pub red_score: u16,
    pub blue_score_prev: u16,
    pub red_score_prev: u16,
    pub round_complete: u16,
    pub rounds_remaining: u16,
    pub player_1: u16,
    pub player_1_points: u16,
    pub player_2: u16,
    pub player_2_points: u16,
    pub player_3: u16,
    pub player_3_points: u16,
    pub kill_stream_player_1: u16,
    pub kill_stream_player_1_count: u16,
    pub game_over: u8,
}
impl TeamPlayWinPanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayWinPanelEvent {
            panel_style: read_value::<u8>(
                stream,
                definition.get_entry(3076948889484157827u64),
                "panel_style",
            )?,
            winning_team: read_value::<u8>(
                stream,
                definition.get_entry(12760025138952247085u64),
                "winning_team",
            )?,
            win_reason: read_value::<u8>(
                stream,
                definition.get_entry(3803421146477351589u64),
                "win_reason",
            )?,
            cappers: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11245576010855792123u64),
                "cappers",
            )?,
            flag_cap_limit: read_value::<u16>(
                stream,
                definition.get_entry(8774624256288798788u64),
                "flag_cap_limit",
            )?,
            blue_score: read_value::<u16>(
                stream,
                definition.get_entry(12881173953286901124u64),
                "blue_score",
            )?,
            red_score: read_value::<u16>(
                stream,
                definition.get_entry(1115892277897790273u64),
                "red_score",
            )?,
            blue_score_prev: read_value::<u16>(
                stream,
                definition.get_entry(12382344057664863052u64),
                "blue_score_prev",
            )?,
            red_score_prev: read_value::<u16>(
                stream,
                definition.get_entry(9684982604781518527u64),
                "red_score_prev",
            )?,
            round_complete: read_value::<u16>(
                stream,
                definition.get_entry(12165785943437780003u64),
                "round_complete",
            )?,
            rounds_remaining: read_value::<u16>(
                stream,
                definition.get_entry(10434023640517397027u64),
                "rounds_remaining",
            )?,
            player_1: read_value::<u16>(
                stream,
                definition.get_entry(2316304829487618708u64),
                "player_1",
            )?,
            player_1_points: read_value::<u16>(
                stream,
                definition.get_entry(5201979123115161946u64),
                "player_1_points",
            )?,
            player_2: read_value::<u16>(
                stream,
                definition.get_entry(2316308128022503341u64),
                "player_2",
            )?,
            player_2_points: read_value::<u16>(
                stream,
                definition.get_entry(17826909355416759905u64),
                "player_2_points",
            )?,
            player_3: read_value::<u16>(
                stream,
                definition.get_entry(2316307028510875130u64),
                "player_3",
            )?,
            player_3_points: read_value::<u16>(
                stream,
                definition.get_entry(3496125461192453068u64),
                "player_3_points",
            )?,
            kill_stream_player_1: read_value::<u16>(
                stream,
                definition.get_entry(5168633764732789397u64),
                "kill_stream_player_1",
            )?,
            kill_stream_player_1_count: read_value::<u16>(
                stream,
                definition.get_entry(15596392604614003293u64),
                "kill_stream_player_1_count",
            )?,
            game_over: read_value::<u8>(
                stream,
                definition.get_entry(17040732377939006530u64),
                "game_over",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3076948889484157827u64 => Ok(self.panel_style.clone().into()),
            12760025138952247085u64 => Ok(self.winning_team.clone().into()),
            3803421146477351589u64 => Ok(self.win_reason.clone().into()),
            11245576010855792123u64 => Ok(self.cappers.clone().into()),
            8774624256288798788u64 => Ok(self.flag_cap_limit.clone().into()),
            12881173953286901124u64 => Ok(self.blue_score.clone().into()),
            1115892277897790273u64 => Ok(self.red_score.clone().into()),
            12382344057664863052u64 => Ok(self.blue_score_prev.clone().into()),
            9684982604781518527u64 => Ok(self.red_score_prev.clone().into()),
            12165785943437780003u64 => Ok(self.round_complete.clone().into()),
            10434023640517397027u64 => Ok(self.rounds_remaining.clone().into()),
            2316304829487618708u64 => Ok(self.player_1.clone().into()),
            5201979123115161946u64 => Ok(self.player_1_points.clone().into()),
            2316308128022503341u64 => Ok(self.player_2.clone().into()),
            17826909355416759905u64 => Ok(self.player_2_points.clone().into()),
            2316307028510875130u64 => Ok(self.player_3.clone().into()),
            3496125461192453068u64 => Ok(self.player_3_points.clone().into()),
            5168633764732789397u64 => Ok(self.kill_stream_player_1.clone().into()),
            15596392604614003293u64 => Ok(self.kill_stream_player_1_count.clone().into()),
            17040732377939006530u64 => Ok(self.game_over.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayWinPanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayTeamBalancedPlayerEvent {
    pub player: u16,
    pub team: u8,
}
impl TeamPlayTeamBalancedPlayerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayTeamBalancedPlayerEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayTeamBalancedPlayer",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlaySetupFinishedEvent {}
impl TeamPlaySetupFinishedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlaySetupFinishedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlaySetupFinished",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayAlertEvent {
    pub alert_type: u16,
}
impl TeamPlayAlertEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayAlertEvent {
            alert_type: read_value::<u16>(
                stream,
                definition.get_entry(5455556148004663490u64),
                "alert_type",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5455556148004663490u64 => Ok(self.alert_type.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayAlert",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TrainingCompleteEvent {
    pub next_map: MaybeUtf8String,
    pub map: MaybeUtf8String,
    pub text: MaybeUtf8String,
}
impl TrainingCompleteEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TrainingCompleteEvent {
            next_map: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(13669032538247983969u64),
                "next_map",
            )?,
            map: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(580780841256168849u64),
                "map",
            )?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            13669032538247983969u64 => Ok(self.next_map.clone().into()),
            580780841256168849u64 => Ok(self.map.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TrainingComplete",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShowFreezePanelEvent {
    pub killer: u16,
}
impl ShowFreezePanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ShowFreezePanelEvent {
            killer: read_value::<u16>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ShowFreezePanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HideFreezePanelEvent {}
impl HideFreezePanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HideFreezePanelEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HideFreezePanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FreezeCamStartedEvent {}
impl FreezeCamStartedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FreezeCamStartedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FreezeCamStarted",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerChangeTeamEvent {}
impl LocalPlayerChangeTeamEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerChangeTeamEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerChangeTeam",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerScoreChangedEvent {
    pub score: u16,
}
impl LocalPlayerScoreChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerScoreChangedEvent {
            score: read_value::<u16>(
                stream,
                definition.get_entry(13911166232573650165u64),
                "score",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            13911166232573650165u64 => Ok(self.score.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerScoreChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerChangeClassEvent {}
impl LocalPlayerChangeClassEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerChangeClassEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerChangeClass",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerRespawnEvent {}
impl LocalPlayerRespawnEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerRespawnEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerRespawn",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BuildingInfoChangedEvent {
    pub building_type: u8,
    pub object_mode: u8,
    pub remove: u8,
}
impl BuildingInfoChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BuildingInfoChangedEvent {
            building_type: read_value::<u8>(
                stream,
                definition.get_entry(11928805672381350942u64),
                "building_type",
            )?,
            object_mode: read_value::<u8>(
                stream,
                definition.get_entry(10575483099853176920u64),
                "object_mode",
            )?,
            remove: read_value::<u8>(
                stream,
                definition.get_entry(18444559702367749501u64),
                "remove",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            11928805672381350942u64 => Ok(self.building_type.clone().into()),
            10575483099853176920u64 => Ok(self.object_mode.clone().into()),
            18444559702367749501u64 => Ok(self.remove.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BuildingInfoChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerChangeDisguiseEvent {
    pub disguised: bool,
}
impl LocalPlayerChangeDisguiseEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerChangeDisguiseEvent {
            disguised: read_value::<bool>(
                stream,
                definition.get_entry(12536453236572645170u64),
                "disguised",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12536453236572645170u64 => Ok(self.disguised.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerChangeDisguise",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerAccountChangedEvent {
    pub old_value: u16,
    pub new_value: u16,
}
impl PlayerAccountChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerAccountChangedEvent {
            old_value: read_value::<u16>(
                stream,
                definition.get_entry(12791125728741340200u64),
                "old_value",
            )?,
            new_value: read_value::<u16>(
                stream,
                definition.get_entry(4557144879184858675u64),
                "new_value",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12791125728741340200u64 => Ok(self.old_value.clone().into()),
            4557144879184858675u64 => Ok(self.new_value.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerAccountChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SpyPdaResetEvent {}
impl SpyPdaResetEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SpyPdaResetEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SpyPdaReset",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FlagStatusUpdateEvent {
    pub user_id: u16,
    pub ent_index: u32,
}
impl FlagStatusUpdateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FlagStatusUpdateEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            ent_index: read_value::<u32>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FlagStatusUpdate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerStatsUpdatedEvent {
    pub force_upload: bool,
}
impl PlayerStatsUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerStatsUpdatedEvent {
            force_upload: read_value::<bool>(
                stream,
                definition.get_entry(9059172244575932677u64),
                "force_upload",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9059172244575932677u64 => Ok(self.force_upload.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerStatsUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayingCommentaryEvent {}
impl PlayingCommentaryEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayingCommentaryEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayingCommentary",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerChargeDeployedEvent {
    pub user_id: u16,
    pub target_id: u16,
}
impl PlayerChargeDeployedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerChargeDeployedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            target_id: read_value::<u16>(
                stream,
                definition.get_entry(10554794794880602069u64),
                "target_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10554794794880602069u64 => Ok(self.target_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerChargeDeployed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerBuiltObjectEvent {
    pub user_id: u16,
    pub object: u16,
    pub index: u16,
}
impl PlayerBuiltObjectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerBuiltObjectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object: read_value::<u16>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerBuiltObject",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerUpgradedObjectEvent {
    pub user_id: u16,
    pub object: u16,
    pub index: u16,
    pub is_builder: bool,
}
impl PlayerUpgradedObjectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerUpgradedObjectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object: read_value::<u16>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            is_builder: read_value::<bool>(
                stream,
                definition.get_entry(16922823115528993136u64),
                "is_builder",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            16922823115528993136u64 => Ok(self.is_builder.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerUpgradedObject",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerCarryObjectEvent {
    pub user_id: u16,
    pub object: u16,
    pub index: u16,
}
impl PlayerCarryObjectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerCarryObjectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object: read_value::<u16>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerCarryObject",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDropObjectEvent {
    pub user_id: u16,
    pub object: u16,
    pub index: u16,
}
impl PlayerDropObjectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDropObjectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object: read_value::<u16>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDropObject",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ObjectRemovedEvent {
    pub user_id: u16,
    pub object_type: u16,
    pub index: u16,
}
impl ObjectRemovedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ObjectRemovedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object_type: read_value::<u16>(
                stream,
                definition.get_entry(6025769100165056826u64),
                "object_type",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            6025769100165056826u64 => Ok(self.object_type.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ObjectRemoved",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ObjectDestroyedEvent {
    pub user_id: u16,
    pub attacker: u16,
    pub assister: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub object_type: u16,
    pub index: u16,
    pub was_building: bool,
}
impl ObjectDestroyedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ObjectDestroyedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            object_type: read_value::<u16>(
                stream,
                definition.get_entry(6025769100165056826u64),
                "object_type",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            was_building: read_value::<bool>(
                stream,
                definition.get_entry(13090762770129151523u64),
                "was_building",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            6025769100165056826u64 => Ok(self.object_type.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            13090762770129151523u64 => Ok(self.was_building.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ObjectDestroyed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ObjectDetonatedEvent {
    pub user_id: u16,
    pub object_type: u16,
    pub index: u16,
}
impl ObjectDetonatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ObjectDetonatedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            object_type: read_value::<u16>(
                stream,
                definition.get_entry(6025769100165056826u64),
                "object_type",
            )?,
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            6025769100165056826u64 => Ok(self.object_type.clone().into()),
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ObjectDetonated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AchievementEarnedEvent {
    pub player: u8,
    pub achievement: u16,
}
impl AchievementEarnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(AchievementEarnedEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            achievement: read_value::<u16>(
                stream,
                definition.get_entry(7071905471600864408u64),
                "achievement",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            7071905471600864408u64 => Ok(self.achievement.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "AchievementEarned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SpecTargetUpdatedEvent {}
impl SpecTargetUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SpecTargetUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SpecTargetUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TournamentStateUpdateEvent {
    pub user_id: u16,
    pub name_change: bool,
    pub ready_state: u16,
    pub new_name: MaybeUtf8String,
}
impl TournamentStateUpdateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TournamentStateUpdateEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            name_change: read_value::<bool>(
                stream,
                definition.get_entry(2507746477842667538u64),
                "name_change",
            )?,
            ready_state: read_value::<u16>(
                stream,
                definition.get_entry(14125289189230288425u64),
                "ready_state",
            )?,
            new_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8904377156710117674u64),
                "new_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2507746477842667538u64 => Ok(self.name_change.clone().into()),
            14125289189230288425u64 => Ok(self.ready_state.clone().into()),
            8904377156710117674u64 => Ok(self.new_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TournamentStateUpdate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TournamentEnableCountdownEvent {}
impl TournamentEnableCountdownEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TournamentEnableCountdownEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TournamentEnableCountdown",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerCalledForMedicEvent {
    pub user_id: u16,
}
impl PlayerCalledForMedicEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerCalledForMedicEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerCalledForMedic",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerAskedForBallEvent {
    pub user_id: u16,
}
impl PlayerAskedForBallEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerAskedForBallEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerAskedForBall",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerBecameObserverEvent {}
impl LocalPlayerBecameObserverEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerBecameObserverEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerBecameObserver",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerIgnitedInvEvent {
    pub pyro_ent_index: u8,
    pub victim_ent_index: u8,
    pub medic_ent_index: u8,
}
impl PlayerIgnitedInvEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerIgnitedInvEvent {
            pyro_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(2481028106190820025u64),
                "pyro_ent_index",
            )?,
            victim_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            medic_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(9117426914612648485u64),
                "medic_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2481028106190820025u64 => Ok(self.pyro_ent_index.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            9117426914612648485u64 => Ok(self.medic_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerIgnitedInv",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerIgnitedEvent {
    pub pyro_ent_index: u8,
    pub victim_ent_index: u8,
    pub weapon_id: u8,
}
impl PlayerIgnitedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerIgnitedEvent {
            pyro_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(2481028106190820025u64),
                "pyro_ent_index",
            )?,
            victim_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            weapon_id: read_value::<u8>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2481028106190820025u64 => Ok(self.pyro_ent_index.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerIgnited",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerExtinguishedEvent {
    pub victim: u8,
    pub healer: u8,
    pub item_definition_index: u16,
}
impl PlayerExtinguishedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerExtinguishedEvent {
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            healer: read_value::<u8>(
                stream,
                definition.get_entry(9195440821534910520u64),
                "healer",
            )?,
            item_definition_index: read_value::<u16>(
                stream,
                definition.get_entry(4926523576391011283u64),
                "item_definition_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            9195440821534910520u64 => Ok(self.healer.clone().into()),
            4926523576391011283u64 => Ok(self.item_definition_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerExtinguished",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerTeleportedEvent {
    pub user_id: u16,
    pub builder_id: u16,
    pub dist: f32,
}
impl PlayerTeleportedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerTeleportedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            builder_id: read_value::<u16>(
                stream,
                definition.get_entry(3387979893847309533u64),
                "builder_id",
            )?,
            dist: read_value::<f32>(
                stream,
                definition.get_entry(14574961654905149033u64),
                "dist",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            3387979893847309533u64 => Ok(self.builder_id.clone().into()),
            14574961654905149033u64 => Ok(self.dist.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerTeleported",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHealedMedicCallEvent {
    pub user_id: u16,
}
impl PlayerHealedMedicCallEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHealedMedicCallEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHealedMedicCall",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerChargeReadyEvent {}
impl LocalPlayerChargeReadyEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerChargeReadyEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerChargeReady",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerWindDownEvent {}
impl LocalPlayerWindDownEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerWindDownEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerWindDown",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerInvulnedEvent {
    pub user_id: u16,
    pub medic_user_id: u16,
}
impl PlayerInvulnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerInvulnedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            medic_user_id: read_value::<u16>(
                stream,
                definition.get_entry(1211611822706104928u64),
                "medic_user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            1211611822706104928u64 => Ok(self.medic_user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerInvulned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EscortSpeedEvent {
    pub team: u8,
    pub speed: u8,
    pub players: u8,
}
impl EscortSpeedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EscortSpeedEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            speed: read_value::<u8>(
                stream,
                definition.get_entry(2486349329025994304u64),
                "speed",
            )?,
            players: read_value::<u8>(
                stream,
                definition.get_entry(11016838732397775657u64),
                "players",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            2486349329025994304u64 => Ok(self.speed.clone().into()),
            11016838732397775657u64 => Ok(self.players.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EscortSpeed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EscortProgressEvent {
    pub team: u8,
    pub progress: f32,
    pub reset: bool,
}
impl EscortProgressEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EscortProgressEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            progress: read_value::<f32>(
                stream,
                definition.get_entry(17983033035584588230u64),
                "progress",
            )?,
            reset: read_value::<bool>(
                stream,
                definition.get_entry(1086335023529244512u64),
                "reset",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            17983033035584588230u64 => Ok(self.progress.clone().into()),
            1086335023529244512u64 => Ok(self.reset.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EscortProgress",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EscortRecedeEvent {
    pub team: u8,
    pub recede_time: f32,
}
impl EscortRecedeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EscortRecedeEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            recede_time: read_value::<f32>(
                stream,
                definition.get_entry(12986815124312535790u64),
                "recede_time",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            12986815124312535790u64 => Ok(self.recede_time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EscortRecede",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameUIActivatedEvent {}
impl GameUIActivatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameUIActivatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameUIActivated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GameUIHiddenEvent {}
impl GameUIHiddenEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GameUIHiddenEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GameUIHidden",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerEscortScoreEvent {
    pub player: u8,
    pub points: u8,
}
impl PlayerEscortScoreEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerEscortScoreEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            points: read_value::<u8>(
                stream,
                definition.get_entry(15925482014108518566u64),
                "points",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            15925482014108518566u64 => Ok(self.points.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerEscortScore",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHealOnHitEvent {
    pub amount: u16,
    pub ent_index: u8,
    pub weapon_def_index: u32,
}
impl PlayerHealOnHitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHealOnHitEvent {
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
            ent_index: read_value::<u8>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            weapon_def_index: read_value::<u32>(
                stream,
                definition.get_entry(4132306594868589054u64),
                "weapon_def_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            4132306594868589054u64 => Ok(self.weapon_def_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHealOnHit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerStealSandvichEvent {
    pub owner: u16,
    pub target: u16,
}
impl PlayerStealSandvichEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerStealSandvichEvent {
            owner: read_value::<u16>(
                stream,
                definition.get_entry(12002927925776846068u64),
                "owner",
            )?,
            target: read_value::<u16>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12002927925776846068u64 => Ok(self.owner.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerStealSandvich",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShowClassLayoutEvent {
    pub show: bool,
}
impl ShowClassLayoutEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ShowClassLayoutEvent {
            show: read_value::<bool>(stream, definition.get_entry(5106060638016120252u64), "show")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5106060638016120252u64 => Ok(self.show.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ShowClassLayout",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShowVsPanelEvent {
    pub show: bool,
}
impl ShowVsPanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ShowVsPanelEvent {
            show: read_value::<bool>(stream, definition.get_entry(5106060638016120252u64), "show")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5106060638016120252u64 => Ok(self.show.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ShowVsPanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDamagedEvent {
    pub amount: u16,
    pub kind: u32,
}
impl PlayerDamagedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDamagedEvent {
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
            kind: read_value::<u32>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDamaged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArenaPlayerNotificationEvent {
    pub player: u8,
    pub message: u8,
}
impl ArenaPlayerNotificationEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ArenaPlayerNotificationEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            message: read_value::<u8>(
                stream,
                definition.get_entry(6080987277291999908u64),
                "message",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            6080987277291999908u64 => Ok(self.message.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ArenaPlayerNotification",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArenaMatchMaxStreakEvent {
    pub team: u8,
    pub streak: u8,
}
impl ArenaMatchMaxStreakEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ArenaMatchMaxStreakEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            streak: read_value::<u8>(
                stream,
                definition.get_entry(5722439984700485459u64),
                "streak",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            5722439984700485459u64 => Ok(self.streak.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ArenaMatchMaxStreak",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArenaRoundStartEvent {}
impl ArenaRoundStartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ArenaRoundStartEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ArenaRoundStart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArenaWinPanelEvent {
    pub panel_style: u8,
    pub winning_team: u8,
    pub win_reason: u8,
    pub cappers: MaybeUtf8String,
    pub flag_cap_limit: u16,
    pub blue_score: u16,
    pub red_score: u16,
    pub blue_score_prev: u16,
    pub red_score_prev: u16,
    pub round_complete: u16,
    pub player_1: u16,
    pub player_1_damage: u16,
    pub player_1_healing: u16,
    pub player_1_lifetime: u16,
    pub player_1_kills: u16,
    pub player_2: u16,
    pub player_2_damage: u16,
    pub player_2_healing: u16,
    pub player_2_lifetime: u16,
    pub player_2_kills: u16,
    pub player_3: u16,
    pub player_3_damage: u16,
    pub player_3_healing: u16,
    pub player_3_lifetime: u16,
    pub player_3_kills: u16,
    pub player_4: u16,
    pub player_4_damage: u16,
    pub player_4_healing: u16,
    pub player_4_lifetime: u16,
    pub player_4_kills: u16,
    pub player_5: u16,
    pub player_5_damage: u16,
    pub player_5_healing: u16,
    pub player_5_lifetime: u16,
    pub player_5_kills: u16,
    pub player_6: u16,
    pub player_6_damage: u16,
    pub player_6_healing: u16,
    pub player_6_lifetime: u16,
    pub player_6_kills: u16,
}
impl ArenaWinPanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ArenaWinPanelEvent {
            panel_style: read_value::<u8>(
                stream,
                definition.get_entry(3076948889484157827u64),
                "panel_style",
            )?,
            winning_team: read_value::<u8>(
                stream,
                definition.get_entry(12760025138952247085u64),
                "winning_team",
            )?,
            win_reason: read_value::<u8>(
                stream,
                definition.get_entry(3803421146477351589u64),
                "win_reason",
            )?,
            cappers: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11245576010855792123u64),
                "cappers",
            )?,
            flag_cap_limit: read_value::<u16>(
                stream,
                definition.get_entry(8774624256288798788u64),
                "flag_cap_limit",
            )?,
            blue_score: read_value::<u16>(
                stream,
                definition.get_entry(12881173953286901124u64),
                "blue_score",
            )?,
            red_score: read_value::<u16>(
                stream,
                definition.get_entry(1115892277897790273u64),
                "red_score",
            )?,
            blue_score_prev: read_value::<u16>(
                stream,
                definition.get_entry(12382344057664863052u64),
                "blue_score_prev",
            )?,
            red_score_prev: read_value::<u16>(
                stream,
                definition.get_entry(9684982604781518527u64),
                "red_score_prev",
            )?,
            round_complete: read_value::<u16>(
                stream,
                definition.get_entry(12165785943437780003u64),
                "round_complete",
            )?,
            player_1: read_value::<u16>(
                stream,
                definition.get_entry(2316304829487618708u64),
                "player_1",
            )?,
            player_1_damage: read_value::<u16>(
                stream,
                definition.get_entry(10394500961929970236u64),
                "player_1_damage",
            )?,
            player_1_healing: read_value::<u16>(
                stream,
                definition.get_entry(4434975577362185857u64),
                "player_1_healing",
            )?,
            player_1_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(9525810424766332458u64),
                "player_1_lifetime",
            )?,
            player_1_kills: read_value::<u16>(
                stream,
                definition.get_entry(9144050188623277698u64),
                "player_1_kills",
            )?,
            player_2: read_value::<u16>(
                stream,
                definition.get_entry(2316308128022503341u64),
                "player_2",
            )?,
            player_2_damage: read_value::<u16>(
                stream,
                definition.get_entry(10233858120128677491u64),
                "player_2_damage",
            )?,
            player_2_healing: read_value::<u16>(
                stream,
                definition.get_entry(14185483197478656496u64),
                "player_2_healing",
            )?,
            player_2_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(4136489886055437333u64),
                "player_2_lifetime",
            )?,
            player_2_kills: read_value::<u16>(
                stream,
                definition.get_entry(4674312054664562187u64),
                "player_2_kills",
            )?,
            player_3: read_value::<u16>(
                stream,
                definition.get_entry(2316307028510875130u64),
                "player_3",
            )?,
            player_3_damage: read_value::<u16>(
                stream,
                definition.get_entry(39363359054721358u64),
                "player_3_damage",
            )?,
            player_3_healing: read_value::<u16>(
                stream,
                definition.get_entry(3236287490998805827u64),
                "player_3_healing",
            )?,
            player_3_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(14621494996524927732u64),
                "player_3_lifetime",
            )?,
            player_3_kills: read_value::<u16>(
                stream,
                definition.get_entry(3863298646261365396u64),
                "player_3_kills",
            )?,
            player_4: read_value::<u16>(
                stream,
                definition.get_entry(2316301530952734075u64),
                "player_4",
            )?,
            player_4_damage: read_value::<u16>(
                stream,
                definition.get_entry(10597470269304895533u64),
                "player_4_damage",
            )?,
            player_4_healing: read_value::<u16>(
                stream,
                definition.get_entry(16447535156948377850u64),
                "player_4_healing",
            )?,
            player_4_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(7059298629593792111u64),
                "player_4_lifetime",
            )?,
            player_4_kills: read_value::<u16>(
                stream,
                definition.get_entry(3991910781166784861u64),
                "player_4_kills",
            )?,
            player_5: read_value::<u16>(
                stream,
                definition.get_entry(2316300431441105864u64),
                "player_5",
            )?,
            player_5_damage: read_value::<u16>(
                stream,
                definition.get_entry(5792678822030670832u64),
                "player_5_damage",
            )?,
            player_5_healing: read_value::<u16>(
                stream,
                definition.get_entry(10549804756926849709u64),
                "player_5_healing",
            )?,
            player_5_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(5937207214365195550u64),
                "player_5_lifetime",
            )?,
            player_5_kills: read_value::<u16>(
                stream,
                definition.get_entry(12801315132847255054u64),
                "player_5_kills",
            )?,
            player_6: read_value::<u16>(
                stream,
                definition.get_entry(2316303729975990497u64),
                "player_6",
            )?,
            player_6_damage: read_value::<u16>(
                stream,
                definition.get_entry(11907089275040254807u64),
                "player_6_damage",
            )?,
            player_6_healing: read_value::<u16>(
                stream,
                definition.get_entry(12917045825324352380u64),
                "player_6_healing",
            )?,
            player_6_lifetime: read_value::<u16>(
                stream,
                definition.get_entry(2459117320880115529u64),
                "player_6_lifetime",
            )?,
            player_6_kills: read_value::<u16>(
                stream,
                definition.get_entry(9062282318485088775u64),
                "player_6_kills",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3076948889484157827u64 => Ok(self.panel_style.clone().into()),
            12760025138952247085u64 => Ok(self.winning_team.clone().into()),
            3803421146477351589u64 => Ok(self.win_reason.clone().into()),
            11245576010855792123u64 => Ok(self.cappers.clone().into()),
            8774624256288798788u64 => Ok(self.flag_cap_limit.clone().into()),
            12881173953286901124u64 => Ok(self.blue_score.clone().into()),
            1115892277897790273u64 => Ok(self.red_score.clone().into()),
            12382344057664863052u64 => Ok(self.blue_score_prev.clone().into()),
            9684982604781518527u64 => Ok(self.red_score_prev.clone().into()),
            12165785943437780003u64 => Ok(self.round_complete.clone().into()),
            2316304829487618708u64 => Ok(self.player_1.clone().into()),
            10394500961929970236u64 => Ok(self.player_1_damage.clone().into()),
            4434975577362185857u64 => Ok(self.player_1_healing.clone().into()),
            9525810424766332458u64 => Ok(self.player_1_lifetime.clone().into()),
            9144050188623277698u64 => Ok(self.player_1_kills.clone().into()),
            2316308128022503341u64 => Ok(self.player_2.clone().into()),
            10233858120128677491u64 => Ok(self.player_2_damage.clone().into()),
            14185483197478656496u64 => Ok(self.player_2_healing.clone().into()),
            4136489886055437333u64 => Ok(self.player_2_lifetime.clone().into()),
            4674312054664562187u64 => Ok(self.player_2_kills.clone().into()),
            2316307028510875130u64 => Ok(self.player_3.clone().into()),
            39363359054721358u64 => Ok(self.player_3_damage.clone().into()),
            3236287490998805827u64 => Ok(self.player_3_healing.clone().into()),
            14621494996524927732u64 => Ok(self.player_3_lifetime.clone().into()),
            3863298646261365396u64 => Ok(self.player_3_kills.clone().into()),
            2316301530952734075u64 => Ok(self.player_4.clone().into()),
            10597470269304895533u64 => Ok(self.player_4_damage.clone().into()),
            16447535156948377850u64 => Ok(self.player_4_healing.clone().into()),
            7059298629593792111u64 => Ok(self.player_4_lifetime.clone().into()),
            3991910781166784861u64 => Ok(self.player_4_kills.clone().into()),
            2316300431441105864u64 => Ok(self.player_5.clone().into()),
            5792678822030670832u64 => Ok(self.player_5_damage.clone().into()),
            10549804756926849709u64 => Ok(self.player_5_healing.clone().into()),
            5937207214365195550u64 => Ok(self.player_5_lifetime.clone().into()),
            12801315132847255054u64 => Ok(self.player_5_kills.clone().into()),
            2316303729975990497u64 => Ok(self.player_6.clone().into()),
            11907089275040254807u64 => Ok(self.player_6_damage.clone().into()),
            12917045825324352380u64 => Ok(self.player_6_healing.clone().into()),
            2459117320880115529u64 => Ok(self.player_6_lifetime.clone().into()),
            9062282318485088775u64 => Ok(self.player_6_kills.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ArenaWinPanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PveWinPanelEvent {
    pub panel_style: u8,
    pub winning_team: u8,
    pub win_reason: u8,
}
impl PveWinPanelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PveWinPanelEvent {
            panel_style: read_value::<u8>(
                stream,
                definition.get_entry(3076948889484157827u64),
                "panel_style",
            )?,
            winning_team: read_value::<u8>(
                stream,
                definition.get_entry(12760025138952247085u64),
                "winning_team",
            )?,
            win_reason: read_value::<u8>(
                stream,
                definition.get_entry(3803421146477351589u64),
                "win_reason",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3076948889484157827u64 => Ok(self.panel_style.clone().into()),
            12760025138952247085u64 => Ok(self.winning_team.clone().into()),
            3803421146477351589u64 => Ok(self.win_reason.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PveWinPanel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AirDashEvent {
    pub player: u8,
}
impl AirDashEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(AirDashEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "AirDash",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LandedEvent {
    pub player: u8,
}
impl LandedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LandedEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "Landed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDamageDodgedEvent {
    pub damage: u16,
}
impl PlayerDamageDodgedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDamageDodgedEvent {
            damage: read_value::<u16>(
                stream,
                definition.get_entry(9179190079975030720u64),
                "damage",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9179190079975030720u64 => Ok(self.damage.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDamageDodged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerStunnedEvent {
    pub stunner: u16,
    pub victim: u16,
    pub victim_capping: bool,
    pub big_stun: bool,
}
impl PlayerStunnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerStunnedEvent {
            stunner: read_value::<u16>(
                stream,
                definition.get_entry(14931815241283043822u64),
                "stunner",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            victim_capping: read_value::<bool>(
                stream,
                definition.get_entry(6103089581884104798u64),
                "victim_capping",
            )?,
            big_stun: read_value::<bool>(
                stream,
                definition.get_entry(2754291295915618874u64),
                "big_stun",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14931815241283043822u64 => Ok(self.stunner.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            6103089581884104798u64 => Ok(self.victim_capping.clone().into()),
            2754291295915618874u64 => Ok(self.big_stun.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerStunned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoutGrandSlamEvent {
    pub scout_id: u16,
    pub target_id: u16,
}
impl ScoutGrandSlamEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ScoutGrandSlamEvent {
            scout_id: read_value::<u16>(
                stream,
                definition.get_entry(7064780806045746163u64),
                "scout_id",
            )?,
            target_id: read_value::<u16>(
                stream,
                definition.get_entry(13239627154349772880u64),
                "target_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7064780806045746163u64 => Ok(self.scout_id.clone().into()),
            13239627154349772880u64 => Ok(self.target_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ScoutGrandSlam",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoutSlamdollLandedEvent {
    pub target_index: u16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl ScoutSlamdollLandedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ScoutSlamdollLandedEvent {
            target_index: read_value::<u16>(
                stream,
                definition.get_entry(654399427416389339u64),
                "target_index",
            )?,
            x: read_value::<f32>(stream, definition.get_entry(12638214688346347271u64), "x")?,
            y: read_value::<f32>(stream, definition.get_entry(12638213588834719060u64), "y")?,
            z: read_value::<f32>(stream, definition.get_entry(12638216887369603693u64), "z")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            654399427416389339u64 => Ok(self.target_index.clone().into()),
            12638214688346347271u64 => Ok(self.x.clone().into()),
            12638213588834719060u64 => Ok(self.y.clone().into()),
            12638216887369603693u64 => Ok(self.z.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ScoutSlamdollLanded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ArrowImpactEvent {
    pub attached_entity: u16,
    pub shooter: u16,
    pub bone_index_attached: u16,
    pub bone_position_x: f32,
    pub bone_position_y: f32,
    pub bone_position_z: f32,
    pub bone_angles_x: f32,
    pub bone_angles_y: f32,
    pub bone_angles_z: f32,
    pub projectile_type: u16,
    pub is_crit: bool,
}
impl ArrowImpactEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ArrowImpactEvent {
            attached_entity: read_value::<u16>(
                stream,
                definition.get_entry(961881214683019338u64),
                "attached_entity",
            )?,
            shooter: read_value::<u16>(
                stream,
                definition.get_entry(13826298252589476871u64),
                "shooter",
            )?,
            bone_index_attached: read_value::<u16>(
                stream,
                definition.get_entry(6360700909751330309u64),
                "bone_index_attached",
            )?,
            bone_position_x: read_value::<f32>(
                stream,
                definition.get_entry(10378941398473653372u64),
                "bone_position_x",
            )?,
            bone_position_y: read_value::<f32>(
                stream,
                definition.get_entry(10378942497985281583u64),
                "bone_position_y",
            )?,
            bone_position_z: read_value::<f32>(
                stream,
                definition.get_entry(10378943597496909794u64),
                "bone_position_z",
            )?,
            bone_angles_x: read_value::<f32>(
                stream,
                definition.get_entry(5043211860521425589u64),
                "bone_angles_x",
            )?,
            bone_angles_y: read_value::<f32>(
                stream,
                definition.get_entry(5043210761009797378u64),
                "bone_angles_y",
            )?,
            bone_angles_z: read_value::<f32>(
                stream,
                definition.get_entry(5043209661498169167u64),
                "bone_angles_z",
            )?,
            projectile_type: read_value::<u16>(
                stream,
                definition.get_entry(9968460680690579194u64),
                "projectile_type",
            )?,
            is_crit: read_value::<bool>(
                stream,
                definition.get_entry(15363801587511353021u64),
                "is_crit",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            961881214683019338u64 => Ok(self.attached_entity.clone().into()),
            13826298252589476871u64 => Ok(self.shooter.clone().into()),
            6360700909751330309u64 => Ok(self.bone_index_attached.clone().into()),
            10378941398473653372u64 => Ok(self.bone_position_x.clone().into()),
            10378942497985281583u64 => Ok(self.bone_position_y.clone().into()),
            10378943597496909794u64 => Ok(self.bone_position_z.clone().into()),
            5043211860521425589u64 => Ok(self.bone_angles_x.clone().into()),
            5043210761009797378u64 => Ok(self.bone_angles_y.clone().into()),
            5043209661498169167u64 => Ok(self.bone_angles_z.clone().into()),
            9968460680690579194u64 => Ok(self.projectile_type.clone().into()),
            15363801587511353021u64 => Ok(self.is_crit.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ArrowImpact",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerJaratedEvent {
    pub thrower_ent_index: u8,
    pub victim_ent_index: u8,
}
impl PlayerJaratedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerJaratedEvent {
            thrower_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7733252424873930810u64),
                "thrower_ent_index",
            )?,
            victim_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7733252424873930810u64 => Ok(self.thrower_ent_index.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerJarated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerJaratedFadeEvent {
    pub thrower_ent_index: u8,
    pub victim_ent_index: u8,
}
impl PlayerJaratedFadeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerJaratedFadeEvent {
            thrower_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7733252424873930810u64),
                "thrower_ent_index",
            )?,
            victim_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7733252424873930810u64 => Ok(self.thrower_ent_index.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerJaratedFade",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerShieldBlockedEvent {
    pub attacker_ent_index: u8,
    pub blocker_ent_index: u8,
}
impl PlayerShieldBlockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerShieldBlockedEvent {
            attacker_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(1824579233645723292u64),
                "attacker_ent_index",
            )?,
            blocker_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(1554237712520490433u64),
                "blocker_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1824579233645723292u64 => Ok(self.attacker_ent_index.clone().into()),
            1554237712520490433u64 => Ok(self.blocker_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerShieldBlocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerPinnedEvent {
    pub pinned: u8,
}
impl PlayerPinnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerPinnedEvent {
            pinned: read_value::<u8>(
                stream,
                definition.get_entry(6882141757131022863u64),
                "pinned",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6882141757131022863u64 => Ok(self.pinned.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerPinned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHealedByMedicEvent {
    pub medic: u8,
}
impl PlayerHealedByMedicEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHealedByMedicEvent {
            medic: read_value::<u8>(
                stream,
                definition.get_entry(12912869923554243305u64),
                "medic",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12912869923554243305u64 => Ok(self.medic.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHealedByMedic",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerSappedObjectEvent {
    pub user_id: u16,
    pub owner_id: u16,
    pub object: u8,
    pub sapper_id: u16,
}
impl PlayerSappedObjectEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerSappedObjectEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            owner_id: read_value::<u16>(
                stream,
                definition.get_entry(3274630577613078265u64),
                "owner_id",
            )?,
            object: read_value::<u8>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
            sapper_id: read_value::<u16>(
                stream,
                definition.get_entry(14334448895032880407u64),
                "sapper_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            3274630577613078265u64 => Ok(self.owner_id.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            14334448895032880407u64 => Ok(self.sapper_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerSappedObject",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ItemFoundEvent {
    pub player: u8,
    pub quality: u8,
    pub method: u8,
    pub item_def: u32,
    pub is_strange: u8,
    pub is_unusual: u8,
    pub wear: f32,
}
impl ItemFoundEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ItemFoundEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            quality: read_value::<u8>(
                stream,
                definition.get_entry(8827161447210483302u64),
                "quality",
            )?,
            method: read_value::<u8>(
                stream,
                definition.get_entry(2525399976365011888u64),
                "method",
            )?,
            item_def: read_value::<u32>(
                stream,
                definition.get_entry(13929934279997928333u64),
                "item_def",
            )?,
            is_strange: read_value::<u8>(
                stream,
                definition.get_entry(4841352240690495167u64),
                "is_strange",
            )?,
            is_unusual: read_value::<u8>(
                stream,
                definition.get_entry(17913208470555068296u64),
                "is_unusual",
            )?,
            wear: read_value::<f32>(stream, definition.get_entry(4427899308794289118u64), "wear")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            8827161447210483302u64 => Ok(self.quality.clone().into()),
            2525399976365011888u64 => Ok(self.method.clone().into()),
            13929934279997928333u64 => Ok(self.item_def.clone().into()),
            4841352240690495167u64 => Ok(self.is_strange.clone().into()),
            17913208470555068296u64 => Ok(self.is_unusual.clone().into()),
            4427899308794289118u64 => Ok(self.wear.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ItemFound",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShowAnnotationEvent {
    pub world_pos_x: f32,
    pub world_pos_y: f32,
    pub world_pos_z: f32,
    pub world_normal_x: f32,
    pub world_normal_y: f32,
    pub world_normal_z: f32,
    pub id: u32,
    pub text: MaybeUtf8String,
    pub lifetime: f32,
    pub visibility_bit_field: u32,
    pub follow_ent_index: u32,
    pub show_distance: bool,
    pub play_sound: MaybeUtf8String,
    pub show_effect: bool,
}
impl ShowAnnotationEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ShowAnnotationEvent {
            world_pos_x: read_value::<f32>(
                stream,
                definition.get_entry(17136446437287500353u64),
                "world_pos_x",
            )?,
            world_pos_y: read_value::<f32>(
                stream,
                definition.get_entry(17136445337775872142u64),
                "world_pos_y",
            )?,
            world_pos_z: read_value::<f32>(
                stream,
                definition.get_entry(17136444238264243931u64),
                "world_pos_z",
            )?,
            world_normal_x: read_value::<f32>(
                stream,
                definition.get_entry(6094703810714717260u64),
                "world_normal_x",
            )?,
            world_normal_y: read_value::<f32>(
                stream,
                definition.get_entry(6094704910226345471u64),
                "world_normal_y",
            )?,
            world_normal_z: read_value::<f32>(
                stream,
                definition.get_entry(6094706009737973682u64),
                "world_normal_z",
            )?,
            id: read_value::<u32>(stream, definition.get_entry(628021283683842752u64), "id")?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
            lifetime: read_value::<f32>(
                stream,
                definition.get_entry(17408443252967694094u64),
                "lifetime",
            )?,
            visibility_bit_field: read_value::<u32>(
                stream,
                definition.get_entry(3514840313863479388u64),
                "visibility_bit_field",
            )?,
            follow_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(5396668150069485958u64),
                "follow_ent_index",
            )?,
            show_distance: read_value::<bool>(
                stream,
                definition.get_entry(10815115409701055510u64),
                "show_distance",
            )?,
            play_sound: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(9580177307637293387u64),
                "play_sound",
            )?,
            show_effect: read_value::<bool>(
                stream,
                definition.get_entry(16061793018411867440u64),
                "show_effect",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17136446437287500353u64 => Ok(self.world_pos_x.clone().into()),
            17136445337775872142u64 => Ok(self.world_pos_y.clone().into()),
            17136444238264243931u64 => Ok(self.world_pos_z.clone().into()),
            6094703810714717260u64 => Ok(self.world_normal_x.clone().into()),
            6094704910226345471u64 => Ok(self.world_normal_y.clone().into()),
            6094706009737973682u64 => Ok(self.world_normal_z.clone().into()),
            628021283683842752u64 => Ok(self.id.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            17408443252967694094u64 => Ok(self.lifetime.clone().into()),
            3514840313863479388u64 => Ok(self.visibility_bit_field.clone().into()),
            5396668150069485958u64 => Ok(self.follow_ent_index.clone().into()),
            10815115409701055510u64 => Ok(self.show_distance.clone().into()),
            9580177307637293387u64 => Ok(self.play_sound.clone().into()),
            16061793018411867440u64 => Ok(self.show_effect.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ShowAnnotation",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HideAnnotationEvent {
    pub id: u32,
}
impl HideAnnotationEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HideAnnotationEvent {
            id: read_value::<u32>(stream, definition.get_entry(628021283683842752u64), "id")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            628021283683842752u64 => Ok(self.id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HideAnnotation",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PostInventoryApplicationEvent {
    pub user_id: u16,
}
impl PostInventoryApplicationEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PostInventoryApplicationEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PostInventoryApplication",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointUnlockUpdatedEvent {
    pub index: u16,
    pub time: f32,
}
impl ControlPointUnlockUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointUnlockUpdatedEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            time: read_value::<f32>(stream, definition.get_entry(2185518981507421060u64), "time")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            2185518981507421060u64 => Ok(self.time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointUnlockUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DeployBuffBannerEvent {
    pub buff_type: u8,
    pub buff_owner: u16,
}
impl DeployBuffBannerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DeployBuffBannerEvent {
            buff_type: read_value::<u8>(
                stream,
                definition.get_entry(15706957908546287009u64),
                "buff_type",
            )?,
            buff_owner: read_value::<u16>(
                stream,
                definition.get_entry(16619542197404164576u64),
                "buff_owner",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            15706957908546287009u64 => Ok(self.buff_type.clone().into()),
            16619542197404164576u64 => Ok(self.buff_owner.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DeployBuffBanner",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerBuffEvent {
    pub user_id: u16,
    pub buff_owner: u16,
    pub buff_type: u8,
}
impl PlayerBuffEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerBuffEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            buff_owner: read_value::<u16>(
                stream,
                definition.get_entry(16619542197404164576u64),
                "buff_owner",
            )?,
            buff_type: read_value::<u8>(
                stream,
                definition.get_entry(15706957908546287009u64),
                "buff_type",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            16619542197404164576u64 => Ok(self.buff_owner.clone().into()),
            15706957908546287009u64 => Ok(self.buff_type.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerBuff",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MedicDeathEvent {
    pub user_id: u16,
    pub attacker: u16,
    pub healing: u16,
    pub charged: bool,
}
impl MedicDeathEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MedicDeathEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            healing: read_value::<u16>(
                stream,
                definition.get_entry(2721180038881757981u64),
                "healing",
            )?,
            charged: read_value::<bool>(
                stream,
                definition.get_entry(17260212114554334191u64),
                "charged",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            2721180038881757981u64 => Ok(self.healing.clone().into()),
            17260212114554334191u64 => Ok(self.charged.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MedicDeath",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OvertimeNagEvent {}
impl OvertimeNagEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(OvertimeNagEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "OvertimeNag",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamsChangedEvent {}
impl TeamsChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamsChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamsChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HalloweenPumpkinGrabEvent {
    pub user_id: u16,
}
impl HalloweenPumpkinGrabEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HalloweenPumpkinGrabEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HalloweenPumpkinGrab",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RocketJumpEvent {
    pub user_id: u16,
    pub play_sound: bool,
}
impl RocketJumpEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RocketJumpEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            play_sound: read_value::<bool>(
                stream,
                definition.get_entry(2035986273219443074u64),
                "play_sound",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2035986273219443074u64 => Ok(self.play_sound.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RocketJump",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RocketJumpLandedEvent {
    pub user_id: u16,
}
impl RocketJumpLandedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RocketJumpLandedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RocketJumpLanded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StickyJumpEvent {
    pub user_id: u16,
    pub play_sound: bool,
}
impl StickyJumpEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(StickyJumpEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            play_sound: read_value::<bool>(
                stream,
                definition.get_entry(2035986273219443074u64),
                "play_sound",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2035986273219443074u64 => Ok(self.play_sound.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "StickyJump",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StickyJumpLandedEvent {
    pub user_id: u16,
}
impl StickyJumpLandedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(StickyJumpLandedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "StickyJumpLanded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RocketPackLaunchEvent {
    pub user_id: u16,
    pub play_sound: bool,
}
impl RocketPackLaunchEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RocketPackLaunchEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            play_sound: read_value::<bool>(
                stream,
                definition.get_entry(2035986273219443074u64),
                "play_sound",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2035986273219443074u64 => Ok(self.play_sound.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RocketPackLaunch",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RocketPackLandedEvent {
    pub user_id: u16,
}
impl RocketPackLandedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RocketPackLandedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RocketPackLanded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MedicDefendedEvent {
    pub user_id: u16,
    pub medic: u16,
}
impl MedicDefendedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MedicDefendedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            medic: read_value::<u16>(
                stream,
                definition.get_entry(12912869923554243305u64),
                "medic",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            12912869923554243305u64 => Ok(self.medic.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MedicDefended",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerHealedEvent {
    pub amount: u16,
}
impl LocalPlayerHealedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerHealedEvent {
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerHealed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDestroyedPipeBombEvent {
    pub user_id: u16,
}
impl PlayerDestroyedPipeBombEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDestroyedPipeBombEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDestroyedPipeBomb",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ObjectDeflectedEvent {
    pub user_id: u16,
    pub owner_id: u16,
    pub weapon_id: u16,
    pub object_ent_index: u16,
}
impl ObjectDeflectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ObjectDeflectedEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            owner_id: read_value::<u16>(
                stream,
                definition.get_entry(3274630577613078265u64),
                "owner_id",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            object_ent_index: read_value::<u16>(
                stream,
                definition.get_entry(8474579522830253112u64),
                "object_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            3274630577613078265u64 => Ok(self.owner_id.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            8474579522830253112u64 => Ok(self.object_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ObjectDeflected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerMvpEvent {
    pub player: u16,
}
impl PlayerMvpEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerMvpEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerMvp",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RaidSpawnMobEvent {}
impl RaidSpawnMobEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RaidSpawnMobEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RaidSpawnMob",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RaidSpawnSquadEvent {}
impl RaidSpawnSquadEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RaidSpawnSquadEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RaidSpawnSquad",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NavBlockedEvent {
    pub area: u32,
    pub blocked: bool,
}
impl NavBlockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(NavBlockedEvent {
            area: read_value::<u32>(stream, definition.get_entry(9894459526856489156u64), "area")?,
            blocked: read_value::<bool>(
                stream,
                definition.get_entry(9150172433819428659u64),
                "blocked",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9894459526856489156u64 => Ok(self.area.clone().into()),
            9150172433819428659u64 => Ok(self.blocked.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "NavBlocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PathTrackPassedEvent {
    pub index: u16,
}
impl PathTrackPassedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PathTrackPassedEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PathTrackPassed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NumCappersChangedEvent {
    pub index: u16,
    pub count: u8,
}
impl NumCappersChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(NumCappersChangedEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            count: read_value::<u8>(
                stream,
                definition.get_entry(12818901015042040436u64),
                "count",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            12818901015042040436u64 => Ok(self.count.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "NumCappersChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerRegenerateEvent {}
impl PlayerRegenerateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerRegenerateEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerRegenerate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UpdateStatusItemEvent {
    pub index: u8,
    pub object: u8,
}
impl UpdateStatusItemEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(UpdateStatusItemEvent {
            index: read_value::<u8>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            object: read_value::<u8>(
                stream,
                definition.get_entry(10231808476453998586u64),
                "object",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            10231808476453998586u64 => Ok(self.object.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "UpdateStatusItem",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StatsResetRoundEvent {}
impl StatsResetRoundEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(StatsResetRoundEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "StatsResetRound",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoreStatsAccumulatedUpdateEvent {}
impl ScoreStatsAccumulatedUpdateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ScoreStatsAccumulatedUpdateEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ScoreStatsAccumulatedUpdate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoreStatsAccumulatedResetEvent {}
impl ScoreStatsAccumulatedResetEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ScoreStatsAccumulatedResetEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ScoreStatsAccumulatedReset",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AchievementEarnedLocalEvent {
    pub achievement: u16,
}
impl AchievementEarnedLocalEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(AchievementEarnedLocalEvent {
            achievement: read_value::<u16>(
                stream,
                definition.get_entry(7071905471600864408u64),
                "achievement",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7071905471600864408u64 => Ok(self.achievement.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "AchievementEarnedLocal",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHealedEvent {
    pub patient: u16,
    pub healer: u16,
    pub amount: u16,
}
impl PlayerHealedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHealedEvent {
            patient: read_value::<u16>(
                stream,
                definition.get_entry(1896141292373524286u64),
                "patient",
            )?,
            healer: read_value::<u16>(
                stream,
                definition.get_entry(9195440821534910520u64),
                "healer",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1896141292373524286u64 => Ok(self.patient.clone().into()),
            9195440821534910520u64 => Ok(self.healer.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHealed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BuildingHealedEvent {
    pub building: u16,
    pub healer: u16,
    pub amount: u16,
}
impl BuildingHealedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BuildingHealedEvent {
            building: read_value::<u16>(
                stream,
                definition.get_entry(18077801548994995317u64),
                "building",
            )?,
            healer: read_value::<u16>(
                stream,
                definition.get_entry(9195440821534910520u64),
                "healer",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18077801548994995317u64 => Ok(self.building.clone().into()),
            9195440821534910520u64 => Ok(self.healer.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BuildingHealed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ItemPickupEvent {
    pub user_id: u16,
    pub item: MaybeUtf8String,
}
impl ItemPickupEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ItemPickupEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            item: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2900776405502981158u64),
                "item",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            2900776405502981158u64 => Ok(self.item.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ItemPickup",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DuelStatusEvent {
    pub killer: u16,
    pub score_type: u16,
    pub initiator: u16,
    pub target: u16,
    pub initiator_score: u16,
    pub target_score: u16,
}
impl DuelStatusEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DuelStatusEvent {
            killer: read_value::<u16>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
            score_type: read_value::<u16>(
                stream,
                definition.get_entry(5898901342300468638u64),
                "score_type",
            )?,
            initiator: read_value::<u16>(
                stream,
                definition.get_entry(7196121162372295066u64),
                "initiator",
            )?,
            target: read_value::<u16>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
            initiator_score: read_value::<u16>(
                stream,
                definition.get_entry(3503682569462171123u64),
                "initiator_score",
            )?,
            target_score: read_value::<u16>(
                stream,
                definition.get_entry(3432502518358366373u64),
                "target_score",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            5898901342300468638u64 => Ok(self.score_type.clone().into()),
            7196121162372295066u64 => Ok(self.initiator.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            3503682569462171123u64 => Ok(self.initiator_score.clone().into()),
            3432502518358366373u64 => Ok(self.target_score.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DuelStatus",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FishNoticeEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub assister: u16,
    pub weapon_log_class_name: MaybeUtf8String,
    pub stun_flags: u16,
    pub death_flags: u16,
    pub silent_kill: bool,
    pub assister_fallback: MaybeUtf8String,
}
impl FishNoticeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FishNoticeEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
            stun_flags: read_value::<u16>(
                stream,
                definition.get_entry(16746745151415897845u64),
                "stun_flags",
            )?,
            death_flags: read_value::<u16>(
                stream,
                definition.get_entry(210841622282264177u64),
                "death_flags",
            )?,
            silent_kill: read_value::<bool>(
                stream,
                definition.get_entry(5449831253309542421u64),
                "silent_kill",
            )?,
            assister_fallback: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2624120833319605424u64),
                "assister_fallback",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            16746745151415897845u64 => Ok(self.stun_flags.clone().into()),
            210841622282264177u64 => Ok(self.death_flags.clone().into()),
            5449831253309542421u64 => Ok(self.silent_kill.clone().into()),
            2624120833319605424u64 => Ok(self.assister_fallback.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FishNotice",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FishNoticeArmEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub assister: u16,
    pub weapon_log_class_name: MaybeUtf8String,
    pub stun_flags: u16,
    pub death_flags: u16,
    pub silent_kill: bool,
    pub assister_fallback: MaybeUtf8String,
}
impl FishNoticeArmEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FishNoticeArmEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
            stun_flags: read_value::<u16>(
                stream,
                definition.get_entry(16746745151415897845u64),
                "stun_flags",
            )?,
            death_flags: read_value::<u16>(
                stream,
                definition.get_entry(210841622282264177u64),
                "death_flags",
            )?,
            silent_kill: read_value::<bool>(
                stream,
                definition.get_entry(5449831253309542421u64),
                "silent_kill",
            )?,
            assister_fallback: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2624120833319605424u64),
                "assister_fallback",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            16746745151415897845u64 => Ok(self.stun_flags.clone().into()),
            210841622282264177u64 => Ok(self.death_flags.clone().into()),
            5449831253309542421u64 => Ok(self.silent_kill.clone().into()),
            2624120833319605424u64 => Ok(self.assister_fallback.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FishNoticeArm",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SlapNoticeEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub assister: u16,
    pub weapon_log_class_name: MaybeUtf8String,
    pub stun_flags: u16,
    pub death_flags: u16,
    pub silent_kill: bool,
    pub assister_fallback: MaybeUtf8String,
}
impl SlapNoticeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SlapNoticeEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
            stun_flags: read_value::<u16>(
                stream,
                definition.get_entry(16746745151415897845u64),
                "stun_flags",
            )?,
            death_flags: read_value::<u16>(
                stream,
                definition.get_entry(210841622282264177u64),
                "death_flags",
            )?,
            silent_kill: read_value::<bool>(
                stream,
                definition.get_entry(5449831253309542421u64),
                "silent_kill",
            )?,
            assister_fallback: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2624120833319605424u64),
                "assister_fallback",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            16746745151415897845u64 => Ok(self.stun_flags.clone().into()),
            210841622282264177u64 => Ok(self.death_flags.clone().into()),
            5449831253309542421u64 => Ok(self.silent_kill.clone().into()),
            2624120833319605424u64 => Ok(self.assister_fallback.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SlapNotice",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ThrowableHitEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub assister: u16,
    pub weapon_log_class_name: MaybeUtf8String,
    pub stun_flags: u16,
    pub death_flags: u16,
    pub silent_kill: bool,
    pub assister_fallback: MaybeUtf8String,
    pub total_hits: u16,
}
impl ThrowableHitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ThrowableHitEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
            stun_flags: read_value::<u16>(
                stream,
                definition.get_entry(16746745151415897845u64),
                "stun_flags",
            )?,
            death_flags: read_value::<u16>(
                stream,
                definition.get_entry(210841622282264177u64),
                "death_flags",
            )?,
            silent_kill: read_value::<bool>(
                stream,
                definition.get_entry(5449831253309542421u64),
                "silent_kill",
            )?,
            assister_fallback: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2624120833319605424u64),
                "assister_fallback",
            )?,
            total_hits: read_value::<u16>(
                stream,
                definition.get_entry(16097486085498709235u64),
                "total_hits",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            16746745151415897845u64 => Ok(self.stun_flags.clone().into()),
            210841622282264177u64 => Ok(self.death_flags.clone().into()),
            5449831253309542421u64 => Ok(self.silent_kill.clone().into()),
            2624120833319605424u64 => Ok(self.assister_fallback.clone().into()),
            16097486085498709235u64 => Ok(self.total_hits.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ThrowableHit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PumpkinLordSummonedEvent {}
impl PumpkinLordSummonedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PumpkinLordSummonedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PumpkinLordSummoned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PumpkinLordKilledEvent {}
impl PumpkinLordKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PumpkinLordKilledEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PumpkinLordKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusSummonedEvent {
    pub level: u16,
}
impl MerasmusSummonedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusSummonedEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusSummoned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusKilledEvent {
    pub level: u16,
}
impl MerasmusKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusKilledEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusEscapeWarningEvent {
    pub level: u16,
    pub time_remaining: u8,
}
impl MerasmusEscapeWarningEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusEscapeWarningEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
            time_remaining: read_value::<u8>(
                stream,
                definition.get_entry(4322130481848759231u64),
                "time_remaining",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            4322130481848759231u64 => Ok(self.time_remaining.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusEscapeWarning",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusEscapedEvent {
    pub level: u16,
}
impl MerasmusEscapedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusEscapedEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusEscaped",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossSummonedEvent {
    pub level: u16,
}
impl EyeballBossSummonedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossSummonedEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossSummoned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossStunnedEvent {
    pub level: u16,
    pub player_ent_index: u8,
}
impl EyeballBossStunnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossStunnedEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
            player_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(13724068420967497014u64),
                "player_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            13724068420967497014u64 => Ok(self.player_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossStunned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossKilledEvent {
    pub level: u16,
}
impl EyeballBossKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossKilledEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossKillerEvent {
    pub level: u16,
    pub player_ent_index: u8,
}
impl EyeballBossKillerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossKillerEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
            player_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(13724068420967497014u64),
                "player_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            13724068420967497014u64 => Ok(self.player_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossKiller",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossEscapeImminentEvent {
    pub level: u16,
    pub time_remaining: u8,
}
impl EyeballBossEscapeImminentEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossEscapeImminentEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
            time_remaining: read_value::<u8>(
                stream,
                definition.get_entry(4322130481848759231u64),
                "time_remaining",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            4322130481848759231u64 => Ok(self.time_remaining.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossEscapeImminent",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EyeballBossEscapedEvent {
    pub level: u16,
}
impl EyeballBossEscapedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EyeballBossEscapedEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EyeballBossEscaped",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NpcHurtEvent {
    pub ent_index: u16,
    pub health: u16,
    pub attacker_player: u16,
    pub weapon_id: u16,
    pub damage_amount: u16,
    pub crit: bool,
    pub boss: u16,
}
impl NpcHurtEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(NpcHurtEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            health: read_value::<u16>(
                stream,
                definition.get_entry(9181103189905877455u64),
                "health",
            )?,
            attacker_player: read_value::<u16>(
                stream,
                definition.get_entry(7636913148849873330u64),
                "attacker_player",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_amount: read_value::<u16>(
                stream,
                definition.get_entry(7439038394412279612u64),
                "damage_amount",
            )?,
            crit: read_value::<bool>(stream, definition.get_entry(1324453635955533101u64), "crit")?,
            boss: read_value::<u16>(
                stream,
                definition.get_entry(14781339113128978092u64),
                "boss",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            9181103189905877455u64 => Ok(self.health.clone().into()),
            7636913148849873330u64 => Ok(self.attacker_player.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            7439038394412279612u64 => Ok(self.damage_amount.clone().into()),
            1324453635955533101u64 => Ok(self.crit.clone().into()),
            14781339113128978092u64 => Ok(self.boss.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "NpcHurt",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ControlPointTimerUpdatedEvent {
    pub index: u16,
    pub time: f32,
}
impl ControlPointTimerUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ControlPointTimerUpdatedEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            time: read_value::<f32>(stream, definition.get_entry(2185518981507421060u64), "time")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            2185518981507421060u64 => Ok(self.time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ControlPointTimerUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHighFiveStartEvent {
    pub ent_index: u8,
}
impl PlayerHighFiveStartEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHighFiveStartEvent {
            ent_index: read_value::<u8>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHighFiveStart",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHighFiveCancelEvent {
    pub ent_index: u8,
}
impl PlayerHighFiveCancelEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHighFiveCancelEvent {
            ent_index: read_value::<u8>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHighFiveCancel",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerHighFiveSuccessEvent {
    pub initiator_ent_index: u8,
    pub partner_ent_index: u8,
}
impl PlayerHighFiveSuccessEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerHighFiveSuccessEvent {
            initiator_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(18349426944415386264u64),
                "initiator_ent_index",
            )?,
            partner_ent_index: read_value::<u8>(
                stream,
                definition.get_entry(6552235651321274101u64),
                "partner_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18349426944415386264u64 => Ok(self.initiator_ent_index.clone().into()),
            6552235651321274101u64 => Ok(self.partner_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerHighFiveSuccess",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerBonusPointsEvent {
    pub points: u16,
    pub player_ent_index: u16,
    pub source_ent_index: u16,
}
impl PlayerBonusPointsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerBonusPointsEvent {
            points: read_value::<u16>(
                stream,
                definition.get_entry(15925482014108518566u64),
                "points",
            )?,
            player_ent_index: read_value::<u16>(
                stream,
                definition.get_entry(13724068420967497014u64),
                "player_ent_index",
            )?,
            source_ent_index: read_value::<u16>(
                stream,
                definition.get_entry(7014721347321999598u64),
                "source_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            15925482014108518566u64 => Ok(self.points.clone().into()),
            13724068420967497014u64 => Ok(self.player_ent_index.clone().into()),
            7014721347321999598u64 => Ok(self.source_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerBonusPoints",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerUpgradedEvent {}
impl PlayerUpgradedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerUpgradedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerUpgraded",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerBuybackEvent {
    pub player: u16,
    pub cost: u16,
}
impl PlayerBuybackEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerBuybackEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            cost: read_value::<u16>(stream, definition.get_entry(859913274118676344u64), "cost")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            859913274118676344u64 => Ok(self.cost.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerBuyback",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerUsedPowerUpBottleEvent {
    pub player: u16,
    pub kind: u16,
    pub time: f32,
}
impl PlayerUsedPowerUpBottleEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerUsedPowerUpBottleEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            kind: read_value::<u16>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
            time: read_value::<f32>(stream, definition.get_entry(2185518981507421060u64), "time")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            2185518981507421060u64 => Ok(self.time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerUsedPowerUpBottle",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ChristmasGiftGrabEvent {
    pub user_id: u16,
}
impl ChristmasGiftGrabEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ChristmasGiftGrabEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ChristmasGiftGrab",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerKilledAchievementZoneEvent {
    pub attacker: u16,
    pub victim: u16,
    pub zone_id: u16,
}
impl PlayerKilledAchievementZoneEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerKilledAchievementZoneEvent {
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            zone_id: read_value::<u16>(
                stream,
                definition.get_entry(15583775021870447765u64),
                "zone_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            15583775021870447765u64 => Ok(self.zone_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerKilledAchievementZone",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyUpdatedEvent {}
impl PartyUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyPrefChangedEvent {}
impl PartyPrefChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyPrefChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyPrefChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyCriteriaChangedEvent {}
impl PartyCriteriaChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyCriteriaChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyCriteriaChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyInvitesChangedEvent {}
impl PartyInvitesChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyInvitesChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyInvitesChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyQueueStateChangedEvent {
    pub match_group: u16,
}
impl PartyQueueStateChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyQueueStateChangedEvent {
            match_group: read_value::<u16>(
                stream,
                definition.get_entry(932658807224826441u64),
                "match_group",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            932658807224826441u64 => Ok(self.match_group.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyQueueStateChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyChatEvent {
    pub steam_id: MaybeUtf8String,
    pub text: MaybeUtf8String,
    pub kind: u16,
}
impl PartyChatEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyChatEvent {
            steam_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2266437675922439198u64),
                "steam_id",
            )?,
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
            kind: read_value::<u16>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2266437675922439198u64 => Ok(self.steam_id.clone().into()),
            18015793717152399486u64 => Ok(self.text.clone().into()),
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyChat",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyMemberJoinEvent {
    pub steam_id: MaybeUtf8String,
}
impl PartyMemberJoinEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyMemberJoinEvent {
            steam_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2266437675922439198u64),
                "steam_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2266437675922439198u64 => Ok(self.steam_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyMemberJoin",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PartyMemberLeaveEvent {
    pub steam_id: MaybeUtf8String,
}
impl PartyMemberLeaveEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PartyMemberLeaveEvent {
            steam_id: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(2266437675922439198u64),
                "steam_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2266437675922439198u64 => Ok(self.steam_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PartyMemberLeave",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MatchInvitesUpdatedEvent {}
impl MatchInvitesUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MatchInvitesUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MatchInvitesUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LobbyUpdatedEvent {}
impl LobbyUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LobbyUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LobbyUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmMissionUpdateEvent {
    pub class: u16,
    pub count: u16,
}
impl MvmMissionUpdateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmMissionUpdateEvent {
            class: read_value::<u16>(
                stream,
                definition.get_entry(15066323702654938015u64),
                "class",
            )?,
            count: read_value::<u16>(
                stream,
                definition.get_entry(12818901015042040436u64),
                "count",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            15066323702654938015u64 => Ok(self.class.clone().into()),
            12818901015042040436u64 => Ok(self.count.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmMissionUpdate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RecalculateHolidaysEvent {}
impl RecalculateHolidaysEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RecalculateHolidaysEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RecalculateHolidays",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerCurrencyChangedEvent {
    pub currency: u16,
}
impl PlayerCurrencyChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerCurrencyChangedEvent {
            currency: read_value::<u16>(
                stream,
                definition.get_entry(8477458519032517654u64),
                "currency",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8477458519032517654u64 => Ok(self.currency.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerCurrencyChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DoomsdayRocketOpenEvent {
    pub team: u8,
}
impl DoomsdayRocketOpenEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DoomsdayRocketOpenEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DoomsdayRocketOpen",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RemoveNemesisRelationshipsEvent {
    pub player: u16,
}
impl RemoveNemesisRelationshipsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RemoveNemesisRelationshipsEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RemoveNemesisRelationships",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmCreditBonusWaveEvent {}
impl MvmCreditBonusWaveEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmCreditBonusWaveEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmCreditBonusWave",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmCreditBonusAllEvent {}
impl MvmCreditBonusAllEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmCreditBonusAllEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmCreditBonusAll",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmCreditBonusAllAdvancedEvent {}
impl MvmCreditBonusAllAdvancedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmCreditBonusAllAdvancedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmCreditBonusAllAdvanced",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmQuickSentryUpgradeEvent {
    pub player: u16,
}
impl MvmQuickSentryUpgradeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmQuickSentryUpgradeEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmQuickSentryUpgrade",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmTankDestroyedByPlayersEvent {}
impl MvmTankDestroyedByPlayersEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmTankDestroyedByPlayersEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmTankDestroyedByPlayers",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmKillRobotDeliveringBombEvent {
    pub player: u16,
}
impl MvmKillRobotDeliveringBombEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmKillRobotDeliveringBombEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmKillRobotDeliveringBomb",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmPickupCurrencyEvent {
    pub player: u16,
    pub currency: u16,
}
impl MvmPickupCurrencyEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmPickupCurrencyEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            currency: read_value::<u16>(
                stream,
                definition.get_entry(8477458519032517654u64),
                "currency",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            8477458519032517654u64 => Ok(self.currency.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmPickupCurrency",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmBombCarrierKilledEvent {
    pub level: u16,
}
impl MvmBombCarrierKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmBombCarrierKilledEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmBombCarrierKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmSentryBusterDetonateEvent {
    pub player: u16,
    pub det_x: f32,
    pub det_y: f32,
    pub det_z: f32,
}
impl MvmSentryBusterDetonateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmSentryBusterDetonateEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            det_x: read_value::<f32>(
                stream,
                definition.get_entry(10880309079248206573u64),
                "det_x",
            )?,
            det_y: read_value::<f32>(
                stream,
                definition.get_entry(10880307979736578362u64),
                "det_y",
            )?,
            det_z: read_value::<f32>(
                stream,
                definition.get_entry(10880306880224950151u64),
                "det_z",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            10880309079248206573u64 => Ok(self.det_x.clone().into()),
            10880307979736578362u64 => Ok(self.det_y.clone().into()),
            10880306880224950151u64 => Ok(self.det_z.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmSentryBusterDetonate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmScoutMarkedForDeathEvent {
    pub player: u16,
}
impl MvmScoutMarkedForDeathEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmScoutMarkedForDeathEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmScoutMarkedForDeath",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmMedicPowerUpSharedEvent {
    pub player: u16,
}
impl MvmMedicPowerUpSharedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmMedicPowerUpSharedEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmMedicPowerUpShared",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmBeginWaveEvent {
    pub wave_index: u16,
    pub max_waves: u16,
    pub advanced: u16,
}
impl MvmBeginWaveEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmBeginWaveEvent {
            wave_index: read_value::<u16>(
                stream,
                definition.get_entry(3508416600111464007u64),
                "wave_index",
            )?,
            max_waves: read_value::<u16>(
                stream,
                definition.get_entry(16186306532262586848u64),
                "max_waves",
            )?,
            advanced: read_value::<u16>(
                stream,
                definition.get_entry(2115851684636468731u64),
                "advanced",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3508416600111464007u64 => Ok(self.wave_index.clone().into()),
            16186306532262586848u64 => Ok(self.max_waves.clone().into()),
            2115851684636468731u64 => Ok(self.advanced.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmBeginWave",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmWaveCompleteEvent {
    pub advanced: bool,
}
impl MvmWaveCompleteEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmWaveCompleteEvent {
            advanced: read_value::<bool>(
                stream,
                definition.get_entry(2115851684636468731u64),
                "advanced",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2115851684636468731u64 => Ok(self.advanced.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmWaveComplete",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmMissionCompleteEvent {
    pub mission: MaybeUtf8String,
}
impl MvmMissionCompleteEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmMissionCompleteEvent {
            mission: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(9969423424304279641u64),
                "mission",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9969423424304279641u64 => Ok(self.mission.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmMissionComplete",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmBombResetByPlayerEvent {
    pub player: u16,
}
impl MvmBombResetByPlayerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmBombResetByPlayerEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmBombResetByPlayer",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmBombAlarmTriggeredEvent {}
impl MvmBombAlarmTriggeredEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmBombAlarmTriggeredEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmBombAlarmTriggered",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmBombDeployResetByPlayerEvent {
    pub player: u16,
}
impl MvmBombDeployResetByPlayerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmBombDeployResetByPlayerEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmBombDeployResetByPlayer",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmWaveFailedEvent {}
impl MvmWaveFailedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmWaveFailedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmWaveFailed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmResetStatsEvent {}
impl MvmResetStatsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmResetStatsEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmResetStats",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DamageResistedEvent {
    pub ent_index: u8,
}
impl DamageResistedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DamageResistedEvent {
            ent_index: read_value::<u8>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DamageResisted",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RevivePlayerNotifyEvent {
    pub ent_index: u16,
    pub marker_ent_index: u16,
}
impl RevivePlayerNotifyEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RevivePlayerNotifyEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            marker_ent_index: read_value::<u16>(
                stream,
                definition.get_entry(3793715953080702003u64),
                "marker_ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            3793715953080702003u64 => Ok(self.marker_ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RevivePlayerNotify",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RevivePlayerStoppedEvent {
    pub ent_index: u16,
}
impl RevivePlayerStoppedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RevivePlayerStoppedEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RevivePlayerStopped",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RevivePlayerCompleteEvent {
    pub ent_index: u16,
}
impl RevivePlayerCompleteEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RevivePlayerCompleteEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RevivePlayerComplete",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerTurnedToGhostEvent {
    pub user_id: u16,
}
impl PlayerTurnedToGhostEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerTurnedToGhostEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerTurnedToGhost",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MedigunShieldBlockedDamageEvent {
    pub user_id: u16,
    pub damage: f32,
}
impl MedigunShieldBlockedDamageEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MedigunShieldBlockedDamageEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            damage: read_value::<f32>(
                stream,
                definition.get_entry(9179190079975030720u64),
                "damage",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            9179190079975030720u64 => Ok(self.damage.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MedigunShieldBlockedDamage",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmAdvWaveCompleteNoGatesEvent {
    pub index: u16,
}
impl MvmAdvWaveCompleteNoGatesEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmAdvWaveCompleteNoGatesEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmAdvWaveCompleteNoGates",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmSniperHeadshotCurrencyEvent {
    pub user_id: u16,
    pub currency: u16,
}
impl MvmSniperHeadshotCurrencyEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmSniperHeadshotCurrencyEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            currency: read_value::<u16>(
                stream,
                definition.get_entry(8477458519032517654u64),
                "currency",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            8477458519032517654u64 => Ok(self.currency.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmSniperHeadshotCurrency",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmMannhattanPitEvent {}
impl MvmMannhattanPitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmMannhattanPitEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmMannhattanPit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FlagCarriedInDetectionZoneEvent {}
impl FlagCarriedInDetectionZoneEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(FlagCarriedInDetectionZoneEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "FlagCarriedInDetectionZone",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmAdvWaveKilledStunRadioEvent {}
impl MvmAdvWaveKilledStunRadioEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmAdvWaveKilledStunRadioEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmAdvWaveKilledStunRadio",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDirectHitStunEvent {
    pub attacker: u16,
    pub victim: u16,
}
impl PlayerDirectHitStunEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDirectHitStunEvent {
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDirectHitStun",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MvmSentryBusterKilledEvent {
    pub sentry_buster: u16,
}
impl MvmSentryBusterKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MvmSentryBusterKilledEvent {
            sentry_buster: read_value::<u16>(
                stream,
                definition.get_entry(7717696824740572250u64),
                "sentry_buster",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7717696824740572250u64 => Ok(self.sentry_buster.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MvmSentryBusterKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UpgradesFileChangedEvent {
    pub path: MaybeUtf8String,
}
impl UpgradesFileChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(UpgradesFileChangedEvent {
            path: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(271672890340345462u64),
                "path",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            271672890340345462u64 => Ok(self.path.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "UpgradesFileChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RdTeamPointsChangedEvent {
    pub points: u16,
    pub team: u8,
    pub method: u8,
}
impl RdTeamPointsChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RdTeamPointsChangedEvent {
            points: read_value::<u16>(
                stream,
                definition.get_entry(15925482014108518566u64),
                "points",
            )?,
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            method: read_value::<u8>(
                stream,
                definition.get_entry(2525399976365011888u64),
                "method",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            15925482014108518566u64 => Ok(self.points.clone().into()),
            18024489754618217260u64 => Ok(self.team.clone().into()),
            2525399976365011888u64 => Ok(self.method.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RdTeamPointsChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RdRulesStateChangedEvent {}
impl RdRulesStateChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RdRulesStateChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RdRulesStateChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RdRobotKilledEvent {
    pub user_id: u16,
    pub victim_ent_index: u32,
    pub inflictor_ent_index: u32,
    pub attacker: u16,
    pub weapon: MaybeUtf8String,
    pub weapon_id: u16,
    pub damage_bits: u32,
    pub custom_kill: u16,
    pub weapon_log_class_name: MaybeUtf8String,
}
impl RdRobotKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RdRobotKilledEvent {
            user_id: read_value::<u16>(
                stream,
                definition.get_entry(17826443226371159423u64),
                "user_id",
            )?,
            victim_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7608903543976233025u64),
                "victim_ent_index",
            )?,
            inflictor_ent_index: read_value::<u32>(
                stream,
                definition.get_entry(7862267791693534473u64),
                "inflictor_ent_index",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11580461223051554305u64),
                "weapon",
            )?,
            weapon_id: read_value::<u16>(
                stream,
                definition.get_entry(5542695206485500884u64),
                "weapon_id",
            )?,
            damage_bits: read_value::<u32>(
                stream,
                definition.get_entry(2104626753992558984u64),
                "damage_bits",
            )?,
            custom_kill: read_value::<u16>(
                stream,
                definition.get_entry(9002408094759571186u64),
                "custom_kill",
            )?,
            weapon_log_class_name: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(8214628514117900939u64),
                "weapon_log_class_name",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17826443226371159423u64 => Ok(self.user_id.clone().into()),
            7608903543976233025u64 => Ok(self.victim_ent_index.clone().into()),
            7862267791693534473u64 => Ok(self.inflictor_ent_index.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            11580461223051554305u64 => Ok(self.weapon.clone().into()),
            5542695206485500884u64 => Ok(self.weapon_id.clone().into()),
            2104626753992558984u64 => Ok(self.damage_bits.clone().into()),
            9002408094759571186u64 => Ok(self.custom_kill.clone().into()),
            8214628514117900939u64 => Ok(self.weapon_log_class_name.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RdRobotKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RdRobotImpactEvent {
    pub ent_index: u16,
    pub impulse_x: f32,
    pub impulse_y: f32,
    pub impulse_z: f32,
}
impl RdRobotImpactEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RdRobotImpactEvent {
            ent_index: read_value::<u16>(
                stream,
                definition.get_entry(17554918082946193550u64),
                "ent_index",
            )?,
            impulse_x: read_value::<f32>(
                stream,
                definition.get_entry(2780540280585913547u64),
                "impulse_x",
            )?,
            impulse_y: read_value::<f32>(
                stream,
                definition.get_entry(2780539181074285336u64),
                "impulse_y",
            )?,
            impulse_z: read_value::<f32>(
                stream,
                definition.get_entry(2780542479609169969u64),
                "impulse_z",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17554918082946193550u64 => Ok(self.ent_index.clone().into()),
            2780540280585913547u64 => Ok(self.impulse_x.clone().into()),
            2780539181074285336u64 => Ok(self.impulse_y.clone().into()),
            2780542479609169969u64 => Ok(self.impulse_z.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RdRobotImpact",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamPlayPreRoundTimeLeftEvent {
    pub time: u16,
}
impl TeamPlayPreRoundTimeLeftEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamPlayPreRoundTimeLeftEvent {
            time: read_value::<u16>(stream, definition.get_entry(2185518981507421060u64), "time")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2185518981507421060u64 => Ok(self.time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamPlayPreRoundTimeLeft",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ParachuteDeployEvent {
    pub index: u16,
}
impl ParachuteDeployEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ParachuteDeployEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ParachuteDeploy",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ParachuteHolsterEvent {
    pub index: u16,
}
impl ParachuteHolsterEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ParachuteHolsterEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ParachuteHolster",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct KillRefillsMeterEvent {
    pub index: u16,
}
impl KillRefillsMeterEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(KillRefillsMeterEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "KillRefillsMeter",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RpsTauntEventEvent {
    pub winner: u16,
    pub winner_rps: u8,
    pub loser: u16,
    pub loser_rps: u8,
}
impl RpsTauntEventEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RpsTauntEventEvent {
            winner: read_value::<u16>(
                stream,
                definition.get_entry(4337804175666422150u64),
                "winner",
            )?,
            winner_rps: read_value::<u8>(
                stream,
                definition.get_entry(10341772402923738934u64),
                "winner_rps",
            )?,
            loser: read_value::<u16>(
                stream,
                definition.get_entry(12743961777105763166u64),
                "loser",
            )?,
            loser_rps: read_value::<u8>(
                stream,
                definition.get_entry(7097735284409203870u64),
                "loser_rps",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            4337804175666422150u64 => Ok(self.winner.clone().into()),
            10341772402923738934u64 => Ok(self.winner_rps.clone().into()),
            12743961777105763166u64 => Ok(self.loser.clone().into()),
            7097735284409203870u64 => Ok(self.loser_rps.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RpsTauntEvent",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CongaKillEvent {
    pub index: u16,
}
impl CongaKillEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CongaKillEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CongaKill",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerInitialSpawnEvent {
    pub index: u16,
}
impl PlayerInitialSpawnEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerInitialSpawnEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerInitialSpawn",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CompetitiveVictoryEvent {}
impl CompetitiveVictoryEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CompetitiveVictoryEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CompetitiveVictory",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CompetitiveStatsUpdateEvent {
    pub index: u16,
    pub kills_rank: u8,
    pub score_rank: u8,
    pub damage_rank: u8,
    pub healing_rank: u8,
    pub support_rank: u8,
}
impl CompetitiveStatsUpdateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CompetitiveStatsUpdateEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            kills_rank: read_value::<u8>(
                stream,
                definition.get_entry(1647429874780590553u64),
                "kills_rank",
            )?,
            score_rank: read_value::<u8>(
                stream,
                definition.get_entry(7011983216168462200u64),
                "score_rank",
            )?,
            damage_rank: read_value::<u8>(
                stream,
                definition.get_entry(1489671713798354735u64),
                "damage_rank",
            )?,
            healing_rank: read_value::<u8>(
                stream,
                definition.get_entry(4364771850637042816u64),
                "healing_rank",
            )?,
            support_rank: read_value::<u8>(
                stream,
                definition.get_entry(177739702519296055u64),
                "support_rank",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            1647429874780590553u64 => Ok(self.kills_rank.clone().into()),
            7011983216168462200u64 => Ok(self.score_rank.clone().into()),
            1489671713798354735u64 => Ok(self.damage_rank.clone().into()),
            4364771850637042816u64 => Ok(self.healing_rank.clone().into()),
            177739702519296055u64 => Ok(self.support_rank.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CompetitiveStatsUpdate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MiniGameWinEvent {
    pub team: u8,
    pub kind: u8,
}
impl MiniGameWinEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MiniGameWinEvent {
            team: read_value::<u8>(
                stream,
                definition.get_entry(18024489754618217260u64),
                "team",
            )?,
            kind: read_value::<u8>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18024489754618217260u64 => Ok(self.team.clone().into()),
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MiniGameWin",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SentryOnGoActiveEvent {
    pub index: u16,
}
impl SentryOnGoActiveEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SentryOnGoActiveEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SentryOnGoActive",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DuckXpLevelUpEvent {
    pub level: u16,
}
impl DuckXpLevelUpEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DuckXpLevelUpEvent {
            level: read_value::<u16>(
                stream,
                definition.get_entry(16779788834081370269u64),
                "level",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16779788834081370269u64 => Ok(self.level.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DuckXpLevelUp",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestLogOpenedEvent {}
impl QuestLogOpenedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestLogOpenedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestLogOpened",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SchemaUpdatedEvent {}
impl SchemaUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SchemaUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SchemaUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LocalPlayerPickupWeaponEvent {}
impl LocalPlayerPickupWeaponEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(LocalPlayerPickupWeaponEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "LocalPlayerPickupWeapon",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RdPlayerScorePointsEvent {
    pub player: u16,
    pub method: u16,
    pub amount: u16,
}
impl RdPlayerScorePointsEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RdPlayerScorePointsEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            method: read_value::<u16>(
                stream,
                definition.get_entry(2525399976365011888u64),
                "method",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            2525399976365011888u64 => Ok(self.method.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RdPlayerScorePoints",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DemomanDetStickiesEvent {
    pub player: u16,
}
impl DemomanDetStickiesEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DemomanDetStickiesEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DemomanDetStickies",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestObjectiveCompletedEvent {
    pub quest_item_id_low: u32,
    pub quest_item_id_hi: u32,
    pub quest_objective_id: u32,
    pub scorer_user_id: u16,
}
impl QuestObjectiveCompletedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestObjectiveCompletedEvent {
            quest_item_id_low: read_value::<u32>(
                stream,
                definition.get_entry(6899404301657350854u64),
                "quest_item_id_low",
            )?,
            quest_item_id_hi: read_value::<u32>(
                stream,
                definition.get_entry(5919305845835660727u64),
                "quest_item_id_hi",
            )?,
            quest_objective_id: read_value::<u32>(
                stream,
                definition.get_entry(16538099295756322479u64),
                "quest_objective_id",
            )?,
            scorer_user_id: read_value::<u16>(
                stream,
                definition.get_entry(10733259024453182903u64),
                "scorer_user_id",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6899404301657350854u64 => Ok(self.quest_item_id_low.clone().into()),
            5919305845835660727u64 => Ok(self.quest_item_id_hi.clone().into()),
            16538099295756322479u64 => Ok(self.quest_objective_id.clone().into()),
            10733259024453182903u64 => Ok(self.scorer_user_id.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestObjectiveCompleted",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerScoreChangedEvent {
    pub player: u8,
    pub delta: u16,
}
impl PlayerScoreChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerScoreChangedEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            delta: read_value::<u16>(
                stream,
                definition.get_entry(5910805692604981441u64),
                "delta",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            5910805692604981441u64 => Ok(self.delta.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerScoreChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct KilledCappingPlayerEvent {
    pub cp: u8,
    pub killer: u8,
    pub victim: u8,
    pub assister: u8,
}
impl KilledCappingPlayerEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(KilledCappingPlayerEvent {
            cp: read_value::<u8>(stream, definition.get_entry(622127901357767142u64), "cp")?,
            killer: read_value::<u8>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            assister: read_value::<u8>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            622127901357767142u64 => Ok(self.cp.clone().into()),
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "KilledCappingPlayer",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EnvironmentalDeathEvent {
    pub killer: u8,
    pub victim: u8,
}
impl EnvironmentalDeathEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EnvironmentalDeathEvent {
            killer: read_value::<u8>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EnvironmentalDeath",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ProjectileDirectHitEvent {
    pub attacker: u8,
    pub victim: u8,
    pub weapon_def_index: u32,
}
impl ProjectileDirectHitEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ProjectileDirectHitEvent {
            attacker: read_value::<u8>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            weapon_def_index: read_value::<u32>(
                stream,
                definition.get_entry(4132306594868589054u64),
                "weapon_def_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            4132306594868589054u64 => Ok(self.weapon_def_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ProjectileDirectHit",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassGetEvent {
    pub owner: u16,
}
impl PassGetEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassGetEvent {
            owner: read_value::<u16>(
                stream,
                definition.get_entry(12002927925776846068u64),
                "owner",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12002927925776846068u64 => Ok(self.owner.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassGet",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassScoreEvent {
    pub scorer: u16,
    pub assister: u16,
    pub points: u8,
}
impl PassScoreEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassScoreEvent {
            scorer: read_value::<u16>(
                stream,
                definition.get_entry(114960585656897893u64),
                "scorer",
            )?,
            assister: read_value::<u16>(
                stream,
                definition.get_entry(17978308754419261977u64),
                "assister",
            )?,
            points: read_value::<u8>(
                stream,
                definition.get_entry(15925482014108518566u64),
                "points",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            114960585656897893u64 => Ok(self.scorer.clone().into()),
            17978308754419261977u64 => Ok(self.assister.clone().into()),
            15925482014108518566u64 => Ok(self.points.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassScore",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassFreeEvent {
    pub owner: u16,
    pub attacker: u16,
}
impl PassFreeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassFreeEvent {
            owner: read_value::<u16>(
                stream,
                definition.get_entry(12002927925776846068u64),
                "owner",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12002927925776846068u64 => Ok(self.owner.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassFree",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassPassCaughtEvent {
    pub passer: u16,
    pub catcher: u16,
    pub dist: f32,
    pub duration: f32,
}
impl PassPassCaughtEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassPassCaughtEvent {
            passer: read_value::<u16>(
                stream,
                definition.get_entry(807676909013642383u64),
                "passer",
            )?,
            catcher: read_value::<u16>(
                stream,
                definition.get_entry(10052723730022137051u64),
                "catcher",
            )?,
            dist: read_value::<f32>(
                stream,
                definition.get_entry(14574961654905149033u64),
                "dist",
            )?,
            duration: read_value::<f32>(
                stream,
                definition.get_entry(10012068961515151501u64),
                "duration",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            807676909013642383u64 => Ok(self.passer.clone().into()),
            10052723730022137051u64 => Ok(self.catcher.clone().into()),
            14574961654905149033u64 => Ok(self.dist.clone().into()),
            10012068961515151501u64 => Ok(self.duration.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassPassCaught",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassBallStolenEvent {
    pub victim: u16,
    pub attacker: u16,
}
impl PassBallStolenEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassBallStolenEvent {
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            attacker: read_value::<u16>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassBallStolen",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PassBallBlockedEvent {
    pub owner: u16,
    pub blocker: u16,
}
impl PassBallBlockedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PassBallBlockedEvent {
            owner: read_value::<u16>(
                stream,
                definition.get_entry(12002927925776846068u64),
                "owner",
            )?,
            blocker: read_value::<u16>(
                stream,
                definition.get_entry(9150196623075249301u64),
                "blocker",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12002927925776846068u64 => Ok(self.owner.clone().into()),
            9150196623075249301u64 => Ok(self.blocker.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PassBallBlocked",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DamagePreventedEvent {
    pub preventor: u16,
    pub victim: u16,
    pub amount: u16,
    pub condition: u16,
}
impl DamagePreventedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DamagePreventedEvent {
            preventor: read_value::<u16>(
                stream,
                definition.get_entry(4517367470558074628u64),
                "preventor",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
            condition: read_value::<u16>(
                stream,
                definition.get_entry(15296338085916210270u64),
                "condition",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            4517367470558074628u64 => Ok(self.preventor.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            15296338085916210270u64 => Ok(self.condition.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DamagePrevented",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HalloweenBossKilledEvent {
    pub boss: u16,
    pub killer: u16,
}
impl HalloweenBossKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HalloweenBossKilledEvent {
            boss: read_value::<u16>(
                stream,
                definition.get_entry(14781339113128978092u64),
                "boss",
            )?,
            killer: read_value::<u16>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14781339113128978092u64 => Ok(self.boss.clone().into()),
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HalloweenBossKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EscapedLootIslandEvent {
    pub player: u16,
}
impl EscapedLootIslandEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EscapedLootIslandEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EscapedLootIsland",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TaggedPlayerAsItEvent {
    pub player: u16,
}
impl TaggedPlayerAsItEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TaggedPlayerAsItEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TaggedPlayerAsIt",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusStunnedEvent {
    pub player: u16,
}
impl MerasmusStunnedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusStunnedEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusStunned",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MerasmusPropFoundEvent {
    pub player: u16,
}
impl MerasmusPropFoundEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MerasmusPropFoundEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MerasmusPropFound",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HalloweenSkeletonKilledEvent {
    pub player: u16,
}
impl HalloweenSkeletonKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HalloweenSkeletonKilledEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HalloweenSkeletonKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SkeletonKilledQuestEvent {
    pub player: u16,
}
impl SkeletonKilledQuestEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SkeletonKilledQuestEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SkeletonKilledQuest",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SkeletonKingKilledQuestEvent {
    pub player: u16,
}
impl SkeletonKingKilledQuestEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SkeletonKingKilledQuestEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SkeletonKingKilledQuest",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EscapeHellEvent {
    pub player: u16,
}
impl EscapeHellEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(EscapeHellEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "EscapeHell",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CrossSpectralBridgeEvent {
    pub player: u16,
}
impl CrossSpectralBridgeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CrossSpectralBridgeEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CrossSpectralBridge",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MiniGameWonEvent {
    pub player: u16,
    pub game: u16,
}
impl MiniGameWonEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MiniGameWonEvent {
            player: read_value::<u16>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            game: read_value::<u16>(
                stream,
                definition.get_entry(10005491431272162599u64),
                "game",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            10005491431272162599u64 => Ok(self.game.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MiniGameWon",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RespawnGhostEvent {
    pub reviver: u16,
    pub ghost: u16,
}
impl RespawnGhostEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RespawnGhostEvent {
            reviver: read_value::<u16>(
                stream,
                definition.get_entry(12335255211204103206u64),
                "reviver",
            )?,
            ghost: read_value::<u16>(
                stream,
                definition.get_entry(7564851073369177808u64),
                "ghost",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12335255211204103206u64 => Ok(self.reviver.clone().into()),
            7564851073369177808u64 => Ok(self.ghost.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RespawnGhost",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct KillInHellEvent {
    pub killer: u16,
    pub victim: u16,
}
impl KillInHellEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(KillInHellEvent {
            killer: read_value::<u16>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "KillInHell",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HalloweenDuckCollectedEvent {
    pub collector: u16,
}
impl HalloweenDuckCollectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HalloweenDuckCollectedEvent {
            collector: read_value::<u16>(
                stream,
                definition.get_entry(10138905367994436056u64),
                "collector",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            10138905367994436056u64 => Ok(self.collector.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HalloweenDuckCollected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SpecialScoreEvent {
    pub player: u8,
}
impl SpecialScoreEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(SpecialScoreEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "SpecialScore",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TeamLeaderKilledEvent {
    pub killer: u8,
    pub victim: u8,
}
impl TeamLeaderKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TeamLeaderKilledEvent {
            killer: read_value::<u8>(
                stream,
                definition.get_entry(7927307102854403058u64),
                "killer",
            )?,
            victim: read_value::<u8>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7927307102854403058u64 => Ok(self.killer.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TeamLeaderKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HalloweenSoulCollectedEvent {
    pub intended_target: u8,
    pub collecting_player: u8,
    pub soul_count: u8,
}
impl HalloweenSoulCollectedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HalloweenSoulCollectedEvent {
            intended_target: read_value::<u8>(
                stream,
                definition.get_entry(6824378530660796482u64),
                "intended_target",
            )?,
            collecting_player: read_value::<u8>(
                stream,
                definition.get_entry(12517516231050690021u64),
                "collecting_player",
            )?,
            soul_count: read_value::<u8>(
                stream,
                definition.get_entry(715456465272287634u64),
                "soul_count",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6824378530660796482u64 => Ok(self.intended_target.clone().into()),
            12517516231050690021u64 => Ok(self.collecting_player.clone().into()),
            715456465272287634u64 => Ok(self.soul_count.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HalloweenSoulCollected",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RecalculateTruceEvent {}
impl RecalculateTruceEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RecalculateTruceEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RecalculateTruce",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DeadRingerCheatDeathEvent {
    pub spy: u8,
    pub attacker: u8,
}
impl DeadRingerCheatDeathEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DeadRingerCheatDeathEvent {
            spy: read_value::<u8>(stream, definition.get_entry(9394673858373567453u64), "spy")?,
            attacker: read_value::<u8>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9394673858373567453u64 => Ok(self.spy.clone().into()),
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DeadRingerCheatDeath",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CrossbowHealEvent {
    pub healer: u8,
    pub target: u8,
    pub amount: u16,
}
impl CrossbowHealEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CrossbowHealEvent {
            healer: read_value::<u8>(
                stream,
                definition.get_entry(9195440821534910520u64),
                "healer",
            )?,
            target: read_value::<u8>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9195440821534910520u64 => Ok(self.healer.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CrossbowHeal",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DamageMitigatedEvent {
    pub mitigator: u8,
    pub damaged: u8,
    pub amount: u16,
    pub item_definition_index: u16,
}
impl DamageMitigatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DamageMitigatedEvent {
            mitigator: read_value::<u8>(
                stream,
                definition.get_entry(6131810397813237369u64),
                "mitigator",
            )?,
            damaged: read_value::<u8>(
                stream,
                definition.get_entry(2938457768903959468u64),
                "damaged",
            )?,
            amount: read_value::<u16>(
                stream,
                definition.get_entry(9301057475299076457u64),
                "amount",
            )?,
            item_definition_index: read_value::<u16>(
                stream,
                definition.get_entry(4926523576391011283u64),
                "item_definition_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            6131810397813237369u64 => Ok(self.mitigator.clone().into()),
            2938457768903959468u64 => Ok(self.damaged.clone().into()),
            9301057475299076457u64 => Ok(self.amount.clone().into()),
            4926523576391011283u64 => Ok(self.item_definition_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DamageMitigated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PayloadPushedEvent {
    pub pusher: u8,
    pub distance: u16,
}
impl PayloadPushedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PayloadPushedEvent {
            pusher: read_value::<u8>(
                stream,
                definition.get_entry(16364632941784438830u64),
                "pusher",
            )?,
            distance: read_value::<u16>(
                stream,
                definition.get_entry(15956180055569954882u64),
                "distance",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16364632941784438830u64 => Ok(self.pusher.clone().into()),
            15956180055569954882u64 => Ok(self.distance.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PayloadPushed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerAbandonedMatchEvent {
    pub game_over: bool,
}
impl PlayerAbandonedMatchEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerAbandonedMatchEvent {
            game_over: read_value::<bool>(
                stream,
                definition.get_entry(17040732377939006530u64),
                "game_over",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17040732377939006530u64 => Ok(self.game_over.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerAbandonedMatch",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClDrawlineEvent {
    pub player: u8,
    pub panel: u8,
    pub line: u8,
    pub x: f32,
    pub y: f32,
}
impl ClDrawlineEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ClDrawlineEvent {
            player: read_value::<u8>(
                stream,
                definition.get_entry(5008278420455340480u64),
                "player",
            )?,
            panel: read_value::<u8>(
                stream,
                definition.get_entry(6829252156396663685u64),
                "panel",
            )?,
            line: read_value::<u8>(
                stream,
                definition.get_entry(13784293248712268039u64),
                "line",
            )?,
            x: read_value::<f32>(stream, definition.get_entry(12638214688346347271u64), "x")?,
            y: read_value::<f32>(stream, definition.get_entry(12638213588834719060u64), "y")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5008278420455340480u64 => Ok(self.player.clone().into()),
            6829252156396663685u64 => Ok(self.panel.clone().into()),
            13784293248712268039u64 => Ok(self.line.clone().into()),
            12638214688346347271u64 => Ok(self.x.clone().into()),
            12638213588834719060u64 => Ok(self.y.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ClDrawline",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RestartTimerTimeEvent {
    pub time: u8,
}
impl RestartTimerTimeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RestartTimerTimeEvent {
            time: read_value::<u8>(stream, definition.get_entry(2185518981507421060u64), "time")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            2185518981507421060u64 => Ok(self.time.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RestartTimerTime",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WinLimitChangedEvent {}
impl WinLimitChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(WinLimitChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "WinLimitChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WinPanelShowScoresEvent {}
impl WinPanelShowScoresEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(WinPanelShowScoresEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "WinPanelShowScores",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TopStreamsRequestFinishedEvent {}
impl TopStreamsRequestFinishedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(TopStreamsRequestFinishedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "TopStreamsRequestFinished",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CompetitiveStateChangedEvent {}
impl CompetitiveStateChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CompetitiveStateChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CompetitiveStateChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GlobalWarDataUpdatedEvent {}
impl GlobalWarDataUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GlobalWarDataUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GlobalWarDataUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StopWatchChangedEvent {}
impl StopWatchChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(StopWatchChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "StopWatchChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DsStopEvent {}
impl DsStopEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DsStopEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DsStop",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DsScreenshotEvent {
    pub delay: f32,
}
impl DsScreenshotEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(DsScreenshotEvent {
            delay: read_value::<f32>(
                stream,
                definition.get_entry(5892760507766377784u64),
                "delay",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            5892760507766377784u64 => Ok(self.delay.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "DsScreenshot",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShowMatchSummaryEvent {}
impl ShowMatchSummaryEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ShowMatchSummaryEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ShowMatchSummary",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ExperienceChangedEvent {}
impl ExperienceChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ExperienceChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ExperienceChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BeginXpLerpEvent {}
impl BeginXpLerpEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(BeginXpLerpEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "BeginXpLerp",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MatchmakerStatsUpdatedEvent {}
impl MatchmakerStatsUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MatchmakerStatsUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MatchmakerStatsUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RematchVotePeriodOverEvent {
    pub success: bool,
}
impl RematchVotePeriodOverEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RematchVotePeriodOverEvent {
            success: read_value::<bool>(
                stream,
                definition.get_entry(1872942068143117264u64),
                "success",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            1872942068143117264u64 => Ok(self.success.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RematchVotePeriodOver",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RematchFailedToCreateEvent {}
impl RematchFailedToCreateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(RematchFailedToCreateEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "RematchFailedToCreate",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerRematchChangeEvent {}
impl PlayerRematchChangeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerRematchChangeEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerRematchChange",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PingUpdatedEvent {}
impl PingUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PingUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PingUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MMStatsUpdatedEvent {}
impl MMStatsUpdatedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MMStatsUpdatedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MMStatsUpdated",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerNextMapVoteChangeEvent {
    pub map_index: u8,
    pub vote: u8,
}
impl PlayerNextMapVoteChangeEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerNextMapVoteChangeEvent {
            map_index: read_value::<u8>(
                stream,
                definition.get_entry(14809078982718236190u64),
                "map_index",
            )?,
            vote: read_value::<u8>(stream, definition.get_entry(3543466495390456501u64), "vote")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            14809078982718236190u64 => Ok(self.map_index.clone().into()),
            3543466495390456501u64 => Ok(self.vote.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerNextMapVoteChange",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct VoteMapsChangedEvent {}
impl VoteMapsChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(VoteMapsChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "VoteMapsChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ProtoDefChangedEvent {
    pub kind: u8,
    pub definition_index: u32,
    pub created: bool,
    pub deleted: bool,
    pub erase_history: bool,
}
impl ProtoDefChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ProtoDefChangedEvent {
            kind: read_value::<u8>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
            definition_index: read_value::<u32>(
                stream,
                definition.get_entry(13641141875887813064u64),
                "definition_index",
            )?,
            created: read_value::<bool>(
                stream,
                definition.get_entry(16258158586571091195u64),
                "created",
            )?,
            deleted: read_value::<bool>(
                stream,
                definition.get_entry(2244448350166952426u64),
                "deleted",
            )?,
            erase_history: read_value::<bool>(
                stream,
                definition.get_entry(7145567783783222582u64),
                "erase_history",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            13641141875887813064u64 => Ok(self.definition_index.clone().into()),
            16258158586571091195u64 => Ok(self.created.clone().into()),
            2244448350166952426u64 => Ok(self.deleted.clone().into()),
            7145567783783222582u64 => Ok(self.erase_history.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ProtoDefChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerDominationEvent {
    pub dominator: u16,
    pub dominated: u16,
    pub dominations: u16,
}
impl PlayerDominationEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerDominationEvent {
            dominator: read_value::<u16>(
                stream,
                definition.get_entry(738416687693650492u64),
                "dominator",
            )?,
            dominated: read_value::<u16>(
                stream,
                definition.get_entry(744074774531288948u64),
                "dominated",
            )?,
            dominations: read_value::<u16>(
                stream,
                definition.get_entry(14151598225611438354u64),
                "dominations",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            738416687693650492u64 => Ok(self.dominator.clone().into()),
            744074774531288948u64 => Ok(self.dominated.clone().into()),
            14151598225611438354u64 => Ok(self.dominations.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerDomination",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PlayerRocketPackPushedEvent {
    pub pusher: u16,
    pub pushed: u16,
}
impl PlayerRocketPackPushedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(PlayerRocketPackPushedEvent {
            pusher: read_value::<u16>(
                stream,
                definition.get_entry(16364632941784438830u64),
                "pusher",
            )?,
            pushed: read_value::<u16>(
                stream,
                definition.get_entry(16364617548621643876u64),
                "pushed",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            16364632941784438830u64 => Ok(self.pusher.clone().into()),
            16364617548621643876u64 => Ok(self.pushed.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "PlayerRocketPackPushed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestRequestEvent {
    pub request: u32,
    pub msg: MaybeUtf8String,
}
impl QuestRequestEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestRequestEvent {
            request: read_value::<u32>(
                stream,
                definition.get_entry(8018452122269601458u64),
                "request",
            )?,
            msg: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(567552616859811458u64),
                "msg",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8018452122269601458u64 => Ok(self.request.clone().into()),
            567552616859811458u64 => Ok(self.msg.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestRequest",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestResponseEvent {
    pub request: u32,
    pub success: bool,
    pub msg: MaybeUtf8String,
}
impl QuestResponseEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestResponseEvent {
            request: read_value::<u32>(
                stream,
                definition.get_entry(8018452122269601458u64),
                "request",
            )?,
            success: read_value::<bool>(
                stream,
                definition.get_entry(1872942068143117264u64),
                "success",
            )?,
            msg: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(567552616859811458u64),
                "msg",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            8018452122269601458u64 => Ok(self.request.clone().into()),
            1872942068143117264u64 => Ok(self.success.clone().into()),
            567552616859811458u64 => Ok(self.msg.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestResponse",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestProgressEvent {
    pub owner: u16,
    pub scorer: u16,
    pub kind: u8,
    pub completed: bool,
    pub quest_definition_index: u32,
}
impl QuestProgressEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestProgressEvent {
            owner: read_value::<u16>(
                stream,
                definition.get_entry(12002927925776846068u64),
                "owner",
            )?,
            scorer: read_value::<u16>(
                stream,
                definition.get_entry(114960585656897893u64),
                "scorer",
            )?,
            kind: read_value::<u8>(
                stream,
                definition.get_entry(12075340201627130925u64),
                "kind",
            )?,
            completed: read_value::<bool>(
                stream,
                definition.get_entry(18293112347497300626u64),
                "completed",
            )?,
            quest_definition_index: read_value::<u32>(
                stream,
                definition.get_entry(16190323556600580467u64),
                "quest_definition_index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            12002927925776846068u64 => Ok(self.owner.clone().into()),
            114960585656897893u64 => Ok(self.scorer.clone().into()),
            12075340201627130925u64 => Ok(self.kind.clone().into()),
            18293112347497300626u64 => Ok(self.completed.clone().into()),
            16190323556600580467u64 => Ok(self.quest_definition_index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestProgress",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ProjectileRemovedEvent {
    pub attacker: u8,
    pub weapon_def_index: u32,
    pub num_hit: u8,
    pub num_direct_hit: u8,
}
impl ProjectileRemovedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ProjectileRemovedEvent {
            attacker: read_value::<u8>(
                stream,
                definition.get_entry(7198542740550218478u64),
                "attacker",
            )?,
            weapon_def_index: read_value::<u32>(
                stream,
                definition.get_entry(4132306594868589054u64),
                "weapon_def_index",
            )?,
            num_hit: read_value::<u8>(
                stream,
                definition.get_entry(12553580167082885753u64),
                "num_hit",
            )?,
            num_direct_hit: read_value::<u8>(
                stream,
                definition.get_entry(8604189910367298615u64),
                "num_direct_hit",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7198542740550218478u64 => Ok(self.attacker.clone().into()),
            4132306594868589054u64 => Ok(self.weapon_def_index.clone().into()),
            12553580167082885753u64 => Ok(self.num_hit.clone().into()),
            8604189910367298615u64 => Ok(self.num_direct_hit.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ProjectileRemoved",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestMapDataChangedEvent {}
impl QuestMapDataChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestMapDataChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestMapDataChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GasDousedPlayerIgnitedEvent {
    pub igniter: u16,
    pub douser: u16,
    pub victim: u16,
}
impl GasDousedPlayerIgnitedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(GasDousedPlayerIgnitedEvent {
            igniter: read_value::<u16>(
                stream,
                definition.get_entry(9421640134696329943u64),
                "igniter",
            )?,
            douser: read_value::<u16>(
                stream,
                definition.get_entry(888696338692993559u64),
                "douser",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9421640134696329943u64 => Ok(self.igniter.clone().into()),
            888696338692993559u64 => Ok(self.douser.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "GasDousedPlayerIgnited",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct QuestTurnInStateEvent {
    pub state: u16,
}
impl QuestTurnInStateEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(QuestTurnInStateEvent {
            state: read_value::<u16>(
                stream,
                definition.get_entry(17177761064896540950u64),
                "state",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17177761064896540950u64 => Ok(self.state.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "QuestTurnInState",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ItemsAcknowledgedEvent {}
impl ItemsAcknowledgedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ItemsAcknowledgedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ItemsAcknowledged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CapperKilledEvent {
    pub blocker: u16,
    pub victim: u16,
}
impl CapperKilledEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(CapperKilledEvent {
            blocker: read_value::<u16>(
                stream,
                definition.get_entry(9150196623075249301u64),
                "blocker",
            )?,
            victim: read_value::<u16>(
                stream,
                definition.get_entry(3120917251440744469u64),
                "victim",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9150196623075249301u64 => Ok(self.blocker.clone().into()),
            3120917251440744469u64 => Ok(self.victim.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "CapperKilled",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MainMenuStabilizedEvent {}
impl MainMenuStabilizedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(MainMenuStabilizedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "MainMenuStabilized",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WorldStatusChangedEvent {}
impl WorldStatusChangedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(WorldStatusChangedEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "WorldStatusChanged",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVStatusEvent {
    pub clients: u32,
    pub slots: u32,
    pub proxies: u16,
    pub master: MaybeUtf8String,
}
impl HLTVStatusEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVStatusEvent {
            clients: read_value::<u32>(
                stream,
                definition.get_entry(7649887508575031031u64),
                "clients",
            )?,
            slots: read_value::<u32>(
                stream,
                definition.get_entry(16612704165544482374u64),
                "slots",
            )?,
            proxies: read_value::<u16>(
                stream,
                definition.get_entry(15307552237670760667u64),
                "proxies",
            )?,
            master: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(17259658682407053043u64),
                "master",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            7649887508575031031u64 => Ok(self.clients.clone().into()),
            16612704165544482374u64 => Ok(self.slots.clone().into()),
            15307552237670760667u64 => Ok(self.proxies.clone().into()),
            17259658682407053043u64 => Ok(self.master.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVStatus",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVCameramanEvent {
    pub index: u16,
}
impl HLTVCameramanEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVCameramanEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVCameraman",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVRankCameraEvent {
    pub index: u8,
    pub rank: f32,
    pub target: u16,
}
impl HLTVRankCameraEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVRankCameraEvent {
            index: read_value::<u8>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            rank: read_value::<f32>(stream, definition.get_entry(7904697092003280967u64), "rank")?,
            target: read_value::<u16>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            7904697092003280967u64 => Ok(self.rank.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVRankCamera",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVRankEntityEvent {
    pub index: u16,
    pub rank: f32,
    pub target: u16,
}
impl HLTVRankEntityEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVRankEntityEvent {
            index: read_value::<u16>(
                stream,
                definition.get_entry(9497966886403524235u64),
                "index",
            )?,
            rank: read_value::<f32>(stream, definition.get_entry(7904697092003280967u64), "rank")?,
            target: read_value::<u16>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            9497966886403524235u64 => Ok(self.index.clone().into()),
            7904697092003280967u64 => Ok(self.rank.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVRankEntity",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVFixedEvent {
    pub pos_x: u32,
    pub pos_y: u32,
    pub pos_z: u32,
    pub theta: u16,
    pub phi: u16,
    pub offset: u16,
    pub fov: f32,
    pub target: u16,
}
impl HLTVFixedEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVFixedEvent {
            pos_x: read_value::<u32>(
                stream,
                definition.get_entry(10101535539947993315u64),
                "pos_x",
            )?,
            pos_y: read_value::<u32>(
                stream,
                definition.get_entry(10101534440436365104u64),
                "pos_y",
            )?,
            pos_z: read_value::<u32>(
                stream,
                definition.get_entry(10101537738971249737u64),
                "pos_z",
            )?,
            theta: read_value::<u16>(
                stream,
                definition.get_entry(16923039152111475483u64),
                "theta",
            )?,
            phi: read_value::<u16>(stream, definition.get_entry(8625054201726242788u64), "phi")?,
            offset: read_value::<u16>(
                stream,
                definition.get_entry(173583165163845066u64),
                "offset",
            )?,
            fov: read_value::<f32>(stream, definition.get_entry(15902909680995393884u64), "fov")?,
            target: read_value::<u16>(
                stream,
                definition.get_entry(1653916590517707752u64),
                "target",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            10101535539947993315u64 => Ok(self.pos_x.clone().into()),
            10101534440436365104u64 => Ok(self.pos_y.clone().into()),
            10101537738971249737u64 => Ok(self.pos_z.clone().into()),
            16923039152111475483u64 => Ok(self.theta.clone().into()),
            8625054201726242788u64 => Ok(self.phi.clone().into()),
            173583165163845066u64 => Ok(self.offset.clone().into()),
            15902909680995393884u64 => Ok(self.fov.clone().into()),
            1653916590517707752u64 => Ok(self.target.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVFixed",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVChaseEvent {
    pub target_1: u16,
    pub target_2: u16,
    pub distance: u16,
    pub theta: u16,
    pub phi: u16,
    pub inertia: u8,
    pub in_eye: u8,
}
impl HLTVChaseEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVChaseEvent {
            target_1: read_value::<u16>(
                stream,
                definition.get_entry(17244538795293747643u64),
                "target_1",
            )?,
            target_2: read_value::<u16>(
                stream,
                definition.get_entry(17244539894805375854u64),
                "target_2",
            )?,
            distance: read_value::<u16>(
                stream,
                definition.get_entry(15956180055569954882u64),
                "distance",
            )?,
            theta: read_value::<u16>(
                stream,
                definition.get_entry(16923039152111475483u64),
                "theta",
            )?,
            phi: read_value::<u16>(stream, definition.get_entry(8625054201726242788u64), "phi")?,
            inertia: read_value::<u8>(
                stream,
                definition.get_entry(16127751084747526033u64),
                "inertia",
            )?,
            in_eye: read_value::<u8>(
                stream,
                definition.get_entry(10269529380097519287u64),
                "in_eye",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            17244538795293747643u64 => Ok(self.target_1.clone().into()),
            17244539894805375854u64 => Ok(self.target_2.clone().into()),
            15956180055569954882u64 => Ok(self.distance.clone().into()),
            16923039152111475483u64 => Ok(self.theta.clone().into()),
            8625054201726242788u64 => Ok(self.phi.clone().into()),
            16127751084747526033u64 => Ok(self.inertia.clone().into()),
            10269529380097519287u64 => Ok(self.in_eye.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVChase",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVMessageEvent {
    pub text: MaybeUtf8String,
}
impl HLTVMessageEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVMessageEvent {
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVMessage",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVTitleEvent {
    pub text: MaybeUtf8String,
}
impl HLTVTitleEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVTitleEvent {
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVTitle",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct HLTVChatEvent {
    pub text: MaybeUtf8String,
}
impl HLTVChatEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(HLTVChatEvent {
            text: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(18015793717152399486u64),
                "text",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            18015793717152399486u64 => Ok(self.text.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "HLTVChat",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplayStartRecordEvent {}
impl ReplayStartRecordEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplayStartRecordEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplayStartRecord",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplaySessionInfoEvent {
    pub sn: MaybeUtf8String,
    pub di: u8,
    pub cb: u32,
    pub st: u32,
}
impl ReplaySessionInfoEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplaySessionInfoEvent {
            sn: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(637602428010181156u64),
                "sn",
            )?,
            di: read_value::<u8>(stream, definition.get_entry(617370314543444270u64), "di")?,
            cb: read_value::<u32>(stream, definition.get_entry(622143294520562096u64), "cb")?,
            st: read_value::<u32>(stream, definition.get_entry(637613423126463266u64), "st")?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            637602428010181156u64 => Ok(self.sn.clone().into()),
            617370314543444270u64 => Ok(self.di.clone().into()),
            622143294520562096u64 => Ok(self.cb.clone().into()),
            637613423126463266u64 => Ok(self.st.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplaySessionInfo",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplayEndRecordEvent {}
impl ReplayEndRecordEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplayEndRecordEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplayEndRecord",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplayReplaysAvailableEvent {}
impl ReplayReplaysAvailableEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplayReplaysAvailableEvent {})
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplayReplaysAvailable",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ReplayServerErrorEvent {
    pub error: MaybeUtf8String,
}
impl ReplayServerErrorEvent {
    #[allow(unused_variables)]
    fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(ReplayServerErrorEvent {
            error: read_value::<MaybeUtf8String>(
                stream,
                definition.get_entry(11489899660447141169u64),
                "error",
            )?,
        })
    }
    #[allow(unused_variables)]
    fn get_field(&self, field: &GameEventEntry) -> Result<GameEventValue> {
        #[allow(clippy::clone_on_copy, clippy::match_single_binding)]
        match field.hash {
            11489899660447141169u64 => Ok(self.error.clone().into()),
            _ => Err(ParseError::MissingGameEventValue {
                ty: "ReplayServerError",
                field: "todo".into(),
            }),
        }
    }
    #[allow(unused_variables)]
    fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        for entry in &definition.entries {
            let value = self
                .get_field(entry)
                .unwrap_or_else(|_| entry.kind.default_value());
            stream.write(&value)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum GameEvent {
    ServerSpawn(Box<ServerSpawnEvent>),
    ServerChangeLevelFailed(ServerChangeLevelFailedEvent),
    ServerShutdown(ServerShutdownEvent),
    ServerCvar(ServerCvarEvent),
    ServerMessage(ServerMessageEvent),
    ServerAddBan(Box<ServerAddBanEvent>),
    ServerRemoveBan(ServerRemoveBanEvent),
    PlayerConnect(PlayerConnectEvent),
    PlayerConnectClient(PlayerConnectClientEvent),
    PlayerInfo(PlayerInfoEvent),
    PlayerDisconnect(PlayerDisconnectEvent),
    PlayerActivate(PlayerActivateEvent),
    PlayerSay(PlayerSayEvent),
    ClientDisconnect(ClientDisconnectEvent),
    ClientBeginConnect(ClientBeginConnectEvent),
    ClientConnected(ClientConnectedEvent),
    ClientFullConnect(ClientFullConnectEvent),
    HostQuit(HostQuitEvent),
    TeamInfo(TeamInfoEvent),
    TeamScore(TeamScoreEvent),
    TeamPlayBroadcastAudio(TeamPlayBroadcastAudioEvent),
    PlayerTeam(PlayerTeamEvent),
    PlayerClass(PlayerClassEvent),
    PlayerDeath(Box<PlayerDeathEvent>),
    PlayerHurt(PlayerHurtEvent),
    PlayerChat(PlayerChatEvent),
    PlayerScore(PlayerScoreEvent),
    PlayerSpawn(PlayerSpawnEvent),
    PlayerShoot(PlayerShootEvent),
    PlayerUse(PlayerUseEvent),
    PlayerChangeName(PlayerChangeNameEvent),
    PlayerHintMessage(PlayerHintMessageEvent),
    BasePlayerTeleported(BasePlayerTeleportedEvent),
    GameInit(GameInitEvent),
    GameNewMap(GameNewMapEvent),
    GameStart(GameStartEvent),
    GameEnd(GameEndEvent),
    RoundStart(RoundStartEvent),
    RoundEnd(RoundEndEvent),
    GameMessage(GameMessageEvent),
    BreakBreakable(BreakBreakableEvent),
    BreakProp(BreakPropEvent),
    EntityKilled(EntityKilledEvent),
    BonusUpdated(BonusUpdatedEvent),
    AchievementEvent(AchievementEventEvent),
    AchievementIncrement(AchievementIncrementEvent),
    PhysgunPickup(PhysgunPickupEvent),
    FlareIgniteNpc(FlareIgniteNpcEvent),
    HelicopterGrenadePuntMiss(HelicopterGrenadePuntMissEvent),
    UserDataDownloaded(UserDataDownloadedEvent),
    RagdollDissolved(RagdollDissolvedEvent),
    HLTVChangedMode(HLTVChangedModeEvent),
    HLTVChangedTarget(HLTVChangedTargetEvent),
    VoteEnded(VoteEndedEvent),
    VoteStarted(VoteStartedEvent),
    VoteChanged(VoteChangedEvent),
    VotePassed(VotePassedEvent),
    VoteFailed(VoteFailedEvent),
    VoteCast(VoteCastEvent),
    VoteOptions(Box<VoteOptionsEvent>),
    ReplaySaved(ReplaySavedEvent),
    EnteredPerformanceMode(EnteredPerformanceModeEvent),
    BrowseReplays(BrowseReplaysEvent),
    ReplayYoutubeStats(ReplayYoutubeStatsEvent),
    InventoryUpdated(InventoryUpdatedEvent),
    CartUpdated(CartUpdatedEvent),
    StorePriceSheetUpdated(StorePriceSheetUpdatedEvent),
    EconInventoryConnected(EconInventoryConnectedEvent),
    ItemSchemaInitialized(ItemSchemaInitializedEvent),
    GcNewSession(GcNewSessionEvent),
    GcLostSession(GcLostSessionEvent),
    IntroFinish(IntroFinishEvent),
    IntroNextCamera(IntroNextCameraEvent),
    PlayerChangeClass(PlayerChangeClassEvent),
    TfMapTimeRemaining(TfMapTimeRemainingEvent),
    TfGameOver(TfGameOverEvent),
    CtfFlagCaptured(CtfFlagCapturedEvent),
    ControlPointInitialized(ControlPointInitializedEvent),
    ControlPointUpdateImages(ControlPointUpdateImagesEvent),
    ControlPointUpdateLayout(ControlPointUpdateLayoutEvent),
    ControlPointUpdateCapping(ControlPointUpdateCappingEvent),
    ControlPointUpdateOwner(ControlPointUpdateOwnerEvent),
    ControlPointStartTouch(ControlPointStartTouchEvent),
    ControlPointEndTouch(ControlPointEndTouchEvent),
    ControlPointPulseElement(ControlPointPulseElementEvent),
    ControlPointFakeCapture(ControlPointFakeCaptureEvent),
    ControlPointFakeCaptureMultiplier(ControlPointFakeCaptureMultiplierEvent),
    TeamPlayRoundSelected(TeamPlayRoundSelectedEvent),
    TeamPlayRoundStart(TeamPlayRoundStartEvent),
    TeamPlayRoundActive(TeamPlayRoundActiveEvent),
    TeamPlayWaitingBegins(TeamPlayWaitingBeginsEvent),
    TeamPlayWaitingEnds(TeamPlayWaitingEndsEvent),
    TeamPlayWaitingAboutToEnd(TeamPlayWaitingAboutToEndEvent),
    TeamPlayRestartRound(TeamPlayRestartRoundEvent),
    TeamPlayReadyRestart(TeamPlayReadyRestartEvent),
    TeamPlayRoundRestartSeconds(TeamPlayRoundRestartSecondsEvent),
    TeamPlayTeamReady(TeamPlayTeamReadyEvent),
    TeamPlayRoundWin(TeamPlayRoundWinEvent),
    TeamPlayUpdateTimer(TeamPlayUpdateTimerEvent),
    TeamPlayRoundStalemate(TeamPlayRoundStalemateEvent),
    TeamPlayOvertimeBegin(TeamPlayOvertimeBeginEvent),
    TeamPlayOvertimeEnd(TeamPlayOvertimeEndEvent),
    TeamPlaySuddenDeathBegin(TeamPlaySuddenDeathBeginEvent),
    TeamPlaySuddenDeathEnd(TeamPlaySuddenDeathEndEvent),
    TeamPlayGameOver(TeamPlayGameOverEvent),
    TeamPlayMapTimeRemaining(TeamPlayMapTimeRemainingEvent),
    TeamPlayTimerFlash(TeamPlayTimerFlashEvent),
    TeamPlayTimerTimeAdded(TeamPlayTimerTimeAddedEvent),
    TeamPlayPointStartCapture(TeamPlayPointStartCaptureEvent),
    TeamPlayPointCaptured(TeamPlayPointCapturedEvent),
    TeamPlayPointLocked(TeamPlayPointLockedEvent),
    TeamPlayPointUnlocked(TeamPlayPointUnlockedEvent),
    TeamPlayCaptureBroken(TeamPlayCaptureBrokenEvent),
    TeamPlayCaptureBlocked(TeamPlayCaptureBlockedEvent),
    TeamPlayFlagEvent(TeamPlayFlagEventEvent),
    TeamPlayWinPanel(TeamPlayWinPanelEvent),
    TeamPlayTeamBalancedPlayer(TeamPlayTeamBalancedPlayerEvent),
    TeamPlaySetupFinished(TeamPlaySetupFinishedEvent),
    TeamPlayAlert(TeamPlayAlertEvent),
    TrainingComplete(TrainingCompleteEvent),
    ShowFreezePanel(ShowFreezePanelEvent),
    HideFreezePanel(HideFreezePanelEvent),
    FreezeCamStarted(FreezeCamStartedEvent),
    LocalPlayerChangeTeam(LocalPlayerChangeTeamEvent),
    LocalPlayerScoreChanged(LocalPlayerScoreChangedEvent),
    LocalPlayerChangeClass(LocalPlayerChangeClassEvent),
    LocalPlayerRespawn(LocalPlayerRespawnEvent),
    BuildingInfoChanged(BuildingInfoChangedEvent),
    LocalPlayerChangeDisguise(LocalPlayerChangeDisguiseEvent),
    PlayerAccountChanged(PlayerAccountChangedEvent),
    SpyPdaReset(SpyPdaResetEvent),
    FlagStatusUpdate(FlagStatusUpdateEvent),
    PlayerStatsUpdated(PlayerStatsUpdatedEvent),
    PlayingCommentary(PlayingCommentaryEvent),
    PlayerChargeDeployed(PlayerChargeDeployedEvent),
    PlayerBuiltObject(PlayerBuiltObjectEvent),
    PlayerUpgradedObject(PlayerUpgradedObjectEvent),
    PlayerCarryObject(PlayerCarryObjectEvent),
    PlayerDropObject(PlayerDropObjectEvent),
    ObjectRemoved(ObjectRemovedEvent),
    ObjectDestroyed(ObjectDestroyedEvent),
    ObjectDetonated(ObjectDetonatedEvent),
    AchievementEarned(AchievementEarnedEvent),
    SpecTargetUpdated(SpecTargetUpdatedEvent),
    TournamentStateUpdate(TournamentStateUpdateEvent),
    TournamentEnableCountdown(TournamentEnableCountdownEvent),
    PlayerCalledForMedic(PlayerCalledForMedicEvent),
    PlayerAskedForBall(PlayerAskedForBallEvent),
    LocalPlayerBecameObserver(LocalPlayerBecameObserverEvent),
    PlayerIgnitedInv(PlayerIgnitedInvEvent),
    PlayerIgnited(PlayerIgnitedEvent),
    PlayerExtinguished(PlayerExtinguishedEvent),
    PlayerTeleported(PlayerTeleportedEvent),
    PlayerHealedMedicCall(PlayerHealedMedicCallEvent),
    LocalPlayerChargeReady(LocalPlayerChargeReadyEvent),
    LocalPlayerWindDown(LocalPlayerWindDownEvent),
    PlayerInvulned(PlayerInvulnedEvent),
    EscortSpeed(EscortSpeedEvent),
    EscortProgress(EscortProgressEvent),
    EscortRecede(EscortRecedeEvent),
    GameUIActivated(GameUIActivatedEvent),
    GameUIHidden(GameUIHiddenEvent),
    PlayerEscortScore(PlayerEscortScoreEvent),
    PlayerHealOnHit(PlayerHealOnHitEvent),
    PlayerStealSandvich(PlayerStealSandvichEvent),
    ShowClassLayout(ShowClassLayoutEvent),
    ShowVsPanel(ShowVsPanelEvent),
    PlayerDamaged(PlayerDamagedEvent),
    ArenaPlayerNotification(ArenaPlayerNotificationEvent),
    ArenaMatchMaxStreak(ArenaMatchMaxStreakEvent),
    ArenaRoundStart(ArenaRoundStartEvent),
    ArenaWinPanel(ArenaWinPanelEvent),
    PveWinPanel(PveWinPanelEvent),
    AirDash(AirDashEvent),
    Landed(LandedEvent),
    PlayerDamageDodged(PlayerDamageDodgedEvent),
    PlayerStunned(PlayerStunnedEvent),
    ScoutGrandSlam(ScoutGrandSlamEvent),
    ScoutSlamdollLanded(ScoutSlamdollLandedEvent),
    ArrowImpact(ArrowImpactEvent),
    PlayerJarated(PlayerJaratedEvent),
    PlayerJaratedFade(PlayerJaratedFadeEvent),
    PlayerShieldBlocked(PlayerShieldBlockedEvent),
    PlayerPinned(PlayerPinnedEvent),
    PlayerHealedByMedic(PlayerHealedByMedicEvent),
    PlayerSappedObject(PlayerSappedObjectEvent),
    ItemFound(ItemFoundEvent),
    ShowAnnotation(ShowAnnotationEvent),
    HideAnnotation(HideAnnotationEvent),
    PostInventoryApplication(PostInventoryApplicationEvent),
    ControlPointUnlockUpdated(ControlPointUnlockUpdatedEvent),
    DeployBuffBanner(DeployBuffBannerEvent),
    PlayerBuff(PlayerBuffEvent),
    MedicDeath(MedicDeathEvent),
    OvertimeNag(OvertimeNagEvent),
    TeamsChanged(TeamsChangedEvent),
    HalloweenPumpkinGrab(HalloweenPumpkinGrabEvent),
    RocketJump(RocketJumpEvent),
    RocketJumpLanded(RocketJumpLandedEvent),
    StickyJump(StickyJumpEvent),
    StickyJumpLanded(StickyJumpLandedEvent),
    RocketPackLaunch(RocketPackLaunchEvent),
    RocketPackLanded(RocketPackLandedEvent),
    MedicDefended(MedicDefendedEvent),
    LocalPlayerHealed(LocalPlayerHealedEvent),
    PlayerDestroyedPipeBomb(PlayerDestroyedPipeBombEvent),
    ObjectDeflected(ObjectDeflectedEvent),
    PlayerMvp(PlayerMvpEvent),
    RaidSpawnMob(RaidSpawnMobEvent),
    RaidSpawnSquad(RaidSpawnSquadEvent),
    NavBlocked(NavBlockedEvent),
    PathTrackPassed(PathTrackPassedEvent),
    NumCappersChanged(NumCappersChangedEvent),
    PlayerRegenerate(PlayerRegenerateEvent),
    UpdateStatusItem(UpdateStatusItemEvent),
    StatsResetRound(StatsResetRoundEvent),
    ScoreStatsAccumulatedUpdate(ScoreStatsAccumulatedUpdateEvent),
    ScoreStatsAccumulatedReset(ScoreStatsAccumulatedResetEvent),
    AchievementEarnedLocal(AchievementEarnedLocalEvent),
    PlayerHealed(PlayerHealedEvent),
    BuildingHealed(BuildingHealedEvent),
    ItemPickup(ItemPickupEvent),
    DuelStatus(DuelStatusEvent),
    FishNotice(Box<FishNoticeEvent>),
    FishNoticeArm(Box<FishNoticeArmEvent>),
    SlapNotice(Box<SlapNoticeEvent>),
    ThrowableHit(Box<ThrowableHitEvent>),
    PumpkinLordSummoned(PumpkinLordSummonedEvent),
    PumpkinLordKilled(PumpkinLordKilledEvent),
    MerasmusSummoned(MerasmusSummonedEvent),
    MerasmusKilled(MerasmusKilledEvent),
    MerasmusEscapeWarning(MerasmusEscapeWarningEvent),
    MerasmusEscaped(MerasmusEscapedEvent),
    EyeballBossSummoned(EyeballBossSummonedEvent),
    EyeballBossStunned(EyeballBossStunnedEvent),
    EyeballBossKilled(EyeballBossKilledEvent),
    EyeballBossKiller(EyeballBossKillerEvent),
    EyeballBossEscapeImminent(EyeballBossEscapeImminentEvent),
    EyeballBossEscaped(EyeballBossEscapedEvent),
    NpcHurt(NpcHurtEvent),
    ControlPointTimerUpdated(ControlPointTimerUpdatedEvent),
    PlayerHighFiveStart(PlayerHighFiveStartEvent),
    PlayerHighFiveCancel(PlayerHighFiveCancelEvent),
    PlayerHighFiveSuccess(PlayerHighFiveSuccessEvent),
    PlayerBonusPoints(PlayerBonusPointsEvent),
    PlayerUpgraded(PlayerUpgradedEvent),
    PlayerBuyback(PlayerBuybackEvent),
    PlayerUsedPowerUpBottle(PlayerUsedPowerUpBottleEvent),
    ChristmasGiftGrab(ChristmasGiftGrabEvent),
    PlayerKilledAchievementZone(PlayerKilledAchievementZoneEvent),
    PartyUpdated(PartyUpdatedEvent),
    PartyPrefChanged(PartyPrefChangedEvent),
    PartyCriteriaChanged(PartyCriteriaChangedEvent),
    PartyInvitesChanged(PartyInvitesChangedEvent),
    PartyQueueStateChanged(PartyQueueStateChangedEvent),
    PartyChat(PartyChatEvent),
    PartyMemberJoin(PartyMemberJoinEvent),
    PartyMemberLeave(PartyMemberLeaveEvent),
    MatchInvitesUpdated(MatchInvitesUpdatedEvent),
    LobbyUpdated(LobbyUpdatedEvent),
    MvmMissionUpdate(MvmMissionUpdateEvent),
    RecalculateHolidays(RecalculateHolidaysEvent),
    PlayerCurrencyChanged(PlayerCurrencyChangedEvent),
    DoomsdayRocketOpen(DoomsdayRocketOpenEvent),
    RemoveNemesisRelationships(RemoveNemesisRelationshipsEvent),
    MvmCreditBonusWave(MvmCreditBonusWaveEvent),
    MvmCreditBonusAll(MvmCreditBonusAllEvent),
    MvmCreditBonusAllAdvanced(MvmCreditBonusAllAdvancedEvent),
    MvmQuickSentryUpgrade(MvmQuickSentryUpgradeEvent),
    MvmTankDestroyedByPlayers(MvmTankDestroyedByPlayersEvent),
    MvmKillRobotDeliveringBomb(MvmKillRobotDeliveringBombEvent),
    MvmPickupCurrency(MvmPickupCurrencyEvent),
    MvmBombCarrierKilled(MvmBombCarrierKilledEvent),
    MvmSentryBusterDetonate(MvmSentryBusterDetonateEvent),
    MvmScoutMarkedForDeath(MvmScoutMarkedForDeathEvent),
    MvmMedicPowerUpShared(MvmMedicPowerUpSharedEvent),
    MvmBeginWave(MvmBeginWaveEvent),
    MvmWaveComplete(MvmWaveCompleteEvent),
    MvmMissionComplete(MvmMissionCompleteEvent),
    MvmBombResetByPlayer(MvmBombResetByPlayerEvent),
    MvmBombAlarmTriggered(MvmBombAlarmTriggeredEvent),
    MvmBombDeployResetByPlayer(MvmBombDeployResetByPlayerEvent),
    MvmWaveFailed(MvmWaveFailedEvent),
    MvmResetStats(MvmResetStatsEvent),
    DamageResisted(DamageResistedEvent),
    RevivePlayerNotify(RevivePlayerNotifyEvent),
    RevivePlayerStopped(RevivePlayerStoppedEvent),
    RevivePlayerComplete(RevivePlayerCompleteEvent),
    PlayerTurnedToGhost(PlayerTurnedToGhostEvent),
    MedigunShieldBlockedDamage(MedigunShieldBlockedDamageEvent),
    MvmAdvWaveCompleteNoGates(MvmAdvWaveCompleteNoGatesEvent),
    MvmSniperHeadshotCurrency(MvmSniperHeadshotCurrencyEvent),
    MvmMannhattanPit(MvmMannhattanPitEvent),
    FlagCarriedInDetectionZone(FlagCarriedInDetectionZoneEvent),
    MvmAdvWaveKilledStunRadio(MvmAdvWaveKilledStunRadioEvent),
    PlayerDirectHitStun(PlayerDirectHitStunEvent),
    MvmSentryBusterKilled(MvmSentryBusterKilledEvent),
    UpgradesFileChanged(UpgradesFileChangedEvent),
    RdTeamPointsChanged(RdTeamPointsChangedEvent),
    RdRulesStateChanged(RdRulesStateChangedEvent),
    RdRobotKilled(RdRobotKilledEvent),
    RdRobotImpact(RdRobotImpactEvent),
    TeamPlayPreRoundTimeLeft(TeamPlayPreRoundTimeLeftEvent),
    ParachuteDeploy(ParachuteDeployEvent),
    ParachuteHolster(ParachuteHolsterEvent),
    KillRefillsMeter(KillRefillsMeterEvent),
    RpsTauntEvent(RpsTauntEventEvent),
    CongaKill(CongaKillEvent),
    PlayerInitialSpawn(PlayerInitialSpawnEvent),
    CompetitiveVictory(CompetitiveVictoryEvent),
    CompetitiveStatsUpdate(CompetitiveStatsUpdateEvent),
    MiniGameWin(MiniGameWinEvent),
    SentryOnGoActive(SentryOnGoActiveEvent),
    DuckXpLevelUp(DuckXpLevelUpEvent),
    QuestLogOpened(QuestLogOpenedEvent),
    SchemaUpdated(SchemaUpdatedEvent),
    LocalPlayerPickupWeapon(LocalPlayerPickupWeaponEvent),
    RdPlayerScorePoints(RdPlayerScorePointsEvent),
    DemomanDetStickies(DemomanDetStickiesEvent),
    QuestObjectiveCompleted(QuestObjectiveCompletedEvent),
    PlayerScoreChanged(PlayerScoreChangedEvent),
    KilledCappingPlayer(KilledCappingPlayerEvent),
    EnvironmentalDeath(EnvironmentalDeathEvent),
    ProjectileDirectHit(ProjectileDirectHitEvent),
    PassGet(PassGetEvent),
    PassScore(PassScoreEvent),
    PassFree(PassFreeEvent),
    PassPassCaught(PassPassCaughtEvent),
    PassBallStolen(PassBallStolenEvent),
    PassBallBlocked(PassBallBlockedEvent),
    DamagePrevented(DamagePreventedEvent),
    HalloweenBossKilled(HalloweenBossKilledEvent),
    EscapedLootIsland(EscapedLootIslandEvent),
    TaggedPlayerAsIt(TaggedPlayerAsItEvent),
    MerasmusStunned(MerasmusStunnedEvent),
    MerasmusPropFound(MerasmusPropFoundEvent),
    HalloweenSkeletonKilled(HalloweenSkeletonKilledEvent),
    SkeletonKilledQuest(SkeletonKilledQuestEvent),
    SkeletonKingKilledQuest(SkeletonKingKilledQuestEvent),
    EscapeHell(EscapeHellEvent),
    CrossSpectralBridge(CrossSpectralBridgeEvent),
    MiniGameWon(MiniGameWonEvent),
    RespawnGhost(RespawnGhostEvent),
    KillInHell(KillInHellEvent),
    HalloweenDuckCollected(HalloweenDuckCollectedEvent),
    SpecialScore(SpecialScoreEvent),
    TeamLeaderKilled(TeamLeaderKilledEvent),
    HalloweenSoulCollected(HalloweenSoulCollectedEvent),
    RecalculateTruce(RecalculateTruceEvent),
    DeadRingerCheatDeath(DeadRingerCheatDeathEvent),
    CrossbowHeal(CrossbowHealEvent),
    DamageMitigated(DamageMitigatedEvent),
    PayloadPushed(PayloadPushedEvent),
    PlayerAbandonedMatch(PlayerAbandonedMatchEvent),
    ClDrawline(ClDrawlineEvent),
    RestartTimerTime(RestartTimerTimeEvent),
    WinLimitChanged(WinLimitChangedEvent),
    WinPanelShowScores(WinPanelShowScoresEvent),
    TopStreamsRequestFinished(TopStreamsRequestFinishedEvent),
    CompetitiveStateChanged(CompetitiveStateChangedEvent),
    GlobalWarDataUpdated(GlobalWarDataUpdatedEvent),
    StopWatchChanged(StopWatchChangedEvent),
    DsStop(DsStopEvent),
    DsScreenshot(DsScreenshotEvent),
    ShowMatchSummary(ShowMatchSummaryEvent),
    ExperienceChanged(ExperienceChangedEvent),
    BeginXpLerp(BeginXpLerpEvent),
    MatchmakerStatsUpdated(MatchmakerStatsUpdatedEvent),
    RematchVotePeriodOver(RematchVotePeriodOverEvent),
    RematchFailedToCreate(RematchFailedToCreateEvent),
    PlayerRematchChange(PlayerRematchChangeEvent),
    PingUpdated(PingUpdatedEvent),
    MMStatsUpdated(MMStatsUpdatedEvent),
    PlayerNextMapVoteChange(PlayerNextMapVoteChangeEvent),
    VoteMapsChanged(VoteMapsChangedEvent),
    ProtoDefChanged(ProtoDefChangedEvent),
    PlayerDomination(PlayerDominationEvent),
    PlayerRocketPackPushed(PlayerRocketPackPushedEvent),
    QuestRequest(QuestRequestEvent),
    QuestResponse(QuestResponseEvent),
    QuestProgress(QuestProgressEvent),
    ProjectileRemoved(ProjectileRemovedEvent),
    QuestMapDataChanged(QuestMapDataChangedEvent),
    GasDousedPlayerIgnited(GasDousedPlayerIgnitedEvent),
    QuestTurnInState(QuestTurnInStateEvent),
    ItemsAcknowledged(ItemsAcknowledgedEvent),
    CapperKilled(CapperKilledEvent),
    MainMenuStabilized(MainMenuStabilizedEvent),
    WorldStatusChanged(WorldStatusChangedEvent),
    HLTVStatus(HLTVStatusEvent),
    HLTVCameraman(HLTVCameramanEvent),
    HLTVRankCamera(HLTVRankCameraEvent),
    HLTVRankEntity(HLTVRankEntityEvent),
    HLTVFixed(HLTVFixedEvent),
    HLTVChase(HLTVChaseEvent),
    HLTVMessage(HLTVMessageEvent),
    HLTVTitle(HLTVTitleEvent),
    HLTVChat(HLTVChatEvent),
    ReplayStartRecord(ReplayStartRecordEvent),
    ReplaySessionInfo(ReplaySessionInfoEvent),
    ReplayEndRecord(ReplayEndRecordEvent),
    ReplayReplaysAvailable(ReplayReplaysAvailableEvent),
    ReplayServerError(ReplayServerErrorEvent),
    Unknown(RawGameEvent),
}
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameEventType {
    #[serde(rename = "server_spawn")]
    ServerSpawn,
    #[serde(rename = "server_changelevel_failed")]
    ServerChangeLevelFailed,
    #[serde(rename = "server_shutdown")]
    ServerShutdown,
    #[serde(rename = "server_cvar")]
    ServerCvar,
    #[serde(rename = "server_message")]
    ServerMessage,
    #[serde(rename = "server_addban")]
    ServerAddBan,
    #[serde(rename = "server_removeban")]
    ServerRemoveBan,
    #[serde(rename = "player_connect")]
    PlayerConnect,
    #[serde(rename = "player_connect_client")]
    PlayerConnectClient,
    #[serde(rename = "player_info")]
    PlayerInfo,
    #[serde(rename = "player_disconnect")]
    PlayerDisconnect,
    #[serde(rename = "player_activate")]
    PlayerActivate,
    #[serde(rename = "player_say")]
    PlayerSay,
    #[serde(rename = "client_disconnect")]
    ClientDisconnect,
    #[serde(rename = "client_beginconnect")]
    ClientBeginConnect,
    #[serde(rename = "client_connected")]
    ClientConnected,
    #[serde(rename = "client_fullconnect")]
    ClientFullConnect,
    #[serde(rename = "host_quit")]
    HostQuit,
    #[serde(rename = "team_info")]
    TeamInfo,
    #[serde(rename = "team_score")]
    TeamScore,
    #[serde(rename = "teamplay_broadcast_audio")]
    TeamPlayBroadcastAudio,
    #[serde(rename = "player_team")]
    PlayerTeam,
    #[serde(rename = "player_class")]
    PlayerClass,
    #[serde(rename = "player_death")]
    PlayerDeath,
    #[serde(rename = "player_hurt")]
    PlayerHurt,
    #[serde(rename = "player_chat")]
    PlayerChat,
    #[serde(rename = "player_score")]
    PlayerScore,
    #[serde(rename = "player_spawn")]
    PlayerSpawn,
    #[serde(rename = "player_shoot")]
    PlayerShoot,
    #[serde(rename = "player_use")]
    PlayerUse,
    #[serde(rename = "player_changename")]
    PlayerChangeName,
    #[serde(rename = "player_hintmessage")]
    PlayerHintMessage,
    #[serde(rename = "base_player_teleported")]
    BasePlayerTeleported,
    #[serde(rename = "game_init")]
    GameInit,
    #[serde(rename = "game_newmap")]
    GameNewMap,
    #[serde(rename = "game_start")]
    GameStart,
    #[serde(rename = "game_end")]
    GameEnd,
    #[serde(rename = "round_start")]
    RoundStart,
    #[serde(rename = "round_end")]
    RoundEnd,
    #[serde(rename = "game_message")]
    GameMessage,
    #[serde(rename = "break_breakable")]
    BreakBreakable,
    #[serde(rename = "break_prop")]
    BreakProp,
    #[serde(rename = "entity_killed")]
    EntityKilled,
    #[serde(rename = "bonus_updated")]
    BonusUpdated,
    #[serde(rename = "achievement_event")]
    AchievementEvent,
    #[serde(rename = "achievement_increment")]
    AchievementIncrement,
    #[serde(rename = "physgun_pickup")]
    PhysgunPickup,
    #[serde(rename = "flare_ignite_npc")]
    FlareIgniteNpc,
    #[serde(rename = "helicopter_grenade_punt_miss")]
    HelicopterGrenadePuntMiss,
    #[serde(rename = "user_data_downloaded")]
    UserDataDownloaded,
    #[serde(rename = "ragdoll_dissolved")]
    RagdollDissolved,
    #[serde(rename = "hltv_changed_mode")]
    HLTVChangedMode,
    #[serde(rename = "hltv_changed_target")]
    HLTVChangedTarget,
    #[serde(rename = "vote_ended")]
    VoteEnded,
    #[serde(rename = "vote_started")]
    VoteStarted,
    #[serde(rename = "vote_changed")]
    VoteChanged,
    #[serde(rename = "vote_passed")]
    VotePassed,
    #[serde(rename = "vote_failed")]
    VoteFailed,
    #[serde(rename = "vote_cast")]
    VoteCast,
    #[serde(rename = "vote_options")]
    VoteOptions,
    #[serde(rename = "replay_saved")]
    ReplaySaved,
    #[serde(rename = "entered_performance_mode")]
    EnteredPerformanceMode,
    #[serde(rename = "browse_replays")]
    BrowseReplays,
    #[serde(rename = "replay_youtube_stats")]
    ReplayYoutubeStats,
    #[serde(rename = "inventory_updated")]
    InventoryUpdated,
    #[serde(rename = "cart_updated")]
    CartUpdated,
    #[serde(rename = "store_pricesheet_updated")]
    StorePriceSheetUpdated,
    #[serde(rename = "econ_inventory_connected")]
    EconInventoryConnected,
    #[serde(rename = "item_schema_initialized")]
    ItemSchemaInitialized,
    #[serde(rename = "gc_new_session")]
    GcNewSession,
    #[serde(rename = "gc_lost_session")]
    GcLostSession,
    #[serde(rename = "intro_finish")]
    IntroFinish,
    #[serde(rename = "intro_nextcamera")]
    IntroNextCamera,
    #[serde(rename = "player_changeclass")]
    PlayerChangeClass,
    #[serde(rename = "tf_map_time_remaining")]
    TfMapTimeRemaining,
    #[serde(rename = "tf_game_over")]
    TfGameOver,
    #[serde(rename = "ctf_flag_captured")]
    CtfFlagCaptured,
    #[serde(rename = "controlpoint_initialized")]
    ControlPointInitialized,
    #[serde(rename = "controlpoint_updateimages")]
    ControlPointUpdateImages,
    #[serde(rename = "controlpoint_updatelayout")]
    ControlPointUpdateLayout,
    #[serde(rename = "controlpoint_updatecapping")]
    ControlPointUpdateCapping,
    #[serde(rename = "controlpoint_updateowner")]
    ControlPointUpdateOwner,
    #[serde(rename = "controlpoint_starttouch")]
    ControlPointStartTouch,
    #[serde(rename = "controlpoint_endtouch")]
    ControlPointEndTouch,
    #[serde(rename = "controlpoint_pulse_element")]
    ControlPointPulseElement,
    #[serde(rename = "controlpoint_fake_capture")]
    ControlPointFakeCapture,
    #[serde(rename = "controlpoint_fake_capture_mult")]
    ControlPointFakeCaptureMultiplier,
    #[serde(rename = "teamplay_round_selected")]
    TeamPlayRoundSelected,
    #[serde(rename = "teamplay_round_start")]
    TeamPlayRoundStart,
    #[serde(rename = "teamplay_round_active")]
    TeamPlayRoundActive,
    #[serde(rename = "teamplay_waiting_begins")]
    TeamPlayWaitingBegins,
    #[serde(rename = "teamplay_waiting_ends")]
    TeamPlayWaitingEnds,
    #[serde(rename = "teamplay_waiting_abouttoend")]
    TeamPlayWaitingAboutToEnd,
    #[serde(rename = "teamplay_restart_round")]
    TeamPlayRestartRound,
    #[serde(rename = "teamplay_ready_restart")]
    TeamPlayReadyRestart,
    #[serde(rename = "teamplay_round_restart_seconds")]
    TeamPlayRoundRestartSeconds,
    #[serde(rename = "teamplay_team_ready")]
    TeamPlayTeamReady,
    #[serde(rename = "teamplay_round_win")]
    TeamPlayRoundWin,
    #[serde(rename = "teamplay_update_timer")]
    TeamPlayUpdateTimer,
    #[serde(rename = "teamplay_round_stalemate")]
    TeamPlayRoundStalemate,
    #[serde(rename = "teamplay_overtime_begin")]
    TeamPlayOvertimeBegin,
    #[serde(rename = "teamplay_overtime_end")]
    TeamPlayOvertimeEnd,
    #[serde(rename = "teamplay_suddendeath_begin")]
    TeamPlaySuddenDeathBegin,
    #[serde(rename = "teamplay_suddendeath_end")]
    TeamPlaySuddenDeathEnd,
    #[serde(rename = "teamplay_game_over")]
    TeamPlayGameOver,
    #[serde(rename = "teamplay_map_time_remaining")]
    TeamPlayMapTimeRemaining,
    #[serde(rename = "teamplay_timer_flash")]
    TeamPlayTimerFlash,
    #[serde(rename = "teamplay_timer_time_added")]
    TeamPlayTimerTimeAdded,
    #[serde(rename = "teamplay_point_startcapture")]
    TeamPlayPointStartCapture,
    #[serde(rename = "teamplay_point_captured")]
    TeamPlayPointCaptured,
    #[serde(rename = "teamplay_point_locked")]
    TeamPlayPointLocked,
    #[serde(rename = "teamplay_point_unlocked")]
    TeamPlayPointUnlocked,
    #[serde(rename = "teamplay_capture_broken")]
    TeamPlayCaptureBroken,
    #[serde(rename = "teamplay_capture_blocked")]
    TeamPlayCaptureBlocked,
    #[serde(rename = "teamplay_flag_event")]
    TeamPlayFlagEvent,
    #[serde(rename = "teamplay_win_panel")]
    TeamPlayWinPanel,
    #[serde(rename = "teamplay_teambalanced_player")]
    TeamPlayTeamBalancedPlayer,
    #[serde(rename = "teamplay_setup_finished")]
    TeamPlaySetupFinished,
    #[serde(rename = "teamplay_alert")]
    TeamPlayAlert,
    #[serde(rename = "training_complete")]
    TrainingComplete,
    #[serde(rename = "show_freezepanel")]
    ShowFreezePanel,
    #[serde(rename = "hide_freezepanel")]
    HideFreezePanel,
    #[serde(rename = "freezecam_started")]
    FreezeCamStarted,
    #[serde(rename = "localplayer_changeteam")]
    LocalPlayerChangeTeam,
    #[serde(rename = "localplayer_score_changed")]
    LocalPlayerScoreChanged,
    #[serde(rename = "localplayer_changeclass")]
    LocalPlayerChangeClass,
    #[serde(rename = "localplayer_respawn")]
    LocalPlayerRespawn,
    #[serde(rename = "building_info_changed")]
    BuildingInfoChanged,
    #[serde(rename = "localplayer_changedisguise")]
    LocalPlayerChangeDisguise,
    #[serde(rename = "player_account_changed")]
    PlayerAccountChanged,
    #[serde(rename = "spy_pda_reset")]
    SpyPdaReset,
    #[serde(rename = "flagstatus_update")]
    FlagStatusUpdate,
    #[serde(rename = "player_stats_updated")]
    PlayerStatsUpdated,
    #[serde(rename = "playing_commentary")]
    PlayingCommentary,
    #[serde(rename = "player_chargedeployed")]
    PlayerChargeDeployed,
    #[serde(rename = "player_builtobject")]
    PlayerBuiltObject,
    #[serde(rename = "player_upgradedobject")]
    PlayerUpgradedObject,
    #[serde(rename = "player_carryobject")]
    PlayerCarryObject,
    #[serde(rename = "player_dropobject")]
    PlayerDropObject,
    #[serde(rename = "object_removed")]
    ObjectRemoved,
    #[serde(rename = "object_destroyed")]
    ObjectDestroyed,
    #[serde(rename = "object_detonated")]
    ObjectDetonated,
    #[serde(rename = "achievement_earned")]
    AchievementEarned,
    #[serde(rename = "spec_target_updated")]
    SpecTargetUpdated,
    #[serde(rename = "tournament_stateupdate")]
    TournamentStateUpdate,
    #[serde(rename = "tournament_enablecountdown")]
    TournamentEnableCountdown,
    #[serde(rename = "player_calledformedic")]
    PlayerCalledForMedic,
    #[serde(rename = "player_askedforball")]
    PlayerAskedForBall,
    #[serde(rename = "localplayer_becameobserver")]
    LocalPlayerBecameObserver,
    #[serde(rename = "player_ignited_inv")]
    PlayerIgnitedInv,
    #[serde(rename = "player_ignited")]
    PlayerIgnited,
    #[serde(rename = "player_extinguished")]
    PlayerExtinguished,
    #[serde(rename = "player_teleported")]
    PlayerTeleported,
    #[serde(rename = "player_healedmediccall")]
    PlayerHealedMedicCall,
    #[serde(rename = "localplayer_chargeready")]
    LocalPlayerChargeReady,
    #[serde(rename = "localplayer_winddown")]
    LocalPlayerWindDown,
    #[serde(rename = "player_invulned")]
    PlayerInvulned,
    #[serde(rename = "escort_speed")]
    EscortSpeed,
    #[serde(rename = "escort_progress")]
    EscortProgress,
    #[serde(rename = "escort_recede")]
    EscortRecede,
    #[serde(rename = "gameui_activated")]
    GameUIActivated,
    #[serde(rename = "gameui_hidden")]
    GameUIHidden,
    #[serde(rename = "player_escort_score")]
    PlayerEscortScore,
    #[serde(rename = "player_healonhit")]
    PlayerHealOnHit,
    #[serde(rename = "player_stealsandvich")]
    PlayerStealSandvich,
    #[serde(rename = "show_class_layout")]
    ShowClassLayout,
    #[serde(rename = "show_vs_panel")]
    ShowVsPanel,
    #[serde(rename = "player_damaged")]
    PlayerDamaged,
    #[serde(rename = "arena_player_notification")]
    ArenaPlayerNotification,
    #[serde(rename = "arena_match_maxstreak")]
    ArenaMatchMaxStreak,
    #[serde(rename = "arena_round_start")]
    ArenaRoundStart,
    #[serde(rename = "arena_win_panel")]
    ArenaWinPanel,
    #[serde(rename = "pve_win_panel")]
    PveWinPanel,
    #[serde(rename = "air_dash")]
    AirDash,
    #[serde(rename = "landed")]
    Landed,
    #[serde(rename = "player_damage_dodged")]
    PlayerDamageDodged,
    #[serde(rename = "player_stunned")]
    PlayerStunned,
    #[serde(rename = "scout_grand_slam")]
    ScoutGrandSlam,
    #[serde(rename = "scout_slamdoll_landed")]
    ScoutSlamdollLanded,
    #[serde(rename = "arrow_impact")]
    ArrowImpact,
    #[serde(rename = "player_jarated")]
    PlayerJarated,
    #[serde(rename = "player_jarated_fade")]
    PlayerJaratedFade,
    #[serde(rename = "player_shield_blocked")]
    PlayerShieldBlocked,
    #[serde(rename = "player_pinned")]
    PlayerPinned,
    #[serde(rename = "player_healedbymedic")]
    PlayerHealedByMedic,
    #[serde(rename = "player_sapped_object")]
    PlayerSappedObject,
    #[serde(rename = "item_found")]
    ItemFound,
    #[serde(rename = "show_annotation")]
    ShowAnnotation,
    #[serde(rename = "hide_annotation")]
    HideAnnotation,
    #[serde(rename = "post_inventory_application")]
    PostInventoryApplication,
    #[serde(rename = "controlpoint_unlock_updated")]
    ControlPointUnlockUpdated,
    #[serde(rename = "deploy_buff_banner")]
    DeployBuffBanner,
    #[serde(rename = "player_buff")]
    PlayerBuff,
    #[serde(rename = "medic_death")]
    MedicDeath,
    #[serde(rename = "overtime_nag")]
    OvertimeNag,
    #[serde(rename = "teams_changed")]
    TeamsChanged,
    #[serde(rename = "halloween_pumpkin_grab")]
    HalloweenPumpkinGrab,
    #[serde(rename = "rocket_jump")]
    RocketJump,
    #[serde(rename = "rocket_jump_landed")]
    RocketJumpLanded,
    #[serde(rename = "sticky_jump")]
    StickyJump,
    #[serde(rename = "sticky_jump_landed")]
    StickyJumpLanded,
    #[serde(rename = "rocketpack_launch")]
    RocketPackLaunch,
    #[serde(rename = "rocketpack_landed")]
    RocketPackLanded,
    #[serde(rename = "medic_defended")]
    MedicDefended,
    #[serde(rename = "localplayer_healed")]
    LocalPlayerHealed,
    #[serde(rename = "player_destroyed_pipebomb")]
    PlayerDestroyedPipeBomb,
    #[serde(rename = "object_deflected")]
    ObjectDeflected,
    #[serde(rename = "player_mvp")]
    PlayerMvp,
    #[serde(rename = "raid_spawn_mob")]
    RaidSpawnMob,
    #[serde(rename = "raid_spawn_squad")]
    RaidSpawnSquad,
    #[serde(rename = "nav_blocked")]
    NavBlocked,
    #[serde(rename = "path_track_passed")]
    PathTrackPassed,
    #[serde(rename = "num_cappers_changed")]
    NumCappersChanged,
    #[serde(rename = "player_regenerate")]
    PlayerRegenerate,
    #[serde(rename = "update_status_item")]
    UpdateStatusItem,
    #[serde(rename = "stats_resetround")]
    StatsResetRound,
    #[serde(rename = "scorestats_accumulated_update")]
    ScoreStatsAccumulatedUpdate,
    #[serde(rename = "scorestats_accumulated_reset")]
    ScoreStatsAccumulatedReset,
    #[serde(rename = "achievement_earned_local")]
    AchievementEarnedLocal,
    #[serde(rename = "player_healed")]
    PlayerHealed,
    #[serde(rename = "building_healed")]
    BuildingHealed,
    #[serde(rename = "item_pickup")]
    ItemPickup,
    #[serde(rename = "duel_status")]
    DuelStatus,
    #[serde(rename = "fish_notice")]
    FishNotice,
    #[serde(rename = "fish_notice__arm")]
    FishNoticeArm,
    #[serde(rename = "slap_notice")]
    SlapNotice,
    #[serde(rename = "throwable_hit")]
    ThrowableHit,
    #[serde(rename = "pumpkin_lord_summoned")]
    PumpkinLordSummoned,
    #[serde(rename = "pumpkin_lord_killed")]
    PumpkinLordKilled,
    #[serde(rename = "merasmus_summoned")]
    MerasmusSummoned,
    #[serde(rename = "merasmus_killed")]
    MerasmusKilled,
    #[serde(rename = "merasmus_escape_warning")]
    MerasmusEscapeWarning,
    #[serde(rename = "merasmus_escaped")]
    MerasmusEscaped,
    #[serde(rename = "eyeball_boss_summoned")]
    EyeballBossSummoned,
    #[serde(rename = "eyeball_boss_stunned")]
    EyeballBossStunned,
    #[serde(rename = "eyeball_boss_killed")]
    EyeballBossKilled,
    #[serde(rename = "eyeball_boss_killer")]
    EyeballBossKiller,
    #[serde(rename = "eyeball_boss_escape_imminent")]
    EyeballBossEscapeImminent,
    #[serde(rename = "eyeball_boss_escaped")]
    EyeballBossEscaped,
    #[serde(rename = "npc_hurt")]
    NpcHurt,
    #[serde(rename = "controlpoint_timer_updated")]
    ControlPointTimerUpdated,
    #[serde(rename = "player_highfive_start")]
    PlayerHighFiveStart,
    #[serde(rename = "player_highfive_cancel")]
    PlayerHighFiveCancel,
    #[serde(rename = "player_highfive_success")]
    PlayerHighFiveSuccess,
    #[serde(rename = "player_bonuspoints")]
    PlayerBonusPoints,
    #[serde(rename = "player_upgraded")]
    PlayerUpgraded,
    #[serde(rename = "player_buyback")]
    PlayerBuyback,
    #[serde(rename = "player_used_powerup_bottle")]
    PlayerUsedPowerUpBottle,
    #[serde(rename = "christmas_gift_grab")]
    ChristmasGiftGrab,
    #[serde(rename = "player_killed_achievement_zone")]
    PlayerKilledAchievementZone,
    #[serde(rename = "party_updated")]
    PartyUpdated,
    #[serde(rename = "party_pref_changed")]
    PartyPrefChanged,
    #[serde(rename = "party_criteria_changed")]
    PartyCriteriaChanged,
    #[serde(rename = "party_invites_changed")]
    PartyInvitesChanged,
    #[serde(rename = "party_queue_state_changed")]
    PartyQueueStateChanged,
    #[serde(rename = "party_chat")]
    PartyChat,
    #[serde(rename = "party_member_join")]
    PartyMemberJoin,
    #[serde(rename = "party_member_leave")]
    PartyMemberLeave,
    #[serde(rename = "match_invites_updated")]
    MatchInvitesUpdated,
    #[serde(rename = "lobby_updated")]
    LobbyUpdated,
    #[serde(rename = "mvm_mission_update")]
    MvmMissionUpdate,
    #[serde(rename = "recalculate_holidays")]
    RecalculateHolidays,
    #[serde(rename = "player_currency_changed")]
    PlayerCurrencyChanged,
    #[serde(rename = "doomsday_rocket_open")]
    DoomsdayRocketOpen,
    #[serde(rename = "remove_nemesis_relationships")]
    RemoveNemesisRelationships,
    #[serde(rename = "mvm_creditbonus_wave")]
    MvmCreditBonusWave,
    #[serde(rename = "mvm_creditbonus_all")]
    MvmCreditBonusAll,
    #[serde(rename = "mvm_creditbonus_all_advanced")]
    MvmCreditBonusAllAdvanced,
    #[serde(rename = "mvm_quick_sentry_upgrade")]
    MvmQuickSentryUpgrade,
    #[serde(rename = "mvm_tank_destroyed_by_players")]
    MvmTankDestroyedByPlayers,
    #[serde(rename = "mvm_kill_robot_delivering_bomb")]
    MvmKillRobotDeliveringBomb,
    #[serde(rename = "mvm_pickup_currency")]
    MvmPickupCurrency,
    #[serde(rename = "mvm_bomb_carrier_killed")]
    MvmBombCarrierKilled,
    #[serde(rename = "mvm_sentrybuster_detonate")]
    MvmSentryBusterDetonate,
    #[serde(rename = "mvm_scout_marked_for_death")]
    MvmScoutMarkedForDeath,
    #[serde(rename = "mvm_medic_powerup_shared")]
    MvmMedicPowerUpShared,
    #[serde(rename = "mvm_begin_wave")]
    MvmBeginWave,
    #[serde(rename = "mvm_wave_complete")]
    MvmWaveComplete,
    #[serde(rename = "mvm_mission_complete")]
    MvmMissionComplete,
    #[serde(rename = "mvm_bomb_reset_by_player")]
    MvmBombResetByPlayer,
    #[serde(rename = "mvm_bomb_alarm_triggered")]
    MvmBombAlarmTriggered,
    #[serde(rename = "mvm_bomb_deploy_reset_by_player")]
    MvmBombDeployResetByPlayer,
    #[serde(rename = "mvm_wave_failed")]
    MvmWaveFailed,
    #[serde(rename = "mvm_reset_stats")]
    MvmResetStats,
    #[serde(rename = "damage_resisted")]
    DamageResisted,
    #[serde(rename = "revive_player_notify")]
    RevivePlayerNotify,
    #[serde(rename = "revive_player_stopped")]
    RevivePlayerStopped,
    #[serde(rename = "revive_player_complete")]
    RevivePlayerComplete,
    #[serde(rename = "player_turned_to_ghost")]
    PlayerTurnedToGhost,
    #[serde(rename = "medigun_shield_blocked_damage")]
    MedigunShieldBlockedDamage,
    #[serde(rename = "mvm_adv_wave_complete_no_gates")]
    MvmAdvWaveCompleteNoGates,
    #[serde(rename = "mvm_sniper_headshot_currency")]
    MvmSniperHeadshotCurrency,
    #[serde(rename = "mvm_mannhattan_pit")]
    MvmMannhattanPit,
    #[serde(rename = "flag_carried_in_detection_zone")]
    FlagCarriedInDetectionZone,
    #[serde(rename = "mvm_adv_wave_killed_stun_radio")]
    MvmAdvWaveKilledStunRadio,
    #[serde(rename = "player_directhit_stun")]
    PlayerDirectHitStun,
    #[serde(rename = "mvm_sentrybuster_killed")]
    MvmSentryBusterKilled,
    #[serde(rename = "upgrades_file_changed")]
    UpgradesFileChanged,
    #[serde(rename = "rd_team_points_changed")]
    RdTeamPointsChanged,
    #[serde(rename = "rd_rules_state_changed")]
    RdRulesStateChanged,
    #[serde(rename = "rd_robot_killed")]
    RdRobotKilled,
    #[serde(rename = "rd_robot_impact")]
    RdRobotImpact,
    #[serde(rename = "teamplay_pre_round_time_left")]
    TeamPlayPreRoundTimeLeft,
    #[serde(rename = "parachute_deploy")]
    ParachuteDeploy,
    #[serde(rename = "parachute_holster")]
    ParachuteHolster,
    #[serde(rename = "kill_refills_meter")]
    KillRefillsMeter,
    #[serde(rename = "rps_taunt_event")]
    RpsTauntEvent,
    #[serde(rename = "conga_kill")]
    CongaKill,
    #[serde(rename = "player_initial_spawn")]
    PlayerInitialSpawn,
    #[serde(rename = "competitive_victory")]
    CompetitiveVictory,
    #[serde(rename = "competitive_stats_update")]
    CompetitiveStatsUpdate,
    #[serde(rename = "minigame_win")]
    MiniGameWin,
    #[serde(rename = "sentry_on_go_active")]
    SentryOnGoActive,
    #[serde(rename = "duck_xp_level_up")]
    DuckXpLevelUp,
    #[serde(rename = "questlog_opened")]
    QuestLogOpened,
    #[serde(rename = "schema_updated")]
    SchemaUpdated,
    #[serde(rename = "localplayer_pickup_weapon")]
    LocalPlayerPickupWeapon,
    #[serde(rename = "rd_player_score_points")]
    RdPlayerScorePoints,
    #[serde(rename = "demoman_det_stickies")]
    DemomanDetStickies,
    #[serde(rename = "quest_objective_completed")]
    QuestObjectiveCompleted,
    #[serde(rename = "player_score_changed")]
    PlayerScoreChanged,
    #[serde(rename = "killed_capping_player")]
    KilledCappingPlayer,
    #[serde(rename = "environmental_death")]
    EnvironmentalDeath,
    #[serde(rename = "projectile_direct_hit")]
    ProjectileDirectHit,
    #[serde(rename = "pass_get")]
    PassGet,
    #[serde(rename = "pass_score")]
    PassScore,
    #[serde(rename = "pass_free")]
    PassFree,
    #[serde(rename = "pass_pass_caught")]
    PassPassCaught,
    #[serde(rename = "pass_ball_stolen")]
    PassBallStolen,
    #[serde(rename = "pass_ball_blocked")]
    PassBallBlocked,
    #[serde(rename = "damage_prevented")]
    DamagePrevented,
    #[serde(rename = "halloween_boss_killed")]
    HalloweenBossKilled,
    #[serde(rename = "escaped_loot_island")]
    EscapedLootIsland,
    #[serde(rename = "tagged_player_as_it")]
    TaggedPlayerAsIt,
    #[serde(rename = "merasmus_stunned")]
    MerasmusStunned,
    #[serde(rename = "merasmus_prop_found")]
    MerasmusPropFound,
    #[serde(rename = "halloween_skeleton_killed")]
    HalloweenSkeletonKilled,
    #[serde(rename = "skeleton_killed_quest")]
    SkeletonKilledQuest,
    #[serde(rename = "skeleton_king_killed_quest")]
    SkeletonKingKilledQuest,
    #[serde(rename = "escape_hell")]
    EscapeHell,
    #[serde(rename = "cross_spectral_bridge")]
    CrossSpectralBridge,
    #[serde(rename = "minigame_won")]
    MiniGameWon,
    #[serde(rename = "respawn_ghost")]
    RespawnGhost,
    #[serde(rename = "kill_in_hell")]
    KillInHell,
    #[serde(rename = "halloween_duck_collected")]
    HalloweenDuckCollected,
    #[serde(rename = "special_score")]
    SpecialScore,
    #[serde(rename = "team_leader_killed")]
    TeamLeaderKilled,
    #[serde(rename = "halloween_soul_collected")]
    HalloweenSoulCollected,
    #[serde(rename = "recalculate_truce")]
    RecalculateTruce,
    #[serde(rename = "deadringer_cheat_death")]
    DeadRingerCheatDeath,
    #[serde(rename = "crossbow_heal")]
    CrossbowHeal,
    #[serde(rename = "damage_mitigated")]
    DamageMitigated,
    #[serde(rename = "payload_pushed")]
    PayloadPushed,
    #[serde(rename = "player_abandoned_match")]
    PlayerAbandonedMatch,
    #[serde(rename = "cl_drawline")]
    ClDrawline,
    #[serde(rename = "restart_timer_time")]
    RestartTimerTime,
    #[serde(rename = "winlimit_changed")]
    WinLimitChanged,
    #[serde(rename = "winpanel_show_scores")]
    WinPanelShowScores,
    #[serde(rename = "top_streams_request_finished")]
    TopStreamsRequestFinished,
    #[serde(rename = "competitive_state_changed")]
    CompetitiveStateChanged,
    #[serde(rename = "global_war_data_updated")]
    GlobalWarDataUpdated,
    #[serde(rename = "stop_watch_changed")]
    StopWatchChanged,
    #[serde(rename = "ds_stop")]
    DsStop,
    #[serde(rename = "ds_screenshot")]
    DsScreenshot,
    #[serde(rename = "show_match_summary")]
    ShowMatchSummary,
    #[serde(rename = "experience_changed")]
    ExperienceChanged,
    #[serde(rename = "begin_xp_lerp")]
    BeginXpLerp,
    #[serde(rename = "matchmaker_stats_updated")]
    MatchmakerStatsUpdated,
    #[serde(rename = "rematch_vote_period_over")]
    RematchVotePeriodOver,
    #[serde(rename = "rematch_failed_to_create")]
    RematchFailedToCreate,
    #[serde(rename = "player_rematch_change")]
    PlayerRematchChange,
    #[serde(rename = "ping_updated")]
    PingUpdated,
    #[serde(rename = "mmstats_updated")]
    MMStatsUpdated,
    #[serde(rename = "player_next_map_vote_change")]
    PlayerNextMapVoteChange,
    #[serde(rename = "vote_maps_changed")]
    VoteMapsChanged,
    #[serde(rename = "proto_def_changed")]
    ProtoDefChanged,
    #[serde(rename = "player_domination")]
    PlayerDomination,
    #[serde(rename = "player_rocketpack_pushed")]
    PlayerRocketPackPushed,
    #[serde(rename = "quest_request")]
    QuestRequest,
    #[serde(rename = "quest_response")]
    QuestResponse,
    #[serde(rename = "quest_progress")]
    QuestProgress,
    #[serde(rename = "projectile_removed")]
    ProjectileRemoved,
    #[serde(rename = "quest_map_data_changed")]
    QuestMapDataChanged,
    #[serde(rename = "gas_doused_player_ignited")]
    GasDousedPlayerIgnited,
    #[serde(rename = "quest_turn_in_state")]
    QuestTurnInState,
    #[serde(rename = "items_acknowledged")]
    ItemsAcknowledged,
    #[serde(rename = "capper_killed")]
    CapperKilled,
    #[serde(rename = "mainmenu_stabilized")]
    MainMenuStabilized,
    #[serde(rename = "world_status_changed")]
    WorldStatusChanged,
    #[serde(rename = "hltv_status")]
    HLTVStatus,
    #[serde(rename = "hltv_cameraman")]
    HLTVCameraman,
    #[serde(rename = "hltv_rank_camera")]
    HLTVRankCamera,
    #[serde(rename = "hltv_rank_entity")]
    HLTVRankEntity,
    #[serde(rename = "hltv_fixed")]
    HLTVFixed,
    #[serde(rename = "hltv_chase")]
    HLTVChase,
    #[serde(rename = "hltv_message")]
    HLTVMessage,
    #[serde(rename = "hltv_title")]
    HLTVTitle,
    #[serde(rename = "hltv_chat")]
    HLTVChat,
    #[serde(rename = "replay_startrecord")]
    ReplayStartRecord,
    #[serde(rename = "replay_sessioninfo")]
    ReplaySessionInfo,
    #[serde(rename = "replay_endrecord")]
    ReplayEndRecord,
    #[serde(rename = "replay_replaysavailable")]
    ReplayReplaysAvailable,
    #[serde(rename = "replay_servererror")]
    ReplayServerError,
    Unknown(String),
}
impl GameEventType {
    pub fn from_type_name(name: &str) -> Self {
        match name {
            "server_spawn" => GameEventType::ServerSpawn,
            "server_changelevel_failed" => GameEventType::ServerChangeLevelFailed,
            "server_shutdown" => GameEventType::ServerShutdown,
            "server_cvar" => GameEventType::ServerCvar,
            "server_message" => GameEventType::ServerMessage,
            "server_addban" => GameEventType::ServerAddBan,
            "server_removeban" => GameEventType::ServerRemoveBan,
            "player_connect" => GameEventType::PlayerConnect,
            "player_connect_client" => GameEventType::PlayerConnectClient,
            "player_info" => GameEventType::PlayerInfo,
            "player_disconnect" => GameEventType::PlayerDisconnect,
            "player_activate" => GameEventType::PlayerActivate,
            "player_say" => GameEventType::PlayerSay,
            "client_disconnect" => GameEventType::ClientDisconnect,
            "client_beginconnect" => GameEventType::ClientBeginConnect,
            "client_connected" => GameEventType::ClientConnected,
            "client_fullconnect" => GameEventType::ClientFullConnect,
            "host_quit" => GameEventType::HostQuit,
            "team_info" => GameEventType::TeamInfo,
            "team_score" => GameEventType::TeamScore,
            "teamplay_broadcast_audio" => GameEventType::TeamPlayBroadcastAudio,
            "player_team" => GameEventType::PlayerTeam,
            "player_class" => GameEventType::PlayerClass,
            "player_death" => GameEventType::PlayerDeath,
            "player_hurt" => GameEventType::PlayerHurt,
            "player_chat" => GameEventType::PlayerChat,
            "player_score" => GameEventType::PlayerScore,
            "player_spawn" => GameEventType::PlayerSpawn,
            "player_shoot" => GameEventType::PlayerShoot,
            "player_use" => GameEventType::PlayerUse,
            "player_changename" => GameEventType::PlayerChangeName,
            "player_hintmessage" => GameEventType::PlayerHintMessage,
            "base_player_teleported" => GameEventType::BasePlayerTeleported,
            "game_init" => GameEventType::GameInit,
            "game_newmap" => GameEventType::GameNewMap,
            "game_start" => GameEventType::GameStart,
            "game_end" => GameEventType::GameEnd,
            "round_start" => GameEventType::RoundStart,
            "round_end" => GameEventType::RoundEnd,
            "game_message" => GameEventType::GameMessage,
            "break_breakable" => GameEventType::BreakBreakable,
            "break_prop" => GameEventType::BreakProp,
            "entity_killed" => GameEventType::EntityKilled,
            "bonus_updated" => GameEventType::BonusUpdated,
            "achievement_event" => GameEventType::AchievementEvent,
            "achievement_increment" => GameEventType::AchievementIncrement,
            "physgun_pickup" => GameEventType::PhysgunPickup,
            "flare_ignite_npc" => GameEventType::FlareIgniteNpc,
            "helicopter_grenade_punt_miss" => GameEventType::HelicopterGrenadePuntMiss,
            "user_data_downloaded" => GameEventType::UserDataDownloaded,
            "ragdoll_dissolved" => GameEventType::RagdollDissolved,
            "hltv_changed_mode" => GameEventType::HLTVChangedMode,
            "hltv_changed_target" => GameEventType::HLTVChangedTarget,
            "vote_ended" => GameEventType::VoteEnded,
            "vote_started" => GameEventType::VoteStarted,
            "vote_changed" => GameEventType::VoteChanged,
            "vote_passed" => GameEventType::VotePassed,
            "vote_failed" => GameEventType::VoteFailed,
            "vote_cast" => GameEventType::VoteCast,
            "vote_options" => GameEventType::VoteOptions,
            "replay_saved" => GameEventType::ReplaySaved,
            "entered_performance_mode" => GameEventType::EnteredPerformanceMode,
            "browse_replays" => GameEventType::BrowseReplays,
            "replay_youtube_stats" => GameEventType::ReplayYoutubeStats,
            "inventory_updated" => GameEventType::InventoryUpdated,
            "cart_updated" => GameEventType::CartUpdated,
            "store_pricesheet_updated" => GameEventType::StorePriceSheetUpdated,
            "econ_inventory_connected" => GameEventType::EconInventoryConnected,
            "item_schema_initialized" => GameEventType::ItemSchemaInitialized,
            "gc_new_session" => GameEventType::GcNewSession,
            "gc_lost_session" => GameEventType::GcLostSession,
            "intro_finish" => GameEventType::IntroFinish,
            "intro_nextcamera" => GameEventType::IntroNextCamera,
            "player_changeclass" => GameEventType::PlayerChangeClass,
            "tf_map_time_remaining" => GameEventType::TfMapTimeRemaining,
            "tf_game_over" => GameEventType::TfGameOver,
            "ctf_flag_captured" => GameEventType::CtfFlagCaptured,
            "controlpoint_initialized" => GameEventType::ControlPointInitialized,
            "controlpoint_updateimages" => GameEventType::ControlPointUpdateImages,
            "controlpoint_updatelayout" => GameEventType::ControlPointUpdateLayout,
            "controlpoint_updatecapping" => GameEventType::ControlPointUpdateCapping,
            "controlpoint_updateowner" => GameEventType::ControlPointUpdateOwner,
            "controlpoint_starttouch" => GameEventType::ControlPointStartTouch,
            "controlpoint_endtouch" => GameEventType::ControlPointEndTouch,
            "controlpoint_pulse_element" => GameEventType::ControlPointPulseElement,
            "controlpoint_fake_capture" => GameEventType::ControlPointFakeCapture,
            "controlpoint_fake_capture_mult" => GameEventType::ControlPointFakeCaptureMultiplier,
            "teamplay_round_selected" => GameEventType::TeamPlayRoundSelected,
            "teamplay_round_start" => GameEventType::TeamPlayRoundStart,
            "teamplay_round_active" => GameEventType::TeamPlayRoundActive,
            "teamplay_waiting_begins" => GameEventType::TeamPlayWaitingBegins,
            "teamplay_waiting_ends" => GameEventType::TeamPlayWaitingEnds,
            "teamplay_waiting_abouttoend" => GameEventType::TeamPlayWaitingAboutToEnd,
            "teamplay_restart_round" => GameEventType::TeamPlayRestartRound,
            "teamplay_ready_restart" => GameEventType::TeamPlayReadyRestart,
            "teamplay_round_restart_seconds" => GameEventType::TeamPlayRoundRestartSeconds,
            "teamplay_team_ready" => GameEventType::TeamPlayTeamReady,
            "teamplay_round_win" => GameEventType::TeamPlayRoundWin,
            "teamplay_update_timer" => GameEventType::TeamPlayUpdateTimer,
            "teamplay_round_stalemate" => GameEventType::TeamPlayRoundStalemate,
            "teamplay_overtime_begin" => GameEventType::TeamPlayOvertimeBegin,
            "teamplay_overtime_end" => GameEventType::TeamPlayOvertimeEnd,
            "teamplay_suddendeath_begin" => GameEventType::TeamPlaySuddenDeathBegin,
            "teamplay_suddendeath_end" => GameEventType::TeamPlaySuddenDeathEnd,
            "teamplay_game_over" => GameEventType::TeamPlayGameOver,
            "teamplay_map_time_remaining" => GameEventType::TeamPlayMapTimeRemaining,
            "teamplay_timer_flash" => GameEventType::TeamPlayTimerFlash,
            "teamplay_timer_time_added" => GameEventType::TeamPlayTimerTimeAdded,
            "teamplay_point_startcapture" => GameEventType::TeamPlayPointStartCapture,
            "teamplay_point_captured" => GameEventType::TeamPlayPointCaptured,
            "teamplay_point_locked" => GameEventType::TeamPlayPointLocked,
            "teamplay_point_unlocked" => GameEventType::TeamPlayPointUnlocked,
            "teamplay_capture_broken" => GameEventType::TeamPlayCaptureBroken,
            "teamplay_capture_blocked" => GameEventType::TeamPlayCaptureBlocked,
            "teamplay_flag_event" => GameEventType::TeamPlayFlagEvent,
            "teamplay_win_panel" => GameEventType::TeamPlayWinPanel,
            "teamplay_teambalanced_player" => GameEventType::TeamPlayTeamBalancedPlayer,
            "teamplay_setup_finished" => GameEventType::TeamPlaySetupFinished,
            "teamplay_alert" => GameEventType::TeamPlayAlert,
            "training_complete" => GameEventType::TrainingComplete,
            "show_freezepanel" => GameEventType::ShowFreezePanel,
            "hide_freezepanel" => GameEventType::HideFreezePanel,
            "freezecam_started" => GameEventType::FreezeCamStarted,
            "localplayer_changeteam" => GameEventType::LocalPlayerChangeTeam,
            "localplayer_score_changed" => GameEventType::LocalPlayerScoreChanged,
            "localplayer_changeclass" => GameEventType::LocalPlayerChangeClass,
            "localplayer_respawn" => GameEventType::LocalPlayerRespawn,
            "building_info_changed" => GameEventType::BuildingInfoChanged,
            "localplayer_changedisguise" => GameEventType::LocalPlayerChangeDisguise,
            "player_account_changed" => GameEventType::PlayerAccountChanged,
            "spy_pda_reset" => GameEventType::SpyPdaReset,
            "flagstatus_update" => GameEventType::FlagStatusUpdate,
            "player_stats_updated" => GameEventType::PlayerStatsUpdated,
            "playing_commentary" => GameEventType::PlayingCommentary,
            "player_chargedeployed" => GameEventType::PlayerChargeDeployed,
            "player_builtobject" => GameEventType::PlayerBuiltObject,
            "player_upgradedobject" => GameEventType::PlayerUpgradedObject,
            "player_carryobject" => GameEventType::PlayerCarryObject,
            "player_dropobject" => GameEventType::PlayerDropObject,
            "object_removed" => GameEventType::ObjectRemoved,
            "object_destroyed" => GameEventType::ObjectDestroyed,
            "object_detonated" => GameEventType::ObjectDetonated,
            "achievement_earned" => GameEventType::AchievementEarned,
            "spec_target_updated" => GameEventType::SpecTargetUpdated,
            "tournament_stateupdate" => GameEventType::TournamentStateUpdate,
            "tournament_enablecountdown" => GameEventType::TournamentEnableCountdown,
            "player_calledformedic" => GameEventType::PlayerCalledForMedic,
            "player_askedforball" => GameEventType::PlayerAskedForBall,
            "localplayer_becameobserver" => GameEventType::LocalPlayerBecameObserver,
            "player_ignited_inv" => GameEventType::PlayerIgnitedInv,
            "player_ignited" => GameEventType::PlayerIgnited,
            "player_extinguished" => GameEventType::PlayerExtinguished,
            "player_teleported" => GameEventType::PlayerTeleported,
            "player_healedmediccall" => GameEventType::PlayerHealedMedicCall,
            "localplayer_chargeready" => GameEventType::LocalPlayerChargeReady,
            "localplayer_winddown" => GameEventType::LocalPlayerWindDown,
            "player_invulned" => GameEventType::PlayerInvulned,
            "escort_speed" => GameEventType::EscortSpeed,
            "escort_progress" => GameEventType::EscortProgress,
            "escort_recede" => GameEventType::EscortRecede,
            "gameui_activated" => GameEventType::GameUIActivated,
            "gameui_hidden" => GameEventType::GameUIHidden,
            "player_escort_score" => GameEventType::PlayerEscortScore,
            "player_healonhit" => GameEventType::PlayerHealOnHit,
            "player_stealsandvich" => GameEventType::PlayerStealSandvich,
            "show_class_layout" => GameEventType::ShowClassLayout,
            "show_vs_panel" => GameEventType::ShowVsPanel,
            "player_damaged" => GameEventType::PlayerDamaged,
            "arena_player_notification" => GameEventType::ArenaPlayerNotification,
            "arena_match_maxstreak" => GameEventType::ArenaMatchMaxStreak,
            "arena_round_start" => GameEventType::ArenaRoundStart,
            "arena_win_panel" => GameEventType::ArenaWinPanel,
            "pve_win_panel" => GameEventType::PveWinPanel,
            "air_dash" => GameEventType::AirDash,
            "landed" => GameEventType::Landed,
            "player_damage_dodged" => GameEventType::PlayerDamageDodged,
            "player_stunned" => GameEventType::PlayerStunned,
            "scout_grand_slam" => GameEventType::ScoutGrandSlam,
            "scout_slamdoll_landed" => GameEventType::ScoutSlamdollLanded,
            "arrow_impact" => GameEventType::ArrowImpact,
            "player_jarated" => GameEventType::PlayerJarated,
            "player_jarated_fade" => GameEventType::PlayerJaratedFade,
            "player_shield_blocked" => GameEventType::PlayerShieldBlocked,
            "player_pinned" => GameEventType::PlayerPinned,
            "player_healedbymedic" => GameEventType::PlayerHealedByMedic,
            "player_sapped_object" => GameEventType::PlayerSappedObject,
            "item_found" => GameEventType::ItemFound,
            "show_annotation" => GameEventType::ShowAnnotation,
            "hide_annotation" => GameEventType::HideAnnotation,
            "post_inventory_application" => GameEventType::PostInventoryApplication,
            "controlpoint_unlock_updated" => GameEventType::ControlPointUnlockUpdated,
            "deploy_buff_banner" => GameEventType::DeployBuffBanner,
            "player_buff" => GameEventType::PlayerBuff,
            "medic_death" => GameEventType::MedicDeath,
            "overtime_nag" => GameEventType::OvertimeNag,
            "teams_changed" => GameEventType::TeamsChanged,
            "halloween_pumpkin_grab" => GameEventType::HalloweenPumpkinGrab,
            "rocket_jump" => GameEventType::RocketJump,
            "rocket_jump_landed" => GameEventType::RocketJumpLanded,
            "sticky_jump" => GameEventType::StickyJump,
            "sticky_jump_landed" => GameEventType::StickyJumpLanded,
            "rocketpack_launch" => GameEventType::RocketPackLaunch,
            "rocketpack_landed" => GameEventType::RocketPackLanded,
            "medic_defended" => GameEventType::MedicDefended,
            "localplayer_healed" => GameEventType::LocalPlayerHealed,
            "player_destroyed_pipebomb" => GameEventType::PlayerDestroyedPipeBomb,
            "object_deflected" => GameEventType::ObjectDeflected,
            "player_mvp" => GameEventType::PlayerMvp,
            "raid_spawn_mob" => GameEventType::RaidSpawnMob,
            "raid_spawn_squad" => GameEventType::RaidSpawnSquad,
            "nav_blocked" => GameEventType::NavBlocked,
            "path_track_passed" => GameEventType::PathTrackPassed,
            "num_cappers_changed" => GameEventType::NumCappersChanged,
            "player_regenerate" => GameEventType::PlayerRegenerate,
            "update_status_item" => GameEventType::UpdateStatusItem,
            "stats_resetround" => GameEventType::StatsResetRound,
            "scorestats_accumulated_update" => GameEventType::ScoreStatsAccumulatedUpdate,
            "scorestats_accumulated_reset" => GameEventType::ScoreStatsAccumulatedReset,
            "achievement_earned_local" => GameEventType::AchievementEarnedLocal,
            "player_healed" => GameEventType::PlayerHealed,
            "building_healed" => GameEventType::BuildingHealed,
            "item_pickup" => GameEventType::ItemPickup,
            "duel_status" => GameEventType::DuelStatus,
            "fish_notice" => GameEventType::FishNotice,
            "fish_notice__arm" => GameEventType::FishNoticeArm,
            "slap_notice" => GameEventType::SlapNotice,
            "throwable_hit" => GameEventType::ThrowableHit,
            "pumpkin_lord_summoned" => GameEventType::PumpkinLordSummoned,
            "pumpkin_lord_killed" => GameEventType::PumpkinLordKilled,
            "merasmus_summoned" => GameEventType::MerasmusSummoned,
            "merasmus_killed" => GameEventType::MerasmusKilled,
            "merasmus_escape_warning" => GameEventType::MerasmusEscapeWarning,
            "merasmus_escaped" => GameEventType::MerasmusEscaped,
            "eyeball_boss_summoned" => GameEventType::EyeballBossSummoned,
            "eyeball_boss_stunned" => GameEventType::EyeballBossStunned,
            "eyeball_boss_killed" => GameEventType::EyeballBossKilled,
            "eyeball_boss_killer" => GameEventType::EyeballBossKiller,
            "eyeball_boss_escape_imminent" => GameEventType::EyeballBossEscapeImminent,
            "eyeball_boss_escaped" => GameEventType::EyeballBossEscaped,
            "npc_hurt" => GameEventType::NpcHurt,
            "controlpoint_timer_updated" => GameEventType::ControlPointTimerUpdated,
            "player_highfive_start" => GameEventType::PlayerHighFiveStart,
            "player_highfive_cancel" => GameEventType::PlayerHighFiveCancel,
            "player_highfive_success" => GameEventType::PlayerHighFiveSuccess,
            "player_bonuspoints" => GameEventType::PlayerBonusPoints,
            "player_upgraded" => GameEventType::PlayerUpgraded,
            "player_buyback" => GameEventType::PlayerBuyback,
            "player_used_powerup_bottle" => GameEventType::PlayerUsedPowerUpBottle,
            "christmas_gift_grab" => GameEventType::ChristmasGiftGrab,
            "player_killed_achievement_zone" => GameEventType::PlayerKilledAchievementZone,
            "party_updated" => GameEventType::PartyUpdated,
            "party_pref_changed" => GameEventType::PartyPrefChanged,
            "party_criteria_changed" => GameEventType::PartyCriteriaChanged,
            "party_invites_changed" => GameEventType::PartyInvitesChanged,
            "party_queue_state_changed" => GameEventType::PartyQueueStateChanged,
            "party_chat" => GameEventType::PartyChat,
            "party_member_join" => GameEventType::PartyMemberJoin,
            "party_member_leave" => GameEventType::PartyMemberLeave,
            "match_invites_updated" => GameEventType::MatchInvitesUpdated,
            "lobby_updated" => GameEventType::LobbyUpdated,
            "mvm_mission_update" => GameEventType::MvmMissionUpdate,
            "recalculate_holidays" => GameEventType::RecalculateHolidays,
            "player_currency_changed" => GameEventType::PlayerCurrencyChanged,
            "doomsday_rocket_open" => GameEventType::DoomsdayRocketOpen,
            "remove_nemesis_relationships" => GameEventType::RemoveNemesisRelationships,
            "mvm_creditbonus_wave" => GameEventType::MvmCreditBonusWave,
            "mvm_creditbonus_all" => GameEventType::MvmCreditBonusAll,
            "mvm_creditbonus_all_advanced" => GameEventType::MvmCreditBonusAllAdvanced,
            "mvm_quick_sentry_upgrade" => GameEventType::MvmQuickSentryUpgrade,
            "mvm_tank_destroyed_by_players" => GameEventType::MvmTankDestroyedByPlayers,
            "mvm_kill_robot_delivering_bomb" => GameEventType::MvmKillRobotDeliveringBomb,
            "mvm_pickup_currency" => GameEventType::MvmPickupCurrency,
            "mvm_bomb_carrier_killed" => GameEventType::MvmBombCarrierKilled,
            "mvm_sentrybuster_detonate" => GameEventType::MvmSentryBusterDetonate,
            "mvm_scout_marked_for_death" => GameEventType::MvmScoutMarkedForDeath,
            "mvm_medic_powerup_shared" => GameEventType::MvmMedicPowerUpShared,
            "mvm_begin_wave" => GameEventType::MvmBeginWave,
            "mvm_wave_complete" => GameEventType::MvmWaveComplete,
            "mvm_mission_complete" => GameEventType::MvmMissionComplete,
            "mvm_bomb_reset_by_player" => GameEventType::MvmBombResetByPlayer,
            "mvm_bomb_alarm_triggered" => GameEventType::MvmBombAlarmTriggered,
            "mvm_bomb_deploy_reset_by_player" => GameEventType::MvmBombDeployResetByPlayer,
            "mvm_wave_failed" => GameEventType::MvmWaveFailed,
            "mvm_reset_stats" => GameEventType::MvmResetStats,
            "damage_resisted" => GameEventType::DamageResisted,
            "revive_player_notify" => GameEventType::RevivePlayerNotify,
            "revive_player_stopped" => GameEventType::RevivePlayerStopped,
            "revive_player_complete" => GameEventType::RevivePlayerComplete,
            "player_turned_to_ghost" => GameEventType::PlayerTurnedToGhost,
            "medigun_shield_blocked_damage" => GameEventType::MedigunShieldBlockedDamage,
            "mvm_adv_wave_complete_no_gates" => GameEventType::MvmAdvWaveCompleteNoGates,
            "mvm_sniper_headshot_currency" => GameEventType::MvmSniperHeadshotCurrency,
            "mvm_mannhattan_pit" => GameEventType::MvmMannhattanPit,
            "flag_carried_in_detection_zone" => GameEventType::FlagCarriedInDetectionZone,
            "mvm_adv_wave_killed_stun_radio" => GameEventType::MvmAdvWaveKilledStunRadio,
            "player_directhit_stun" => GameEventType::PlayerDirectHitStun,
            "mvm_sentrybuster_killed" => GameEventType::MvmSentryBusterKilled,
            "upgrades_file_changed" => GameEventType::UpgradesFileChanged,
            "rd_team_points_changed" => GameEventType::RdTeamPointsChanged,
            "rd_rules_state_changed" => GameEventType::RdRulesStateChanged,
            "rd_robot_killed" => GameEventType::RdRobotKilled,
            "rd_robot_impact" => GameEventType::RdRobotImpact,
            "teamplay_pre_round_time_left" => GameEventType::TeamPlayPreRoundTimeLeft,
            "parachute_deploy" => GameEventType::ParachuteDeploy,
            "parachute_holster" => GameEventType::ParachuteHolster,
            "kill_refills_meter" => GameEventType::KillRefillsMeter,
            "rps_taunt_event" => GameEventType::RpsTauntEvent,
            "conga_kill" => GameEventType::CongaKill,
            "player_initial_spawn" => GameEventType::PlayerInitialSpawn,
            "competitive_victory" => GameEventType::CompetitiveVictory,
            "competitive_stats_update" => GameEventType::CompetitiveStatsUpdate,
            "minigame_win" => GameEventType::MiniGameWin,
            "sentry_on_go_active" => GameEventType::SentryOnGoActive,
            "duck_xp_level_up" => GameEventType::DuckXpLevelUp,
            "questlog_opened" => GameEventType::QuestLogOpened,
            "schema_updated" => GameEventType::SchemaUpdated,
            "localplayer_pickup_weapon" => GameEventType::LocalPlayerPickupWeapon,
            "rd_player_score_points" => GameEventType::RdPlayerScorePoints,
            "demoman_det_stickies" => GameEventType::DemomanDetStickies,
            "quest_objective_completed" => GameEventType::QuestObjectiveCompleted,
            "player_score_changed" => GameEventType::PlayerScoreChanged,
            "killed_capping_player" => GameEventType::KilledCappingPlayer,
            "environmental_death" => GameEventType::EnvironmentalDeath,
            "projectile_direct_hit" => GameEventType::ProjectileDirectHit,
            "pass_get" => GameEventType::PassGet,
            "pass_score" => GameEventType::PassScore,
            "pass_free" => GameEventType::PassFree,
            "pass_pass_caught" => GameEventType::PassPassCaught,
            "pass_ball_stolen" => GameEventType::PassBallStolen,
            "pass_ball_blocked" => GameEventType::PassBallBlocked,
            "damage_prevented" => GameEventType::DamagePrevented,
            "halloween_boss_killed" => GameEventType::HalloweenBossKilled,
            "escaped_loot_island" => GameEventType::EscapedLootIsland,
            "tagged_player_as_it" => GameEventType::TaggedPlayerAsIt,
            "merasmus_stunned" => GameEventType::MerasmusStunned,
            "merasmus_prop_found" => GameEventType::MerasmusPropFound,
            "halloween_skeleton_killed" => GameEventType::HalloweenSkeletonKilled,
            "skeleton_killed_quest" => GameEventType::SkeletonKilledQuest,
            "skeleton_king_killed_quest" => GameEventType::SkeletonKingKilledQuest,
            "escape_hell" => GameEventType::EscapeHell,
            "cross_spectral_bridge" => GameEventType::CrossSpectralBridge,
            "minigame_won" => GameEventType::MiniGameWon,
            "respawn_ghost" => GameEventType::RespawnGhost,
            "kill_in_hell" => GameEventType::KillInHell,
            "halloween_duck_collected" => GameEventType::HalloweenDuckCollected,
            "special_score" => GameEventType::SpecialScore,
            "team_leader_killed" => GameEventType::TeamLeaderKilled,
            "halloween_soul_collected" => GameEventType::HalloweenSoulCollected,
            "recalculate_truce" => GameEventType::RecalculateTruce,
            "deadringer_cheat_death" => GameEventType::DeadRingerCheatDeath,
            "crossbow_heal" => GameEventType::CrossbowHeal,
            "damage_mitigated" => GameEventType::DamageMitigated,
            "payload_pushed" => GameEventType::PayloadPushed,
            "player_abandoned_match" => GameEventType::PlayerAbandonedMatch,
            "cl_drawline" => GameEventType::ClDrawline,
            "restart_timer_time" => GameEventType::RestartTimerTime,
            "winlimit_changed" => GameEventType::WinLimitChanged,
            "winpanel_show_scores" => GameEventType::WinPanelShowScores,
            "top_streams_request_finished" => GameEventType::TopStreamsRequestFinished,
            "competitive_state_changed" => GameEventType::CompetitiveStateChanged,
            "global_war_data_updated" => GameEventType::GlobalWarDataUpdated,
            "stop_watch_changed" => GameEventType::StopWatchChanged,
            "ds_stop" => GameEventType::DsStop,
            "ds_screenshot" => GameEventType::DsScreenshot,
            "show_match_summary" => GameEventType::ShowMatchSummary,
            "experience_changed" => GameEventType::ExperienceChanged,
            "begin_xp_lerp" => GameEventType::BeginXpLerp,
            "matchmaker_stats_updated" => GameEventType::MatchmakerStatsUpdated,
            "rematch_vote_period_over" => GameEventType::RematchVotePeriodOver,
            "rematch_failed_to_create" => GameEventType::RematchFailedToCreate,
            "player_rematch_change" => GameEventType::PlayerRematchChange,
            "ping_updated" => GameEventType::PingUpdated,
            "mmstats_updated" => GameEventType::MMStatsUpdated,
            "player_next_map_vote_change" => GameEventType::PlayerNextMapVoteChange,
            "vote_maps_changed" => GameEventType::VoteMapsChanged,
            "proto_def_changed" => GameEventType::ProtoDefChanged,
            "player_domination" => GameEventType::PlayerDomination,
            "player_rocketpack_pushed" => GameEventType::PlayerRocketPackPushed,
            "quest_request" => GameEventType::QuestRequest,
            "quest_response" => GameEventType::QuestResponse,
            "quest_progress" => GameEventType::QuestProgress,
            "projectile_removed" => GameEventType::ProjectileRemoved,
            "quest_map_data_changed" => GameEventType::QuestMapDataChanged,
            "gas_doused_player_ignited" => GameEventType::GasDousedPlayerIgnited,
            "quest_turn_in_state" => GameEventType::QuestTurnInState,
            "items_acknowledged" => GameEventType::ItemsAcknowledged,
            "capper_killed" => GameEventType::CapperKilled,
            "mainmenu_stabilized" => GameEventType::MainMenuStabilized,
            "world_status_changed" => GameEventType::WorldStatusChanged,
            "hltv_status" => GameEventType::HLTVStatus,
            "hltv_cameraman" => GameEventType::HLTVCameraman,
            "hltv_rank_camera" => GameEventType::HLTVRankCamera,
            "hltv_rank_entity" => GameEventType::HLTVRankEntity,
            "hltv_fixed" => GameEventType::HLTVFixed,
            "hltv_chase" => GameEventType::HLTVChase,
            "hltv_message" => GameEventType::HLTVMessage,
            "hltv_title" => GameEventType::HLTVTitle,
            "hltv_chat" => GameEventType::HLTVChat,
            "replay_startrecord" => GameEventType::ReplayStartRecord,
            "replay_sessioninfo" => GameEventType::ReplaySessionInfo,
            "replay_endrecord" => GameEventType::ReplayEndRecord,
            "replay_replaysavailable" => GameEventType::ReplayReplaysAvailable,
            "replay_servererror" => GameEventType::ReplayServerError,
            ty => GameEventType::Unknown(ty.into()),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            GameEventType::ServerSpawn => "server_spawn",
            GameEventType::ServerChangeLevelFailed => "server_changelevel_failed",
            GameEventType::ServerShutdown => "server_shutdown",
            GameEventType::ServerCvar => "server_cvar",
            GameEventType::ServerMessage => "server_message",
            GameEventType::ServerAddBan => "server_addban",
            GameEventType::ServerRemoveBan => "server_removeban",
            GameEventType::PlayerConnect => "player_connect",
            GameEventType::PlayerConnectClient => "player_connect_client",
            GameEventType::PlayerInfo => "player_info",
            GameEventType::PlayerDisconnect => "player_disconnect",
            GameEventType::PlayerActivate => "player_activate",
            GameEventType::PlayerSay => "player_say",
            GameEventType::ClientDisconnect => "client_disconnect",
            GameEventType::ClientBeginConnect => "client_beginconnect",
            GameEventType::ClientConnected => "client_connected",
            GameEventType::ClientFullConnect => "client_fullconnect",
            GameEventType::HostQuit => "host_quit",
            GameEventType::TeamInfo => "team_info",
            GameEventType::TeamScore => "team_score",
            GameEventType::TeamPlayBroadcastAudio => "teamplay_broadcast_audio",
            GameEventType::PlayerTeam => "player_team",
            GameEventType::PlayerClass => "player_class",
            GameEventType::PlayerDeath => "player_death",
            GameEventType::PlayerHurt => "player_hurt",
            GameEventType::PlayerChat => "player_chat",
            GameEventType::PlayerScore => "player_score",
            GameEventType::PlayerSpawn => "player_spawn",
            GameEventType::PlayerShoot => "player_shoot",
            GameEventType::PlayerUse => "player_use",
            GameEventType::PlayerChangeName => "player_changename",
            GameEventType::PlayerHintMessage => "player_hintmessage",
            GameEventType::BasePlayerTeleported => "base_player_teleported",
            GameEventType::GameInit => "game_init",
            GameEventType::GameNewMap => "game_newmap",
            GameEventType::GameStart => "game_start",
            GameEventType::GameEnd => "game_end",
            GameEventType::RoundStart => "round_start",
            GameEventType::RoundEnd => "round_end",
            GameEventType::GameMessage => "game_message",
            GameEventType::BreakBreakable => "break_breakable",
            GameEventType::BreakProp => "break_prop",
            GameEventType::EntityKilled => "entity_killed",
            GameEventType::BonusUpdated => "bonus_updated",
            GameEventType::AchievementEvent => "achievement_event",
            GameEventType::AchievementIncrement => "achievement_increment",
            GameEventType::PhysgunPickup => "physgun_pickup",
            GameEventType::FlareIgniteNpc => "flare_ignite_npc",
            GameEventType::HelicopterGrenadePuntMiss => "helicopter_grenade_punt_miss",
            GameEventType::UserDataDownloaded => "user_data_downloaded",
            GameEventType::RagdollDissolved => "ragdoll_dissolved",
            GameEventType::HLTVChangedMode => "hltv_changed_mode",
            GameEventType::HLTVChangedTarget => "hltv_changed_target",
            GameEventType::VoteEnded => "vote_ended",
            GameEventType::VoteStarted => "vote_started",
            GameEventType::VoteChanged => "vote_changed",
            GameEventType::VotePassed => "vote_passed",
            GameEventType::VoteFailed => "vote_failed",
            GameEventType::VoteCast => "vote_cast",
            GameEventType::VoteOptions => "vote_options",
            GameEventType::ReplaySaved => "replay_saved",
            GameEventType::EnteredPerformanceMode => "entered_performance_mode",
            GameEventType::BrowseReplays => "browse_replays",
            GameEventType::ReplayYoutubeStats => "replay_youtube_stats",
            GameEventType::InventoryUpdated => "inventory_updated",
            GameEventType::CartUpdated => "cart_updated",
            GameEventType::StorePriceSheetUpdated => "store_pricesheet_updated",
            GameEventType::EconInventoryConnected => "econ_inventory_connected",
            GameEventType::ItemSchemaInitialized => "item_schema_initialized",
            GameEventType::GcNewSession => "gc_new_session",
            GameEventType::GcLostSession => "gc_lost_session",
            GameEventType::IntroFinish => "intro_finish",
            GameEventType::IntroNextCamera => "intro_nextcamera",
            GameEventType::PlayerChangeClass => "player_changeclass",
            GameEventType::TfMapTimeRemaining => "tf_map_time_remaining",
            GameEventType::TfGameOver => "tf_game_over",
            GameEventType::CtfFlagCaptured => "ctf_flag_captured",
            GameEventType::ControlPointInitialized => "controlpoint_initialized",
            GameEventType::ControlPointUpdateImages => "controlpoint_updateimages",
            GameEventType::ControlPointUpdateLayout => "controlpoint_updatelayout",
            GameEventType::ControlPointUpdateCapping => "controlpoint_updatecapping",
            GameEventType::ControlPointUpdateOwner => "controlpoint_updateowner",
            GameEventType::ControlPointStartTouch => "controlpoint_starttouch",
            GameEventType::ControlPointEndTouch => "controlpoint_endtouch",
            GameEventType::ControlPointPulseElement => "controlpoint_pulse_element",
            GameEventType::ControlPointFakeCapture => "controlpoint_fake_capture",
            GameEventType::ControlPointFakeCaptureMultiplier => "controlpoint_fake_capture_mult",
            GameEventType::TeamPlayRoundSelected => "teamplay_round_selected",
            GameEventType::TeamPlayRoundStart => "teamplay_round_start",
            GameEventType::TeamPlayRoundActive => "teamplay_round_active",
            GameEventType::TeamPlayWaitingBegins => "teamplay_waiting_begins",
            GameEventType::TeamPlayWaitingEnds => "teamplay_waiting_ends",
            GameEventType::TeamPlayWaitingAboutToEnd => "teamplay_waiting_abouttoend",
            GameEventType::TeamPlayRestartRound => "teamplay_restart_round",
            GameEventType::TeamPlayReadyRestart => "teamplay_ready_restart",
            GameEventType::TeamPlayRoundRestartSeconds => "teamplay_round_restart_seconds",
            GameEventType::TeamPlayTeamReady => "teamplay_team_ready",
            GameEventType::TeamPlayRoundWin => "teamplay_round_win",
            GameEventType::TeamPlayUpdateTimer => "teamplay_update_timer",
            GameEventType::TeamPlayRoundStalemate => "teamplay_round_stalemate",
            GameEventType::TeamPlayOvertimeBegin => "teamplay_overtime_begin",
            GameEventType::TeamPlayOvertimeEnd => "teamplay_overtime_end",
            GameEventType::TeamPlaySuddenDeathBegin => "teamplay_suddendeath_begin",
            GameEventType::TeamPlaySuddenDeathEnd => "teamplay_suddendeath_end",
            GameEventType::TeamPlayGameOver => "teamplay_game_over",
            GameEventType::TeamPlayMapTimeRemaining => "teamplay_map_time_remaining",
            GameEventType::TeamPlayTimerFlash => "teamplay_timer_flash",
            GameEventType::TeamPlayTimerTimeAdded => "teamplay_timer_time_added",
            GameEventType::TeamPlayPointStartCapture => "teamplay_point_startcapture",
            GameEventType::TeamPlayPointCaptured => "teamplay_point_captured",
            GameEventType::TeamPlayPointLocked => "teamplay_point_locked",
            GameEventType::TeamPlayPointUnlocked => "teamplay_point_unlocked",
            GameEventType::TeamPlayCaptureBroken => "teamplay_capture_broken",
            GameEventType::TeamPlayCaptureBlocked => "teamplay_capture_blocked",
            GameEventType::TeamPlayFlagEvent => "teamplay_flag_event",
            GameEventType::TeamPlayWinPanel => "teamplay_win_panel",
            GameEventType::TeamPlayTeamBalancedPlayer => "teamplay_teambalanced_player",
            GameEventType::TeamPlaySetupFinished => "teamplay_setup_finished",
            GameEventType::TeamPlayAlert => "teamplay_alert",
            GameEventType::TrainingComplete => "training_complete",
            GameEventType::ShowFreezePanel => "show_freezepanel",
            GameEventType::HideFreezePanel => "hide_freezepanel",
            GameEventType::FreezeCamStarted => "freezecam_started",
            GameEventType::LocalPlayerChangeTeam => "localplayer_changeteam",
            GameEventType::LocalPlayerScoreChanged => "localplayer_score_changed",
            GameEventType::LocalPlayerChangeClass => "localplayer_changeclass",
            GameEventType::LocalPlayerRespawn => "localplayer_respawn",
            GameEventType::BuildingInfoChanged => "building_info_changed",
            GameEventType::LocalPlayerChangeDisguise => "localplayer_changedisguise",
            GameEventType::PlayerAccountChanged => "player_account_changed",
            GameEventType::SpyPdaReset => "spy_pda_reset",
            GameEventType::FlagStatusUpdate => "flagstatus_update",
            GameEventType::PlayerStatsUpdated => "player_stats_updated",
            GameEventType::PlayingCommentary => "playing_commentary",
            GameEventType::PlayerChargeDeployed => "player_chargedeployed",
            GameEventType::PlayerBuiltObject => "player_builtobject",
            GameEventType::PlayerUpgradedObject => "player_upgradedobject",
            GameEventType::PlayerCarryObject => "player_carryobject",
            GameEventType::PlayerDropObject => "player_dropobject",
            GameEventType::ObjectRemoved => "object_removed",
            GameEventType::ObjectDestroyed => "object_destroyed",
            GameEventType::ObjectDetonated => "object_detonated",
            GameEventType::AchievementEarned => "achievement_earned",
            GameEventType::SpecTargetUpdated => "spec_target_updated",
            GameEventType::TournamentStateUpdate => "tournament_stateupdate",
            GameEventType::TournamentEnableCountdown => "tournament_enablecountdown",
            GameEventType::PlayerCalledForMedic => "player_calledformedic",
            GameEventType::PlayerAskedForBall => "player_askedforball",
            GameEventType::LocalPlayerBecameObserver => "localplayer_becameobserver",
            GameEventType::PlayerIgnitedInv => "player_ignited_inv",
            GameEventType::PlayerIgnited => "player_ignited",
            GameEventType::PlayerExtinguished => "player_extinguished",
            GameEventType::PlayerTeleported => "player_teleported",
            GameEventType::PlayerHealedMedicCall => "player_healedmediccall",
            GameEventType::LocalPlayerChargeReady => "localplayer_chargeready",
            GameEventType::LocalPlayerWindDown => "localplayer_winddown",
            GameEventType::PlayerInvulned => "player_invulned",
            GameEventType::EscortSpeed => "escort_speed",
            GameEventType::EscortProgress => "escort_progress",
            GameEventType::EscortRecede => "escort_recede",
            GameEventType::GameUIActivated => "gameui_activated",
            GameEventType::GameUIHidden => "gameui_hidden",
            GameEventType::PlayerEscortScore => "player_escort_score",
            GameEventType::PlayerHealOnHit => "player_healonhit",
            GameEventType::PlayerStealSandvich => "player_stealsandvich",
            GameEventType::ShowClassLayout => "show_class_layout",
            GameEventType::ShowVsPanel => "show_vs_panel",
            GameEventType::PlayerDamaged => "player_damaged",
            GameEventType::ArenaPlayerNotification => "arena_player_notification",
            GameEventType::ArenaMatchMaxStreak => "arena_match_maxstreak",
            GameEventType::ArenaRoundStart => "arena_round_start",
            GameEventType::ArenaWinPanel => "arena_win_panel",
            GameEventType::PveWinPanel => "pve_win_panel",
            GameEventType::AirDash => "air_dash",
            GameEventType::Landed => "landed",
            GameEventType::PlayerDamageDodged => "player_damage_dodged",
            GameEventType::PlayerStunned => "player_stunned",
            GameEventType::ScoutGrandSlam => "scout_grand_slam",
            GameEventType::ScoutSlamdollLanded => "scout_slamdoll_landed",
            GameEventType::ArrowImpact => "arrow_impact",
            GameEventType::PlayerJarated => "player_jarated",
            GameEventType::PlayerJaratedFade => "player_jarated_fade",
            GameEventType::PlayerShieldBlocked => "player_shield_blocked",
            GameEventType::PlayerPinned => "player_pinned",
            GameEventType::PlayerHealedByMedic => "player_healedbymedic",
            GameEventType::PlayerSappedObject => "player_sapped_object",
            GameEventType::ItemFound => "item_found",
            GameEventType::ShowAnnotation => "show_annotation",
            GameEventType::HideAnnotation => "hide_annotation",
            GameEventType::PostInventoryApplication => "post_inventory_application",
            GameEventType::ControlPointUnlockUpdated => "controlpoint_unlock_updated",
            GameEventType::DeployBuffBanner => "deploy_buff_banner",
            GameEventType::PlayerBuff => "player_buff",
            GameEventType::MedicDeath => "medic_death",
            GameEventType::OvertimeNag => "overtime_nag",
            GameEventType::TeamsChanged => "teams_changed",
            GameEventType::HalloweenPumpkinGrab => "halloween_pumpkin_grab",
            GameEventType::RocketJump => "rocket_jump",
            GameEventType::RocketJumpLanded => "rocket_jump_landed",
            GameEventType::StickyJump => "sticky_jump",
            GameEventType::StickyJumpLanded => "sticky_jump_landed",
            GameEventType::RocketPackLaunch => "rocketpack_launch",
            GameEventType::RocketPackLanded => "rocketpack_landed",
            GameEventType::MedicDefended => "medic_defended",
            GameEventType::LocalPlayerHealed => "localplayer_healed",
            GameEventType::PlayerDestroyedPipeBomb => "player_destroyed_pipebomb",
            GameEventType::ObjectDeflected => "object_deflected",
            GameEventType::PlayerMvp => "player_mvp",
            GameEventType::RaidSpawnMob => "raid_spawn_mob",
            GameEventType::RaidSpawnSquad => "raid_spawn_squad",
            GameEventType::NavBlocked => "nav_blocked",
            GameEventType::PathTrackPassed => "path_track_passed",
            GameEventType::NumCappersChanged => "num_cappers_changed",
            GameEventType::PlayerRegenerate => "player_regenerate",
            GameEventType::UpdateStatusItem => "update_status_item",
            GameEventType::StatsResetRound => "stats_resetround",
            GameEventType::ScoreStatsAccumulatedUpdate => "scorestats_accumulated_update",
            GameEventType::ScoreStatsAccumulatedReset => "scorestats_accumulated_reset",
            GameEventType::AchievementEarnedLocal => "achievement_earned_local",
            GameEventType::PlayerHealed => "player_healed",
            GameEventType::BuildingHealed => "building_healed",
            GameEventType::ItemPickup => "item_pickup",
            GameEventType::DuelStatus => "duel_status",
            GameEventType::FishNotice => "fish_notice",
            GameEventType::FishNoticeArm => "fish_notice__arm",
            GameEventType::SlapNotice => "slap_notice",
            GameEventType::ThrowableHit => "throwable_hit",
            GameEventType::PumpkinLordSummoned => "pumpkin_lord_summoned",
            GameEventType::PumpkinLordKilled => "pumpkin_lord_killed",
            GameEventType::MerasmusSummoned => "merasmus_summoned",
            GameEventType::MerasmusKilled => "merasmus_killed",
            GameEventType::MerasmusEscapeWarning => "merasmus_escape_warning",
            GameEventType::MerasmusEscaped => "merasmus_escaped",
            GameEventType::EyeballBossSummoned => "eyeball_boss_summoned",
            GameEventType::EyeballBossStunned => "eyeball_boss_stunned",
            GameEventType::EyeballBossKilled => "eyeball_boss_killed",
            GameEventType::EyeballBossKiller => "eyeball_boss_killer",
            GameEventType::EyeballBossEscapeImminent => "eyeball_boss_escape_imminent",
            GameEventType::EyeballBossEscaped => "eyeball_boss_escaped",
            GameEventType::NpcHurt => "npc_hurt",
            GameEventType::ControlPointTimerUpdated => "controlpoint_timer_updated",
            GameEventType::PlayerHighFiveStart => "player_highfive_start",
            GameEventType::PlayerHighFiveCancel => "player_highfive_cancel",
            GameEventType::PlayerHighFiveSuccess => "player_highfive_success",
            GameEventType::PlayerBonusPoints => "player_bonuspoints",
            GameEventType::PlayerUpgraded => "player_upgraded",
            GameEventType::PlayerBuyback => "player_buyback",
            GameEventType::PlayerUsedPowerUpBottle => "player_used_powerup_bottle",
            GameEventType::ChristmasGiftGrab => "christmas_gift_grab",
            GameEventType::PlayerKilledAchievementZone => "player_killed_achievement_zone",
            GameEventType::PartyUpdated => "party_updated",
            GameEventType::PartyPrefChanged => "party_pref_changed",
            GameEventType::PartyCriteriaChanged => "party_criteria_changed",
            GameEventType::PartyInvitesChanged => "party_invites_changed",
            GameEventType::PartyQueueStateChanged => "party_queue_state_changed",
            GameEventType::PartyChat => "party_chat",
            GameEventType::PartyMemberJoin => "party_member_join",
            GameEventType::PartyMemberLeave => "party_member_leave",
            GameEventType::MatchInvitesUpdated => "match_invites_updated",
            GameEventType::LobbyUpdated => "lobby_updated",
            GameEventType::MvmMissionUpdate => "mvm_mission_update",
            GameEventType::RecalculateHolidays => "recalculate_holidays",
            GameEventType::PlayerCurrencyChanged => "player_currency_changed",
            GameEventType::DoomsdayRocketOpen => "doomsday_rocket_open",
            GameEventType::RemoveNemesisRelationships => "remove_nemesis_relationships",
            GameEventType::MvmCreditBonusWave => "mvm_creditbonus_wave",
            GameEventType::MvmCreditBonusAll => "mvm_creditbonus_all",
            GameEventType::MvmCreditBonusAllAdvanced => "mvm_creditbonus_all_advanced",
            GameEventType::MvmQuickSentryUpgrade => "mvm_quick_sentry_upgrade",
            GameEventType::MvmTankDestroyedByPlayers => "mvm_tank_destroyed_by_players",
            GameEventType::MvmKillRobotDeliveringBomb => "mvm_kill_robot_delivering_bomb",
            GameEventType::MvmPickupCurrency => "mvm_pickup_currency",
            GameEventType::MvmBombCarrierKilled => "mvm_bomb_carrier_killed",
            GameEventType::MvmSentryBusterDetonate => "mvm_sentrybuster_detonate",
            GameEventType::MvmScoutMarkedForDeath => "mvm_scout_marked_for_death",
            GameEventType::MvmMedicPowerUpShared => "mvm_medic_powerup_shared",
            GameEventType::MvmBeginWave => "mvm_begin_wave",
            GameEventType::MvmWaveComplete => "mvm_wave_complete",
            GameEventType::MvmMissionComplete => "mvm_mission_complete",
            GameEventType::MvmBombResetByPlayer => "mvm_bomb_reset_by_player",
            GameEventType::MvmBombAlarmTriggered => "mvm_bomb_alarm_triggered",
            GameEventType::MvmBombDeployResetByPlayer => "mvm_bomb_deploy_reset_by_player",
            GameEventType::MvmWaveFailed => "mvm_wave_failed",
            GameEventType::MvmResetStats => "mvm_reset_stats",
            GameEventType::DamageResisted => "damage_resisted",
            GameEventType::RevivePlayerNotify => "revive_player_notify",
            GameEventType::RevivePlayerStopped => "revive_player_stopped",
            GameEventType::RevivePlayerComplete => "revive_player_complete",
            GameEventType::PlayerTurnedToGhost => "player_turned_to_ghost",
            GameEventType::MedigunShieldBlockedDamage => "medigun_shield_blocked_damage",
            GameEventType::MvmAdvWaveCompleteNoGates => "mvm_adv_wave_complete_no_gates",
            GameEventType::MvmSniperHeadshotCurrency => "mvm_sniper_headshot_currency",
            GameEventType::MvmMannhattanPit => "mvm_mannhattan_pit",
            GameEventType::FlagCarriedInDetectionZone => "flag_carried_in_detection_zone",
            GameEventType::MvmAdvWaveKilledStunRadio => "mvm_adv_wave_killed_stun_radio",
            GameEventType::PlayerDirectHitStun => "player_directhit_stun",
            GameEventType::MvmSentryBusterKilled => "mvm_sentrybuster_killed",
            GameEventType::UpgradesFileChanged => "upgrades_file_changed",
            GameEventType::RdTeamPointsChanged => "rd_team_points_changed",
            GameEventType::RdRulesStateChanged => "rd_rules_state_changed",
            GameEventType::RdRobotKilled => "rd_robot_killed",
            GameEventType::RdRobotImpact => "rd_robot_impact",
            GameEventType::TeamPlayPreRoundTimeLeft => "teamplay_pre_round_time_left",
            GameEventType::ParachuteDeploy => "parachute_deploy",
            GameEventType::ParachuteHolster => "parachute_holster",
            GameEventType::KillRefillsMeter => "kill_refills_meter",
            GameEventType::RpsTauntEvent => "rps_taunt_event",
            GameEventType::CongaKill => "conga_kill",
            GameEventType::PlayerInitialSpawn => "player_initial_spawn",
            GameEventType::CompetitiveVictory => "competitive_victory",
            GameEventType::CompetitiveStatsUpdate => "competitive_stats_update",
            GameEventType::MiniGameWin => "minigame_win",
            GameEventType::SentryOnGoActive => "sentry_on_go_active",
            GameEventType::DuckXpLevelUp => "duck_xp_level_up",
            GameEventType::QuestLogOpened => "questlog_opened",
            GameEventType::SchemaUpdated => "schema_updated",
            GameEventType::LocalPlayerPickupWeapon => "localplayer_pickup_weapon",
            GameEventType::RdPlayerScorePoints => "rd_player_score_points",
            GameEventType::DemomanDetStickies => "demoman_det_stickies",
            GameEventType::QuestObjectiveCompleted => "quest_objective_completed",
            GameEventType::PlayerScoreChanged => "player_score_changed",
            GameEventType::KilledCappingPlayer => "killed_capping_player",
            GameEventType::EnvironmentalDeath => "environmental_death",
            GameEventType::ProjectileDirectHit => "projectile_direct_hit",
            GameEventType::PassGet => "pass_get",
            GameEventType::PassScore => "pass_score",
            GameEventType::PassFree => "pass_free",
            GameEventType::PassPassCaught => "pass_pass_caught",
            GameEventType::PassBallStolen => "pass_ball_stolen",
            GameEventType::PassBallBlocked => "pass_ball_blocked",
            GameEventType::DamagePrevented => "damage_prevented",
            GameEventType::HalloweenBossKilled => "halloween_boss_killed",
            GameEventType::EscapedLootIsland => "escaped_loot_island",
            GameEventType::TaggedPlayerAsIt => "tagged_player_as_it",
            GameEventType::MerasmusStunned => "merasmus_stunned",
            GameEventType::MerasmusPropFound => "merasmus_prop_found",
            GameEventType::HalloweenSkeletonKilled => "halloween_skeleton_killed",
            GameEventType::SkeletonKilledQuest => "skeleton_killed_quest",
            GameEventType::SkeletonKingKilledQuest => "skeleton_king_killed_quest",
            GameEventType::EscapeHell => "escape_hell",
            GameEventType::CrossSpectralBridge => "cross_spectral_bridge",
            GameEventType::MiniGameWon => "minigame_won",
            GameEventType::RespawnGhost => "respawn_ghost",
            GameEventType::KillInHell => "kill_in_hell",
            GameEventType::HalloweenDuckCollected => "halloween_duck_collected",
            GameEventType::SpecialScore => "special_score",
            GameEventType::TeamLeaderKilled => "team_leader_killed",
            GameEventType::HalloweenSoulCollected => "halloween_soul_collected",
            GameEventType::RecalculateTruce => "recalculate_truce",
            GameEventType::DeadRingerCheatDeath => "deadringer_cheat_death",
            GameEventType::CrossbowHeal => "crossbow_heal",
            GameEventType::DamageMitigated => "damage_mitigated",
            GameEventType::PayloadPushed => "payload_pushed",
            GameEventType::PlayerAbandonedMatch => "player_abandoned_match",
            GameEventType::ClDrawline => "cl_drawline",
            GameEventType::RestartTimerTime => "restart_timer_time",
            GameEventType::WinLimitChanged => "winlimit_changed",
            GameEventType::WinPanelShowScores => "winpanel_show_scores",
            GameEventType::TopStreamsRequestFinished => "top_streams_request_finished",
            GameEventType::CompetitiveStateChanged => "competitive_state_changed",
            GameEventType::GlobalWarDataUpdated => "global_war_data_updated",
            GameEventType::StopWatchChanged => "stop_watch_changed",
            GameEventType::DsStop => "ds_stop",
            GameEventType::DsScreenshot => "ds_screenshot",
            GameEventType::ShowMatchSummary => "show_match_summary",
            GameEventType::ExperienceChanged => "experience_changed",
            GameEventType::BeginXpLerp => "begin_xp_lerp",
            GameEventType::MatchmakerStatsUpdated => "matchmaker_stats_updated",
            GameEventType::RematchVotePeriodOver => "rematch_vote_period_over",
            GameEventType::RematchFailedToCreate => "rematch_failed_to_create",
            GameEventType::PlayerRematchChange => "player_rematch_change",
            GameEventType::PingUpdated => "ping_updated",
            GameEventType::MMStatsUpdated => "mmstats_updated",
            GameEventType::PlayerNextMapVoteChange => "player_next_map_vote_change",
            GameEventType::VoteMapsChanged => "vote_maps_changed",
            GameEventType::ProtoDefChanged => "proto_def_changed",
            GameEventType::PlayerDomination => "player_domination",
            GameEventType::PlayerRocketPackPushed => "player_rocketpack_pushed",
            GameEventType::QuestRequest => "quest_request",
            GameEventType::QuestResponse => "quest_response",
            GameEventType::QuestProgress => "quest_progress",
            GameEventType::ProjectileRemoved => "projectile_removed",
            GameEventType::QuestMapDataChanged => "quest_map_data_changed",
            GameEventType::GasDousedPlayerIgnited => "gas_doused_player_ignited",
            GameEventType::QuestTurnInState => "quest_turn_in_state",
            GameEventType::ItemsAcknowledged => "items_acknowledged",
            GameEventType::CapperKilled => "capper_killed",
            GameEventType::MainMenuStabilized => "mainmenu_stabilized",
            GameEventType::WorldStatusChanged => "world_status_changed",
            GameEventType::HLTVStatus => "hltv_status",
            GameEventType::HLTVCameraman => "hltv_cameraman",
            GameEventType::HLTVRankCamera => "hltv_rank_camera",
            GameEventType::HLTVRankEntity => "hltv_rank_entity",
            GameEventType::HLTVFixed => "hltv_fixed",
            GameEventType::HLTVChase => "hltv_chase",
            GameEventType::HLTVMessage => "hltv_message",
            GameEventType::HLTVTitle => "hltv_title",
            GameEventType::HLTVChat => "hltv_chat",
            GameEventType::ReplayStartRecord => "replay_startrecord",
            GameEventType::ReplaySessionInfo => "replay_sessioninfo",
            GameEventType::ReplayEndRecord => "replay_endrecord",
            GameEventType::ReplayReplaysAvailable => "replay_replaysavailable",
            GameEventType::ReplayServerError => "replay_servererror",
            GameEventType::Unknown(ty) => ty,
        }
    }
}
impl GameEvent {
    pub fn read(stream: &mut Stream, definition: &GameEventDefinition) -> Result<Self> {
        Ok(match definition.event_type {
            GameEventType::ServerSpawn => {
                GameEvent::ServerSpawn(Box::new(<ServerSpawnEvent>::read(stream, definition)?))
            }
            GameEventType::ServerChangeLevelFailed => GameEvent::ServerChangeLevelFailed(
                ServerChangeLevelFailedEvent::read(stream, definition)?,
            ),
            GameEventType::ServerShutdown => {
                GameEvent::ServerShutdown(ServerShutdownEvent::read(stream, definition)?)
            }
            GameEventType::ServerCvar => {
                GameEvent::ServerCvar(ServerCvarEvent::read(stream, definition)?)
            }
            GameEventType::ServerMessage => {
                GameEvent::ServerMessage(ServerMessageEvent::read(stream, definition)?)
            }
            GameEventType::ServerAddBan => {
                GameEvent::ServerAddBan(Box::new(<ServerAddBanEvent>::read(stream, definition)?))
            }
            GameEventType::ServerRemoveBan => {
                GameEvent::ServerRemoveBan(ServerRemoveBanEvent::read(stream, definition)?)
            }
            GameEventType::PlayerConnect => {
                GameEvent::PlayerConnect(PlayerConnectEvent::read(stream, definition)?)
            }
            GameEventType::PlayerConnectClient => {
                GameEvent::PlayerConnectClient(PlayerConnectClientEvent::read(stream, definition)?)
            }
            GameEventType::PlayerInfo => {
                GameEvent::PlayerInfo(PlayerInfoEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDisconnect => {
                GameEvent::PlayerDisconnect(PlayerDisconnectEvent::read(stream, definition)?)
            }
            GameEventType::PlayerActivate => {
                GameEvent::PlayerActivate(PlayerActivateEvent::read(stream, definition)?)
            }
            GameEventType::PlayerSay => {
                GameEvent::PlayerSay(PlayerSayEvent::read(stream, definition)?)
            }
            GameEventType::ClientDisconnect => {
                GameEvent::ClientDisconnect(ClientDisconnectEvent::read(stream, definition)?)
            }
            GameEventType::ClientBeginConnect => {
                GameEvent::ClientBeginConnect(ClientBeginConnectEvent::read(stream, definition)?)
            }
            GameEventType::ClientConnected => {
                GameEvent::ClientConnected(ClientConnectedEvent::read(stream, definition)?)
            }
            GameEventType::ClientFullConnect => {
                GameEvent::ClientFullConnect(ClientFullConnectEvent::read(stream, definition)?)
            }
            GameEventType::HostQuit => {
                GameEvent::HostQuit(HostQuitEvent::read(stream, definition)?)
            }
            GameEventType::TeamInfo => {
                GameEvent::TeamInfo(TeamInfoEvent::read(stream, definition)?)
            }
            GameEventType::TeamScore => {
                GameEvent::TeamScore(TeamScoreEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayBroadcastAudio => GameEvent::TeamPlayBroadcastAudio(
                TeamPlayBroadcastAudioEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerTeam => {
                GameEvent::PlayerTeam(PlayerTeamEvent::read(stream, definition)?)
            }
            GameEventType::PlayerClass => {
                GameEvent::PlayerClass(PlayerClassEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDeath => {
                GameEvent::PlayerDeath(Box::new(<PlayerDeathEvent>::read(stream, definition)?))
            }
            GameEventType::PlayerHurt => {
                GameEvent::PlayerHurt(PlayerHurtEvent::read(stream, definition)?)
            }
            GameEventType::PlayerChat => {
                GameEvent::PlayerChat(PlayerChatEvent::read(stream, definition)?)
            }
            GameEventType::PlayerScore => {
                GameEvent::PlayerScore(PlayerScoreEvent::read(stream, definition)?)
            }
            GameEventType::PlayerSpawn => {
                GameEvent::PlayerSpawn(PlayerSpawnEvent::read(stream, definition)?)
            }
            GameEventType::PlayerShoot => {
                GameEvent::PlayerShoot(PlayerShootEvent::read(stream, definition)?)
            }
            GameEventType::PlayerUse => {
                GameEvent::PlayerUse(PlayerUseEvent::read(stream, definition)?)
            }
            GameEventType::PlayerChangeName => {
                GameEvent::PlayerChangeName(PlayerChangeNameEvent::read(stream, definition)?)
            }
            GameEventType::PlayerHintMessage => {
                GameEvent::PlayerHintMessage(PlayerHintMessageEvent::read(stream, definition)?)
            }
            GameEventType::BasePlayerTeleported => GameEvent::BasePlayerTeleported(
                BasePlayerTeleportedEvent::read(stream, definition)?,
            ),
            GameEventType::GameInit => {
                GameEvent::GameInit(GameInitEvent::read(stream, definition)?)
            }
            GameEventType::GameNewMap => {
                GameEvent::GameNewMap(GameNewMapEvent::read(stream, definition)?)
            }
            GameEventType::GameStart => {
                GameEvent::GameStart(GameStartEvent::read(stream, definition)?)
            }
            GameEventType::GameEnd => GameEvent::GameEnd(GameEndEvent::read(stream, definition)?),
            GameEventType::RoundStart => {
                GameEvent::RoundStart(RoundStartEvent::read(stream, definition)?)
            }
            GameEventType::RoundEnd => {
                GameEvent::RoundEnd(RoundEndEvent::read(stream, definition)?)
            }
            GameEventType::GameMessage => {
                GameEvent::GameMessage(GameMessageEvent::read(stream, definition)?)
            }
            GameEventType::BreakBreakable => {
                GameEvent::BreakBreakable(BreakBreakableEvent::read(stream, definition)?)
            }
            GameEventType::BreakProp => {
                GameEvent::BreakProp(BreakPropEvent::read(stream, definition)?)
            }
            GameEventType::EntityKilled => {
                GameEvent::EntityKilled(EntityKilledEvent::read(stream, definition)?)
            }
            GameEventType::BonusUpdated => {
                GameEvent::BonusUpdated(BonusUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::AchievementEvent => {
                GameEvent::AchievementEvent(AchievementEventEvent::read(stream, definition)?)
            }
            GameEventType::AchievementIncrement => GameEvent::AchievementIncrement(
                AchievementIncrementEvent::read(stream, definition)?,
            ),
            GameEventType::PhysgunPickup => {
                GameEvent::PhysgunPickup(PhysgunPickupEvent::read(stream, definition)?)
            }
            GameEventType::FlareIgniteNpc => {
                GameEvent::FlareIgniteNpc(FlareIgniteNpcEvent::read(stream, definition)?)
            }
            GameEventType::HelicopterGrenadePuntMiss => GameEvent::HelicopterGrenadePuntMiss(
                HelicopterGrenadePuntMissEvent::read(stream, definition)?,
            ),
            GameEventType::UserDataDownloaded => {
                GameEvent::UserDataDownloaded(UserDataDownloadedEvent::read(stream, definition)?)
            }
            GameEventType::RagdollDissolved => {
                GameEvent::RagdollDissolved(RagdollDissolvedEvent::read(stream, definition)?)
            }
            GameEventType::HLTVChangedMode => {
                GameEvent::HLTVChangedMode(HLTVChangedModeEvent::read(stream, definition)?)
            }
            GameEventType::HLTVChangedTarget => {
                GameEvent::HLTVChangedTarget(HLTVChangedTargetEvent::read(stream, definition)?)
            }
            GameEventType::VoteEnded => {
                GameEvent::VoteEnded(VoteEndedEvent::read(stream, definition)?)
            }
            GameEventType::VoteStarted => {
                GameEvent::VoteStarted(VoteStartedEvent::read(stream, definition)?)
            }
            GameEventType::VoteChanged => {
                GameEvent::VoteChanged(VoteChangedEvent::read(stream, definition)?)
            }
            GameEventType::VotePassed => {
                GameEvent::VotePassed(VotePassedEvent::read(stream, definition)?)
            }
            GameEventType::VoteFailed => {
                GameEvent::VoteFailed(VoteFailedEvent::read(stream, definition)?)
            }
            GameEventType::VoteCast => {
                GameEvent::VoteCast(VoteCastEvent::read(stream, definition)?)
            }
            GameEventType::VoteOptions => {
                GameEvent::VoteOptions(Box::new(<VoteOptionsEvent>::read(stream, definition)?))
            }
            GameEventType::ReplaySaved => {
                GameEvent::ReplaySaved(ReplaySavedEvent::read(stream, definition)?)
            }
            GameEventType::EnteredPerformanceMode => GameEvent::EnteredPerformanceMode(
                EnteredPerformanceModeEvent::read(stream, definition)?,
            ),
            GameEventType::BrowseReplays => {
                GameEvent::BrowseReplays(BrowseReplaysEvent::read(stream, definition)?)
            }
            GameEventType::ReplayYoutubeStats => {
                GameEvent::ReplayYoutubeStats(ReplayYoutubeStatsEvent::read(stream, definition)?)
            }
            GameEventType::InventoryUpdated => {
                GameEvent::InventoryUpdated(InventoryUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::CartUpdated => {
                GameEvent::CartUpdated(CartUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::StorePriceSheetUpdated => GameEvent::StorePriceSheetUpdated(
                StorePriceSheetUpdatedEvent::read(stream, definition)?,
            ),
            GameEventType::EconInventoryConnected => GameEvent::EconInventoryConnected(
                EconInventoryConnectedEvent::read(stream, definition)?,
            ),
            GameEventType::ItemSchemaInitialized => GameEvent::ItemSchemaInitialized(
                ItemSchemaInitializedEvent::read(stream, definition)?,
            ),
            GameEventType::GcNewSession => {
                GameEvent::GcNewSession(GcNewSessionEvent::read(stream, definition)?)
            }
            GameEventType::GcLostSession => {
                GameEvent::GcLostSession(GcLostSessionEvent::read(stream, definition)?)
            }
            GameEventType::IntroFinish => {
                GameEvent::IntroFinish(IntroFinishEvent::read(stream, definition)?)
            }
            GameEventType::IntroNextCamera => {
                GameEvent::IntroNextCamera(IntroNextCameraEvent::read(stream, definition)?)
            }
            GameEventType::PlayerChangeClass => {
                GameEvent::PlayerChangeClass(PlayerChangeClassEvent::read(stream, definition)?)
            }
            GameEventType::TfMapTimeRemaining => {
                GameEvent::TfMapTimeRemaining(TfMapTimeRemainingEvent::read(stream, definition)?)
            }
            GameEventType::TfGameOver => {
                GameEvent::TfGameOver(TfGameOverEvent::read(stream, definition)?)
            }
            GameEventType::CtfFlagCaptured => {
                GameEvent::CtfFlagCaptured(CtfFlagCapturedEvent::read(stream, definition)?)
            }
            GameEventType::ControlPointInitialized => GameEvent::ControlPointInitialized(
                ControlPointInitializedEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointUpdateImages => GameEvent::ControlPointUpdateImages(
                ControlPointUpdateImagesEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointUpdateLayout => GameEvent::ControlPointUpdateLayout(
                ControlPointUpdateLayoutEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointUpdateCapping => GameEvent::ControlPointUpdateCapping(
                ControlPointUpdateCappingEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointUpdateOwner => GameEvent::ControlPointUpdateOwner(
                ControlPointUpdateOwnerEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointStartTouch => GameEvent::ControlPointStartTouch(
                ControlPointStartTouchEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointEndTouch => GameEvent::ControlPointEndTouch(
                ControlPointEndTouchEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointPulseElement => GameEvent::ControlPointPulseElement(
                ControlPointPulseElementEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointFakeCapture => GameEvent::ControlPointFakeCapture(
                ControlPointFakeCaptureEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointFakeCaptureMultiplier => {
                GameEvent::ControlPointFakeCaptureMultiplier(
                    ControlPointFakeCaptureMultiplierEvent::read(stream, definition)?,
                )
            }
            GameEventType::TeamPlayRoundSelected => GameEvent::TeamPlayRoundSelected(
                TeamPlayRoundSelectedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayRoundStart => {
                GameEvent::TeamPlayRoundStart(TeamPlayRoundStartEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayRoundActive => {
                GameEvent::TeamPlayRoundActive(TeamPlayRoundActiveEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayWaitingBegins => GameEvent::TeamPlayWaitingBegins(
                TeamPlayWaitingBeginsEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayWaitingEnds => {
                GameEvent::TeamPlayWaitingEnds(TeamPlayWaitingEndsEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayWaitingAboutToEnd => GameEvent::TeamPlayWaitingAboutToEnd(
                TeamPlayWaitingAboutToEndEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayRestartRound => GameEvent::TeamPlayRestartRound(
                TeamPlayRestartRoundEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayReadyRestart => GameEvent::TeamPlayReadyRestart(
                TeamPlayReadyRestartEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayRoundRestartSeconds => GameEvent::TeamPlayRoundRestartSeconds(
                TeamPlayRoundRestartSecondsEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayTeamReady => {
                GameEvent::TeamPlayTeamReady(TeamPlayTeamReadyEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayRoundWin => {
                GameEvent::TeamPlayRoundWin(TeamPlayRoundWinEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayUpdateTimer => {
                GameEvent::TeamPlayUpdateTimer(TeamPlayUpdateTimerEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayRoundStalemate => GameEvent::TeamPlayRoundStalemate(
                TeamPlayRoundStalemateEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayOvertimeBegin => GameEvent::TeamPlayOvertimeBegin(
                TeamPlayOvertimeBeginEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayOvertimeEnd => {
                GameEvent::TeamPlayOvertimeEnd(TeamPlayOvertimeEndEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlaySuddenDeathBegin => GameEvent::TeamPlaySuddenDeathBegin(
                TeamPlaySuddenDeathBeginEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlaySuddenDeathEnd => GameEvent::TeamPlaySuddenDeathEnd(
                TeamPlaySuddenDeathEndEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayGameOver => {
                GameEvent::TeamPlayGameOver(TeamPlayGameOverEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayMapTimeRemaining => GameEvent::TeamPlayMapTimeRemaining(
                TeamPlayMapTimeRemainingEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayTimerFlash => {
                GameEvent::TeamPlayTimerFlash(TeamPlayTimerFlashEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayTimerTimeAdded => GameEvent::TeamPlayTimerTimeAdded(
                TeamPlayTimerTimeAddedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayPointStartCapture => GameEvent::TeamPlayPointStartCapture(
                TeamPlayPointStartCaptureEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayPointCaptured => GameEvent::TeamPlayPointCaptured(
                TeamPlayPointCapturedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayPointLocked => {
                GameEvent::TeamPlayPointLocked(TeamPlayPointLockedEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayPointUnlocked => GameEvent::TeamPlayPointUnlocked(
                TeamPlayPointUnlockedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayCaptureBroken => GameEvent::TeamPlayCaptureBroken(
                TeamPlayCaptureBrokenEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayCaptureBlocked => GameEvent::TeamPlayCaptureBlocked(
                TeamPlayCaptureBlockedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayFlagEvent => {
                GameEvent::TeamPlayFlagEvent(TeamPlayFlagEventEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayWinPanel => {
                GameEvent::TeamPlayWinPanel(TeamPlayWinPanelEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayTeamBalancedPlayer => GameEvent::TeamPlayTeamBalancedPlayer(
                TeamPlayTeamBalancedPlayerEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlaySetupFinished => GameEvent::TeamPlaySetupFinished(
                TeamPlaySetupFinishedEvent::read(stream, definition)?,
            ),
            GameEventType::TeamPlayAlert => {
                GameEvent::TeamPlayAlert(TeamPlayAlertEvent::read(stream, definition)?)
            }
            GameEventType::TrainingComplete => {
                GameEvent::TrainingComplete(TrainingCompleteEvent::read(stream, definition)?)
            }
            GameEventType::ShowFreezePanel => {
                GameEvent::ShowFreezePanel(ShowFreezePanelEvent::read(stream, definition)?)
            }
            GameEventType::HideFreezePanel => {
                GameEvent::HideFreezePanel(HideFreezePanelEvent::read(stream, definition)?)
            }
            GameEventType::FreezeCamStarted => {
                GameEvent::FreezeCamStarted(FreezeCamStartedEvent::read(stream, definition)?)
            }
            GameEventType::LocalPlayerChangeTeam => GameEvent::LocalPlayerChangeTeam(
                LocalPlayerChangeTeamEvent::read(stream, definition)?,
            ),
            GameEventType::LocalPlayerScoreChanged => GameEvent::LocalPlayerScoreChanged(
                LocalPlayerScoreChangedEvent::read(stream, definition)?,
            ),
            GameEventType::LocalPlayerChangeClass => GameEvent::LocalPlayerChangeClass(
                LocalPlayerChangeClassEvent::read(stream, definition)?,
            ),
            GameEventType::LocalPlayerRespawn => {
                GameEvent::LocalPlayerRespawn(LocalPlayerRespawnEvent::read(stream, definition)?)
            }
            GameEventType::BuildingInfoChanged => {
                GameEvent::BuildingInfoChanged(BuildingInfoChangedEvent::read(stream, definition)?)
            }
            GameEventType::LocalPlayerChangeDisguise => GameEvent::LocalPlayerChangeDisguise(
                LocalPlayerChangeDisguiseEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerAccountChanged => GameEvent::PlayerAccountChanged(
                PlayerAccountChangedEvent::read(stream, definition)?,
            ),
            GameEventType::SpyPdaReset => {
                GameEvent::SpyPdaReset(SpyPdaResetEvent::read(stream, definition)?)
            }
            GameEventType::FlagStatusUpdate => {
                GameEvent::FlagStatusUpdate(FlagStatusUpdateEvent::read(stream, definition)?)
            }
            GameEventType::PlayerStatsUpdated => {
                GameEvent::PlayerStatsUpdated(PlayerStatsUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::PlayingCommentary => {
                GameEvent::PlayingCommentary(PlayingCommentaryEvent::read(stream, definition)?)
            }
            GameEventType::PlayerChargeDeployed => GameEvent::PlayerChargeDeployed(
                PlayerChargeDeployedEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerBuiltObject => {
                GameEvent::PlayerBuiltObject(PlayerBuiltObjectEvent::read(stream, definition)?)
            }
            GameEventType::PlayerUpgradedObject => GameEvent::PlayerUpgradedObject(
                PlayerUpgradedObjectEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerCarryObject => {
                GameEvent::PlayerCarryObject(PlayerCarryObjectEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDropObject => {
                GameEvent::PlayerDropObject(PlayerDropObjectEvent::read(stream, definition)?)
            }
            GameEventType::ObjectRemoved => {
                GameEvent::ObjectRemoved(ObjectRemovedEvent::read(stream, definition)?)
            }
            GameEventType::ObjectDestroyed => {
                GameEvent::ObjectDestroyed(ObjectDestroyedEvent::read(stream, definition)?)
            }
            GameEventType::ObjectDetonated => {
                GameEvent::ObjectDetonated(ObjectDetonatedEvent::read(stream, definition)?)
            }
            GameEventType::AchievementEarned => {
                GameEvent::AchievementEarned(AchievementEarnedEvent::read(stream, definition)?)
            }
            GameEventType::SpecTargetUpdated => {
                GameEvent::SpecTargetUpdated(SpecTargetUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::TournamentStateUpdate => GameEvent::TournamentStateUpdate(
                TournamentStateUpdateEvent::read(stream, definition)?,
            ),
            GameEventType::TournamentEnableCountdown => GameEvent::TournamentEnableCountdown(
                TournamentEnableCountdownEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerCalledForMedic => GameEvent::PlayerCalledForMedic(
                PlayerCalledForMedicEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerAskedForBall => {
                GameEvent::PlayerAskedForBall(PlayerAskedForBallEvent::read(stream, definition)?)
            }
            GameEventType::LocalPlayerBecameObserver => GameEvent::LocalPlayerBecameObserver(
                LocalPlayerBecameObserverEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerIgnitedInv => {
                GameEvent::PlayerIgnitedInv(PlayerIgnitedInvEvent::read(stream, definition)?)
            }
            GameEventType::PlayerIgnited => {
                GameEvent::PlayerIgnited(PlayerIgnitedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerExtinguished => {
                GameEvent::PlayerExtinguished(PlayerExtinguishedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerTeleported => {
                GameEvent::PlayerTeleported(PlayerTeleportedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerHealedMedicCall => GameEvent::PlayerHealedMedicCall(
                PlayerHealedMedicCallEvent::read(stream, definition)?,
            ),
            GameEventType::LocalPlayerChargeReady => GameEvent::LocalPlayerChargeReady(
                LocalPlayerChargeReadyEvent::read(stream, definition)?,
            ),
            GameEventType::LocalPlayerWindDown => {
                GameEvent::LocalPlayerWindDown(LocalPlayerWindDownEvent::read(stream, definition)?)
            }
            GameEventType::PlayerInvulned => {
                GameEvent::PlayerInvulned(PlayerInvulnedEvent::read(stream, definition)?)
            }
            GameEventType::EscortSpeed => {
                GameEvent::EscortSpeed(EscortSpeedEvent::read(stream, definition)?)
            }
            GameEventType::EscortProgress => {
                GameEvent::EscortProgress(EscortProgressEvent::read(stream, definition)?)
            }
            GameEventType::EscortRecede => {
                GameEvent::EscortRecede(EscortRecedeEvent::read(stream, definition)?)
            }
            GameEventType::GameUIActivated => {
                GameEvent::GameUIActivated(GameUIActivatedEvent::read(stream, definition)?)
            }
            GameEventType::GameUIHidden => {
                GameEvent::GameUIHidden(GameUIHiddenEvent::read(stream, definition)?)
            }
            GameEventType::PlayerEscortScore => {
                GameEvent::PlayerEscortScore(PlayerEscortScoreEvent::read(stream, definition)?)
            }
            GameEventType::PlayerHealOnHit => {
                GameEvent::PlayerHealOnHit(PlayerHealOnHitEvent::read(stream, definition)?)
            }
            GameEventType::PlayerStealSandvich => {
                GameEvent::PlayerStealSandvich(PlayerStealSandvichEvent::read(stream, definition)?)
            }
            GameEventType::ShowClassLayout => {
                GameEvent::ShowClassLayout(ShowClassLayoutEvent::read(stream, definition)?)
            }
            GameEventType::ShowVsPanel => {
                GameEvent::ShowVsPanel(ShowVsPanelEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDamaged => {
                GameEvent::PlayerDamaged(PlayerDamagedEvent::read(stream, definition)?)
            }
            GameEventType::ArenaPlayerNotification => GameEvent::ArenaPlayerNotification(
                ArenaPlayerNotificationEvent::read(stream, definition)?,
            ),
            GameEventType::ArenaMatchMaxStreak => {
                GameEvent::ArenaMatchMaxStreak(ArenaMatchMaxStreakEvent::read(stream, definition)?)
            }
            GameEventType::ArenaRoundStart => {
                GameEvent::ArenaRoundStart(ArenaRoundStartEvent::read(stream, definition)?)
            }
            GameEventType::ArenaWinPanel => {
                GameEvent::ArenaWinPanel(ArenaWinPanelEvent::read(stream, definition)?)
            }
            GameEventType::PveWinPanel => {
                GameEvent::PveWinPanel(PveWinPanelEvent::read(stream, definition)?)
            }
            GameEventType::AirDash => GameEvent::AirDash(AirDashEvent::read(stream, definition)?),
            GameEventType::Landed => GameEvent::Landed(LandedEvent::read(stream, definition)?),
            GameEventType::PlayerDamageDodged => {
                GameEvent::PlayerDamageDodged(PlayerDamageDodgedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerStunned => {
                GameEvent::PlayerStunned(PlayerStunnedEvent::read(stream, definition)?)
            }
            GameEventType::ScoutGrandSlam => {
                GameEvent::ScoutGrandSlam(ScoutGrandSlamEvent::read(stream, definition)?)
            }
            GameEventType::ScoutSlamdollLanded => {
                GameEvent::ScoutSlamdollLanded(ScoutSlamdollLandedEvent::read(stream, definition)?)
            }
            GameEventType::ArrowImpact => {
                GameEvent::ArrowImpact(ArrowImpactEvent::read(stream, definition)?)
            }
            GameEventType::PlayerJarated => {
                GameEvent::PlayerJarated(PlayerJaratedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerJaratedFade => {
                GameEvent::PlayerJaratedFade(PlayerJaratedFadeEvent::read(stream, definition)?)
            }
            GameEventType::PlayerShieldBlocked => {
                GameEvent::PlayerShieldBlocked(PlayerShieldBlockedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerPinned => {
                GameEvent::PlayerPinned(PlayerPinnedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerHealedByMedic => {
                GameEvent::PlayerHealedByMedic(PlayerHealedByMedicEvent::read(stream, definition)?)
            }
            GameEventType::PlayerSappedObject => {
                GameEvent::PlayerSappedObject(PlayerSappedObjectEvent::read(stream, definition)?)
            }
            GameEventType::ItemFound => {
                GameEvent::ItemFound(ItemFoundEvent::read(stream, definition)?)
            }
            GameEventType::ShowAnnotation => {
                GameEvent::ShowAnnotation(ShowAnnotationEvent::read(stream, definition)?)
            }
            GameEventType::HideAnnotation => {
                GameEvent::HideAnnotation(HideAnnotationEvent::read(stream, definition)?)
            }
            GameEventType::PostInventoryApplication => GameEvent::PostInventoryApplication(
                PostInventoryApplicationEvent::read(stream, definition)?,
            ),
            GameEventType::ControlPointUnlockUpdated => GameEvent::ControlPointUnlockUpdated(
                ControlPointUnlockUpdatedEvent::read(stream, definition)?,
            ),
            GameEventType::DeployBuffBanner => {
                GameEvent::DeployBuffBanner(DeployBuffBannerEvent::read(stream, definition)?)
            }
            GameEventType::PlayerBuff => {
                GameEvent::PlayerBuff(PlayerBuffEvent::read(stream, definition)?)
            }
            GameEventType::MedicDeath => {
                GameEvent::MedicDeath(MedicDeathEvent::read(stream, definition)?)
            }
            GameEventType::OvertimeNag => {
                GameEvent::OvertimeNag(OvertimeNagEvent::read(stream, definition)?)
            }
            GameEventType::TeamsChanged => {
                GameEvent::TeamsChanged(TeamsChangedEvent::read(stream, definition)?)
            }
            GameEventType::HalloweenPumpkinGrab => GameEvent::HalloweenPumpkinGrab(
                HalloweenPumpkinGrabEvent::read(stream, definition)?,
            ),
            GameEventType::RocketJump => {
                GameEvent::RocketJump(RocketJumpEvent::read(stream, definition)?)
            }
            GameEventType::RocketJumpLanded => {
                GameEvent::RocketJumpLanded(RocketJumpLandedEvent::read(stream, definition)?)
            }
            GameEventType::StickyJump => {
                GameEvent::StickyJump(StickyJumpEvent::read(stream, definition)?)
            }
            GameEventType::StickyJumpLanded => {
                GameEvent::StickyJumpLanded(StickyJumpLandedEvent::read(stream, definition)?)
            }
            GameEventType::RocketPackLaunch => {
                GameEvent::RocketPackLaunch(RocketPackLaunchEvent::read(stream, definition)?)
            }
            GameEventType::RocketPackLanded => {
                GameEvent::RocketPackLanded(RocketPackLandedEvent::read(stream, definition)?)
            }
            GameEventType::MedicDefended => {
                GameEvent::MedicDefended(MedicDefendedEvent::read(stream, definition)?)
            }
            GameEventType::LocalPlayerHealed => {
                GameEvent::LocalPlayerHealed(LocalPlayerHealedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDestroyedPipeBomb => GameEvent::PlayerDestroyedPipeBomb(
                PlayerDestroyedPipeBombEvent::read(stream, definition)?,
            ),
            GameEventType::ObjectDeflected => {
                GameEvent::ObjectDeflected(ObjectDeflectedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerMvp => {
                GameEvent::PlayerMvp(PlayerMvpEvent::read(stream, definition)?)
            }
            GameEventType::RaidSpawnMob => {
                GameEvent::RaidSpawnMob(RaidSpawnMobEvent::read(stream, definition)?)
            }
            GameEventType::RaidSpawnSquad => {
                GameEvent::RaidSpawnSquad(RaidSpawnSquadEvent::read(stream, definition)?)
            }
            GameEventType::NavBlocked => {
                GameEvent::NavBlocked(NavBlockedEvent::read(stream, definition)?)
            }
            GameEventType::PathTrackPassed => {
                GameEvent::PathTrackPassed(PathTrackPassedEvent::read(stream, definition)?)
            }
            GameEventType::NumCappersChanged => {
                GameEvent::NumCappersChanged(NumCappersChangedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerRegenerate => {
                GameEvent::PlayerRegenerate(PlayerRegenerateEvent::read(stream, definition)?)
            }
            GameEventType::UpdateStatusItem => {
                GameEvent::UpdateStatusItem(UpdateStatusItemEvent::read(stream, definition)?)
            }
            GameEventType::StatsResetRound => {
                GameEvent::StatsResetRound(StatsResetRoundEvent::read(stream, definition)?)
            }
            GameEventType::ScoreStatsAccumulatedUpdate => GameEvent::ScoreStatsAccumulatedUpdate(
                ScoreStatsAccumulatedUpdateEvent::read(stream, definition)?,
            ),
            GameEventType::ScoreStatsAccumulatedReset => GameEvent::ScoreStatsAccumulatedReset(
                ScoreStatsAccumulatedResetEvent::read(stream, definition)?,
            ),
            GameEventType::AchievementEarnedLocal => GameEvent::AchievementEarnedLocal(
                AchievementEarnedLocalEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerHealed => {
                GameEvent::PlayerHealed(PlayerHealedEvent::read(stream, definition)?)
            }
            GameEventType::BuildingHealed => {
                GameEvent::BuildingHealed(BuildingHealedEvent::read(stream, definition)?)
            }
            GameEventType::ItemPickup => {
                GameEvent::ItemPickup(ItemPickupEvent::read(stream, definition)?)
            }
            GameEventType::DuelStatus => {
                GameEvent::DuelStatus(DuelStatusEvent::read(stream, definition)?)
            }
            GameEventType::FishNotice => {
                GameEvent::FishNotice(Box::new(<FishNoticeEvent>::read(stream, definition)?))
            }
            GameEventType::FishNoticeArm => {
                GameEvent::FishNoticeArm(Box::new(<FishNoticeArmEvent>::read(stream, definition)?))
            }
            GameEventType::SlapNotice => {
                GameEvent::SlapNotice(Box::new(<SlapNoticeEvent>::read(stream, definition)?))
            }
            GameEventType::ThrowableHit => {
                GameEvent::ThrowableHit(Box::new(<ThrowableHitEvent>::read(stream, definition)?))
            }
            GameEventType::PumpkinLordSummoned => {
                GameEvent::PumpkinLordSummoned(PumpkinLordSummonedEvent::read(stream, definition)?)
            }
            GameEventType::PumpkinLordKilled => {
                GameEvent::PumpkinLordKilled(PumpkinLordKilledEvent::read(stream, definition)?)
            }
            GameEventType::MerasmusSummoned => {
                GameEvent::MerasmusSummoned(MerasmusSummonedEvent::read(stream, definition)?)
            }
            GameEventType::MerasmusKilled => {
                GameEvent::MerasmusKilled(MerasmusKilledEvent::read(stream, definition)?)
            }
            GameEventType::MerasmusEscapeWarning => GameEvent::MerasmusEscapeWarning(
                MerasmusEscapeWarningEvent::read(stream, definition)?,
            ),
            GameEventType::MerasmusEscaped => {
                GameEvent::MerasmusEscaped(MerasmusEscapedEvent::read(stream, definition)?)
            }
            GameEventType::EyeballBossSummoned => {
                GameEvent::EyeballBossSummoned(EyeballBossSummonedEvent::read(stream, definition)?)
            }
            GameEventType::EyeballBossStunned => {
                GameEvent::EyeballBossStunned(EyeballBossStunnedEvent::read(stream, definition)?)
            }
            GameEventType::EyeballBossKilled => {
                GameEvent::EyeballBossKilled(EyeballBossKilledEvent::read(stream, definition)?)
            }
            GameEventType::EyeballBossKiller => {
                GameEvent::EyeballBossKiller(EyeballBossKillerEvent::read(stream, definition)?)
            }
            GameEventType::EyeballBossEscapeImminent => GameEvent::EyeballBossEscapeImminent(
                EyeballBossEscapeImminentEvent::read(stream, definition)?,
            ),
            GameEventType::EyeballBossEscaped => {
                GameEvent::EyeballBossEscaped(EyeballBossEscapedEvent::read(stream, definition)?)
            }
            GameEventType::NpcHurt => GameEvent::NpcHurt(NpcHurtEvent::read(stream, definition)?),
            GameEventType::ControlPointTimerUpdated => GameEvent::ControlPointTimerUpdated(
                ControlPointTimerUpdatedEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerHighFiveStart => {
                GameEvent::PlayerHighFiveStart(PlayerHighFiveStartEvent::read(stream, definition)?)
            }
            GameEventType::PlayerHighFiveCancel => GameEvent::PlayerHighFiveCancel(
                PlayerHighFiveCancelEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerHighFiveSuccess => GameEvent::PlayerHighFiveSuccess(
                PlayerHighFiveSuccessEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerBonusPoints => {
                GameEvent::PlayerBonusPoints(PlayerBonusPointsEvent::read(stream, definition)?)
            }
            GameEventType::PlayerUpgraded => {
                GameEvent::PlayerUpgraded(PlayerUpgradedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerBuyback => {
                GameEvent::PlayerBuyback(PlayerBuybackEvent::read(stream, definition)?)
            }
            GameEventType::PlayerUsedPowerUpBottle => GameEvent::PlayerUsedPowerUpBottle(
                PlayerUsedPowerUpBottleEvent::read(stream, definition)?,
            ),
            GameEventType::ChristmasGiftGrab => {
                GameEvent::ChristmasGiftGrab(ChristmasGiftGrabEvent::read(stream, definition)?)
            }
            GameEventType::PlayerKilledAchievementZone => GameEvent::PlayerKilledAchievementZone(
                PlayerKilledAchievementZoneEvent::read(stream, definition)?,
            ),
            GameEventType::PartyUpdated => {
                GameEvent::PartyUpdated(PartyUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::PartyPrefChanged => {
                GameEvent::PartyPrefChanged(PartyPrefChangedEvent::read(stream, definition)?)
            }
            GameEventType::PartyCriteriaChanged => GameEvent::PartyCriteriaChanged(
                PartyCriteriaChangedEvent::read(stream, definition)?,
            ),
            GameEventType::PartyInvitesChanged => {
                GameEvent::PartyInvitesChanged(PartyInvitesChangedEvent::read(stream, definition)?)
            }
            GameEventType::PartyQueueStateChanged => GameEvent::PartyQueueStateChanged(
                PartyQueueStateChangedEvent::read(stream, definition)?,
            ),
            GameEventType::PartyChat => {
                GameEvent::PartyChat(PartyChatEvent::read(stream, definition)?)
            }
            GameEventType::PartyMemberJoin => {
                GameEvent::PartyMemberJoin(PartyMemberJoinEvent::read(stream, definition)?)
            }
            GameEventType::PartyMemberLeave => {
                GameEvent::PartyMemberLeave(PartyMemberLeaveEvent::read(stream, definition)?)
            }
            GameEventType::MatchInvitesUpdated => {
                GameEvent::MatchInvitesUpdated(MatchInvitesUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::LobbyUpdated => {
                GameEvent::LobbyUpdated(LobbyUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::MvmMissionUpdate => {
                GameEvent::MvmMissionUpdate(MvmMissionUpdateEvent::read(stream, definition)?)
            }
            GameEventType::RecalculateHolidays => {
                GameEvent::RecalculateHolidays(RecalculateHolidaysEvent::read(stream, definition)?)
            }
            GameEventType::PlayerCurrencyChanged => GameEvent::PlayerCurrencyChanged(
                PlayerCurrencyChangedEvent::read(stream, definition)?,
            ),
            GameEventType::DoomsdayRocketOpen => {
                GameEvent::DoomsdayRocketOpen(DoomsdayRocketOpenEvent::read(stream, definition)?)
            }
            GameEventType::RemoveNemesisRelationships => GameEvent::RemoveNemesisRelationships(
                RemoveNemesisRelationshipsEvent::read(stream, definition)?,
            ),
            GameEventType::MvmCreditBonusWave => {
                GameEvent::MvmCreditBonusWave(MvmCreditBonusWaveEvent::read(stream, definition)?)
            }
            GameEventType::MvmCreditBonusAll => {
                GameEvent::MvmCreditBonusAll(MvmCreditBonusAllEvent::read(stream, definition)?)
            }
            GameEventType::MvmCreditBonusAllAdvanced => GameEvent::MvmCreditBonusAllAdvanced(
                MvmCreditBonusAllAdvancedEvent::read(stream, definition)?,
            ),
            GameEventType::MvmQuickSentryUpgrade => GameEvent::MvmQuickSentryUpgrade(
                MvmQuickSentryUpgradeEvent::read(stream, definition)?,
            ),
            GameEventType::MvmTankDestroyedByPlayers => GameEvent::MvmTankDestroyedByPlayers(
                MvmTankDestroyedByPlayersEvent::read(stream, definition)?,
            ),
            GameEventType::MvmKillRobotDeliveringBomb => GameEvent::MvmKillRobotDeliveringBomb(
                MvmKillRobotDeliveringBombEvent::read(stream, definition)?,
            ),
            GameEventType::MvmPickupCurrency => {
                GameEvent::MvmPickupCurrency(MvmPickupCurrencyEvent::read(stream, definition)?)
            }
            GameEventType::MvmBombCarrierKilled => GameEvent::MvmBombCarrierKilled(
                MvmBombCarrierKilledEvent::read(stream, definition)?,
            ),
            GameEventType::MvmSentryBusterDetonate => GameEvent::MvmSentryBusterDetonate(
                MvmSentryBusterDetonateEvent::read(stream, definition)?,
            ),
            GameEventType::MvmScoutMarkedForDeath => GameEvent::MvmScoutMarkedForDeath(
                MvmScoutMarkedForDeathEvent::read(stream, definition)?,
            ),
            GameEventType::MvmMedicPowerUpShared => GameEvent::MvmMedicPowerUpShared(
                MvmMedicPowerUpSharedEvent::read(stream, definition)?,
            ),
            GameEventType::MvmBeginWave => {
                GameEvent::MvmBeginWave(MvmBeginWaveEvent::read(stream, definition)?)
            }
            GameEventType::MvmWaveComplete => {
                GameEvent::MvmWaveComplete(MvmWaveCompleteEvent::read(stream, definition)?)
            }
            GameEventType::MvmMissionComplete => {
                GameEvent::MvmMissionComplete(MvmMissionCompleteEvent::read(stream, definition)?)
            }
            GameEventType::MvmBombResetByPlayer => GameEvent::MvmBombResetByPlayer(
                MvmBombResetByPlayerEvent::read(stream, definition)?,
            ),
            GameEventType::MvmBombAlarmTriggered => GameEvent::MvmBombAlarmTriggered(
                MvmBombAlarmTriggeredEvent::read(stream, definition)?,
            ),
            GameEventType::MvmBombDeployResetByPlayer => GameEvent::MvmBombDeployResetByPlayer(
                MvmBombDeployResetByPlayerEvent::read(stream, definition)?,
            ),
            GameEventType::MvmWaveFailed => {
                GameEvent::MvmWaveFailed(MvmWaveFailedEvent::read(stream, definition)?)
            }
            GameEventType::MvmResetStats => {
                GameEvent::MvmResetStats(MvmResetStatsEvent::read(stream, definition)?)
            }
            GameEventType::DamageResisted => {
                GameEvent::DamageResisted(DamageResistedEvent::read(stream, definition)?)
            }
            GameEventType::RevivePlayerNotify => {
                GameEvent::RevivePlayerNotify(RevivePlayerNotifyEvent::read(stream, definition)?)
            }
            GameEventType::RevivePlayerStopped => {
                GameEvent::RevivePlayerStopped(RevivePlayerStoppedEvent::read(stream, definition)?)
            }
            GameEventType::RevivePlayerComplete => GameEvent::RevivePlayerComplete(
                RevivePlayerCompleteEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerTurnedToGhost => {
                GameEvent::PlayerTurnedToGhost(PlayerTurnedToGhostEvent::read(stream, definition)?)
            }
            GameEventType::MedigunShieldBlockedDamage => GameEvent::MedigunShieldBlockedDamage(
                MedigunShieldBlockedDamageEvent::read(stream, definition)?,
            ),
            GameEventType::MvmAdvWaveCompleteNoGates => GameEvent::MvmAdvWaveCompleteNoGates(
                MvmAdvWaveCompleteNoGatesEvent::read(stream, definition)?,
            ),
            GameEventType::MvmSniperHeadshotCurrency => GameEvent::MvmSniperHeadshotCurrency(
                MvmSniperHeadshotCurrencyEvent::read(stream, definition)?,
            ),
            GameEventType::MvmMannhattanPit => {
                GameEvent::MvmMannhattanPit(MvmMannhattanPitEvent::read(stream, definition)?)
            }
            GameEventType::FlagCarriedInDetectionZone => GameEvent::FlagCarriedInDetectionZone(
                FlagCarriedInDetectionZoneEvent::read(stream, definition)?,
            ),
            GameEventType::MvmAdvWaveKilledStunRadio => GameEvent::MvmAdvWaveKilledStunRadio(
                MvmAdvWaveKilledStunRadioEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerDirectHitStun => {
                GameEvent::PlayerDirectHitStun(PlayerDirectHitStunEvent::read(stream, definition)?)
            }
            GameEventType::MvmSentryBusterKilled => GameEvent::MvmSentryBusterKilled(
                MvmSentryBusterKilledEvent::read(stream, definition)?,
            ),
            GameEventType::UpgradesFileChanged => {
                GameEvent::UpgradesFileChanged(UpgradesFileChangedEvent::read(stream, definition)?)
            }
            GameEventType::RdTeamPointsChanged => {
                GameEvent::RdTeamPointsChanged(RdTeamPointsChangedEvent::read(stream, definition)?)
            }
            GameEventType::RdRulesStateChanged => {
                GameEvent::RdRulesStateChanged(RdRulesStateChangedEvent::read(stream, definition)?)
            }
            GameEventType::RdRobotKilled => {
                GameEvent::RdRobotKilled(RdRobotKilledEvent::read(stream, definition)?)
            }
            GameEventType::RdRobotImpact => {
                GameEvent::RdRobotImpact(RdRobotImpactEvent::read(stream, definition)?)
            }
            GameEventType::TeamPlayPreRoundTimeLeft => GameEvent::TeamPlayPreRoundTimeLeft(
                TeamPlayPreRoundTimeLeftEvent::read(stream, definition)?,
            ),
            GameEventType::ParachuteDeploy => {
                GameEvent::ParachuteDeploy(ParachuteDeployEvent::read(stream, definition)?)
            }
            GameEventType::ParachuteHolster => {
                GameEvent::ParachuteHolster(ParachuteHolsterEvent::read(stream, definition)?)
            }
            GameEventType::KillRefillsMeter => {
                GameEvent::KillRefillsMeter(KillRefillsMeterEvent::read(stream, definition)?)
            }
            GameEventType::RpsTauntEvent => {
                GameEvent::RpsTauntEvent(RpsTauntEventEvent::read(stream, definition)?)
            }
            GameEventType::CongaKill => {
                GameEvent::CongaKill(CongaKillEvent::read(stream, definition)?)
            }
            GameEventType::PlayerInitialSpawn => {
                GameEvent::PlayerInitialSpawn(PlayerInitialSpawnEvent::read(stream, definition)?)
            }
            GameEventType::CompetitiveVictory => {
                GameEvent::CompetitiveVictory(CompetitiveVictoryEvent::read(stream, definition)?)
            }
            GameEventType::CompetitiveStatsUpdate => GameEvent::CompetitiveStatsUpdate(
                CompetitiveStatsUpdateEvent::read(stream, definition)?,
            ),
            GameEventType::MiniGameWin => {
                GameEvent::MiniGameWin(MiniGameWinEvent::read(stream, definition)?)
            }
            GameEventType::SentryOnGoActive => {
                GameEvent::SentryOnGoActive(SentryOnGoActiveEvent::read(stream, definition)?)
            }
            GameEventType::DuckXpLevelUp => {
                GameEvent::DuckXpLevelUp(DuckXpLevelUpEvent::read(stream, definition)?)
            }
            GameEventType::QuestLogOpened => {
                GameEvent::QuestLogOpened(QuestLogOpenedEvent::read(stream, definition)?)
            }
            GameEventType::SchemaUpdated => {
                GameEvent::SchemaUpdated(SchemaUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::LocalPlayerPickupWeapon => GameEvent::LocalPlayerPickupWeapon(
                LocalPlayerPickupWeaponEvent::read(stream, definition)?,
            ),
            GameEventType::RdPlayerScorePoints => {
                GameEvent::RdPlayerScorePoints(RdPlayerScorePointsEvent::read(stream, definition)?)
            }
            GameEventType::DemomanDetStickies => {
                GameEvent::DemomanDetStickies(DemomanDetStickiesEvent::read(stream, definition)?)
            }
            GameEventType::QuestObjectiveCompleted => GameEvent::QuestObjectiveCompleted(
                QuestObjectiveCompletedEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerScoreChanged => {
                GameEvent::PlayerScoreChanged(PlayerScoreChangedEvent::read(stream, definition)?)
            }
            GameEventType::KilledCappingPlayer => {
                GameEvent::KilledCappingPlayer(KilledCappingPlayerEvent::read(stream, definition)?)
            }
            GameEventType::EnvironmentalDeath => {
                GameEvent::EnvironmentalDeath(EnvironmentalDeathEvent::read(stream, definition)?)
            }
            GameEventType::ProjectileDirectHit => {
                GameEvent::ProjectileDirectHit(ProjectileDirectHitEvent::read(stream, definition)?)
            }
            GameEventType::PassGet => GameEvent::PassGet(PassGetEvent::read(stream, definition)?),
            GameEventType::PassScore => {
                GameEvent::PassScore(PassScoreEvent::read(stream, definition)?)
            }
            GameEventType::PassFree => {
                GameEvent::PassFree(PassFreeEvent::read(stream, definition)?)
            }
            GameEventType::PassPassCaught => {
                GameEvent::PassPassCaught(PassPassCaughtEvent::read(stream, definition)?)
            }
            GameEventType::PassBallStolen => {
                GameEvent::PassBallStolen(PassBallStolenEvent::read(stream, definition)?)
            }
            GameEventType::PassBallBlocked => {
                GameEvent::PassBallBlocked(PassBallBlockedEvent::read(stream, definition)?)
            }
            GameEventType::DamagePrevented => {
                GameEvent::DamagePrevented(DamagePreventedEvent::read(stream, definition)?)
            }
            GameEventType::HalloweenBossKilled => {
                GameEvent::HalloweenBossKilled(HalloweenBossKilledEvent::read(stream, definition)?)
            }
            GameEventType::EscapedLootIsland => {
                GameEvent::EscapedLootIsland(EscapedLootIslandEvent::read(stream, definition)?)
            }
            GameEventType::TaggedPlayerAsIt => {
                GameEvent::TaggedPlayerAsIt(TaggedPlayerAsItEvent::read(stream, definition)?)
            }
            GameEventType::MerasmusStunned => {
                GameEvent::MerasmusStunned(MerasmusStunnedEvent::read(stream, definition)?)
            }
            GameEventType::MerasmusPropFound => {
                GameEvent::MerasmusPropFound(MerasmusPropFoundEvent::read(stream, definition)?)
            }
            GameEventType::HalloweenSkeletonKilled => GameEvent::HalloweenSkeletonKilled(
                HalloweenSkeletonKilledEvent::read(stream, definition)?,
            ),
            GameEventType::SkeletonKilledQuest => {
                GameEvent::SkeletonKilledQuest(SkeletonKilledQuestEvent::read(stream, definition)?)
            }
            GameEventType::SkeletonKingKilledQuest => GameEvent::SkeletonKingKilledQuest(
                SkeletonKingKilledQuestEvent::read(stream, definition)?,
            ),
            GameEventType::EscapeHell => {
                GameEvent::EscapeHell(EscapeHellEvent::read(stream, definition)?)
            }
            GameEventType::CrossSpectralBridge => {
                GameEvent::CrossSpectralBridge(CrossSpectralBridgeEvent::read(stream, definition)?)
            }
            GameEventType::MiniGameWon => {
                GameEvent::MiniGameWon(MiniGameWonEvent::read(stream, definition)?)
            }
            GameEventType::RespawnGhost => {
                GameEvent::RespawnGhost(RespawnGhostEvent::read(stream, definition)?)
            }
            GameEventType::KillInHell => {
                GameEvent::KillInHell(KillInHellEvent::read(stream, definition)?)
            }
            GameEventType::HalloweenDuckCollected => GameEvent::HalloweenDuckCollected(
                HalloweenDuckCollectedEvent::read(stream, definition)?,
            ),
            GameEventType::SpecialScore => {
                GameEvent::SpecialScore(SpecialScoreEvent::read(stream, definition)?)
            }
            GameEventType::TeamLeaderKilled => {
                GameEvent::TeamLeaderKilled(TeamLeaderKilledEvent::read(stream, definition)?)
            }
            GameEventType::HalloweenSoulCollected => GameEvent::HalloweenSoulCollected(
                HalloweenSoulCollectedEvent::read(stream, definition)?,
            ),
            GameEventType::RecalculateTruce => {
                GameEvent::RecalculateTruce(RecalculateTruceEvent::read(stream, definition)?)
            }
            GameEventType::DeadRingerCheatDeath => GameEvent::DeadRingerCheatDeath(
                DeadRingerCheatDeathEvent::read(stream, definition)?,
            ),
            GameEventType::CrossbowHeal => {
                GameEvent::CrossbowHeal(CrossbowHealEvent::read(stream, definition)?)
            }
            GameEventType::DamageMitigated => {
                GameEvent::DamageMitigated(DamageMitigatedEvent::read(stream, definition)?)
            }
            GameEventType::PayloadPushed => {
                GameEvent::PayloadPushed(PayloadPushedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerAbandonedMatch => GameEvent::PlayerAbandonedMatch(
                PlayerAbandonedMatchEvent::read(stream, definition)?,
            ),
            GameEventType::ClDrawline => {
                GameEvent::ClDrawline(ClDrawlineEvent::read(stream, definition)?)
            }
            GameEventType::RestartTimerTime => {
                GameEvent::RestartTimerTime(RestartTimerTimeEvent::read(stream, definition)?)
            }
            GameEventType::WinLimitChanged => {
                GameEvent::WinLimitChanged(WinLimitChangedEvent::read(stream, definition)?)
            }
            GameEventType::WinPanelShowScores => {
                GameEvent::WinPanelShowScores(WinPanelShowScoresEvent::read(stream, definition)?)
            }
            GameEventType::TopStreamsRequestFinished => GameEvent::TopStreamsRequestFinished(
                TopStreamsRequestFinishedEvent::read(stream, definition)?,
            ),
            GameEventType::CompetitiveStateChanged => GameEvent::CompetitiveStateChanged(
                CompetitiveStateChangedEvent::read(stream, definition)?,
            ),
            GameEventType::GlobalWarDataUpdated => GameEvent::GlobalWarDataUpdated(
                GlobalWarDataUpdatedEvent::read(stream, definition)?,
            ),
            GameEventType::StopWatchChanged => {
                GameEvent::StopWatchChanged(StopWatchChangedEvent::read(stream, definition)?)
            }
            GameEventType::DsStop => GameEvent::DsStop(DsStopEvent::read(stream, definition)?),
            GameEventType::DsScreenshot => {
                GameEvent::DsScreenshot(DsScreenshotEvent::read(stream, definition)?)
            }
            GameEventType::ShowMatchSummary => {
                GameEvent::ShowMatchSummary(ShowMatchSummaryEvent::read(stream, definition)?)
            }
            GameEventType::ExperienceChanged => {
                GameEvent::ExperienceChanged(ExperienceChangedEvent::read(stream, definition)?)
            }
            GameEventType::BeginXpLerp => {
                GameEvent::BeginXpLerp(BeginXpLerpEvent::read(stream, definition)?)
            }
            GameEventType::MatchmakerStatsUpdated => GameEvent::MatchmakerStatsUpdated(
                MatchmakerStatsUpdatedEvent::read(stream, definition)?,
            ),
            GameEventType::RematchVotePeriodOver => GameEvent::RematchVotePeriodOver(
                RematchVotePeriodOverEvent::read(stream, definition)?,
            ),
            GameEventType::RematchFailedToCreate => GameEvent::RematchFailedToCreate(
                RematchFailedToCreateEvent::read(stream, definition)?,
            ),
            GameEventType::PlayerRematchChange => {
                GameEvent::PlayerRematchChange(PlayerRematchChangeEvent::read(stream, definition)?)
            }
            GameEventType::PingUpdated => {
                GameEvent::PingUpdated(PingUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::MMStatsUpdated => {
                GameEvent::MMStatsUpdated(MMStatsUpdatedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerNextMapVoteChange => GameEvent::PlayerNextMapVoteChange(
                PlayerNextMapVoteChangeEvent::read(stream, definition)?,
            ),
            GameEventType::VoteMapsChanged => {
                GameEvent::VoteMapsChanged(VoteMapsChangedEvent::read(stream, definition)?)
            }
            GameEventType::ProtoDefChanged => {
                GameEvent::ProtoDefChanged(ProtoDefChangedEvent::read(stream, definition)?)
            }
            GameEventType::PlayerDomination => {
                GameEvent::PlayerDomination(PlayerDominationEvent::read(stream, definition)?)
            }
            GameEventType::PlayerRocketPackPushed => GameEvent::PlayerRocketPackPushed(
                PlayerRocketPackPushedEvent::read(stream, definition)?,
            ),
            GameEventType::QuestRequest => {
                GameEvent::QuestRequest(QuestRequestEvent::read(stream, definition)?)
            }
            GameEventType::QuestResponse => {
                GameEvent::QuestResponse(QuestResponseEvent::read(stream, definition)?)
            }
            GameEventType::QuestProgress => {
                GameEvent::QuestProgress(QuestProgressEvent::read(stream, definition)?)
            }
            GameEventType::ProjectileRemoved => {
                GameEvent::ProjectileRemoved(ProjectileRemovedEvent::read(stream, definition)?)
            }
            GameEventType::QuestMapDataChanged => {
                GameEvent::QuestMapDataChanged(QuestMapDataChangedEvent::read(stream, definition)?)
            }
            GameEventType::GasDousedPlayerIgnited => GameEvent::GasDousedPlayerIgnited(
                GasDousedPlayerIgnitedEvent::read(stream, definition)?,
            ),
            GameEventType::QuestTurnInState => {
                GameEvent::QuestTurnInState(QuestTurnInStateEvent::read(stream, definition)?)
            }
            GameEventType::ItemsAcknowledged => {
                GameEvent::ItemsAcknowledged(ItemsAcknowledgedEvent::read(stream, definition)?)
            }
            GameEventType::CapperKilled => {
                GameEvent::CapperKilled(CapperKilledEvent::read(stream, definition)?)
            }
            GameEventType::MainMenuStabilized => {
                GameEvent::MainMenuStabilized(MainMenuStabilizedEvent::read(stream, definition)?)
            }
            GameEventType::WorldStatusChanged => {
                GameEvent::WorldStatusChanged(WorldStatusChangedEvent::read(stream, definition)?)
            }
            GameEventType::HLTVStatus => {
                GameEvent::HLTVStatus(HLTVStatusEvent::read(stream, definition)?)
            }
            GameEventType::HLTVCameraman => {
                GameEvent::HLTVCameraman(HLTVCameramanEvent::read(stream, definition)?)
            }
            GameEventType::HLTVRankCamera => {
                GameEvent::HLTVRankCamera(HLTVRankCameraEvent::read(stream, definition)?)
            }
            GameEventType::HLTVRankEntity => {
                GameEvent::HLTVRankEntity(HLTVRankEntityEvent::read(stream, definition)?)
            }
            GameEventType::HLTVFixed => {
                GameEvent::HLTVFixed(HLTVFixedEvent::read(stream, definition)?)
            }
            GameEventType::HLTVChase => {
                GameEvent::HLTVChase(HLTVChaseEvent::read(stream, definition)?)
            }
            GameEventType::HLTVMessage => {
                GameEvent::HLTVMessage(HLTVMessageEvent::read(stream, definition)?)
            }
            GameEventType::HLTVTitle => {
                GameEvent::HLTVTitle(HLTVTitleEvent::read(stream, definition)?)
            }
            GameEventType::HLTVChat => {
                GameEvent::HLTVChat(HLTVChatEvent::read(stream, definition)?)
            }
            GameEventType::ReplayStartRecord => {
                GameEvent::ReplayStartRecord(ReplayStartRecordEvent::read(stream, definition)?)
            }
            GameEventType::ReplaySessionInfo => {
                GameEvent::ReplaySessionInfo(ReplaySessionInfoEvent::read(stream, definition)?)
            }
            GameEventType::ReplayEndRecord => {
                GameEvent::ReplayEndRecord(ReplayEndRecordEvent::read(stream, definition)?)
            }
            GameEventType::ReplayReplaysAvailable => GameEvent::ReplayReplaysAvailable(
                ReplayReplaysAvailableEvent::read(stream, definition)?,
            ),
            GameEventType::ReplayServerError => {
                GameEvent::ReplayServerError(ReplayServerErrorEvent::read(stream, definition)?)
            }
            GameEventType::Unknown(_) => {
                GameEvent::Unknown(RawGameEvent::read(stream, definition)?)
            }
        })
    }
    pub fn write(
        &self,
        stream: &mut BitWriteStream<LittleEndian>,
        definition: &GameEventDefinition,
    ) -> Result<()> {
        match &self {
            GameEvent::ServerSpawn(event) => event.write(stream, definition),
            GameEvent::ServerChangeLevelFailed(event) => event.write(stream, definition),
            GameEvent::ServerShutdown(event) => event.write(stream, definition),
            GameEvent::ServerCvar(event) => event.write(stream, definition),
            GameEvent::ServerMessage(event) => event.write(stream, definition),
            GameEvent::ServerAddBan(event) => event.write(stream, definition),
            GameEvent::ServerRemoveBan(event) => event.write(stream, definition),
            GameEvent::PlayerConnect(event) => event.write(stream, definition),
            GameEvent::PlayerConnectClient(event) => event.write(stream, definition),
            GameEvent::PlayerInfo(event) => event.write(stream, definition),
            GameEvent::PlayerDisconnect(event) => event.write(stream, definition),
            GameEvent::PlayerActivate(event) => event.write(stream, definition),
            GameEvent::PlayerSay(event) => event.write(stream, definition),
            GameEvent::ClientDisconnect(event) => event.write(stream, definition),
            GameEvent::ClientBeginConnect(event) => event.write(stream, definition),
            GameEvent::ClientConnected(event) => event.write(stream, definition),
            GameEvent::ClientFullConnect(event) => event.write(stream, definition),
            GameEvent::HostQuit(event) => event.write(stream, definition),
            GameEvent::TeamInfo(event) => event.write(stream, definition),
            GameEvent::TeamScore(event) => event.write(stream, definition),
            GameEvent::TeamPlayBroadcastAudio(event) => event.write(stream, definition),
            GameEvent::PlayerTeam(event) => event.write(stream, definition),
            GameEvent::PlayerClass(event) => event.write(stream, definition),
            GameEvent::PlayerDeath(event) => event.write(stream, definition),
            GameEvent::PlayerHurt(event) => event.write(stream, definition),
            GameEvent::PlayerChat(event) => event.write(stream, definition),
            GameEvent::PlayerScore(event) => event.write(stream, definition),
            GameEvent::PlayerSpawn(event) => event.write(stream, definition),
            GameEvent::PlayerShoot(event) => event.write(stream, definition),
            GameEvent::PlayerUse(event) => event.write(stream, definition),
            GameEvent::PlayerChangeName(event) => event.write(stream, definition),
            GameEvent::PlayerHintMessage(event) => event.write(stream, definition),
            GameEvent::BasePlayerTeleported(event) => event.write(stream, definition),
            GameEvent::GameInit(event) => event.write(stream, definition),
            GameEvent::GameNewMap(event) => event.write(stream, definition),
            GameEvent::GameStart(event) => event.write(stream, definition),
            GameEvent::GameEnd(event) => event.write(stream, definition),
            GameEvent::RoundStart(event) => event.write(stream, definition),
            GameEvent::RoundEnd(event) => event.write(stream, definition),
            GameEvent::GameMessage(event) => event.write(stream, definition),
            GameEvent::BreakBreakable(event) => event.write(stream, definition),
            GameEvent::BreakProp(event) => event.write(stream, definition),
            GameEvent::EntityKilled(event) => event.write(stream, definition),
            GameEvent::BonusUpdated(event) => event.write(stream, definition),
            GameEvent::AchievementEvent(event) => event.write(stream, definition),
            GameEvent::AchievementIncrement(event) => event.write(stream, definition),
            GameEvent::PhysgunPickup(event) => event.write(stream, definition),
            GameEvent::FlareIgniteNpc(event) => event.write(stream, definition),
            GameEvent::HelicopterGrenadePuntMiss(event) => event.write(stream, definition),
            GameEvent::UserDataDownloaded(event) => event.write(stream, definition),
            GameEvent::RagdollDissolved(event) => event.write(stream, definition),
            GameEvent::HLTVChangedMode(event) => event.write(stream, definition),
            GameEvent::HLTVChangedTarget(event) => event.write(stream, definition),
            GameEvent::VoteEnded(event) => event.write(stream, definition),
            GameEvent::VoteStarted(event) => event.write(stream, definition),
            GameEvent::VoteChanged(event) => event.write(stream, definition),
            GameEvent::VotePassed(event) => event.write(stream, definition),
            GameEvent::VoteFailed(event) => event.write(stream, definition),
            GameEvent::VoteCast(event) => event.write(stream, definition),
            GameEvent::VoteOptions(event) => event.write(stream, definition),
            GameEvent::ReplaySaved(event) => event.write(stream, definition),
            GameEvent::EnteredPerformanceMode(event) => event.write(stream, definition),
            GameEvent::BrowseReplays(event) => event.write(stream, definition),
            GameEvent::ReplayYoutubeStats(event) => event.write(stream, definition),
            GameEvent::InventoryUpdated(event) => event.write(stream, definition),
            GameEvent::CartUpdated(event) => event.write(stream, definition),
            GameEvent::StorePriceSheetUpdated(event) => event.write(stream, definition),
            GameEvent::EconInventoryConnected(event) => event.write(stream, definition),
            GameEvent::ItemSchemaInitialized(event) => event.write(stream, definition),
            GameEvent::GcNewSession(event) => event.write(stream, definition),
            GameEvent::GcLostSession(event) => event.write(stream, definition),
            GameEvent::IntroFinish(event) => event.write(stream, definition),
            GameEvent::IntroNextCamera(event) => event.write(stream, definition),
            GameEvent::PlayerChangeClass(event) => event.write(stream, definition),
            GameEvent::TfMapTimeRemaining(event) => event.write(stream, definition),
            GameEvent::TfGameOver(event) => event.write(stream, definition),
            GameEvent::CtfFlagCaptured(event) => event.write(stream, definition),
            GameEvent::ControlPointInitialized(event) => event.write(stream, definition),
            GameEvent::ControlPointUpdateImages(event) => event.write(stream, definition),
            GameEvent::ControlPointUpdateLayout(event) => event.write(stream, definition),
            GameEvent::ControlPointUpdateCapping(event) => event.write(stream, definition),
            GameEvent::ControlPointUpdateOwner(event) => event.write(stream, definition),
            GameEvent::ControlPointStartTouch(event) => event.write(stream, definition),
            GameEvent::ControlPointEndTouch(event) => event.write(stream, definition),
            GameEvent::ControlPointPulseElement(event) => event.write(stream, definition),
            GameEvent::ControlPointFakeCapture(event) => event.write(stream, definition),
            GameEvent::ControlPointFakeCaptureMultiplier(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundSelected(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundStart(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundActive(event) => event.write(stream, definition),
            GameEvent::TeamPlayWaitingBegins(event) => event.write(stream, definition),
            GameEvent::TeamPlayWaitingEnds(event) => event.write(stream, definition),
            GameEvent::TeamPlayWaitingAboutToEnd(event) => event.write(stream, definition),
            GameEvent::TeamPlayRestartRound(event) => event.write(stream, definition),
            GameEvent::TeamPlayReadyRestart(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundRestartSeconds(event) => event.write(stream, definition),
            GameEvent::TeamPlayTeamReady(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundWin(event) => event.write(stream, definition),
            GameEvent::TeamPlayUpdateTimer(event) => event.write(stream, definition),
            GameEvent::TeamPlayRoundStalemate(event) => event.write(stream, definition),
            GameEvent::TeamPlayOvertimeBegin(event) => event.write(stream, definition),
            GameEvent::TeamPlayOvertimeEnd(event) => event.write(stream, definition),
            GameEvent::TeamPlaySuddenDeathBegin(event) => event.write(stream, definition),
            GameEvent::TeamPlaySuddenDeathEnd(event) => event.write(stream, definition),
            GameEvent::TeamPlayGameOver(event) => event.write(stream, definition),
            GameEvent::TeamPlayMapTimeRemaining(event) => event.write(stream, definition),
            GameEvent::TeamPlayTimerFlash(event) => event.write(stream, definition),
            GameEvent::TeamPlayTimerTimeAdded(event) => event.write(stream, definition),
            GameEvent::TeamPlayPointStartCapture(event) => event.write(stream, definition),
            GameEvent::TeamPlayPointCaptured(event) => event.write(stream, definition),
            GameEvent::TeamPlayPointLocked(event) => event.write(stream, definition),
            GameEvent::TeamPlayPointUnlocked(event) => event.write(stream, definition),
            GameEvent::TeamPlayCaptureBroken(event) => event.write(stream, definition),
            GameEvent::TeamPlayCaptureBlocked(event) => event.write(stream, definition),
            GameEvent::TeamPlayFlagEvent(event) => event.write(stream, definition),
            GameEvent::TeamPlayWinPanel(event) => event.write(stream, definition),
            GameEvent::TeamPlayTeamBalancedPlayer(event) => event.write(stream, definition),
            GameEvent::TeamPlaySetupFinished(event) => event.write(stream, definition),
            GameEvent::TeamPlayAlert(event) => event.write(stream, definition),
            GameEvent::TrainingComplete(event) => event.write(stream, definition),
            GameEvent::ShowFreezePanel(event) => event.write(stream, definition),
            GameEvent::HideFreezePanel(event) => event.write(stream, definition),
            GameEvent::FreezeCamStarted(event) => event.write(stream, definition),
            GameEvent::LocalPlayerChangeTeam(event) => event.write(stream, definition),
            GameEvent::LocalPlayerScoreChanged(event) => event.write(stream, definition),
            GameEvent::LocalPlayerChangeClass(event) => event.write(stream, definition),
            GameEvent::LocalPlayerRespawn(event) => event.write(stream, definition),
            GameEvent::BuildingInfoChanged(event) => event.write(stream, definition),
            GameEvent::LocalPlayerChangeDisguise(event) => event.write(stream, definition),
            GameEvent::PlayerAccountChanged(event) => event.write(stream, definition),
            GameEvent::SpyPdaReset(event) => event.write(stream, definition),
            GameEvent::FlagStatusUpdate(event) => event.write(stream, definition),
            GameEvent::PlayerStatsUpdated(event) => event.write(stream, definition),
            GameEvent::PlayingCommentary(event) => event.write(stream, definition),
            GameEvent::PlayerChargeDeployed(event) => event.write(stream, definition),
            GameEvent::PlayerBuiltObject(event) => event.write(stream, definition),
            GameEvent::PlayerUpgradedObject(event) => event.write(stream, definition),
            GameEvent::PlayerCarryObject(event) => event.write(stream, definition),
            GameEvent::PlayerDropObject(event) => event.write(stream, definition),
            GameEvent::ObjectRemoved(event) => event.write(stream, definition),
            GameEvent::ObjectDestroyed(event) => event.write(stream, definition),
            GameEvent::ObjectDetonated(event) => event.write(stream, definition),
            GameEvent::AchievementEarned(event) => event.write(stream, definition),
            GameEvent::SpecTargetUpdated(event) => event.write(stream, definition),
            GameEvent::TournamentStateUpdate(event) => event.write(stream, definition),
            GameEvent::TournamentEnableCountdown(event) => event.write(stream, definition),
            GameEvent::PlayerCalledForMedic(event) => event.write(stream, definition),
            GameEvent::PlayerAskedForBall(event) => event.write(stream, definition),
            GameEvent::LocalPlayerBecameObserver(event) => event.write(stream, definition),
            GameEvent::PlayerIgnitedInv(event) => event.write(stream, definition),
            GameEvent::PlayerIgnited(event) => event.write(stream, definition),
            GameEvent::PlayerExtinguished(event) => event.write(stream, definition),
            GameEvent::PlayerTeleported(event) => event.write(stream, definition),
            GameEvent::PlayerHealedMedicCall(event) => event.write(stream, definition),
            GameEvent::LocalPlayerChargeReady(event) => event.write(stream, definition),
            GameEvent::LocalPlayerWindDown(event) => event.write(stream, definition),
            GameEvent::PlayerInvulned(event) => event.write(stream, definition),
            GameEvent::EscortSpeed(event) => event.write(stream, definition),
            GameEvent::EscortProgress(event) => event.write(stream, definition),
            GameEvent::EscortRecede(event) => event.write(stream, definition),
            GameEvent::GameUIActivated(event) => event.write(stream, definition),
            GameEvent::GameUIHidden(event) => event.write(stream, definition),
            GameEvent::PlayerEscortScore(event) => event.write(stream, definition),
            GameEvent::PlayerHealOnHit(event) => event.write(stream, definition),
            GameEvent::PlayerStealSandvich(event) => event.write(stream, definition),
            GameEvent::ShowClassLayout(event) => event.write(stream, definition),
            GameEvent::ShowVsPanel(event) => event.write(stream, definition),
            GameEvent::PlayerDamaged(event) => event.write(stream, definition),
            GameEvent::ArenaPlayerNotification(event) => event.write(stream, definition),
            GameEvent::ArenaMatchMaxStreak(event) => event.write(stream, definition),
            GameEvent::ArenaRoundStart(event) => event.write(stream, definition),
            GameEvent::ArenaWinPanel(event) => event.write(stream, definition),
            GameEvent::PveWinPanel(event) => event.write(stream, definition),
            GameEvent::AirDash(event) => event.write(stream, definition),
            GameEvent::Landed(event) => event.write(stream, definition),
            GameEvent::PlayerDamageDodged(event) => event.write(stream, definition),
            GameEvent::PlayerStunned(event) => event.write(stream, definition),
            GameEvent::ScoutGrandSlam(event) => event.write(stream, definition),
            GameEvent::ScoutSlamdollLanded(event) => event.write(stream, definition),
            GameEvent::ArrowImpact(event) => event.write(stream, definition),
            GameEvent::PlayerJarated(event) => event.write(stream, definition),
            GameEvent::PlayerJaratedFade(event) => event.write(stream, definition),
            GameEvent::PlayerShieldBlocked(event) => event.write(stream, definition),
            GameEvent::PlayerPinned(event) => event.write(stream, definition),
            GameEvent::PlayerHealedByMedic(event) => event.write(stream, definition),
            GameEvent::PlayerSappedObject(event) => event.write(stream, definition),
            GameEvent::ItemFound(event) => event.write(stream, definition),
            GameEvent::ShowAnnotation(event) => event.write(stream, definition),
            GameEvent::HideAnnotation(event) => event.write(stream, definition),
            GameEvent::PostInventoryApplication(event) => event.write(stream, definition),
            GameEvent::ControlPointUnlockUpdated(event) => event.write(stream, definition),
            GameEvent::DeployBuffBanner(event) => event.write(stream, definition),
            GameEvent::PlayerBuff(event) => event.write(stream, definition),
            GameEvent::MedicDeath(event) => event.write(stream, definition),
            GameEvent::OvertimeNag(event) => event.write(stream, definition),
            GameEvent::TeamsChanged(event) => event.write(stream, definition),
            GameEvent::HalloweenPumpkinGrab(event) => event.write(stream, definition),
            GameEvent::RocketJump(event) => event.write(stream, definition),
            GameEvent::RocketJumpLanded(event) => event.write(stream, definition),
            GameEvent::StickyJump(event) => event.write(stream, definition),
            GameEvent::StickyJumpLanded(event) => event.write(stream, definition),
            GameEvent::RocketPackLaunch(event) => event.write(stream, definition),
            GameEvent::RocketPackLanded(event) => event.write(stream, definition),
            GameEvent::MedicDefended(event) => event.write(stream, definition),
            GameEvent::LocalPlayerHealed(event) => event.write(stream, definition),
            GameEvent::PlayerDestroyedPipeBomb(event) => event.write(stream, definition),
            GameEvent::ObjectDeflected(event) => event.write(stream, definition),
            GameEvent::PlayerMvp(event) => event.write(stream, definition),
            GameEvent::RaidSpawnMob(event) => event.write(stream, definition),
            GameEvent::RaidSpawnSquad(event) => event.write(stream, definition),
            GameEvent::NavBlocked(event) => event.write(stream, definition),
            GameEvent::PathTrackPassed(event) => event.write(stream, definition),
            GameEvent::NumCappersChanged(event) => event.write(stream, definition),
            GameEvent::PlayerRegenerate(event) => event.write(stream, definition),
            GameEvent::UpdateStatusItem(event) => event.write(stream, definition),
            GameEvent::StatsResetRound(event) => event.write(stream, definition),
            GameEvent::ScoreStatsAccumulatedUpdate(event) => event.write(stream, definition),
            GameEvent::ScoreStatsAccumulatedReset(event) => event.write(stream, definition),
            GameEvent::AchievementEarnedLocal(event) => event.write(stream, definition),
            GameEvent::PlayerHealed(event) => event.write(stream, definition),
            GameEvent::BuildingHealed(event) => event.write(stream, definition),
            GameEvent::ItemPickup(event) => event.write(stream, definition),
            GameEvent::DuelStatus(event) => event.write(stream, definition),
            GameEvent::FishNotice(event) => event.write(stream, definition),
            GameEvent::FishNoticeArm(event) => event.write(stream, definition),
            GameEvent::SlapNotice(event) => event.write(stream, definition),
            GameEvent::ThrowableHit(event) => event.write(stream, definition),
            GameEvent::PumpkinLordSummoned(event) => event.write(stream, definition),
            GameEvent::PumpkinLordKilled(event) => event.write(stream, definition),
            GameEvent::MerasmusSummoned(event) => event.write(stream, definition),
            GameEvent::MerasmusKilled(event) => event.write(stream, definition),
            GameEvent::MerasmusEscapeWarning(event) => event.write(stream, definition),
            GameEvent::MerasmusEscaped(event) => event.write(stream, definition),
            GameEvent::EyeballBossSummoned(event) => event.write(stream, definition),
            GameEvent::EyeballBossStunned(event) => event.write(stream, definition),
            GameEvent::EyeballBossKilled(event) => event.write(stream, definition),
            GameEvent::EyeballBossKiller(event) => event.write(stream, definition),
            GameEvent::EyeballBossEscapeImminent(event) => event.write(stream, definition),
            GameEvent::EyeballBossEscaped(event) => event.write(stream, definition),
            GameEvent::NpcHurt(event) => event.write(stream, definition),
            GameEvent::ControlPointTimerUpdated(event) => event.write(stream, definition),
            GameEvent::PlayerHighFiveStart(event) => event.write(stream, definition),
            GameEvent::PlayerHighFiveCancel(event) => event.write(stream, definition),
            GameEvent::PlayerHighFiveSuccess(event) => event.write(stream, definition),
            GameEvent::PlayerBonusPoints(event) => event.write(stream, definition),
            GameEvent::PlayerUpgraded(event) => event.write(stream, definition),
            GameEvent::PlayerBuyback(event) => event.write(stream, definition),
            GameEvent::PlayerUsedPowerUpBottle(event) => event.write(stream, definition),
            GameEvent::ChristmasGiftGrab(event) => event.write(stream, definition),
            GameEvent::PlayerKilledAchievementZone(event) => event.write(stream, definition),
            GameEvent::PartyUpdated(event) => event.write(stream, definition),
            GameEvent::PartyPrefChanged(event) => event.write(stream, definition),
            GameEvent::PartyCriteriaChanged(event) => event.write(stream, definition),
            GameEvent::PartyInvitesChanged(event) => event.write(stream, definition),
            GameEvent::PartyQueueStateChanged(event) => event.write(stream, definition),
            GameEvent::PartyChat(event) => event.write(stream, definition),
            GameEvent::PartyMemberJoin(event) => event.write(stream, definition),
            GameEvent::PartyMemberLeave(event) => event.write(stream, definition),
            GameEvent::MatchInvitesUpdated(event) => event.write(stream, definition),
            GameEvent::LobbyUpdated(event) => event.write(stream, definition),
            GameEvent::MvmMissionUpdate(event) => event.write(stream, definition),
            GameEvent::RecalculateHolidays(event) => event.write(stream, definition),
            GameEvent::PlayerCurrencyChanged(event) => event.write(stream, definition),
            GameEvent::DoomsdayRocketOpen(event) => event.write(stream, definition),
            GameEvent::RemoveNemesisRelationships(event) => event.write(stream, definition),
            GameEvent::MvmCreditBonusWave(event) => event.write(stream, definition),
            GameEvent::MvmCreditBonusAll(event) => event.write(stream, definition),
            GameEvent::MvmCreditBonusAllAdvanced(event) => event.write(stream, definition),
            GameEvent::MvmQuickSentryUpgrade(event) => event.write(stream, definition),
            GameEvent::MvmTankDestroyedByPlayers(event) => event.write(stream, definition),
            GameEvent::MvmKillRobotDeliveringBomb(event) => event.write(stream, definition),
            GameEvent::MvmPickupCurrency(event) => event.write(stream, definition),
            GameEvent::MvmBombCarrierKilled(event) => event.write(stream, definition),
            GameEvent::MvmSentryBusterDetonate(event) => event.write(stream, definition),
            GameEvent::MvmScoutMarkedForDeath(event) => event.write(stream, definition),
            GameEvent::MvmMedicPowerUpShared(event) => event.write(stream, definition),
            GameEvent::MvmBeginWave(event) => event.write(stream, definition),
            GameEvent::MvmWaveComplete(event) => event.write(stream, definition),
            GameEvent::MvmMissionComplete(event) => event.write(stream, definition),
            GameEvent::MvmBombResetByPlayer(event) => event.write(stream, definition),
            GameEvent::MvmBombAlarmTriggered(event) => event.write(stream, definition),
            GameEvent::MvmBombDeployResetByPlayer(event) => event.write(stream, definition),
            GameEvent::MvmWaveFailed(event) => event.write(stream, definition),
            GameEvent::MvmResetStats(event) => event.write(stream, definition),
            GameEvent::DamageResisted(event) => event.write(stream, definition),
            GameEvent::RevivePlayerNotify(event) => event.write(stream, definition),
            GameEvent::RevivePlayerStopped(event) => event.write(stream, definition),
            GameEvent::RevivePlayerComplete(event) => event.write(stream, definition),
            GameEvent::PlayerTurnedToGhost(event) => event.write(stream, definition),
            GameEvent::MedigunShieldBlockedDamage(event) => event.write(stream, definition),
            GameEvent::MvmAdvWaveCompleteNoGates(event) => event.write(stream, definition),
            GameEvent::MvmSniperHeadshotCurrency(event) => event.write(stream, definition),
            GameEvent::MvmMannhattanPit(event) => event.write(stream, definition),
            GameEvent::FlagCarriedInDetectionZone(event) => event.write(stream, definition),
            GameEvent::MvmAdvWaveKilledStunRadio(event) => event.write(stream, definition),
            GameEvent::PlayerDirectHitStun(event) => event.write(stream, definition),
            GameEvent::MvmSentryBusterKilled(event) => event.write(stream, definition),
            GameEvent::UpgradesFileChanged(event) => event.write(stream, definition),
            GameEvent::RdTeamPointsChanged(event) => event.write(stream, definition),
            GameEvent::RdRulesStateChanged(event) => event.write(stream, definition),
            GameEvent::RdRobotKilled(event) => event.write(stream, definition),
            GameEvent::RdRobotImpact(event) => event.write(stream, definition),
            GameEvent::TeamPlayPreRoundTimeLeft(event) => event.write(stream, definition),
            GameEvent::ParachuteDeploy(event) => event.write(stream, definition),
            GameEvent::ParachuteHolster(event) => event.write(stream, definition),
            GameEvent::KillRefillsMeter(event) => event.write(stream, definition),
            GameEvent::RpsTauntEvent(event) => event.write(stream, definition),
            GameEvent::CongaKill(event) => event.write(stream, definition),
            GameEvent::PlayerInitialSpawn(event) => event.write(stream, definition),
            GameEvent::CompetitiveVictory(event) => event.write(stream, definition),
            GameEvent::CompetitiveStatsUpdate(event) => event.write(stream, definition),
            GameEvent::MiniGameWin(event) => event.write(stream, definition),
            GameEvent::SentryOnGoActive(event) => event.write(stream, definition),
            GameEvent::DuckXpLevelUp(event) => event.write(stream, definition),
            GameEvent::QuestLogOpened(event) => event.write(stream, definition),
            GameEvent::SchemaUpdated(event) => event.write(stream, definition),
            GameEvent::LocalPlayerPickupWeapon(event) => event.write(stream, definition),
            GameEvent::RdPlayerScorePoints(event) => event.write(stream, definition),
            GameEvent::DemomanDetStickies(event) => event.write(stream, definition),
            GameEvent::QuestObjectiveCompleted(event) => event.write(stream, definition),
            GameEvent::PlayerScoreChanged(event) => event.write(stream, definition),
            GameEvent::KilledCappingPlayer(event) => event.write(stream, definition),
            GameEvent::EnvironmentalDeath(event) => event.write(stream, definition),
            GameEvent::ProjectileDirectHit(event) => event.write(stream, definition),
            GameEvent::PassGet(event) => event.write(stream, definition),
            GameEvent::PassScore(event) => event.write(stream, definition),
            GameEvent::PassFree(event) => event.write(stream, definition),
            GameEvent::PassPassCaught(event) => event.write(stream, definition),
            GameEvent::PassBallStolen(event) => event.write(stream, definition),
            GameEvent::PassBallBlocked(event) => event.write(stream, definition),
            GameEvent::DamagePrevented(event) => event.write(stream, definition),
            GameEvent::HalloweenBossKilled(event) => event.write(stream, definition),
            GameEvent::EscapedLootIsland(event) => event.write(stream, definition),
            GameEvent::TaggedPlayerAsIt(event) => event.write(stream, definition),
            GameEvent::MerasmusStunned(event) => event.write(stream, definition),
            GameEvent::MerasmusPropFound(event) => event.write(stream, definition),
            GameEvent::HalloweenSkeletonKilled(event) => event.write(stream, definition),
            GameEvent::SkeletonKilledQuest(event) => event.write(stream, definition),
            GameEvent::SkeletonKingKilledQuest(event) => event.write(stream, definition),
            GameEvent::EscapeHell(event) => event.write(stream, definition),
            GameEvent::CrossSpectralBridge(event) => event.write(stream, definition),
            GameEvent::MiniGameWon(event) => event.write(stream, definition),
            GameEvent::RespawnGhost(event) => event.write(stream, definition),
            GameEvent::KillInHell(event) => event.write(stream, definition),
            GameEvent::HalloweenDuckCollected(event) => event.write(stream, definition),
            GameEvent::SpecialScore(event) => event.write(stream, definition),
            GameEvent::TeamLeaderKilled(event) => event.write(stream, definition),
            GameEvent::HalloweenSoulCollected(event) => event.write(stream, definition),
            GameEvent::RecalculateTruce(event) => event.write(stream, definition),
            GameEvent::DeadRingerCheatDeath(event) => event.write(stream, definition),
            GameEvent::CrossbowHeal(event) => event.write(stream, definition),
            GameEvent::DamageMitigated(event) => event.write(stream, definition),
            GameEvent::PayloadPushed(event) => event.write(stream, definition),
            GameEvent::PlayerAbandonedMatch(event) => event.write(stream, definition),
            GameEvent::ClDrawline(event) => event.write(stream, definition),
            GameEvent::RestartTimerTime(event) => event.write(stream, definition),
            GameEvent::WinLimitChanged(event) => event.write(stream, definition),
            GameEvent::WinPanelShowScores(event) => event.write(stream, definition),
            GameEvent::TopStreamsRequestFinished(event) => event.write(stream, definition),
            GameEvent::CompetitiveStateChanged(event) => event.write(stream, definition),
            GameEvent::GlobalWarDataUpdated(event) => event.write(stream, definition),
            GameEvent::StopWatchChanged(event) => event.write(stream, definition),
            GameEvent::DsStop(event) => event.write(stream, definition),
            GameEvent::DsScreenshot(event) => event.write(stream, definition),
            GameEvent::ShowMatchSummary(event) => event.write(stream, definition),
            GameEvent::ExperienceChanged(event) => event.write(stream, definition),
            GameEvent::BeginXpLerp(event) => event.write(stream, definition),
            GameEvent::MatchmakerStatsUpdated(event) => event.write(stream, definition),
            GameEvent::RematchVotePeriodOver(event) => event.write(stream, definition),
            GameEvent::RematchFailedToCreate(event) => event.write(stream, definition),
            GameEvent::PlayerRematchChange(event) => event.write(stream, definition),
            GameEvent::PingUpdated(event) => event.write(stream, definition),
            GameEvent::MMStatsUpdated(event) => event.write(stream, definition),
            GameEvent::PlayerNextMapVoteChange(event) => event.write(stream, definition),
            GameEvent::VoteMapsChanged(event) => event.write(stream, definition),
            GameEvent::ProtoDefChanged(event) => event.write(stream, definition),
            GameEvent::PlayerDomination(event) => event.write(stream, definition),
            GameEvent::PlayerRocketPackPushed(event) => event.write(stream, definition),
            GameEvent::QuestRequest(event) => event.write(stream, definition),
            GameEvent::QuestResponse(event) => event.write(stream, definition),
            GameEvent::QuestProgress(event) => event.write(stream, definition),
            GameEvent::ProjectileRemoved(event) => event.write(stream, definition),
            GameEvent::QuestMapDataChanged(event) => event.write(stream, definition),
            GameEvent::GasDousedPlayerIgnited(event) => event.write(stream, definition),
            GameEvent::QuestTurnInState(event) => event.write(stream, definition),
            GameEvent::ItemsAcknowledged(event) => event.write(stream, definition),
            GameEvent::CapperKilled(event) => event.write(stream, definition),
            GameEvent::MainMenuStabilized(event) => event.write(stream, definition),
            GameEvent::WorldStatusChanged(event) => event.write(stream, definition),
            GameEvent::HLTVStatus(event) => event.write(stream, definition),
            GameEvent::HLTVCameraman(event) => event.write(stream, definition),
            GameEvent::HLTVRankCamera(event) => event.write(stream, definition),
            GameEvent::HLTVRankEntity(event) => event.write(stream, definition),
            GameEvent::HLTVFixed(event) => event.write(stream, definition),
            GameEvent::HLTVChase(event) => event.write(stream, definition),
            GameEvent::HLTVMessage(event) => event.write(stream, definition),
            GameEvent::HLTVTitle(event) => event.write(stream, definition),
            GameEvent::HLTVChat(event) => event.write(stream, definition),
            GameEvent::ReplayStartRecord(event) => event.write(stream, definition),
            GameEvent::ReplaySessionInfo(event) => event.write(stream, definition),
            GameEvent::ReplayEndRecord(event) => event.write(stream, definition),
            GameEvent::ReplayReplaysAvailable(event) => event.write(stream, definition),
            GameEvent::ReplayServerError(event) => event.write(stream, definition),
            GameEvent::Unknown(raw) => Ok(raw.write(stream)?),
        }
    }
    pub fn event_type(&self) -> GameEventType {
        match &self {
            GameEvent::ServerSpawn(_) => GameEventType::ServerSpawn,
            GameEvent::ServerChangeLevelFailed(_) => GameEventType::ServerChangeLevelFailed,
            GameEvent::ServerShutdown(_) => GameEventType::ServerShutdown,
            GameEvent::ServerCvar(_) => GameEventType::ServerCvar,
            GameEvent::ServerMessage(_) => GameEventType::ServerMessage,
            GameEvent::ServerAddBan(_) => GameEventType::ServerAddBan,
            GameEvent::ServerRemoveBan(_) => GameEventType::ServerRemoveBan,
            GameEvent::PlayerConnect(_) => GameEventType::PlayerConnect,
            GameEvent::PlayerConnectClient(_) => GameEventType::PlayerConnectClient,
            GameEvent::PlayerInfo(_) => GameEventType::PlayerInfo,
            GameEvent::PlayerDisconnect(_) => GameEventType::PlayerDisconnect,
            GameEvent::PlayerActivate(_) => GameEventType::PlayerActivate,
            GameEvent::PlayerSay(_) => GameEventType::PlayerSay,
            GameEvent::ClientDisconnect(_) => GameEventType::ClientDisconnect,
            GameEvent::ClientBeginConnect(_) => GameEventType::ClientBeginConnect,
            GameEvent::ClientConnected(_) => GameEventType::ClientConnected,
            GameEvent::ClientFullConnect(_) => GameEventType::ClientFullConnect,
            GameEvent::HostQuit(_) => GameEventType::HostQuit,
            GameEvent::TeamInfo(_) => GameEventType::TeamInfo,
            GameEvent::TeamScore(_) => GameEventType::TeamScore,
            GameEvent::TeamPlayBroadcastAudio(_) => GameEventType::TeamPlayBroadcastAudio,
            GameEvent::PlayerTeam(_) => GameEventType::PlayerTeam,
            GameEvent::PlayerClass(_) => GameEventType::PlayerClass,
            GameEvent::PlayerDeath(_) => GameEventType::PlayerDeath,
            GameEvent::PlayerHurt(_) => GameEventType::PlayerHurt,
            GameEvent::PlayerChat(_) => GameEventType::PlayerChat,
            GameEvent::PlayerScore(_) => GameEventType::PlayerScore,
            GameEvent::PlayerSpawn(_) => GameEventType::PlayerSpawn,
            GameEvent::PlayerShoot(_) => GameEventType::PlayerShoot,
            GameEvent::PlayerUse(_) => GameEventType::PlayerUse,
            GameEvent::PlayerChangeName(_) => GameEventType::PlayerChangeName,
            GameEvent::PlayerHintMessage(_) => GameEventType::PlayerHintMessage,
            GameEvent::BasePlayerTeleported(_) => GameEventType::BasePlayerTeleported,
            GameEvent::GameInit(_) => GameEventType::GameInit,
            GameEvent::GameNewMap(_) => GameEventType::GameNewMap,
            GameEvent::GameStart(_) => GameEventType::GameStart,
            GameEvent::GameEnd(_) => GameEventType::GameEnd,
            GameEvent::RoundStart(_) => GameEventType::RoundStart,
            GameEvent::RoundEnd(_) => GameEventType::RoundEnd,
            GameEvent::GameMessage(_) => GameEventType::GameMessage,
            GameEvent::BreakBreakable(_) => GameEventType::BreakBreakable,
            GameEvent::BreakProp(_) => GameEventType::BreakProp,
            GameEvent::EntityKilled(_) => GameEventType::EntityKilled,
            GameEvent::BonusUpdated(_) => GameEventType::BonusUpdated,
            GameEvent::AchievementEvent(_) => GameEventType::AchievementEvent,
            GameEvent::AchievementIncrement(_) => GameEventType::AchievementIncrement,
            GameEvent::PhysgunPickup(_) => GameEventType::PhysgunPickup,
            GameEvent::FlareIgniteNpc(_) => GameEventType::FlareIgniteNpc,
            GameEvent::HelicopterGrenadePuntMiss(_) => GameEventType::HelicopterGrenadePuntMiss,
            GameEvent::UserDataDownloaded(_) => GameEventType::UserDataDownloaded,
            GameEvent::RagdollDissolved(_) => GameEventType::RagdollDissolved,
            GameEvent::HLTVChangedMode(_) => GameEventType::HLTVChangedMode,
            GameEvent::HLTVChangedTarget(_) => GameEventType::HLTVChangedTarget,
            GameEvent::VoteEnded(_) => GameEventType::VoteEnded,
            GameEvent::VoteStarted(_) => GameEventType::VoteStarted,
            GameEvent::VoteChanged(_) => GameEventType::VoteChanged,
            GameEvent::VotePassed(_) => GameEventType::VotePassed,
            GameEvent::VoteFailed(_) => GameEventType::VoteFailed,
            GameEvent::VoteCast(_) => GameEventType::VoteCast,
            GameEvent::VoteOptions(_) => GameEventType::VoteOptions,
            GameEvent::ReplaySaved(_) => GameEventType::ReplaySaved,
            GameEvent::EnteredPerformanceMode(_) => GameEventType::EnteredPerformanceMode,
            GameEvent::BrowseReplays(_) => GameEventType::BrowseReplays,
            GameEvent::ReplayYoutubeStats(_) => GameEventType::ReplayYoutubeStats,
            GameEvent::InventoryUpdated(_) => GameEventType::InventoryUpdated,
            GameEvent::CartUpdated(_) => GameEventType::CartUpdated,
            GameEvent::StorePriceSheetUpdated(_) => GameEventType::StorePriceSheetUpdated,
            GameEvent::EconInventoryConnected(_) => GameEventType::EconInventoryConnected,
            GameEvent::ItemSchemaInitialized(_) => GameEventType::ItemSchemaInitialized,
            GameEvent::GcNewSession(_) => GameEventType::GcNewSession,
            GameEvent::GcLostSession(_) => GameEventType::GcLostSession,
            GameEvent::IntroFinish(_) => GameEventType::IntroFinish,
            GameEvent::IntroNextCamera(_) => GameEventType::IntroNextCamera,
            GameEvent::PlayerChangeClass(_) => GameEventType::PlayerChangeClass,
            GameEvent::TfMapTimeRemaining(_) => GameEventType::TfMapTimeRemaining,
            GameEvent::TfGameOver(_) => GameEventType::TfGameOver,
            GameEvent::CtfFlagCaptured(_) => GameEventType::CtfFlagCaptured,
            GameEvent::ControlPointInitialized(_) => GameEventType::ControlPointInitialized,
            GameEvent::ControlPointUpdateImages(_) => GameEventType::ControlPointUpdateImages,
            GameEvent::ControlPointUpdateLayout(_) => GameEventType::ControlPointUpdateLayout,
            GameEvent::ControlPointUpdateCapping(_) => GameEventType::ControlPointUpdateCapping,
            GameEvent::ControlPointUpdateOwner(_) => GameEventType::ControlPointUpdateOwner,
            GameEvent::ControlPointStartTouch(_) => GameEventType::ControlPointStartTouch,
            GameEvent::ControlPointEndTouch(_) => GameEventType::ControlPointEndTouch,
            GameEvent::ControlPointPulseElement(_) => GameEventType::ControlPointPulseElement,
            GameEvent::ControlPointFakeCapture(_) => GameEventType::ControlPointFakeCapture,
            GameEvent::ControlPointFakeCaptureMultiplier(_) => {
                GameEventType::ControlPointFakeCaptureMultiplier
            }
            GameEvent::TeamPlayRoundSelected(_) => GameEventType::TeamPlayRoundSelected,
            GameEvent::TeamPlayRoundStart(_) => GameEventType::TeamPlayRoundStart,
            GameEvent::TeamPlayRoundActive(_) => GameEventType::TeamPlayRoundActive,
            GameEvent::TeamPlayWaitingBegins(_) => GameEventType::TeamPlayWaitingBegins,
            GameEvent::TeamPlayWaitingEnds(_) => GameEventType::TeamPlayWaitingEnds,
            GameEvent::TeamPlayWaitingAboutToEnd(_) => GameEventType::TeamPlayWaitingAboutToEnd,
            GameEvent::TeamPlayRestartRound(_) => GameEventType::TeamPlayRestartRound,
            GameEvent::TeamPlayReadyRestart(_) => GameEventType::TeamPlayReadyRestart,
            GameEvent::TeamPlayRoundRestartSeconds(_) => GameEventType::TeamPlayRoundRestartSeconds,
            GameEvent::TeamPlayTeamReady(_) => GameEventType::TeamPlayTeamReady,
            GameEvent::TeamPlayRoundWin(_) => GameEventType::TeamPlayRoundWin,
            GameEvent::TeamPlayUpdateTimer(_) => GameEventType::TeamPlayUpdateTimer,
            GameEvent::TeamPlayRoundStalemate(_) => GameEventType::TeamPlayRoundStalemate,
            GameEvent::TeamPlayOvertimeBegin(_) => GameEventType::TeamPlayOvertimeBegin,
            GameEvent::TeamPlayOvertimeEnd(_) => GameEventType::TeamPlayOvertimeEnd,
            GameEvent::TeamPlaySuddenDeathBegin(_) => GameEventType::TeamPlaySuddenDeathBegin,
            GameEvent::TeamPlaySuddenDeathEnd(_) => GameEventType::TeamPlaySuddenDeathEnd,
            GameEvent::TeamPlayGameOver(_) => GameEventType::TeamPlayGameOver,
            GameEvent::TeamPlayMapTimeRemaining(_) => GameEventType::TeamPlayMapTimeRemaining,
            GameEvent::TeamPlayTimerFlash(_) => GameEventType::TeamPlayTimerFlash,
            GameEvent::TeamPlayTimerTimeAdded(_) => GameEventType::TeamPlayTimerTimeAdded,
            GameEvent::TeamPlayPointStartCapture(_) => GameEventType::TeamPlayPointStartCapture,
            GameEvent::TeamPlayPointCaptured(_) => GameEventType::TeamPlayPointCaptured,
            GameEvent::TeamPlayPointLocked(_) => GameEventType::TeamPlayPointLocked,
            GameEvent::TeamPlayPointUnlocked(_) => GameEventType::TeamPlayPointUnlocked,
            GameEvent::TeamPlayCaptureBroken(_) => GameEventType::TeamPlayCaptureBroken,
            GameEvent::TeamPlayCaptureBlocked(_) => GameEventType::TeamPlayCaptureBlocked,
            GameEvent::TeamPlayFlagEvent(_) => GameEventType::TeamPlayFlagEvent,
            GameEvent::TeamPlayWinPanel(_) => GameEventType::TeamPlayWinPanel,
            GameEvent::TeamPlayTeamBalancedPlayer(_) => GameEventType::TeamPlayTeamBalancedPlayer,
            GameEvent::TeamPlaySetupFinished(_) => GameEventType::TeamPlaySetupFinished,
            GameEvent::TeamPlayAlert(_) => GameEventType::TeamPlayAlert,
            GameEvent::TrainingComplete(_) => GameEventType::TrainingComplete,
            GameEvent::ShowFreezePanel(_) => GameEventType::ShowFreezePanel,
            GameEvent::HideFreezePanel(_) => GameEventType::HideFreezePanel,
            GameEvent::FreezeCamStarted(_) => GameEventType::FreezeCamStarted,
            GameEvent::LocalPlayerChangeTeam(_) => GameEventType::LocalPlayerChangeTeam,
            GameEvent::LocalPlayerScoreChanged(_) => GameEventType::LocalPlayerScoreChanged,
            GameEvent::LocalPlayerChangeClass(_) => GameEventType::LocalPlayerChangeClass,
            GameEvent::LocalPlayerRespawn(_) => GameEventType::LocalPlayerRespawn,
            GameEvent::BuildingInfoChanged(_) => GameEventType::BuildingInfoChanged,
            GameEvent::LocalPlayerChangeDisguise(_) => GameEventType::LocalPlayerChangeDisguise,
            GameEvent::PlayerAccountChanged(_) => GameEventType::PlayerAccountChanged,
            GameEvent::SpyPdaReset(_) => GameEventType::SpyPdaReset,
            GameEvent::FlagStatusUpdate(_) => GameEventType::FlagStatusUpdate,
            GameEvent::PlayerStatsUpdated(_) => GameEventType::PlayerStatsUpdated,
            GameEvent::PlayingCommentary(_) => GameEventType::PlayingCommentary,
            GameEvent::PlayerChargeDeployed(_) => GameEventType::PlayerChargeDeployed,
            GameEvent::PlayerBuiltObject(_) => GameEventType::PlayerBuiltObject,
            GameEvent::PlayerUpgradedObject(_) => GameEventType::PlayerUpgradedObject,
            GameEvent::PlayerCarryObject(_) => GameEventType::PlayerCarryObject,
            GameEvent::PlayerDropObject(_) => GameEventType::PlayerDropObject,
            GameEvent::ObjectRemoved(_) => GameEventType::ObjectRemoved,
            GameEvent::ObjectDestroyed(_) => GameEventType::ObjectDestroyed,
            GameEvent::ObjectDetonated(_) => GameEventType::ObjectDetonated,
            GameEvent::AchievementEarned(_) => GameEventType::AchievementEarned,
            GameEvent::SpecTargetUpdated(_) => GameEventType::SpecTargetUpdated,
            GameEvent::TournamentStateUpdate(_) => GameEventType::TournamentStateUpdate,
            GameEvent::TournamentEnableCountdown(_) => GameEventType::TournamentEnableCountdown,
            GameEvent::PlayerCalledForMedic(_) => GameEventType::PlayerCalledForMedic,
            GameEvent::PlayerAskedForBall(_) => GameEventType::PlayerAskedForBall,
            GameEvent::LocalPlayerBecameObserver(_) => GameEventType::LocalPlayerBecameObserver,
            GameEvent::PlayerIgnitedInv(_) => GameEventType::PlayerIgnitedInv,
            GameEvent::PlayerIgnited(_) => GameEventType::PlayerIgnited,
            GameEvent::PlayerExtinguished(_) => GameEventType::PlayerExtinguished,
            GameEvent::PlayerTeleported(_) => GameEventType::PlayerTeleported,
            GameEvent::PlayerHealedMedicCall(_) => GameEventType::PlayerHealedMedicCall,
            GameEvent::LocalPlayerChargeReady(_) => GameEventType::LocalPlayerChargeReady,
            GameEvent::LocalPlayerWindDown(_) => GameEventType::LocalPlayerWindDown,
            GameEvent::PlayerInvulned(_) => GameEventType::PlayerInvulned,
            GameEvent::EscortSpeed(_) => GameEventType::EscortSpeed,
            GameEvent::EscortProgress(_) => GameEventType::EscortProgress,
            GameEvent::EscortRecede(_) => GameEventType::EscortRecede,
            GameEvent::GameUIActivated(_) => GameEventType::GameUIActivated,
            GameEvent::GameUIHidden(_) => GameEventType::GameUIHidden,
            GameEvent::PlayerEscortScore(_) => GameEventType::PlayerEscortScore,
            GameEvent::PlayerHealOnHit(_) => GameEventType::PlayerHealOnHit,
            GameEvent::PlayerStealSandvich(_) => GameEventType::PlayerStealSandvich,
            GameEvent::ShowClassLayout(_) => GameEventType::ShowClassLayout,
            GameEvent::ShowVsPanel(_) => GameEventType::ShowVsPanel,
            GameEvent::PlayerDamaged(_) => GameEventType::PlayerDamaged,
            GameEvent::ArenaPlayerNotification(_) => GameEventType::ArenaPlayerNotification,
            GameEvent::ArenaMatchMaxStreak(_) => GameEventType::ArenaMatchMaxStreak,
            GameEvent::ArenaRoundStart(_) => GameEventType::ArenaRoundStart,
            GameEvent::ArenaWinPanel(_) => GameEventType::ArenaWinPanel,
            GameEvent::PveWinPanel(_) => GameEventType::PveWinPanel,
            GameEvent::AirDash(_) => GameEventType::AirDash,
            GameEvent::Landed(_) => GameEventType::Landed,
            GameEvent::PlayerDamageDodged(_) => GameEventType::PlayerDamageDodged,
            GameEvent::PlayerStunned(_) => GameEventType::PlayerStunned,
            GameEvent::ScoutGrandSlam(_) => GameEventType::ScoutGrandSlam,
            GameEvent::ScoutSlamdollLanded(_) => GameEventType::ScoutSlamdollLanded,
            GameEvent::ArrowImpact(_) => GameEventType::ArrowImpact,
            GameEvent::PlayerJarated(_) => GameEventType::PlayerJarated,
            GameEvent::PlayerJaratedFade(_) => GameEventType::PlayerJaratedFade,
            GameEvent::PlayerShieldBlocked(_) => GameEventType::PlayerShieldBlocked,
            GameEvent::PlayerPinned(_) => GameEventType::PlayerPinned,
            GameEvent::PlayerHealedByMedic(_) => GameEventType::PlayerHealedByMedic,
            GameEvent::PlayerSappedObject(_) => GameEventType::PlayerSappedObject,
            GameEvent::ItemFound(_) => GameEventType::ItemFound,
            GameEvent::ShowAnnotation(_) => GameEventType::ShowAnnotation,
            GameEvent::HideAnnotation(_) => GameEventType::HideAnnotation,
            GameEvent::PostInventoryApplication(_) => GameEventType::PostInventoryApplication,
            GameEvent::ControlPointUnlockUpdated(_) => GameEventType::ControlPointUnlockUpdated,
            GameEvent::DeployBuffBanner(_) => GameEventType::DeployBuffBanner,
            GameEvent::PlayerBuff(_) => GameEventType::PlayerBuff,
            GameEvent::MedicDeath(_) => GameEventType::MedicDeath,
            GameEvent::OvertimeNag(_) => GameEventType::OvertimeNag,
            GameEvent::TeamsChanged(_) => GameEventType::TeamsChanged,
            GameEvent::HalloweenPumpkinGrab(_) => GameEventType::HalloweenPumpkinGrab,
            GameEvent::RocketJump(_) => GameEventType::RocketJump,
            GameEvent::RocketJumpLanded(_) => GameEventType::RocketJumpLanded,
            GameEvent::StickyJump(_) => GameEventType::StickyJump,
            GameEvent::StickyJumpLanded(_) => GameEventType::StickyJumpLanded,
            GameEvent::RocketPackLaunch(_) => GameEventType::RocketPackLaunch,
            GameEvent::RocketPackLanded(_) => GameEventType::RocketPackLanded,
            GameEvent::MedicDefended(_) => GameEventType::MedicDefended,
            GameEvent::LocalPlayerHealed(_) => GameEventType::LocalPlayerHealed,
            GameEvent::PlayerDestroyedPipeBomb(_) => GameEventType::PlayerDestroyedPipeBomb,
            GameEvent::ObjectDeflected(_) => GameEventType::ObjectDeflected,
            GameEvent::PlayerMvp(_) => GameEventType::PlayerMvp,
            GameEvent::RaidSpawnMob(_) => GameEventType::RaidSpawnMob,
            GameEvent::RaidSpawnSquad(_) => GameEventType::RaidSpawnSquad,
            GameEvent::NavBlocked(_) => GameEventType::NavBlocked,
            GameEvent::PathTrackPassed(_) => GameEventType::PathTrackPassed,
            GameEvent::NumCappersChanged(_) => GameEventType::NumCappersChanged,
            GameEvent::PlayerRegenerate(_) => GameEventType::PlayerRegenerate,
            GameEvent::UpdateStatusItem(_) => GameEventType::UpdateStatusItem,
            GameEvent::StatsResetRound(_) => GameEventType::StatsResetRound,
            GameEvent::ScoreStatsAccumulatedUpdate(_) => GameEventType::ScoreStatsAccumulatedUpdate,
            GameEvent::ScoreStatsAccumulatedReset(_) => GameEventType::ScoreStatsAccumulatedReset,
            GameEvent::AchievementEarnedLocal(_) => GameEventType::AchievementEarnedLocal,
            GameEvent::PlayerHealed(_) => GameEventType::PlayerHealed,
            GameEvent::BuildingHealed(_) => GameEventType::BuildingHealed,
            GameEvent::ItemPickup(_) => GameEventType::ItemPickup,
            GameEvent::DuelStatus(_) => GameEventType::DuelStatus,
            GameEvent::FishNotice(_) => GameEventType::FishNotice,
            GameEvent::FishNoticeArm(_) => GameEventType::FishNoticeArm,
            GameEvent::SlapNotice(_) => GameEventType::SlapNotice,
            GameEvent::ThrowableHit(_) => GameEventType::ThrowableHit,
            GameEvent::PumpkinLordSummoned(_) => GameEventType::PumpkinLordSummoned,
            GameEvent::PumpkinLordKilled(_) => GameEventType::PumpkinLordKilled,
            GameEvent::MerasmusSummoned(_) => GameEventType::MerasmusSummoned,
            GameEvent::MerasmusKilled(_) => GameEventType::MerasmusKilled,
            GameEvent::MerasmusEscapeWarning(_) => GameEventType::MerasmusEscapeWarning,
            GameEvent::MerasmusEscaped(_) => GameEventType::MerasmusEscaped,
            GameEvent::EyeballBossSummoned(_) => GameEventType::EyeballBossSummoned,
            GameEvent::EyeballBossStunned(_) => GameEventType::EyeballBossStunned,
            GameEvent::EyeballBossKilled(_) => GameEventType::EyeballBossKilled,
            GameEvent::EyeballBossKiller(_) => GameEventType::EyeballBossKiller,
            GameEvent::EyeballBossEscapeImminent(_) => GameEventType::EyeballBossEscapeImminent,
            GameEvent::EyeballBossEscaped(_) => GameEventType::EyeballBossEscaped,
            GameEvent::NpcHurt(_) => GameEventType::NpcHurt,
            GameEvent::ControlPointTimerUpdated(_) => GameEventType::ControlPointTimerUpdated,
            GameEvent::PlayerHighFiveStart(_) => GameEventType::PlayerHighFiveStart,
            GameEvent::PlayerHighFiveCancel(_) => GameEventType::PlayerHighFiveCancel,
            GameEvent::PlayerHighFiveSuccess(_) => GameEventType::PlayerHighFiveSuccess,
            GameEvent::PlayerBonusPoints(_) => GameEventType::PlayerBonusPoints,
            GameEvent::PlayerUpgraded(_) => GameEventType::PlayerUpgraded,
            GameEvent::PlayerBuyback(_) => GameEventType::PlayerBuyback,
            GameEvent::PlayerUsedPowerUpBottle(_) => GameEventType::PlayerUsedPowerUpBottle,
            GameEvent::ChristmasGiftGrab(_) => GameEventType::ChristmasGiftGrab,
            GameEvent::PlayerKilledAchievementZone(_) => GameEventType::PlayerKilledAchievementZone,
            GameEvent::PartyUpdated(_) => GameEventType::PartyUpdated,
            GameEvent::PartyPrefChanged(_) => GameEventType::PartyPrefChanged,
            GameEvent::PartyCriteriaChanged(_) => GameEventType::PartyCriteriaChanged,
            GameEvent::PartyInvitesChanged(_) => GameEventType::PartyInvitesChanged,
            GameEvent::PartyQueueStateChanged(_) => GameEventType::PartyQueueStateChanged,
            GameEvent::PartyChat(_) => GameEventType::PartyChat,
            GameEvent::PartyMemberJoin(_) => GameEventType::PartyMemberJoin,
            GameEvent::PartyMemberLeave(_) => GameEventType::PartyMemberLeave,
            GameEvent::MatchInvitesUpdated(_) => GameEventType::MatchInvitesUpdated,
            GameEvent::LobbyUpdated(_) => GameEventType::LobbyUpdated,
            GameEvent::MvmMissionUpdate(_) => GameEventType::MvmMissionUpdate,
            GameEvent::RecalculateHolidays(_) => GameEventType::RecalculateHolidays,
            GameEvent::PlayerCurrencyChanged(_) => GameEventType::PlayerCurrencyChanged,
            GameEvent::DoomsdayRocketOpen(_) => GameEventType::DoomsdayRocketOpen,
            GameEvent::RemoveNemesisRelationships(_) => GameEventType::RemoveNemesisRelationships,
            GameEvent::MvmCreditBonusWave(_) => GameEventType::MvmCreditBonusWave,
            GameEvent::MvmCreditBonusAll(_) => GameEventType::MvmCreditBonusAll,
            GameEvent::MvmCreditBonusAllAdvanced(_) => GameEventType::MvmCreditBonusAllAdvanced,
            GameEvent::MvmQuickSentryUpgrade(_) => GameEventType::MvmQuickSentryUpgrade,
            GameEvent::MvmTankDestroyedByPlayers(_) => GameEventType::MvmTankDestroyedByPlayers,
            GameEvent::MvmKillRobotDeliveringBomb(_) => GameEventType::MvmKillRobotDeliveringBomb,
            GameEvent::MvmPickupCurrency(_) => GameEventType::MvmPickupCurrency,
            GameEvent::MvmBombCarrierKilled(_) => GameEventType::MvmBombCarrierKilled,
            GameEvent::MvmSentryBusterDetonate(_) => GameEventType::MvmSentryBusterDetonate,
            GameEvent::MvmScoutMarkedForDeath(_) => GameEventType::MvmScoutMarkedForDeath,
            GameEvent::MvmMedicPowerUpShared(_) => GameEventType::MvmMedicPowerUpShared,
            GameEvent::MvmBeginWave(_) => GameEventType::MvmBeginWave,
            GameEvent::MvmWaveComplete(_) => GameEventType::MvmWaveComplete,
            GameEvent::MvmMissionComplete(_) => GameEventType::MvmMissionComplete,
            GameEvent::MvmBombResetByPlayer(_) => GameEventType::MvmBombResetByPlayer,
            GameEvent::MvmBombAlarmTriggered(_) => GameEventType::MvmBombAlarmTriggered,
            GameEvent::MvmBombDeployResetByPlayer(_) => GameEventType::MvmBombDeployResetByPlayer,
            GameEvent::MvmWaveFailed(_) => GameEventType::MvmWaveFailed,
            GameEvent::MvmResetStats(_) => GameEventType::MvmResetStats,
            GameEvent::DamageResisted(_) => GameEventType::DamageResisted,
            GameEvent::RevivePlayerNotify(_) => GameEventType::RevivePlayerNotify,
            GameEvent::RevivePlayerStopped(_) => GameEventType::RevivePlayerStopped,
            GameEvent::RevivePlayerComplete(_) => GameEventType::RevivePlayerComplete,
            GameEvent::PlayerTurnedToGhost(_) => GameEventType::PlayerTurnedToGhost,
            GameEvent::MedigunShieldBlockedDamage(_) => GameEventType::MedigunShieldBlockedDamage,
            GameEvent::MvmAdvWaveCompleteNoGates(_) => GameEventType::MvmAdvWaveCompleteNoGates,
            GameEvent::MvmSniperHeadshotCurrency(_) => GameEventType::MvmSniperHeadshotCurrency,
            GameEvent::MvmMannhattanPit(_) => GameEventType::MvmMannhattanPit,
            GameEvent::FlagCarriedInDetectionZone(_) => GameEventType::FlagCarriedInDetectionZone,
            GameEvent::MvmAdvWaveKilledStunRadio(_) => GameEventType::MvmAdvWaveKilledStunRadio,
            GameEvent::PlayerDirectHitStun(_) => GameEventType::PlayerDirectHitStun,
            GameEvent::MvmSentryBusterKilled(_) => GameEventType::MvmSentryBusterKilled,
            GameEvent::UpgradesFileChanged(_) => GameEventType::UpgradesFileChanged,
            GameEvent::RdTeamPointsChanged(_) => GameEventType::RdTeamPointsChanged,
            GameEvent::RdRulesStateChanged(_) => GameEventType::RdRulesStateChanged,
            GameEvent::RdRobotKilled(_) => GameEventType::RdRobotKilled,
            GameEvent::RdRobotImpact(_) => GameEventType::RdRobotImpact,
            GameEvent::TeamPlayPreRoundTimeLeft(_) => GameEventType::TeamPlayPreRoundTimeLeft,
            GameEvent::ParachuteDeploy(_) => GameEventType::ParachuteDeploy,
            GameEvent::ParachuteHolster(_) => GameEventType::ParachuteHolster,
            GameEvent::KillRefillsMeter(_) => GameEventType::KillRefillsMeter,
            GameEvent::RpsTauntEvent(_) => GameEventType::RpsTauntEvent,
            GameEvent::CongaKill(_) => GameEventType::CongaKill,
            GameEvent::PlayerInitialSpawn(_) => GameEventType::PlayerInitialSpawn,
            GameEvent::CompetitiveVictory(_) => GameEventType::CompetitiveVictory,
            GameEvent::CompetitiveStatsUpdate(_) => GameEventType::CompetitiveStatsUpdate,
            GameEvent::MiniGameWin(_) => GameEventType::MiniGameWin,
            GameEvent::SentryOnGoActive(_) => GameEventType::SentryOnGoActive,
            GameEvent::DuckXpLevelUp(_) => GameEventType::DuckXpLevelUp,
            GameEvent::QuestLogOpened(_) => GameEventType::QuestLogOpened,
            GameEvent::SchemaUpdated(_) => GameEventType::SchemaUpdated,
            GameEvent::LocalPlayerPickupWeapon(_) => GameEventType::LocalPlayerPickupWeapon,
            GameEvent::RdPlayerScorePoints(_) => GameEventType::RdPlayerScorePoints,
            GameEvent::DemomanDetStickies(_) => GameEventType::DemomanDetStickies,
            GameEvent::QuestObjectiveCompleted(_) => GameEventType::QuestObjectiveCompleted,
            GameEvent::PlayerScoreChanged(_) => GameEventType::PlayerScoreChanged,
            GameEvent::KilledCappingPlayer(_) => GameEventType::KilledCappingPlayer,
            GameEvent::EnvironmentalDeath(_) => GameEventType::EnvironmentalDeath,
            GameEvent::ProjectileDirectHit(_) => GameEventType::ProjectileDirectHit,
            GameEvent::PassGet(_) => GameEventType::PassGet,
            GameEvent::PassScore(_) => GameEventType::PassScore,
            GameEvent::PassFree(_) => GameEventType::PassFree,
            GameEvent::PassPassCaught(_) => GameEventType::PassPassCaught,
            GameEvent::PassBallStolen(_) => GameEventType::PassBallStolen,
            GameEvent::PassBallBlocked(_) => GameEventType::PassBallBlocked,
            GameEvent::DamagePrevented(_) => GameEventType::DamagePrevented,
            GameEvent::HalloweenBossKilled(_) => GameEventType::HalloweenBossKilled,
            GameEvent::EscapedLootIsland(_) => GameEventType::EscapedLootIsland,
            GameEvent::TaggedPlayerAsIt(_) => GameEventType::TaggedPlayerAsIt,
            GameEvent::MerasmusStunned(_) => GameEventType::MerasmusStunned,
            GameEvent::MerasmusPropFound(_) => GameEventType::MerasmusPropFound,
            GameEvent::HalloweenSkeletonKilled(_) => GameEventType::HalloweenSkeletonKilled,
            GameEvent::SkeletonKilledQuest(_) => GameEventType::SkeletonKilledQuest,
            GameEvent::SkeletonKingKilledQuest(_) => GameEventType::SkeletonKingKilledQuest,
            GameEvent::EscapeHell(_) => GameEventType::EscapeHell,
            GameEvent::CrossSpectralBridge(_) => GameEventType::CrossSpectralBridge,
            GameEvent::MiniGameWon(_) => GameEventType::MiniGameWon,
            GameEvent::RespawnGhost(_) => GameEventType::RespawnGhost,
            GameEvent::KillInHell(_) => GameEventType::KillInHell,
            GameEvent::HalloweenDuckCollected(_) => GameEventType::HalloweenDuckCollected,
            GameEvent::SpecialScore(_) => GameEventType::SpecialScore,
            GameEvent::TeamLeaderKilled(_) => GameEventType::TeamLeaderKilled,
            GameEvent::HalloweenSoulCollected(_) => GameEventType::HalloweenSoulCollected,
            GameEvent::RecalculateTruce(_) => GameEventType::RecalculateTruce,
            GameEvent::DeadRingerCheatDeath(_) => GameEventType::DeadRingerCheatDeath,
            GameEvent::CrossbowHeal(_) => GameEventType::CrossbowHeal,
            GameEvent::DamageMitigated(_) => GameEventType::DamageMitigated,
            GameEvent::PayloadPushed(_) => GameEventType::PayloadPushed,
            GameEvent::PlayerAbandonedMatch(_) => GameEventType::PlayerAbandonedMatch,
            GameEvent::ClDrawline(_) => GameEventType::ClDrawline,
            GameEvent::RestartTimerTime(_) => GameEventType::RestartTimerTime,
            GameEvent::WinLimitChanged(_) => GameEventType::WinLimitChanged,
            GameEvent::WinPanelShowScores(_) => GameEventType::WinPanelShowScores,
            GameEvent::TopStreamsRequestFinished(_) => GameEventType::TopStreamsRequestFinished,
            GameEvent::CompetitiveStateChanged(_) => GameEventType::CompetitiveStateChanged,
            GameEvent::GlobalWarDataUpdated(_) => GameEventType::GlobalWarDataUpdated,
            GameEvent::StopWatchChanged(_) => GameEventType::StopWatchChanged,
            GameEvent::DsStop(_) => GameEventType::DsStop,
            GameEvent::DsScreenshot(_) => GameEventType::DsScreenshot,
            GameEvent::ShowMatchSummary(_) => GameEventType::ShowMatchSummary,
            GameEvent::ExperienceChanged(_) => GameEventType::ExperienceChanged,
            GameEvent::BeginXpLerp(_) => GameEventType::BeginXpLerp,
            GameEvent::MatchmakerStatsUpdated(_) => GameEventType::MatchmakerStatsUpdated,
            GameEvent::RematchVotePeriodOver(_) => GameEventType::RematchVotePeriodOver,
            GameEvent::RematchFailedToCreate(_) => GameEventType::RematchFailedToCreate,
            GameEvent::PlayerRematchChange(_) => GameEventType::PlayerRematchChange,
            GameEvent::PingUpdated(_) => GameEventType::PingUpdated,
            GameEvent::MMStatsUpdated(_) => GameEventType::MMStatsUpdated,
            GameEvent::PlayerNextMapVoteChange(_) => GameEventType::PlayerNextMapVoteChange,
            GameEvent::VoteMapsChanged(_) => GameEventType::VoteMapsChanged,
            GameEvent::ProtoDefChanged(_) => GameEventType::ProtoDefChanged,
            GameEvent::PlayerDomination(_) => GameEventType::PlayerDomination,
            GameEvent::PlayerRocketPackPushed(_) => GameEventType::PlayerRocketPackPushed,
            GameEvent::QuestRequest(_) => GameEventType::QuestRequest,
            GameEvent::QuestResponse(_) => GameEventType::QuestResponse,
            GameEvent::QuestProgress(_) => GameEventType::QuestProgress,
            GameEvent::ProjectileRemoved(_) => GameEventType::ProjectileRemoved,
            GameEvent::QuestMapDataChanged(_) => GameEventType::QuestMapDataChanged,
            GameEvent::GasDousedPlayerIgnited(_) => GameEventType::GasDousedPlayerIgnited,
            GameEvent::QuestTurnInState(_) => GameEventType::QuestTurnInState,
            GameEvent::ItemsAcknowledged(_) => GameEventType::ItemsAcknowledged,
            GameEvent::CapperKilled(_) => GameEventType::CapperKilled,
            GameEvent::MainMenuStabilized(_) => GameEventType::MainMenuStabilized,
            GameEvent::WorldStatusChanged(_) => GameEventType::WorldStatusChanged,
            GameEvent::HLTVStatus(_) => GameEventType::HLTVStatus,
            GameEvent::HLTVCameraman(_) => GameEventType::HLTVCameraman,
            GameEvent::HLTVRankCamera(_) => GameEventType::HLTVRankCamera,
            GameEvent::HLTVRankEntity(_) => GameEventType::HLTVRankEntity,
            GameEvent::HLTVFixed(_) => GameEventType::HLTVFixed,
            GameEvent::HLTVChase(_) => GameEventType::HLTVChase,
            GameEvent::HLTVMessage(_) => GameEventType::HLTVMessage,
            GameEvent::HLTVTitle(_) => GameEventType::HLTVTitle,
            GameEvent::HLTVChat(_) => GameEventType::HLTVChat,
            GameEvent::ReplayStartRecord(_) => GameEventType::ReplayStartRecord,
            GameEvent::ReplaySessionInfo(_) => GameEventType::ReplaySessionInfo,
            GameEvent::ReplayEndRecord(_) => GameEventType::ReplayEndRecord,
            GameEvent::ReplayReplaysAvailable(_) => GameEventType::ReplayReplaysAvailable,
            GameEvent::ReplayServerError(_) => GameEventType::ReplayServerError,
            GameEvent::Unknown(raw) => raw.event_type.clone(),
        }
    }
}
pub fn get_sizes() -> fnv::FnvHashMap<&'static str, usize> {
    [
        ("ServerSpawn", size_of::<ServerSpawnEvent>()),
        (
            "ServerChangeLevelFailed",
            size_of::<ServerChangeLevelFailedEvent>(),
        ),
        ("ServerShutdown", size_of::<ServerShutdownEvent>()),
        ("ServerCvar", size_of::<ServerCvarEvent>()),
        ("ServerMessage", size_of::<ServerMessageEvent>()),
        ("ServerAddBan", size_of::<ServerAddBanEvent>()),
        ("ServerRemoveBan", size_of::<ServerRemoveBanEvent>()),
        ("PlayerConnect", size_of::<PlayerConnectEvent>()),
        ("PlayerConnectClient", size_of::<PlayerConnectClientEvent>()),
        ("PlayerInfo", size_of::<PlayerInfoEvent>()),
        ("PlayerDisconnect", size_of::<PlayerDisconnectEvent>()),
        ("PlayerActivate", size_of::<PlayerActivateEvent>()),
        ("PlayerSay", size_of::<PlayerSayEvent>()),
        ("ClientDisconnect", size_of::<ClientDisconnectEvent>()),
        ("ClientBeginConnect", size_of::<ClientBeginConnectEvent>()),
        ("ClientConnected", size_of::<ClientConnectedEvent>()),
        ("ClientFullConnect", size_of::<ClientFullConnectEvent>()),
        ("HostQuit", size_of::<HostQuitEvent>()),
        ("TeamInfo", size_of::<TeamInfoEvent>()),
        ("TeamScore", size_of::<TeamScoreEvent>()),
        (
            "TeamPlayBroadcastAudio",
            size_of::<TeamPlayBroadcastAudioEvent>(),
        ),
        ("PlayerTeam", size_of::<PlayerTeamEvent>()),
        ("PlayerClass", size_of::<PlayerClassEvent>()),
        ("PlayerDeath", size_of::<PlayerDeathEvent>()),
        ("PlayerHurt", size_of::<PlayerHurtEvent>()),
        ("PlayerChat", size_of::<PlayerChatEvent>()),
        ("PlayerScore", size_of::<PlayerScoreEvent>()),
        ("PlayerSpawn", size_of::<PlayerSpawnEvent>()),
        ("PlayerShoot", size_of::<PlayerShootEvent>()),
        ("PlayerUse", size_of::<PlayerUseEvent>()),
        ("PlayerChangeName", size_of::<PlayerChangeNameEvent>()),
        ("PlayerHintMessage", size_of::<PlayerHintMessageEvent>()),
        (
            "BasePlayerTeleported",
            size_of::<BasePlayerTeleportedEvent>(),
        ),
        ("GameInit", size_of::<GameInitEvent>()),
        ("GameNewMap", size_of::<GameNewMapEvent>()),
        ("GameStart", size_of::<GameStartEvent>()),
        ("GameEnd", size_of::<GameEndEvent>()),
        ("RoundStart", size_of::<RoundStartEvent>()),
        ("RoundEnd", size_of::<RoundEndEvent>()),
        ("GameMessage", size_of::<GameMessageEvent>()),
        ("BreakBreakable", size_of::<BreakBreakableEvent>()),
        ("BreakProp", size_of::<BreakPropEvent>()),
        ("EntityKilled", size_of::<EntityKilledEvent>()),
        ("BonusUpdated", size_of::<BonusUpdatedEvent>()),
        ("AchievementEvent", size_of::<AchievementEventEvent>()),
        (
            "AchievementIncrement",
            size_of::<AchievementIncrementEvent>(),
        ),
        ("PhysgunPickup", size_of::<PhysgunPickupEvent>()),
        ("FlareIgniteNpc", size_of::<FlareIgniteNpcEvent>()),
        (
            "HelicopterGrenadePuntMiss",
            size_of::<HelicopterGrenadePuntMissEvent>(),
        ),
        ("UserDataDownloaded", size_of::<UserDataDownloadedEvent>()),
        ("RagdollDissolved", size_of::<RagdollDissolvedEvent>()),
        ("HLTVChangedMode", size_of::<HLTVChangedModeEvent>()),
        ("HLTVChangedTarget", size_of::<HLTVChangedTargetEvent>()),
        ("VoteEnded", size_of::<VoteEndedEvent>()),
        ("VoteStarted", size_of::<VoteStartedEvent>()),
        ("VoteChanged", size_of::<VoteChangedEvent>()),
        ("VotePassed", size_of::<VotePassedEvent>()),
        ("VoteFailed", size_of::<VoteFailedEvent>()),
        ("VoteCast", size_of::<VoteCastEvent>()),
        ("VoteOptions", size_of::<VoteOptionsEvent>()),
        ("ReplaySaved", size_of::<ReplaySavedEvent>()),
        (
            "EnteredPerformanceMode",
            size_of::<EnteredPerformanceModeEvent>(),
        ),
        ("BrowseReplays", size_of::<BrowseReplaysEvent>()),
        ("ReplayYoutubeStats", size_of::<ReplayYoutubeStatsEvent>()),
        ("InventoryUpdated", size_of::<InventoryUpdatedEvent>()),
        ("CartUpdated", size_of::<CartUpdatedEvent>()),
        (
            "StorePriceSheetUpdated",
            size_of::<StorePriceSheetUpdatedEvent>(),
        ),
        (
            "EconInventoryConnected",
            size_of::<EconInventoryConnectedEvent>(),
        ),
        (
            "ItemSchemaInitialized",
            size_of::<ItemSchemaInitializedEvent>(),
        ),
        ("GcNewSession", size_of::<GcNewSessionEvent>()),
        ("GcLostSession", size_of::<GcLostSessionEvent>()),
        ("IntroFinish", size_of::<IntroFinishEvent>()),
        ("IntroNextCamera", size_of::<IntroNextCameraEvent>()),
        ("PlayerChangeClass", size_of::<PlayerChangeClassEvent>()),
        ("TfMapTimeRemaining", size_of::<TfMapTimeRemainingEvent>()),
        ("TfGameOver", size_of::<TfGameOverEvent>()),
        ("CtfFlagCaptured", size_of::<CtfFlagCapturedEvent>()),
        (
            "ControlPointInitialized",
            size_of::<ControlPointInitializedEvent>(),
        ),
        (
            "ControlPointUpdateImages",
            size_of::<ControlPointUpdateImagesEvent>(),
        ),
        (
            "ControlPointUpdateLayout",
            size_of::<ControlPointUpdateLayoutEvent>(),
        ),
        (
            "ControlPointUpdateCapping",
            size_of::<ControlPointUpdateCappingEvent>(),
        ),
        (
            "ControlPointUpdateOwner",
            size_of::<ControlPointUpdateOwnerEvent>(),
        ),
        (
            "ControlPointStartTouch",
            size_of::<ControlPointStartTouchEvent>(),
        ),
        (
            "ControlPointEndTouch",
            size_of::<ControlPointEndTouchEvent>(),
        ),
        (
            "ControlPointPulseElement",
            size_of::<ControlPointPulseElementEvent>(),
        ),
        (
            "ControlPointFakeCapture",
            size_of::<ControlPointFakeCaptureEvent>(),
        ),
        (
            "ControlPointFakeCaptureMultiplier",
            size_of::<ControlPointFakeCaptureMultiplierEvent>(),
        ),
        (
            "TeamPlayRoundSelected",
            size_of::<TeamPlayRoundSelectedEvent>(),
        ),
        ("TeamPlayRoundStart", size_of::<TeamPlayRoundStartEvent>()),
        ("TeamPlayRoundActive", size_of::<TeamPlayRoundActiveEvent>()),
        (
            "TeamPlayWaitingBegins",
            size_of::<TeamPlayWaitingBeginsEvent>(),
        ),
        ("TeamPlayWaitingEnds", size_of::<TeamPlayWaitingEndsEvent>()),
        (
            "TeamPlayWaitingAboutToEnd",
            size_of::<TeamPlayWaitingAboutToEndEvent>(),
        ),
        (
            "TeamPlayRestartRound",
            size_of::<TeamPlayRestartRoundEvent>(),
        ),
        (
            "TeamPlayReadyRestart",
            size_of::<TeamPlayReadyRestartEvent>(),
        ),
        (
            "TeamPlayRoundRestartSeconds",
            size_of::<TeamPlayRoundRestartSecondsEvent>(),
        ),
        ("TeamPlayTeamReady", size_of::<TeamPlayTeamReadyEvent>()),
        ("TeamPlayRoundWin", size_of::<TeamPlayRoundWinEvent>()),
        ("TeamPlayUpdateTimer", size_of::<TeamPlayUpdateTimerEvent>()),
        (
            "TeamPlayRoundStalemate",
            size_of::<TeamPlayRoundStalemateEvent>(),
        ),
        (
            "TeamPlayOvertimeBegin",
            size_of::<TeamPlayOvertimeBeginEvent>(),
        ),
        ("TeamPlayOvertimeEnd", size_of::<TeamPlayOvertimeEndEvent>()),
        (
            "TeamPlaySuddenDeathBegin",
            size_of::<TeamPlaySuddenDeathBeginEvent>(),
        ),
        (
            "TeamPlaySuddenDeathEnd",
            size_of::<TeamPlaySuddenDeathEndEvent>(),
        ),
        ("TeamPlayGameOver", size_of::<TeamPlayGameOverEvent>()),
        (
            "TeamPlayMapTimeRemaining",
            size_of::<TeamPlayMapTimeRemainingEvent>(),
        ),
        ("TeamPlayTimerFlash", size_of::<TeamPlayTimerFlashEvent>()),
        (
            "TeamPlayTimerTimeAdded",
            size_of::<TeamPlayTimerTimeAddedEvent>(),
        ),
        (
            "TeamPlayPointStartCapture",
            size_of::<TeamPlayPointStartCaptureEvent>(),
        ),
        (
            "TeamPlayPointCaptured",
            size_of::<TeamPlayPointCapturedEvent>(),
        ),
        ("TeamPlayPointLocked", size_of::<TeamPlayPointLockedEvent>()),
        (
            "TeamPlayPointUnlocked",
            size_of::<TeamPlayPointUnlockedEvent>(),
        ),
        (
            "TeamPlayCaptureBroken",
            size_of::<TeamPlayCaptureBrokenEvent>(),
        ),
        (
            "TeamPlayCaptureBlocked",
            size_of::<TeamPlayCaptureBlockedEvent>(),
        ),
        ("TeamPlayFlagEvent", size_of::<TeamPlayFlagEventEvent>()),
        ("TeamPlayWinPanel", size_of::<TeamPlayWinPanelEvent>()),
        (
            "TeamPlayTeamBalancedPlayer",
            size_of::<TeamPlayTeamBalancedPlayerEvent>(),
        ),
        (
            "TeamPlaySetupFinished",
            size_of::<TeamPlaySetupFinishedEvent>(),
        ),
        ("TeamPlayAlert", size_of::<TeamPlayAlertEvent>()),
        ("TrainingComplete", size_of::<TrainingCompleteEvent>()),
        ("ShowFreezePanel", size_of::<ShowFreezePanelEvent>()),
        ("HideFreezePanel", size_of::<HideFreezePanelEvent>()),
        ("FreezeCamStarted", size_of::<FreezeCamStartedEvent>()),
        (
            "LocalPlayerChangeTeam",
            size_of::<LocalPlayerChangeTeamEvent>(),
        ),
        (
            "LocalPlayerScoreChanged",
            size_of::<LocalPlayerScoreChangedEvent>(),
        ),
        (
            "LocalPlayerChangeClass",
            size_of::<LocalPlayerChangeClassEvent>(),
        ),
        ("LocalPlayerRespawn", size_of::<LocalPlayerRespawnEvent>()),
        ("BuildingInfoChanged", size_of::<BuildingInfoChangedEvent>()),
        (
            "LocalPlayerChangeDisguise",
            size_of::<LocalPlayerChangeDisguiseEvent>(),
        ),
        (
            "PlayerAccountChanged",
            size_of::<PlayerAccountChangedEvent>(),
        ),
        ("SpyPdaReset", size_of::<SpyPdaResetEvent>()),
        ("FlagStatusUpdate", size_of::<FlagStatusUpdateEvent>()),
        ("PlayerStatsUpdated", size_of::<PlayerStatsUpdatedEvent>()),
        ("PlayingCommentary", size_of::<PlayingCommentaryEvent>()),
        (
            "PlayerChargeDeployed",
            size_of::<PlayerChargeDeployedEvent>(),
        ),
        ("PlayerBuiltObject", size_of::<PlayerBuiltObjectEvent>()),
        (
            "PlayerUpgradedObject",
            size_of::<PlayerUpgradedObjectEvent>(),
        ),
        ("PlayerCarryObject", size_of::<PlayerCarryObjectEvent>()),
        ("PlayerDropObject", size_of::<PlayerDropObjectEvent>()),
        ("ObjectRemoved", size_of::<ObjectRemovedEvent>()),
        ("ObjectDestroyed", size_of::<ObjectDestroyedEvent>()),
        ("ObjectDetonated", size_of::<ObjectDetonatedEvent>()),
        ("AchievementEarned", size_of::<AchievementEarnedEvent>()),
        ("SpecTargetUpdated", size_of::<SpecTargetUpdatedEvent>()),
        (
            "TournamentStateUpdate",
            size_of::<TournamentStateUpdateEvent>(),
        ),
        (
            "TournamentEnableCountdown",
            size_of::<TournamentEnableCountdownEvent>(),
        ),
        (
            "PlayerCalledForMedic",
            size_of::<PlayerCalledForMedicEvent>(),
        ),
        ("PlayerAskedForBall", size_of::<PlayerAskedForBallEvent>()),
        (
            "LocalPlayerBecameObserver",
            size_of::<LocalPlayerBecameObserverEvent>(),
        ),
        ("PlayerIgnitedInv", size_of::<PlayerIgnitedInvEvent>()),
        ("PlayerIgnited", size_of::<PlayerIgnitedEvent>()),
        ("PlayerExtinguished", size_of::<PlayerExtinguishedEvent>()),
        ("PlayerTeleported", size_of::<PlayerTeleportedEvent>()),
        (
            "PlayerHealedMedicCall",
            size_of::<PlayerHealedMedicCallEvent>(),
        ),
        (
            "LocalPlayerChargeReady",
            size_of::<LocalPlayerChargeReadyEvent>(),
        ),
        ("LocalPlayerWindDown", size_of::<LocalPlayerWindDownEvent>()),
        ("PlayerInvulned", size_of::<PlayerInvulnedEvent>()),
        ("EscortSpeed", size_of::<EscortSpeedEvent>()),
        ("EscortProgress", size_of::<EscortProgressEvent>()),
        ("EscortRecede", size_of::<EscortRecedeEvent>()),
        ("GameUIActivated", size_of::<GameUIActivatedEvent>()),
        ("GameUIHidden", size_of::<GameUIHiddenEvent>()),
        ("PlayerEscortScore", size_of::<PlayerEscortScoreEvent>()),
        ("PlayerHealOnHit", size_of::<PlayerHealOnHitEvent>()),
        ("PlayerStealSandvich", size_of::<PlayerStealSandvichEvent>()),
        ("ShowClassLayout", size_of::<ShowClassLayoutEvent>()),
        ("ShowVsPanel", size_of::<ShowVsPanelEvent>()),
        ("PlayerDamaged", size_of::<PlayerDamagedEvent>()),
        (
            "ArenaPlayerNotification",
            size_of::<ArenaPlayerNotificationEvent>(),
        ),
        ("ArenaMatchMaxStreak", size_of::<ArenaMatchMaxStreakEvent>()),
        ("ArenaRoundStart", size_of::<ArenaRoundStartEvent>()),
        ("ArenaWinPanel", size_of::<ArenaWinPanelEvent>()),
        ("PveWinPanel", size_of::<PveWinPanelEvent>()),
        ("AirDash", size_of::<AirDashEvent>()),
        ("Landed", size_of::<LandedEvent>()),
        ("PlayerDamageDodged", size_of::<PlayerDamageDodgedEvent>()),
        ("PlayerStunned", size_of::<PlayerStunnedEvent>()),
        ("ScoutGrandSlam", size_of::<ScoutGrandSlamEvent>()),
        ("ScoutSlamdollLanded", size_of::<ScoutSlamdollLandedEvent>()),
        ("ArrowImpact", size_of::<ArrowImpactEvent>()),
        ("PlayerJarated", size_of::<PlayerJaratedEvent>()),
        ("PlayerJaratedFade", size_of::<PlayerJaratedFadeEvent>()),
        ("PlayerShieldBlocked", size_of::<PlayerShieldBlockedEvent>()),
        ("PlayerPinned", size_of::<PlayerPinnedEvent>()),
        ("PlayerHealedByMedic", size_of::<PlayerHealedByMedicEvent>()),
        ("PlayerSappedObject", size_of::<PlayerSappedObjectEvent>()),
        ("ItemFound", size_of::<ItemFoundEvent>()),
        ("ShowAnnotation", size_of::<ShowAnnotationEvent>()),
        ("HideAnnotation", size_of::<HideAnnotationEvent>()),
        (
            "PostInventoryApplication",
            size_of::<PostInventoryApplicationEvent>(),
        ),
        (
            "ControlPointUnlockUpdated",
            size_of::<ControlPointUnlockUpdatedEvent>(),
        ),
        ("DeployBuffBanner", size_of::<DeployBuffBannerEvent>()),
        ("PlayerBuff", size_of::<PlayerBuffEvent>()),
        ("MedicDeath", size_of::<MedicDeathEvent>()),
        ("OvertimeNag", size_of::<OvertimeNagEvent>()),
        ("TeamsChanged", size_of::<TeamsChangedEvent>()),
        (
            "HalloweenPumpkinGrab",
            size_of::<HalloweenPumpkinGrabEvent>(),
        ),
        ("RocketJump", size_of::<RocketJumpEvent>()),
        ("RocketJumpLanded", size_of::<RocketJumpLandedEvent>()),
        ("StickyJump", size_of::<StickyJumpEvent>()),
        ("StickyJumpLanded", size_of::<StickyJumpLandedEvent>()),
        ("RocketPackLaunch", size_of::<RocketPackLaunchEvent>()),
        ("RocketPackLanded", size_of::<RocketPackLandedEvent>()),
        ("MedicDefended", size_of::<MedicDefendedEvent>()),
        ("LocalPlayerHealed", size_of::<LocalPlayerHealedEvent>()),
        (
            "PlayerDestroyedPipeBomb",
            size_of::<PlayerDestroyedPipeBombEvent>(),
        ),
        ("ObjectDeflected", size_of::<ObjectDeflectedEvent>()),
        ("PlayerMvp", size_of::<PlayerMvpEvent>()),
        ("RaidSpawnMob", size_of::<RaidSpawnMobEvent>()),
        ("RaidSpawnSquad", size_of::<RaidSpawnSquadEvent>()),
        ("NavBlocked", size_of::<NavBlockedEvent>()),
        ("PathTrackPassed", size_of::<PathTrackPassedEvent>()),
        ("NumCappersChanged", size_of::<NumCappersChangedEvent>()),
        ("PlayerRegenerate", size_of::<PlayerRegenerateEvent>()),
        ("UpdateStatusItem", size_of::<UpdateStatusItemEvent>()),
        ("StatsResetRound", size_of::<StatsResetRoundEvent>()),
        (
            "ScoreStatsAccumulatedUpdate",
            size_of::<ScoreStatsAccumulatedUpdateEvent>(),
        ),
        (
            "ScoreStatsAccumulatedReset",
            size_of::<ScoreStatsAccumulatedResetEvent>(),
        ),
        (
            "AchievementEarnedLocal",
            size_of::<AchievementEarnedLocalEvent>(),
        ),
        ("PlayerHealed", size_of::<PlayerHealedEvent>()),
        ("BuildingHealed", size_of::<BuildingHealedEvent>()),
        ("ItemPickup", size_of::<ItemPickupEvent>()),
        ("DuelStatus", size_of::<DuelStatusEvent>()),
        ("FishNotice", size_of::<FishNoticeEvent>()),
        ("FishNoticeArm", size_of::<FishNoticeArmEvent>()),
        ("SlapNotice", size_of::<SlapNoticeEvent>()),
        ("ThrowableHit", size_of::<ThrowableHitEvent>()),
        ("PumpkinLordSummoned", size_of::<PumpkinLordSummonedEvent>()),
        ("PumpkinLordKilled", size_of::<PumpkinLordKilledEvent>()),
        ("MerasmusSummoned", size_of::<MerasmusSummonedEvent>()),
        ("MerasmusKilled", size_of::<MerasmusKilledEvent>()),
        (
            "MerasmusEscapeWarning",
            size_of::<MerasmusEscapeWarningEvent>(),
        ),
        ("MerasmusEscaped", size_of::<MerasmusEscapedEvent>()),
        ("EyeballBossSummoned", size_of::<EyeballBossSummonedEvent>()),
        ("EyeballBossStunned", size_of::<EyeballBossStunnedEvent>()),
        ("EyeballBossKilled", size_of::<EyeballBossKilledEvent>()),
        ("EyeballBossKiller", size_of::<EyeballBossKillerEvent>()),
        (
            "EyeballBossEscapeImminent",
            size_of::<EyeballBossEscapeImminentEvent>(),
        ),
        ("EyeballBossEscaped", size_of::<EyeballBossEscapedEvent>()),
        ("NpcHurt", size_of::<NpcHurtEvent>()),
        (
            "ControlPointTimerUpdated",
            size_of::<ControlPointTimerUpdatedEvent>(),
        ),
        ("PlayerHighFiveStart", size_of::<PlayerHighFiveStartEvent>()),
        (
            "PlayerHighFiveCancel",
            size_of::<PlayerHighFiveCancelEvent>(),
        ),
        (
            "PlayerHighFiveSuccess",
            size_of::<PlayerHighFiveSuccessEvent>(),
        ),
        ("PlayerBonusPoints", size_of::<PlayerBonusPointsEvent>()),
        ("PlayerUpgraded", size_of::<PlayerUpgradedEvent>()),
        ("PlayerBuyback", size_of::<PlayerBuybackEvent>()),
        (
            "PlayerUsedPowerUpBottle",
            size_of::<PlayerUsedPowerUpBottleEvent>(),
        ),
        ("ChristmasGiftGrab", size_of::<ChristmasGiftGrabEvent>()),
        (
            "PlayerKilledAchievementZone",
            size_of::<PlayerKilledAchievementZoneEvent>(),
        ),
        ("PartyUpdated", size_of::<PartyUpdatedEvent>()),
        ("PartyPrefChanged", size_of::<PartyPrefChangedEvent>()),
        (
            "PartyCriteriaChanged",
            size_of::<PartyCriteriaChangedEvent>(),
        ),
        ("PartyInvitesChanged", size_of::<PartyInvitesChangedEvent>()),
        (
            "PartyQueueStateChanged",
            size_of::<PartyQueueStateChangedEvent>(),
        ),
        ("PartyChat", size_of::<PartyChatEvent>()),
        ("PartyMemberJoin", size_of::<PartyMemberJoinEvent>()),
        ("PartyMemberLeave", size_of::<PartyMemberLeaveEvent>()),
        ("MatchInvitesUpdated", size_of::<MatchInvitesUpdatedEvent>()),
        ("LobbyUpdated", size_of::<LobbyUpdatedEvent>()),
        ("MvmMissionUpdate", size_of::<MvmMissionUpdateEvent>()),
        ("RecalculateHolidays", size_of::<RecalculateHolidaysEvent>()),
        (
            "PlayerCurrencyChanged",
            size_of::<PlayerCurrencyChangedEvent>(),
        ),
        ("DoomsdayRocketOpen", size_of::<DoomsdayRocketOpenEvent>()),
        (
            "RemoveNemesisRelationships",
            size_of::<RemoveNemesisRelationshipsEvent>(),
        ),
        ("MvmCreditBonusWave", size_of::<MvmCreditBonusWaveEvent>()),
        ("MvmCreditBonusAll", size_of::<MvmCreditBonusAllEvent>()),
        (
            "MvmCreditBonusAllAdvanced",
            size_of::<MvmCreditBonusAllAdvancedEvent>(),
        ),
        (
            "MvmQuickSentryUpgrade",
            size_of::<MvmQuickSentryUpgradeEvent>(),
        ),
        (
            "MvmTankDestroyedByPlayers",
            size_of::<MvmTankDestroyedByPlayersEvent>(),
        ),
        (
            "MvmKillRobotDeliveringBomb",
            size_of::<MvmKillRobotDeliveringBombEvent>(),
        ),
        ("MvmPickupCurrency", size_of::<MvmPickupCurrencyEvent>()),
        (
            "MvmBombCarrierKilled",
            size_of::<MvmBombCarrierKilledEvent>(),
        ),
        (
            "MvmSentryBusterDetonate",
            size_of::<MvmSentryBusterDetonateEvent>(),
        ),
        (
            "MvmScoutMarkedForDeath",
            size_of::<MvmScoutMarkedForDeathEvent>(),
        ),
        (
            "MvmMedicPowerUpShared",
            size_of::<MvmMedicPowerUpSharedEvent>(),
        ),
        ("MvmBeginWave", size_of::<MvmBeginWaveEvent>()),
        ("MvmWaveComplete", size_of::<MvmWaveCompleteEvent>()),
        ("MvmMissionComplete", size_of::<MvmMissionCompleteEvent>()),
        (
            "MvmBombResetByPlayer",
            size_of::<MvmBombResetByPlayerEvent>(),
        ),
        (
            "MvmBombAlarmTriggered",
            size_of::<MvmBombAlarmTriggeredEvent>(),
        ),
        (
            "MvmBombDeployResetByPlayer",
            size_of::<MvmBombDeployResetByPlayerEvent>(),
        ),
        ("MvmWaveFailed", size_of::<MvmWaveFailedEvent>()),
        ("MvmResetStats", size_of::<MvmResetStatsEvent>()),
        ("DamageResisted", size_of::<DamageResistedEvent>()),
        ("RevivePlayerNotify", size_of::<RevivePlayerNotifyEvent>()),
        ("RevivePlayerStopped", size_of::<RevivePlayerStoppedEvent>()),
        (
            "RevivePlayerComplete",
            size_of::<RevivePlayerCompleteEvent>(),
        ),
        ("PlayerTurnedToGhost", size_of::<PlayerTurnedToGhostEvent>()),
        (
            "MedigunShieldBlockedDamage",
            size_of::<MedigunShieldBlockedDamageEvent>(),
        ),
        (
            "MvmAdvWaveCompleteNoGates",
            size_of::<MvmAdvWaveCompleteNoGatesEvent>(),
        ),
        (
            "MvmSniperHeadshotCurrency",
            size_of::<MvmSniperHeadshotCurrencyEvent>(),
        ),
        ("MvmMannhattanPit", size_of::<MvmMannhattanPitEvent>()),
        (
            "FlagCarriedInDetectionZone",
            size_of::<FlagCarriedInDetectionZoneEvent>(),
        ),
        (
            "MvmAdvWaveKilledStunRadio",
            size_of::<MvmAdvWaveKilledStunRadioEvent>(),
        ),
        ("PlayerDirectHitStun", size_of::<PlayerDirectHitStunEvent>()),
        (
            "MvmSentryBusterKilled",
            size_of::<MvmSentryBusterKilledEvent>(),
        ),
        ("UpgradesFileChanged", size_of::<UpgradesFileChangedEvent>()),
        ("RdTeamPointsChanged", size_of::<RdTeamPointsChangedEvent>()),
        ("RdRulesStateChanged", size_of::<RdRulesStateChangedEvent>()),
        ("RdRobotKilled", size_of::<RdRobotKilledEvent>()),
        ("RdRobotImpact", size_of::<RdRobotImpactEvent>()),
        (
            "TeamPlayPreRoundTimeLeft",
            size_of::<TeamPlayPreRoundTimeLeftEvent>(),
        ),
        ("ParachuteDeploy", size_of::<ParachuteDeployEvent>()),
        ("ParachuteHolster", size_of::<ParachuteHolsterEvent>()),
        ("KillRefillsMeter", size_of::<KillRefillsMeterEvent>()),
        ("RpsTauntEvent", size_of::<RpsTauntEventEvent>()),
        ("CongaKill", size_of::<CongaKillEvent>()),
        ("PlayerInitialSpawn", size_of::<PlayerInitialSpawnEvent>()),
        ("CompetitiveVictory", size_of::<CompetitiveVictoryEvent>()),
        (
            "CompetitiveStatsUpdate",
            size_of::<CompetitiveStatsUpdateEvent>(),
        ),
        ("MiniGameWin", size_of::<MiniGameWinEvent>()),
        ("SentryOnGoActive", size_of::<SentryOnGoActiveEvent>()),
        ("DuckXpLevelUp", size_of::<DuckXpLevelUpEvent>()),
        ("QuestLogOpened", size_of::<QuestLogOpenedEvent>()),
        ("SchemaUpdated", size_of::<SchemaUpdatedEvent>()),
        (
            "LocalPlayerPickupWeapon",
            size_of::<LocalPlayerPickupWeaponEvent>(),
        ),
        ("RdPlayerScorePoints", size_of::<RdPlayerScorePointsEvent>()),
        ("DemomanDetStickies", size_of::<DemomanDetStickiesEvent>()),
        (
            "QuestObjectiveCompleted",
            size_of::<QuestObjectiveCompletedEvent>(),
        ),
        ("PlayerScoreChanged", size_of::<PlayerScoreChangedEvent>()),
        ("KilledCappingPlayer", size_of::<KilledCappingPlayerEvent>()),
        ("EnvironmentalDeath", size_of::<EnvironmentalDeathEvent>()),
        ("ProjectileDirectHit", size_of::<ProjectileDirectHitEvent>()),
        ("PassGet", size_of::<PassGetEvent>()),
        ("PassScore", size_of::<PassScoreEvent>()),
        ("PassFree", size_of::<PassFreeEvent>()),
        ("PassPassCaught", size_of::<PassPassCaughtEvent>()),
        ("PassBallStolen", size_of::<PassBallStolenEvent>()),
        ("PassBallBlocked", size_of::<PassBallBlockedEvent>()),
        ("DamagePrevented", size_of::<DamagePreventedEvent>()),
        ("HalloweenBossKilled", size_of::<HalloweenBossKilledEvent>()),
        ("EscapedLootIsland", size_of::<EscapedLootIslandEvent>()),
        ("TaggedPlayerAsIt", size_of::<TaggedPlayerAsItEvent>()),
        ("MerasmusStunned", size_of::<MerasmusStunnedEvent>()),
        ("MerasmusPropFound", size_of::<MerasmusPropFoundEvent>()),
        (
            "HalloweenSkeletonKilled",
            size_of::<HalloweenSkeletonKilledEvent>(),
        ),
        ("SkeletonKilledQuest", size_of::<SkeletonKilledQuestEvent>()),
        (
            "SkeletonKingKilledQuest",
            size_of::<SkeletonKingKilledQuestEvent>(),
        ),
        ("EscapeHell", size_of::<EscapeHellEvent>()),
        ("CrossSpectralBridge", size_of::<CrossSpectralBridgeEvent>()),
        ("MiniGameWon", size_of::<MiniGameWonEvent>()),
        ("RespawnGhost", size_of::<RespawnGhostEvent>()),
        ("KillInHell", size_of::<KillInHellEvent>()),
        (
            "HalloweenDuckCollected",
            size_of::<HalloweenDuckCollectedEvent>(),
        ),
        ("SpecialScore", size_of::<SpecialScoreEvent>()),
        ("TeamLeaderKilled", size_of::<TeamLeaderKilledEvent>()),
        (
            "HalloweenSoulCollected",
            size_of::<HalloweenSoulCollectedEvent>(),
        ),
        ("RecalculateTruce", size_of::<RecalculateTruceEvent>()),
        (
            "DeadRingerCheatDeath",
            size_of::<DeadRingerCheatDeathEvent>(),
        ),
        ("CrossbowHeal", size_of::<CrossbowHealEvent>()),
        ("DamageMitigated", size_of::<DamageMitigatedEvent>()),
        ("PayloadPushed", size_of::<PayloadPushedEvent>()),
        (
            "PlayerAbandonedMatch",
            size_of::<PlayerAbandonedMatchEvent>(),
        ),
        ("ClDrawline", size_of::<ClDrawlineEvent>()),
        ("RestartTimerTime", size_of::<RestartTimerTimeEvent>()),
        ("WinLimitChanged", size_of::<WinLimitChangedEvent>()),
        ("WinPanelShowScores", size_of::<WinPanelShowScoresEvent>()),
        (
            "TopStreamsRequestFinished",
            size_of::<TopStreamsRequestFinishedEvent>(),
        ),
        (
            "CompetitiveStateChanged",
            size_of::<CompetitiveStateChangedEvent>(),
        ),
        (
            "GlobalWarDataUpdated",
            size_of::<GlobalWarDataUpdatedEvent>(),
        ),
        ("StopWatchChanged", size_of::<StopWatchChangedEvent>()),
        ("DsStop", size_of::<DsStopEvent>()),
        ("DsScreenshot", size_of::<DsScreenshotEvent>()),
        ("ShowMatchSummary", size_of::<ShowMatchSummaryEvent>()),
        ("ExperienceChanged", size_of::<ExperienceChangedEvent>()),
        ("BeginXpLerp", size_of::<BeginXpLerpEvent>()),
        (
            "MatchmakerStatsUpdated",
            size_of::<MatchmakerStatsUpdatedEvent>(),
        ),
        (
            "RematchVotePeriodOver",
            size_of::<RematchVotePeriodOverEvent>(),
        ),
        (
            "RematchFailedToCreate",
            size_of::<RematchFailedToCreateEvent>(),
        ),
        ("PlayerRematchChange", size_of::<PlayerRematchChangeEvent>()),
        ("PingUpdated", size_of::<PingUpdatedEvent>()),
        ("MMStatsUpdated", size_of::<MMStatsUpdatedEvent>()),
        (
            "PlayerNextMapVoteChange",
            size_of::<PlayerNextMapVoteChangeEvent>(),
        ),
        ("VoteMapsChanged", size_of::<VoteMapsChangedEvent>()),
        ("ProtoDefChanged", size_of::<ProtoDefChangedEvent>()),
        ("PlayerDomination", size_of::<PlayerDominationEvent>()),
        (
            "PlayerRocketPackPushed",
            size_of::<PlayerRocketPackPushedEvent>(),
        ),
        ("QuestRequest", size_of::<QuestRequestEvent>()),
        ("QuestResponse", size_of::<QuestResponseEvent>()),
        ("QuestProgress", size_of::<QuestProgressEvent>()),
        ("ProjectileRemoved", size_of::<ProjectileRemovedEvent>()),
        ("QuestMapDataChanged", size_of::<QuestMapDataChangedEvent>()),
        (
            "GasDousedPlayerIgnited",
            size_of::<GasDousedPlayerIgnitedEvent>(),
        ),
        ("QuestTurnInState", size_of::<QuestTurnInStateEvent>()),
        ("ItemsAcknowledged", size_of::<ItemsAcknowledgedEvent>()),
        ("CapperKilled", size_of::<CapperKilledEvent>()),
        ("MainMenuStabilized", size_of::<MainMenuStabilizedEvent>()),
        ("WorldStatusChanged", size_of::<WorldStatusChangedEvent>()),
        ("HLTVStatus", size_of::<HLTVStatusEvent>()),
        ("HLTVCameraman", size_of::<HLTVCameramanEvent>()),
        ("HLTVRankCamera", size_of::<HLTVRankCameraEvent>()),
        ("HLTVRankEntity", size_of::<HLTVRankEntityEvent>()),
        ("HLTVFixed", size_of::<HLTVFixedEvent>()),
        ("HLTVChase", size_of::<HLTVChaseEvent>()),
        ("HLTVMessage", size_of::<HLTVMessageEvent>()),
        ("HLTVTitle", size_of::<HLTVTitleEvent>()),
        ("HLTVChat", size_of::<HLTVChatEvent>()),
        ("ReplayStartRecord", size_of::<ReplayStartRecordEvent>()),
        ("ReplaySessionInfo", size_of::<ReplaySessionInfoEvent>()),
        ("ReplayEndRecord", size_of::<ReplayEndRecordEvent>()),
        (
            "ReplayReplaysAvailable",
            size_of::<ReplayReplaysAvailableEvent>(),
        ),
        ("ReplayServerError", size_of::<ReplayServerErrorEvent>()),
    ]
    .iter()
    .copied()
    .collect()
}
