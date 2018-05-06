
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct ShutdownCompletePacket;

impl ShutdownCompletePacket {
    pub fn new() -> ShutdownCompletePacket
    {
        ShutdownCompletePacket
    }
}
