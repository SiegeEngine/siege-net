
use errors::*;
use remote::Remote;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct InitAckPacket {
    pub public_key: [u8; 32],
    nonce_response_1: [u8; 32],
    nonce_response_2: [u8; 32],
}
impl InitAckPacket {
    pub fn new(remote: &Remote, nonce_response: &[u8])
               -> Result<InitAckPacket>
    {
        if remote.eph_private_key.is_none() {
            return Err("Ephemeral private key already used.".into());
        }

        let mut nonce_response_1: [u8; 32] = [0; 32];
        nonce_response_1.copy_from_slice(&nonce_response[0..32]);

        let mut nonce_response_2: [u8; 32] = [0; 32];
        nonce_response_2.copy_from_slice(&nonce_response[32..64]);

        let mut public_key = [0_u8; 32];
        try!(remote.eph_private_key.as_ref().unwrap().compute_public_key(&mut public_key));

        Ok(InitAckPacket {
            public_key: public_key,
            nonce_response_1: nonce_response_1,
            nonce_response_2: nonce_response_2,
        })
    }

    pub fn get_nonce_response(&self) -> [u8; 64]
    {
        let mut signature: [u8; 64] = [0; 64];
        signature[0..32].copy_from_slice(&self.nonce_response_1[..]);
        signature[32..64].copy_from_slice(&self.nonce_response_2[..]);
        signature
    }
}

#[test]
fn test() {
    use std::str::FromStr;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use ring::rand::SystemRandom;
    use remote::Remote;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    #[repr(u8)]
    enum Packet {
        InitAck(InitAckPacket),
    }

    let remote_addr: SocketAddr = FromStr::from_str("0.0.0.0:0").unwrap();
    let mut remote = Remote::new(remote_addr, Arc::new(SystemRandom::new())).unwrap();

    let init_ack_packet = InitAckPacket::new(&remote, &[99; 64]).unwrap();
    let packet = Packet::InitAck(init_ack_packet.clone());
    let mut bytes: Vec<u8> = remote.serialize_reply_packet(&packet, 177).unwrap();
    let (packet2,_,stale) = remote.deserialize_packet(&mut bytes[..]).unwrap();
    assert_eq!(stale,false);
    match packet2 {
        Packet::InitAck(init_ack_packet2) => {
            assert_eq!(init_ack_packet2, init_ack_packet);
        },
    }
}
