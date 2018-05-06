
use errors::*;
use std::sync::Arc;
use std::net::SocketAddr;
use ring::rand::{SystemRandom, SecureRandom};
use ring::agreement::{EphemeralPrivateKey, X25519, agree_ephemeral};
use ring::signature::ED25519;
use ring::error::Unspecified;
use untrusted::Input;
use timestamp::Timestamp;
use packets::{Packet, Header};

/// Information about the remote entity you are communicating with
pub struct Remote {
    /// Random number generator
    pub rng: Arc<SystemRandom>,

    /// The remote's IP address and port
    pub addr: SocketAddr,

    /// The next sequence number we will use when sending packets to the remote
    pub next_local_seq_number: u32,

    /// The last sequence number we received in packets sent from the remote
    pub last_remote_seq_number: u32,

    /// An ephemeral private key used for establishing a session key.  Only used
    /// for the first packet exchange, and then set to None.
    pub eph_private_key: Option<EphemeralPrivateKey>,

    /// The shared session key used for AEAD of packet contents.  Starts as zeroes
    /// until a better session key has been established via key exchange.
    pub session_key: [u8; 16],

    /// A nonce used to help verify the remote is authentic.  Only used by the Client
    /// and only used during the first packet exchange.
    pub nonce: [u8; 12],

    /// The time when we sent the last three messages that we expect to get replies
    /// from, along with their sequence numbers.  This allows us to coorelate reply
    /// timestamps and do proper clock synchronization.
    pub sent_pings: [(u32, Timestamp); 3],

    /// Index into sent_pings circular array, where we write to next.
    pub sent_ping_write_index: usize,

    /// The minimum possible offset of the remote's clock as compared to our clock.
    pub offset_min: Option<i32>,

    /// The maximum possible offset of the remote's clock as compared to our clock.
    pub offset_max: Option<i32>,
}

impl Remote {
    pub fn new(addr: SocketAddr, rng: Arc<SystemRandom>) -> Result<Remote>
    {
        let mut nonce: [u8; 12] = [0; 12]; // 96 bit nonce
        rng.fill(&mut nonce[..12]).unwrap();

        let eph_private_key = EphemeralPrivateKey::generate(&X25519, &*rng)?;

        Ok(Remote {
            rng: rng,
            addr: addr,
            next_local_seq_number: 1,
            last_remote_seq_number: 0,
            eph_private_key: Some(eph_private_key),
            session_key: [0; 16],
            nonce: nonce,
            sent_pings: [(0, Timestamp::now()); 3],
            sent_ping_write_index: 0,
            offset_min: None,
            offset_max: None,
        })
    }

    pub fn serialize_packet(&mut self,
                            packet: &Packet,
                            magic: u32,
                            version: u32) -> Result<Vec<u8>>
    {
        self._serialize_packet(packet, magic, version, None)
    }

    pub fn serialize_reply_packet(&mut self,
                                  packet: &Packet,
                                  magic: u32,
                                  version: u32,
                                  in_reply_to: u32)
                                  -> Result<Vec<u8>>
    {
        self._serialize_packet(packet, magic, version, Some(in_reply_to))
    }

