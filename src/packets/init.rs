
use errors::*;
use remote::Remote;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct InitPacket {
    pub public_key: [u8; 32],
    pub nonce: [u8; 12],
}

impl InitPacket {
    pub fn new(remote: &mut Remote) -> Result<InitPacket>
    {
        if remote.eph_private_key.is_none() {
            return Err("Ephemeral private key already used.".into());
        }
        let mut public_key = [0_u8; 32];
        try!(remote.eph_private_key.as_ref().unwrap().compute_public_key(&mut public_key));

        remote.roll_nonce();

        Ok(InitPacket {
            public_key: public_key,
            nonce: remote.nonce,
        })
    }
}

#[test]
fn test() {
    use std::str::FromStr;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use remote::Remote;
    use ring::rand::SystemRandom;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    #[repr(u8)]
    enum Packet {
        Init(InitPacket),
    }

    let remote_addr: SocketAddr = FromStr::from_str("0.0.0.0:0").unwrap();
    let mut remote = Remote::new(remote_addr, Arc::new(SystemRandom::new())).unwrap();

    let init_packet = InitPacket::new(&mut remote).unwrap();
    let packet = Packet::Init(init_packet.clone());
    let mut bytes: Vec<u8> = remote.serialize_packet(&packet).unwrap();
    let (packet2,_,stale) = remote.deserialize_packet(&mut bytes[..]).unwrap();
    assert_eq!(stale,false);
    match packet2 {
        Packet::Init(init_packet2) => {
            assert_eq!(init_packet2, init_packet);
        }
    }
}
