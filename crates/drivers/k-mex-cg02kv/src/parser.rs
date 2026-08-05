use crate::protocol::Packet;

pub struct Parser;

impl Parser {
    pub fn parse(_data: &[u8]) -> Option<Packet> {
        None
    }
}