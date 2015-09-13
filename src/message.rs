/**
 * This defines the class that will be used to encode/decode general messages
 */
use rustc_serialize::{json, Encodable, Decodable};
use std::fmt;

#[derive(RustcDecodable, RustcEncodable, Debug, Eq, PartialEq)]
pub struct Message {
    ack: bool,      // Determines whether this message requires the reciever to acknowledge
    m_id: u32,      // Message ID. Just a unique integer
    p_id: u32,      // ID of the parser that will parse this message
    message: String, // The actual message payload
}

impl Message {
    pub fn get_parser_id(&self) -> u32 {
        self.p_id
    }
}

mod tests {
    use super::*;
    use std::collections::{HashMap};
    use rustc_serialize::json;
    use core_messages::Ping;


    #[test]
    fn test_encode() {

        let payload = Ping::new();
        let payload_str = json::encode(&payload).unwrap();

        let pimessage = Message {
            ack: true,
            m_id: 2,
            p_id: 2,
            message: payload_str
        };

        let encoded = json::encode(&pimessage).unwrap();
        println!("{}", encoded);

        let decode: Message = json::decode(&encoded).unwrap();
        let decoded_payload: Ping = json::decode(&decode.message).unwrap();

        assert_eq!(decode, pimessage);
        assert_eq!(decoded_payload, payload);
    }
}

