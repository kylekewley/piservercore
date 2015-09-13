/**
 * This defines the class that will be used to encode/decode general messages
 */
use rustc_serialize::json;

use std::collections::{HashMap};

#[derive(RustcDecodable, RustcEncodable)]
pub struct Message {
    ack: bool,      // Determines whether this message requires the reciever to acknowledge
    m_id: u32,      // Message ID. Just a unique integer
    p_id: u32,      // ID of the parser that will parse this message
    message: HashMap<String, String>, // The actual message payload
}

impl Message {

    pub fn get_parser_id(&self) -> u32 {
        self.p_id
    }

}
