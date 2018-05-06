
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct ShutdownPacket;

impl ShutdownPacket {
    pub fn new() -> ShutdownPacket
    {
        ShutdownPacket
    }
}
