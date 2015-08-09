/**
 * This defines the struct used for sending and recieving messages from a byte stream
 *
 * Messages are sent out using the Message_capnp struct preceeded by 4 bytes defining the length of
 * the message not including the four bytes. These bytes are sent in big endian byte order.
 */
extern crate capnp;

use std::io::{self, Write, Read, Result, Error, ErrorKind, BufReader, BufRead, BufWriter};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::thread;
use std::net::TcpStream;

use capnp::message::{MessageReader, ReaderOptions, MallocMessageBuilder, MessageBuilder};
use capnp::{serialize, message};
use capnp::serialize::OwnedSpaceMessageReader;

use message_capnp::message as Message;
use ack_capnp::ack as Ack;

use parser::Parser;

const PREFIX_SIZE: usize = 4;

#[allow(dead_code)]
pub struct Messanger {
    ostream: Arc<Mutex<TcpStream>>,
    istream: TcpStream,
    oqueue: Arc<Mutex<Vec<MallocMessageBuilder>>>,
    parser: Arc<Parser>,
}

impl Messanger {
    pub fn with_connection(client: TcpStream, parser: Arc<Parser>) -> Result<Messanger> {
        let ostream = Arc::new(Mutex::new(try!(client.try_clone())));
        let istream = try!(client.try_clone());
        let oqueue: Arc<Mutex<Vec<MallocMessageBuilder>>> = Arc::new(Mutex::new(Vec::new()));

        Ok(Messanger {
            ostream: ostream,
            istream: istream,
            oqueue: oqueue,
            parser: parser
        })
    }

    /**
     * This function is called on a new thread by the main thread for the
     * individual client. It will block until a message is recieved or there is an io error and
     * return the result.
     */
    pub fn recv_message(istream: &mut Read) -> ::capnp::Result<OwnedSpaceMessageReader> {
        let size = try!({
            Messanger::read_message_size(istream)
        });

        // Create a buffer so we can read the message
        let buf_read = BufReader::new(istream);
        let mut stream = buf_read.take(size as u64);

        serialize::read_message(&mut stream, ReaderOptions::new())
    }

    /**
     * Convert from an OwnedSpaceMessageReader to an actual ProtoBuf message
     */
    pub fn convert_to_message<'b>(reader: &'b OwnedSpaceMessageReader) -> ::capnp::Result<Message::Reader<'b>> {
        reader.get_root::<Message::Reader>()
    }

    /// Read the first PREFIX_SIZE bytes from the stream interpreted as big endian
    /// Return the message size, or an error if there is an io error.
    pub fn read_message_size(stream: &mut Read) -> Result<u64> {
        let stream = stream.take(PREFIX_SIZE as u64);

        // Shift each byte into a u64 up to PREFIX_SIZE bytes
        let mut length: u64 = 0;
        let mut byte_count = 0;
        for result in stream.bytes() {
            byte_count += 1;
            match result {
                Ok(byte) => {
                    length = length<<8;
                    length |= byte as u64;
                },
                Err(e) => {return Err(e);}
            }
        }

        if byte_count != PREFIX_SIZE {
            return Err(Error::new(ErrorKind::InvalidInput, "Not enough bytes in the stream"));
        }

        return Ok(length)
    }

    /// Write the size of the message into the output stream
    fn write_message_size(stream: &mut Write, size: u32) -> Result<()> {
        let mut buffer = [0u8; PREFIX_SIZE];

        let mut size = size;
        let mask = 0x000000FFu32;

        for i in 0..PREFIX_SIZE {
            // Add each byte to the buffer
            buffer[i] = (size & mask) as u8;
            size = size >> 8;
        }

        stream.write_all(& buffer)
    }

    pub fn send_message<T: Write>(ostream: &mut T, message: &mut MallocMessageBuilder) -> Result<()> {
        let message_size = serialize::compute_serialized_size_in_words(message);
        Messanger::write_message_size(ostream, message_size as u32).unwrap();
        serialize::write_message(ostream, message)
    }

    pub fn add_to_send_queue(&mut self, message: MallocMessageBuilder) {
        let mut queue = self.oqueue.lock().unwrap();
        queue.push(message);
    }

    /// This call loops forever, creating a new thread to handle reading from
    /// the stream. The blocking thread will handle messages as they come in, 
    /// and write new messages when they are added to the queue.
    pub fn handle_client_stream(&mut self) -> Result<()> {
        let (tx, rx) = channel();

        let mut istream = self.istream.try_clone().unwrap();

        // Spawn a new thread to listen to incoming messages
        thread::spawn(move|| {
            loop {
                let m = Messanger::recv_message(&mut istream).unwrap();
                tx.send(m).unwrap();
            }
        });

        loop {
            {
                // Send the first item in the queue
                let message = {
                    let mut queue = self.oqueue.lock().unwrap();
                    queue.pop()
                };

                match message {
                    Some(mut m) => {
                        let ostream = self.ostream.lock();
                        match ostream {
                            Ok(mut guard) => {
                                Messanger::send_message(&mut *guard, &mut m).unwrap();
                            },
                            Err(_) => { /* Mutex poisoned */ }
                        }
                    },
                    None => { /* Nothing in the queue */}
                }
            }

            // Try to recieve messages
            match rx.try_recv() {
                Ok(r) => {
                    let message = Messanger::convert_to_message(&r);
                    match message {
                        Ok(m) => {
                            let response = self.parser.parse_message(&m);
                            if response.is_some() {
                                self.add_to_send_queue(response.unwrap());
                            }
                        },
                        Err(_) => {}
                    }
                },
                Err(_) => {
                    // Do nothing
                }
            }
        }
    }
}
#[cfg(test)]
mod test {

