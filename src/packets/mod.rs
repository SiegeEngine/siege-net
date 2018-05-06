
pub mod header;
pub use self::header::Header;

pub mod flags;
pub use self::flags::Flags;

mod init;
pub use self::init::InitPacket;
mod init_ack;
pub use self::init_ack::InitAckPacket;
mod heartbeat;
pub use self::heartbeat::HeartbeatPacket;
mod heartbeat_ack;
pub use self::heartbeat_ack::HeartbeatAckPacket;
mod shutdown;
pub use self::shutdown::ShutdownPacket;
mod shutdown_complete;
pub use self::shutdown_complete::ShutdownCompletePacket;
mod upgrade_required;
pub use self::upgrade_required::UpgradeRequiredPacket;

// The maximum size of a packet, according to the protocol.
pub const MAX_PROTO_PACKET: usize = 1500;

use errors::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum Packet {
    Init(InitPacket),
    InitAck(InitAckPacket),
    UpgradeRequired(UpgradeRequiredPacket),
    Heartbeat(HeartbeatPacket),
    HeartbeatAck(HeartbeatAckPacket),
    Shutdown(ShutdownPacket),
    ShutdownComplete(ShutdownCompletePacket),
}
impl Packet {
    pub fn name(&self) -> &'static str
    {
        match *self {
            Packet::Init(_) => "Init",
            Packet::InitAck(_) => "InitAck",
            Packet::UpgradeRequired(_) => "UpgradeRequired",
            Packet::Heartbeat(_) => "Heartbeat",
            Packet::HeartbeatAck(_) => "HeartbeatAck",
            Packet::Shutdown(_) => "Shutdown",
            Packet::ShutdownComplete(_) => "ShutdownComplete",
        }
    }
}

// Returns Ok(true) if correct version, Ok(false) if wrong version, Err(_) if
// not a Siege packet.
pub fn validate_magic_and_version(
    correct_magic: u32, // MAGIC is 20 bits
    current_version: u32, // VERSION is 12 bits
    bytes: &[u8]) -> Result<bool>
{
    use bincode::deserialize;
    assert!(correct_magic <= 0xFFFFF); // only 20 bits of space for MAGIC
    assert!(current_version <= 0xFFF); // only 12 bits of space for VERSION

    if bytes.len() < 4 { return Err(ErrorKind::InvalidPacket.into()); }
    let magic_and_version: u32 = try!(deserialize(&bytes[0..4]));
    let magic = magic_and_version & 0xFFFFF000;
    let version =  magic_and_version & 0xFFF;
    if magic != correct_magic {
        debug!("Packet has wrong magic number");
        Err(ErrorKind::InvalidPacket.into())
    }
    else if version != current_version {
        debug!("Packet has wrong version");
        Ok(false)
    } else {
        Ok(true)
    }
}


#[test]
fn test() {
    use std::str::FromStr;
    use std::sync::Arc;
    use std::net::SocketAddr;
    use ring::rand::SystemRandom;
    use remote::Remote;

    let remote_addr: SocketAddr = FromStr::from_str("127.0.0.1:4444").unwrap();
    let mut remote = Remote::new(remote_addr, Arc::new(SystemRandom::new())).unwrap();

    let init_ack_packet = InitAckPacket::new(&remote, &[0; 64]).unwrap();
    let packet = Packet::InitAck(init_ack_packet.clone());
    let mut bytes: Vec<u8> = remote.serialize_packet(&packet).unwrap();
    let (packet2,_,stale) = remote.deserialize_packet(&mut bytes[..]).unwrap();
    assert_eq!(stale,false);
    match packet2 {
        Packet::InitAck(init_ack_packet2) => {
            assert_eq!(init_ack_packet, init_ack_packet2);
        },
        _ => panic!("Ser/De failed for InitAckPacket"),
    }
}

#[test]
fn test_validate_magic_and_version() {
    use bincode::Infinite;

    let mav = 0xFF00 | 0x18;
    let bytes: Vec<u8> = ::bincode::serialize(&mav, Infinite).unwrap();
    match validate_magic_and_version(&*bytes, 0xFF00, 0x18) {
        Ok(true) => {},
        _ => panic!("validate_magic_and_version() failed on valid input"),
    }

    let mav = 0xFF00 | 254_u32;
    let bytes: Vec<u8> = ::bincode::serialize(&mav, Infinite).unwrap();
    match validate_magic_and_version(&*bytes, 0xFF00, 0x18) {
        Ok(false) => {},
        _ => panic!("validate_magic_and_version() yielded wrong result on bad version"),
    }

    let mav = 0;
    let bytes: Vec<u8> = ::bincode::serialize(&mav, Infinite).unwrap();
    match validate_magic_and_version(&*bytes, 0xFF00, 0x18) {
        Err(_) => {},
        _ => panic!("validate_magic_and_version() did not fail on bad packet"),
    }
}
