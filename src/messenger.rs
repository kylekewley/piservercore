/**
 * This defines the struct used for sending and recieving messages from a byte stream
 *
 * Messages are sent out using the Message_capnp struct preceeded by 4 bytes defining the length of
 * the message not including the four bytes. These bytes are sent in big endian byte order.
 */
use std::io::{self, Write, Read, Result, Error, ErrorKind, BufReader, BufRead, BufWriter};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::thread;
use std::net::TcpStream;

use parser::Parser;

const PREFIX_SIZE: usize = 4;


#[allow(dead_code)]
pub struct Messenger {
    ostream: Arc<Mutex<TcpStream>>,
    istream: TcpStream,
    oqueue: Arc<Mutex<Vec<Message>>>,
    parser: Arc<Parser>,
}

impl Messenger {
    pub fn with_connection(client: TcpStream, parser: Arc<Parser>) -> Result<Messenger> {
        let ostream = Arc::new(Mutex::new(try!(client.try_clone())));
        let istream = try!(client.try_clone());
        let oqueue: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));

        Ok(Messenger {
            ostream: ostream,
            istream: istream,
            oqueue: oqueue,
            parser: parser
        })
    }

    pub fn get_oqueue(&self) -> Arc<Mutex<Vec<Message>>> {
        return self.oqueue.clone();
    }

    /**
     * This function is called on a new thread by the main thread for the
     * individual client. It will block until a message is recieved or there is an io error and
     * return the result.
     */
    pub fn recv_message(istream: &mut Read) -> Result<String> {
        let size = try!({
            Messenger::read_message_size(istream)
        });

        // Create a buffer so we can read the message
        let buf_read = BufReader::new(istream);
        let mut stream = buf_read.take(size as u64);

        let mut recv_str = "";
        let read_len = try!(stream.read_to_string(&mut recv_str));

        if (read_len != size) {
            return Err(Error::new(ErrorKind::InvalidInput, "Not enough bytes in the stream"));
        }

        Ok(recv_str);
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

    pub fn send_message<T: Write>(ostream: &mut T, message: &Message) -> Result<()> {
        let message_str = json::encode(&message).unwrap();
        let message_size = message_str.len();
        Messenger::write_message_size(ostream, message_size as u32).unwrap();
        ostream.write_all(message.as_bytes())
    }

    pub fn add_to_send_queue(&mut self, message: Message) {
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
                let m = Messenger::recv_message(&mut istream).unwrap();
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
                                Messenger::send_message(&mut *guard, &mut m).unwrap();
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
                    let message: Message = json::decode(&r).unwrap();
                    let response = self.parser.parse_message(&message);
                    if response.is_some() {
                        self.add_to_send_queue(response.unwrap());
                    }
                },
                Err(_) => {}
            }
        }
    }
}
#[cfg(test)]
mod test {
}