    use super::{Messanger};

    use std::io::{Cursor, Result, BufStream, Read, Write};
    use std::net::{TcpStream, TcpListener, SocketAddr};
    use std::sync::mpsc::channel;
    use std::{thread, convert};

    use capnp::{serialize};
    use capnp::{MessageBuilder, MallocMessageBuilder};

    use message_capnp::message as Message;
    use ack_capnp::ack as Ack;
    use error_capnp::error as Error;


    #[test]
    #[should_panic]
    fn test_invalid_message_prefix() {
        let mut c = Cursor::new(vec![0x12u8]);
        Messanger::read_message_size(&mut c).unwrap();
    }

    fn get_single_connection(listener: TcpListener) -> Result<(TcpStream, SocketAddr)> {
        listener.accept()
    }

    fn create_iostream(port: u16) -> Result<(TcpStream, TcpStream)> {
        let host = "127.0.0.1";
        let server = try!(TcpListener::bind((host, port)));

        let (tx, rx) = channel();

        thread::spawn(move||{tx.send(get_single_connection(server));});

        let client = TcpStream::connect((host, port)).unwrap();
        let (serv, addr) = match rx.recv() {
            Ok(s) => try!(s),
            Err(e) => panic!(e)
        };
        Ok((client, serv))
    }

    #[test]
    fn test_io() {
        let (mut client, mut server) = create_iostream(12345).unwrap();

        // Send the string
        let string = String::from_str("Hello World!");
        let length = string.len();
        server.write_fmt(format_args!("{}", string));

        // Limit the client to read the string length
        let mut c = client.take(length as u64);
        let mut read_str = String::new();

        // Check the read value
        c.read_to_string(&mut read_str);
        assert_eq!(string, read_str);
    }

    #[test]
    fn test_send_recv_message() {
        let id = 123u64;
        let parser_id = 11u32;
        let recv_ack = false;
        let ack_id = 0x42324319u64;
        let ack_status = Ack::Status::Failure;
        let error_code = 3u32;
        let error_message = "Error test string";
        let error_blame_id = 543u64;

        let (mut client, mut server) = create_iostream(12346).unwrap();

        let mut pimessage = MallocMessageBuilder::new_default();
        let size = {
            let mut pi = pimessage.init_root::<Message::Builder>();

            pi.set_message_id(id);
            pi.set_parser_id(parser_id);
            pi.set_recv_ack(recv_ack);

            // Configure the inner message
            {
                let ackmessage = pi.borrow().init_message();
                let mut ack = ackmessage.init_as::<Ack::Builder>();

                ack.set_message_id(ack_id);
                ack.set_status(ack_status);

                {
                    // Configure the error
                    let mut err = ack.init_error();

                    err.set_code(error_code);
                    err.set_message(error_message);
                    err.set_blame_id(error_blame_id);
                }
            }


            pi.total_size()
        };

        let size = size.unwrap().word_count;
        let size2 = serialize::compute_serialized_size_in_words(&mut pimessage);
        Messanger::send_message(&mut server, &mut pimessage);

        let m = Messanger::recv_message(&mut client).unwrap();
        let message = Messanger::convert_to_message(&m).unwrap();

        println!("Size: {} id: {} parser_id: {}", message.total_size().unwrap().word_count, message.get_message_id(), message.get_parser_id());
        assert_eq!(message.total_size().unwrap().word_count, size);
        assert_eq!(message.get_message_id(), id);
        assert_eq!(message.get_parser_id(), parser_id);
        assert_eq!(message.get_recv_ack(), recv_ack);


        // Verify the inner message
        let ack = message.get_message();
        let ack = ack.get_as::<Ack::Reader>().unwrap();
        assert_eq!(ack.get_message_id(), ack_id);

        if ack.get_status().unwrap() != ack_status {
            panic!("Ack status incorrect");
        }

        // Verify the inner error in the ack
        let err = ack.get_error().unwrap();
        assert_eq!(err.get_code(), error_code);
        assert_eq!(err.get_message().unwrap(), error_message);
        assert_eq!(err.get_blame_id(), error_blame_id);

    }
}