    fn _serialize_packet(
        &mut self,
        packet: &Packet,
        magic: u32,
        version: u32,
        in_reply_to: Option<u32>)
        -> Result<Vec<u8>>
    {
        use std::io::Cursor;
        use ring::aead::{AES_128_GCM, SealingKey, seal_in_place};
        use bincode::{serialized_size, serialize_into};

        // Build the header
        let seq = self.next_seq_number();
        let now = Timestamp::now();
        match packet {
            // Save timestamp for the following kinds of outbound packets
            &Packet::Init(_) | &Packet::Heartbeat(_) => {
                self.sent_pings[self.sent_ping_write_index] = (seq, now);
                self.sent_ping_write_index = (self.sent_ping_write_index + 1) % 3;
            },
            _ => {},
        }

        let header = Header::new(now, seq, in_reply_to, 1500);

        // Prepare serialization area
        const MAGIC_AND_VERSION_SIZE: usize = 4;
        const NONCE_SIZE: usize = 12;
        const SUFFIX_SIZE: usize = 16; //  AES_128_GCM.max_overhead_len() is 16;
        let fullsize =
            MAGIC_AND_VERSION_SIZE +
            NONCE_SIZE +
            serialized_size(&header)? as usize +
            serialized_size(packet)? as usize +
            SUFFIX_SIZE;
        let bytes: Vec<u8> = Vec::with_capacity(fullsize);

        // Serialize in the magic and version
        let mut bytes: Vec<u8> = {
            let mut cursor = Cursor::new(bytes);
            let magic_and_version: u32 = magic | version;
            serialize_into(&mut cursor, &magic_and_version)?;
            cursor.into_inner()
        };

        // Write in a random nonce
        bytes.extend([0; 12].into_iter());
        self.rng.fill(&mut bytes[4..4+12]).unwrap();

        // Serialize in the header
        let bytes: Vec<u8> = {
            let len = bytes.len();
            let mut cursor = Cursor::new(bytes);
            cursor.set_position(len as u64);
            serialize_into(&mut cursor, &header)?;
            cursor.into_inner()
        };

        // Serialize in the packet
        let mut bytes: Vec<u8> = {
            let len = bytes.len();
            let mut cursor = Cursor::new(bytes);
            cursor.set_position(len as u64);
            serialize_into(&mut cursor, packet)?;
            cursor.into_inner()
        };

        // Encrypt/Sign
        bytes.extend([0; SUFFIX_SIZE].into_iter());
        // mav = magic and version.  This is the "associated data"
        let mavbytes = &bytes[0..4].to_vec(); // copy to appease borrow checker ;-(
        let nonce = &bytes[4..4+12].to_vec(); // copy to appease borrow checker :-(
        let sealing_key = try!(SealingKey::new(&AES_128_GCM, &self.session_key));
        let size = try!(seal_in_place(&sealing_key, &*nonce, &*mavbytes,
                                      &mut bytes[16..], SUFFIX_SIZE));
        bytes.truncate(16+size);

        Ok(bytes)
    }

    // Returns the packet along with the sequence number from the header (for in-reply-to),
    // and whether or not the packet is stale (out of order or potentially a duplicate).
    pub fn deserialize_packet(
        &mut self,
        bytes: &mut [u8])
        -> Result<(Packet, u32, bool)>
    {
        use ring::aead::{AES_128_GCM, OpeningKey, open_in_place};
        use bincode::{deserialize, serialized_size};

        // Decrypt
        let mavbytes = &bytes[0..4].to_vec(); // copy to appease borrow checker ;-(
        let nonce = &bytes[4..4+12].to_vec(); // copy to appease borrow checker ;-(
        let opening_key = try!(OpeningKey::new(&AES_128_GCM, &self.session_key));
        let slice = open_in_place(&opening_key, &*nonce, &*mavbytes, 0, &mut bytes[16..])?;

        // Deserialize the header
        let header: Header = deserialize(slice)?;

        // Deserialize the packet body
        let offset = serialized_size(&header)? as usize;
        let packet: Packet = deserialize(&slice[offset..])?;

        let mut stale: bool = false;

        // Process the header
        {
            // Bump last seq number, if greater
            if header.sequence_number > self.last_remote_seq_number {
                self.last_remote_seq_number = header.sequence_number;
            } else {
                // Packet is either out-of-order or is a duplicate
                stale = true;
            }

            // Clock synchronization
            if header.in_reply_to != 0 {
                // Get our saved ping time (if not overwritten already)
                for i in 0..3 {
                    let (seq, stamp) = self.sent_pings[i];
                    if seq == header.in_reply_to {
                        self.synchronize(
                            stamp,
                            Timestamp::from_raw(header.timestamp),
                            Timestamp::now());
                        break;
                    }
                }
            }
        }

        // Return the packet
        Ok((packet, header.sequence_number, stale))
    }

    pub fn next_seq_number(&mut self) -> u32
    {
        let output = self.next_local_seq_number;
        self.next_local_seq_number += 1;
        output
    }

    pub fn roll_nonce(&mut self) {
        self.rng.fill(&mut self.nonce[..12]).unwrap();
    }

