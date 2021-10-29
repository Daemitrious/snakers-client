use crate::{Area, Player, TcpStream};

pub struct Client {
    pub stream: TcpStream,
    pub area: Area,
    pub player: Player,
}
