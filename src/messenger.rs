/**
 * This defines the struct used for sending and recieving messages from a byte stream
 *
 * Messages are sent out using the Message_capnp struct preceeded by 4 bytes defining the length of
 * the message not including the four bytes. These bytes are sent in big endian byte order.
 */
extern crate capnp;


use std::io::{self, Write, Read, Result, Error, ErrorKind};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::thread;
use std::net::TcpStream;

use capnp::message::{MessageReader, ReaderOptions, MallocMessageBuilder, MessageBuilder};
use capnp::io::{InputStream, OutputStream, BufferedOutputStreamWrapper};
use capnp::{serialize_packed, message};
use capnp::serialize::OwnedSpaceMessageReader;

use Message_capnp::message as Message;
use Ack_capnp::ack as Ack;

const PREFIX_SIZE: usize = 4;

#[allow(dead_code)]
struct Messenger {
    ostream: Arc<Mutex<TcpStream>>,
    istream: TcpStream,
    oqueue: Arc<Mutex<Vec<(MallocMessageBuilder, u32)>>>,
}

#[allow(dead_code)]
#[allow(unused_attributes)]
impl Messenger {

    /**
     * This function is called on a new thread by the main thread for the
     * individual client. It will block until a message is recieved or there is an io error and
     * return the result.
     */
    fn recv_message(istream: &mut Read) -> ::capnp::Result<OwnedSpaceMessageReader> {
        let size = try!({
            Messenger::read_message_size(istream)
        });

        let mut stream = istream.take(size as u64);
        serialize_packed::new_reader_unbuffered(&mut stream, ReaderOptions::new())
    }

    fn convert_to_message<'b>(reader: &'b OwnedSpaceMessageReader) -> ::capnp::Result<Message::Reader<'b>> {
        reader.get_root::<Message::Reader>()
    }


    /// Read the first PREFIX_SIZE bytes from the stream interpreted as big endian
    /// Return the message size, or an error if there is an io error.
    fn read_message_size(stream: &mut Read) -> Result<u64> {
        let stream = stream.take(PREFIX_SIZE as u64);

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

    fn write_message_size(stream: &mut OutputStream, size: u32) -> Result<()> {
        let mut buffer = [0u8; PREFIX_SIZE];

        let mut size = size;
        let mask = 0x000000FFu32;

        for i in 0..PREFIX_SIZE {
            buffer[i] = (size & mask) as u8;
            size = size >> 8;
        }

        stream.write(& buffer)
    }

    fn send_message<T: OutputStream>(ostream: &mut T, message: &mut MallocMessageBuilder, message_size: u32) -> Result<()> {
        Messenger::write_message_size(ostream, message_size);
        serialize_packed::write_packed_message_unbuffered(ostream, message)
    }

    pub fn add_to_send_queue(&mut self, message: MallocMessageBuilder, size: u32) {
        let mut queue = self.oqueue.lock().unwrap();
        queue.push((message, size));
    }

    /// TODO: Add a clientManager class that manages client groups
    /// TODO: Add a parser class that keeps track of parsers and id's
    pub fn with_manager_and_parser() {
    }

    /// This call loops forever, creating a new thread to handle reading from
    /// the stream. The blocking thread will handle messages as they come in, 
    /// and write new messages when they are added to the queue.
    pub fn handle_client_stream(&mut self, stream: TcpStream) -> Result<()> {
        let ostream = Arc::new(Mutex::new(try!(stream.try_clone())));
        let mut istream = stream;

        let (tx, rx) = channel();

        thread::spawn(move|| {
            loop {
                let m = Messenger::recv_message(&mut istream).unwrap();
                tx.send(m).unwrap();
            }
        });

        loop {
            {
                // Send all messages in the queue
                let mut queue = self.oqueue.lock().unwrap();
                if queue.len() > 0 {
                    match ostream.try_lock() {
                        Ok(guard) => {
                            let (mut m, s) = queue.pop().unwrap();
                            let ostream = ostream.clone();
                            thread::spawn(move|| {
                                let mut ostream = ostream.lock().unwrap();
                                Messenger::send_message(&mut *ostream, &mut m, s).unwrap();
                            });
                        },
                        Err(e) => { /* Mutex in use or poisoned... Continue */ }
                    };
                }
            }

            // Try to recieve messages
            match rx.try_recv() {
                Ok(r) => {
                    let message = Messenger::convert_to_message(&r).unwrap();
                    //TODO: Send to the parser
                },
                Err(e) => {
                    // Do nothing
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {

    use super::{Messenger};

    use std::io::{Cursor, Result, BufStream, Read, Write};
    use std::net::{TcpStream, TcpListener, SocketAddr};
    use std::old_io::util::IterReader;
    use std::old_io::{Reader, Writer};
    use std::sync::mpsc::channel;
    use std::{thread, convert};

    use capnp::serialize_packed;
    use capnp::{MessageBuilder, MallocMessageBuilder};

    use Message_capnp::message as Message;
    use Ack_capnp::ack as Ack;
    use Error_capnp::error as Error;

    #[test]
    fn test_be_message_prefix() {
        let num = 0x12345678;

        let mut v = Vec::new();
        v.write_be_u32(num);

        let mut c = Cursor::new(v);

        match Messenger::read_message_size(&mut c) {
            Ok(size) => assert_eq!(size, num as u64),
            Err(e) => panic!(e)
        };
    }

    #[test]
    #[should_panic]
    fn test_invalid_message_prefix() {
        let mut c = Cursor::new(vec![0x12u8]);
        Messenger::read_message_size(&mut c).unwrap();
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
        Messenger::send_message(&mut server, &mut pimessage, size as u32);

        let m = Messenger::recv_message(&mut client).unwrap();
        let message = Messenger::convert_to_message(&m).unwrap();

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
