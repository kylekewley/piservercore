#![allow(unused_features)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(path)]
#![feature(old_io)]
#![feature(collections)]
extern crate capnp;

mod Ack_capnp {
    include!("./schema/Ack_capnp.rs");
}
mod Error_capnp {
    include!("./schema/Error_capnp.rs");
}
mod Message_capnp {
    include!("./schema/Message_capnp.rs");
}
mod Messenger {
    include!("./messenger.rs");
}
