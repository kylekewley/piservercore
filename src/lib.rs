#![allow(unused_features)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(path)]
#![feature(old_io)]
#![feature(collections)]
extern crate capnp;

pub mod ack_capnp;
pub mod error_capnp;
pub mod message_capnp;
pub mod messenger;
