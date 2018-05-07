//! Siege-net protocol
//!
//! This module implements a Client and a Server for the Siege-net protocol, a
//! messaging protocol build on top of UDP designed for low latency gaming.

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;
extern crate ring;
extern crate untrusted;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;

mod errors;
mod timestamp;
pub mod packets;
mod remote;

pub use errors::*;
pub use timestamp::Timestamp;
pub use remote::Remote;
