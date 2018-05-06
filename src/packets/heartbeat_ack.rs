
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct HeartbeatAckPacket;

impl HeartbeatAckPacket {
    pub fn new() -> HeartbeatAckPacket
    {
        HeartbeatAckPacket
    }
}
