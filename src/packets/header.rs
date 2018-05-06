
use super::MAX_PROTO_PACKET;
use packets::flags::Flags;
use timestamp::Timestamp;

// Header format for every siege-net packet
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct Header {
    pub timestamp: u32,
    pub sequence_number: u32,
    pub in_reply_to: u32,
    pub recv_window_size: u16,
    pub flags: Flags,
    reserved: u8,
}
impl Header {
    pub fn new(timestamp: Timestamp,
               sequence_number: u32,
               in_reply_to: Option<u32>,
               recv_window_size: u16)
               -> Header
    {
        Header {
            timestamp: *timestamp,
            sequence_number: sequence_number,
            in_reply_to: match in_reply_to {
                Some(u) => u,
                None => 0
            },
            recv_window_size: recv_window_size,
            flags: Flags::new(),
            reserved: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.recv_window_size <= MAX_PROTO_PACKET as u16
            && self.reserved == 0
    }
}

#[test]
fn test_header() {
    use std::str::FromStr;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use ring::rand::SystemRandom;
    use remote::Remote;

    let addr: SocketAddr = FromStr::from_str("127.0.0.1:12345").unwrap();

    let mut remote = Remote::new(addr, Arc::new(SystemRandom::new())).unwrap();

    let header = Header::new(Timestamp::now(), remote.next_seq_number(), Some(1), 1500);

    assert!(header.is_valid());
    assert_eq!(header.sequence_number, 1);
    assert_eq!(header.flags, Flags::new());
    assert_eq!(header.recv_window_size, 1500);
    assert_eq!(header.in_reply_to, 1);
}
