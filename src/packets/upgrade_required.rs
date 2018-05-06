
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct UpgradeRequiredPacket {
    pub version: u32,
}

impl UpgradeRequiredPacket {
    pub fn new(version: u32) -> UpgradeRequiredPacket
    {
        UpgradeRequiredPacket {
            version: version
        }
    }
}
