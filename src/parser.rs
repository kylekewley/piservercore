/**
 * This defines the struct to be used for tracking the functions used for responding to different
 * messeges based on their message ID. Functions can be registered for certain IDs and when a
 * message comes in with that ID, it will be passed to the proper function.
 */

extern crate capnp;

use std::collections::{HashMap};
use std::sync::{Arc, Mutex};

use capnp::message::{MessageReader, ReaderOptions, MallocMessageBuilder, MessageBuilder};
use capnp::{serialize, message};
use capnp::serialize::OwnedSpaceMessageReader;

use message_capnp::message as Message;
use ack_capnp::ack as Ack;

pub struct Parser {
    parsers: HashMap<u32, Box<Fn(&Message::Reader) ->
        Option<MallocMessageBuilder>+Send+Sync>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { parsers: HashMap::new() }
    }

    pub fn parse_message(&self, message: &Message::Reader) -> Option<MallocMessageBuilder> {
        let parser_id = message.get_parser_id();
        let f = self.parsers.get(&parser_id);

        if f.is_some() {
            let f = f.unwrap();
            let result = f(message);

            return result;
        }

        // No parser registered for the ID. Ignore the message
        None
    }

    /**
      * Register the function to be called for a message with the given parser id
      *
      * @return true if the parser was registered. false if a parser already exists for the given
      * ID
      */

    pub fn register_parser(&mut self, parser_id: u32, parser: Box<Fn(&Message::Reader) ->
                           Option<MallocMessageBuilder>+Send+Sync>) -> bool {

        if self.parsers.contains_key(&parser_id) {
            return false;
        }

        self.parsers.insert(parser_id, parser);

        true
    }
}
