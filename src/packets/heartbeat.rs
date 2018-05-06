
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct HeartbeatPacket;

impl HeartbeatPacket {
    pub fn new() -> HeartbeatPacket
    {
        HeartbeatPacket
    }
}