    pub fn compute_session_key(&mut self, remote_public_key: &[u8; 32])
                               -> Result<()>
    {
        let eph = match self.eph_private_key.take() {
            Some(eph) => eph,
            None => return Err("Ephemeral private key was already used.".into()),
        };
        self.session_key = agree_ephemeral(
            eph, &X25519, Input::from(remote_public_key),
            ErrorKind::Crypto(Unspecified).into(),
            key_derivation_function)?;

        Ok(())
    }

    fn synchronize(&mut self, sent: Timestamp, bounce: Timestamp, recv: Timestamp)
    {
        // `sent` and `received` are in OUR timeframe.
        // `bounce` is in THEIR timeframe.

        // Because of the order of these events, we know:
        //      sent - 1 <= (bounce + offset) <= recv + 1
        // (the 1 millisecond adjustments are because all of these numbers are rounded
        //  to the nearest millisecond)
        // We don't know the offset, but we can bound it with a little algebra:
        //      offset >= sent - 1 - bounce
        //      offset <= recv + 1 - bounce
        // Therefore:
        let offset_min = (sent - bounce) - 1;
        let offset_max = (recv - bounce) + 1;

        if self.offset_min.is_none() {
            // First time, eh?
            self.offset_min = Some(offset_min);
            self.offset_max = Some(offset_max);
            self.print_sync_debug();
            return;
        }

        let smin = self.offset_min.unwrap();
        let smax = self.offset_max.unwrap();

        // Check for clock drift (where the new window does not overlap the existing
        // one)
        if offset_min > smax {
            let old = (smax + smin) / 2;
            self.offset_min = Some(offset_min);
            self.offset_max = Some(offset_max);
            let new = (offset_min + offset_max) / 2;
            warn!("Clock drift: offset shifted by +{}", new - old);
            self.print_sync_debug();
            return;
        }
        if offset_max < smin {
            let old = (smax + smin) / 2;
            self.offset_min = Some(offset_min);
            self.offset_max = Some(offset_max);
            let new = (offset_min + offset_max) / 2;
            warn!("Clock drift: offset shifted by {}", new - old);
            self.print_sync_debug();
            return;
        }

        // Use these offset bounds if they are better than what we already have
        if offset_min > smin {
            self.offset_min = Some(offset_min);
            self.print_sync_debug();
        }
        if offset_max < smax {
            self.offset_max = Some(offset_max);
            self.print_sync_debug();
        }
    }

    #[inline]
    fn print_sync_debug(&self) {
        debug!("CLOCK-SYNC: within {}ms [offset is between {}..{}]",
               self.offset_max.unwrap() - self.offset_min.unwrap(),
               self.offset_min.unwrap(), self.offset_max.unwrap());
    }

    // Get the remote's current time, as best as we can determine
    pub fn now(&self) -> Result<Timestamp>
    {
        let min = self.offset_min.ok_or(Error::from_kind(ErrorKind::NotSynchronized))?;
        let max = self.offset_max.ok_or(Error::from_kind(ErrorKind::NotSynchronized))?;
        if max < min { return Err(ErrorKind::NotSynchronized.into()); }
        let offset: i32 = (max + min)/2; // use the middle of the window
        Ok(Timestamp::now() + offset)
    }

    pub fn clock_window_size(&self) -> Option<i32>
    {
        match self.offset_max {
            Some(max) => match self.offset_min {
                Some(min) => Some(max - min),
                None => None,
            },
            None => None
        }
    }

    pub fn validate_nonce_signature(&self, signature: &[u8], server_public_key: &[u8])
                                    -> Result<()>
    {
        ::ring::signature::verify(&ED25519,
                                  Input::from(server_public_key),
                                  Input::from(&self.nonce),
                                  Input::from(signature))
            .map_err(|_| ErrorKind::RemoteFailedChallenge.into())
    }
}

fn key_derivation_function(input: &[u8]) -> Result<[u8; 16]>
{
    let mut output: [u8; 16] = [0; 16];
    output.copy_from_slice(&input[0..16]);
    Ok(output)
}
