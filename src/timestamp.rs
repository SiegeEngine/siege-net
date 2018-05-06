use std::time::Instant;
use std::ops::{Deref, Sub, Add};
use std::fmt;

lazy_static! {
    static ref START_INSTANT: Instant = Instant::now();
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Timestamp(u32);

impl Timestamp {
    pub fn now() -> Timestamp
    {
        let stamp = START_INSTANT.elapsed();
        Timestamp((stamp.as_secs() * 1000) as u32 + (stamp.subsec_nanos() / 1000000))
    }

    pub fn from_raw(raw: u32) -> Timestamp {
        Timestamp(raw)
    }
}

impl Deref for Timestamp {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Sub for Timestamp {
    type Output = i32;

    fn sub(self, other: Timestamp) -> i32 {
        ((self.0 as i64) - (other.0 as i64)) as i32
    }
}

impl Add<i32> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: i32) -> Timestamp {
        Timestamp((self.0 as i64 + rhs as i64) as u32)
    }
}

#[test]
fn test() {
    let t1 = Timestamp::now();
    ::std::thread::sleep(::std::time::Duration::from_secs(1));
    let t2 = Timestamp::now();

    let offset = t2 - t1;
    assert!(offset > 0);
    assert_eq!(t1 + offset, t2)
}
