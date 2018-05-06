
use std::fmt;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[repr(C)]
pub struct Flags(u8);

static FLAGS_FIRST: u8 = 0x01;
static FLAGS_LAST: u8 = 0x02;
static FLAGS_MULTIPLE: u8 = 0x04;
static FLAGS_IN_ORDER: u8 = 0x08;
static FLAGS_ACK: u8 = 0x10;

impl Flags {

    pub fn new() -> Flags {
        let mut out = Flags(0);
        out.set(FLAGS_FIRST);
        out.set(FLAGS_LAST);
        out
    }

    #[inline]
    fn set(&mut self, flag: u8) {
        self.0 |= flag
    }

    #[inline]
    fn unset(&mut self, flag: u8) {
        self.0 &= !flag
    }

    #[inline]
    fn isset(&self, flag: u8) -> bool
    {
        self.0 & flag != 0
    }

    /// Turn on FIRST flag (packet is first in a series)
    pub fn set_first(mut self) -> Flags {
        self.set(FLAGS_FIRST); self
    }

    /// Turn off FIRST flag (packet is first in a series)
    pub fn unset_first(mut self) -> Flags {
        self.unset(FLAGS_FIRST); self
    }

    /// Is the FIRST flag set?
    pub fn is_first(&self) -> bool {
        self.isset(FLAGS_FIRST)
    }

    /// Turn on LAST flag (packet is last in a series)
    pub fn set_last(mut self) -> Flags {
        self.set(FLAGS_LAST); self
    }

    /// Turn off LAST flag (packet is last in a series)
    pub fn unset_last(mut self) -> Flags {
        self.unset(FLAGS_LAST); self
    }

    /// Is the LAST flag set?
    pub fn is_last(&self) -> bool {
        self.isset(FLAGS_LAST)
    }

    /// Turn on MULTIPLE flag
    pub fn set_multiple(mut self) -> Flags {
        self.set(FLAGS_MULTIPLE); self
    }

    /// Turn off MULTIPLE flag
    pub fn unset_multiple(mut self) -> Flags {
        self.unset(FLAGS_MULTIPLE); self
    }

    /// Is the MULTIPLE flag set?
    pub fn is_multiple(&self) -> bool {
        self.isset(FLAGS_MULTIPLE)
    }

    /// Turn on IN_ORDER flag
    pub fn set_in_order(mut self) -> Flags {
        self.set(FLAGS_IN_ORDER); self
    }

    /// Turn off IN_ORDER flag
    pub fn unset_in_order(mut self) -> Flags {
        self.unset(FLAGS_IN_ORDER); self
    }

    /// Is the IN_ORDER flag set?
    pub fn is_in_order(&self) -> bool {
        self.isset(FLAGS_IN_ORDER)
    }

    /// Turn on ACK flag
    pub fn set_ack(mut self) -> Flags {
        self.set(FLAGS_ACK); self
    }

    /// Turn off ACK flag
    pub fn unset_ack(mut self) -> Flags {
        self.unset(FLAGS_ACK); self
    }

    /// Is the ACK flag set?
    pub fn is_ack(&self) -> bool {
        self.isset(FLAGS_ACK)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}{}{}{}{}",
               if self.is_first() { "F" } else { "_" },
               if self.is_last() { "L" } else { "_" },
               if self.is_multiple() { "M" } else { "_" },
               if self.is_in_order() { "O" } else { "_" },
               if self.is_ack() { "A" } else { "_" },
               )
    }
}

#[test]
fn test_sup_flags_1() {
    assert!(Flags::new() == Flags::new().set_first());
    assert!(Flags::new() == Flags::new().set_last());
    assert!(Flags::new().unset_first().is_last());
    assert!(Flags::new().unset_last().is_first());
}
