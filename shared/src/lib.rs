#![no_std]

pub trait Actor: Send + Sync + core::fmt::Debug {
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CoolActor {
    value: u32,
}

impl Actor for CoolActor {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NewParams {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NewReturn {}

impl CoolActor {
    pub fn new(_params: NewParams) -> CoolActor {
        let actor = CoolActor {
            value: 0,
        };

        actor
    }
}


#[inline]
pub fn from_i64(val: i64) -> u64 {
    u64::from_le_bytes(val.to_le_bytes())
}

#[inline]
pub fn from_u64(val: u64) -> i64 {
    i64::from_le_bytes(val.to_le_bytes())
}

#[inline]
pub fn from_u32(val: u32) -> i32 {
    i32::from_le_bytes(val.to_le_bytes())
}

#[inline]
pub fn from_i32(val: i32) -> u32 {
    u32::from_le_bytes(val.to_le_bytes())
}
