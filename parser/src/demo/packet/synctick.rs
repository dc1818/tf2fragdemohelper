use crate::demo::data::DemoTick;
use bitbuffer::BitRead;
#[cfg(feature = "write")]
use bitbuffer::BitWrite;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(BitRead, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "write", derive(BitWrite))]
pub struct SyncTickPacket {
    pub tick: DemoTick,
}
