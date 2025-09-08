const MAGIC: [u8; 6] = *b"i3-ipc";
const MAGIC_LEN: usize = 6;

const EVENT_FLAG: u32 = 0x80000000;

pub mod codec;
pub mod command;
pub mod connection;
pub mod packet;
pub mod reply;
