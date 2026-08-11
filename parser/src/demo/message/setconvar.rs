use bitbuffer::{BitRead, BitReadStream, Endianness};
#[cfg(feature = "write")]
use bitbuffer::{BitWrite, BitWriteStream};
use serde::{Deserialize, Serialize};

use crate::ReadResult;

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "write", derive(BitWrite))]
pub struct ConVar {
    pub key: String,
    pub value: String,
}

impl<E: Endianness> BitRead<'_, E> for ConVar {
    fn read(stream: &mut BitReadStream<'_, E>) -> ReadResult<Self> {
        let key = stream
            .read()
            .unwrap_or_else(|_| "Malformed cvar name".to_string());
        let value = stream
            .read()
            .unwrap_or_else(|_| "Malformed cvar value".to_string());
        Ok(ConVar { key, value })
    }
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, BitRead, PartialEq, Serialize, Deserialize, Clone)]
pub struct SetConVarMessage {
    pub length: u8,
    #[size = "length"]
    pub vars: Vec<ConVar>,
}

#[cfg(feature = "write")]
impl<E: Endianness> BitWrite<E> for SetConVarMessage {
    fn write(&self, stream: &mut BitWriteStream<E>) -> ReadResult<()> {
        self.length.write(stream)?;
        self.vars.write(stream)
    }
}
#[test]
#[cfg(feature = "write")]
fn test_set_con_var_roundtrip() {
    crate::test_roundtrip_write(SetConVarMessage {
        length: 0,
        vars: Vec::new(),
    });
    crate::test_roundtrip_write(SetConVarMessage {
        length: 2,
        vars: vec![
            ConVar {
                key: "foo1".to_string(),
                value: "bar1".to_string(),
            },
            ConVar {
                key: "foo2".to_string(),
                value: "bar2".to_string(),
            },
        ],
    });
}
