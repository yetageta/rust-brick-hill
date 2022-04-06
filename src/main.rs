use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

mod buffer;
use buffer::{Buffer};

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 80];

    loop { 
        match stream.read(&mut data) {
            Ok(size) => {
                if size == 0 {
                    break;
                }
                let mut buffer = Buffer::new(Some(data));
                buffer.read_uint_v();
                buffer.zlib_uncompress();

                let packet_type = buffer.read_byte();
                
                if packet_type == 3 {
                    let command = buffer.read_string();
                    let args = buffer.read_string();
                    println!("Command: {}, Args: {}", command, args);
                } else if packet_type == 1 {
                    let mut authentication_packet = Buffer::new(None);
                    authentication_packet.write_byte(1); // Mark as authentication
                    authentication_packet.write_uint32(0); // Net ID
                    authentication_packet.write_uint32(100); // Brick Count
                    authentication_packet.write_uint32(1); // User ID
                    authentication_packet.write_string("Player 1".to_string()); // Username
                    authentication_packet.write_byte(0); // Admin
                    authentication_packet.write_byte(0); //  Membership Type
                    authentication_packet.write_uint_v();

                    match stream.write(&authentication_packet.data) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Error when sending: {}", e)
                        },
                    };
                }
            },
            Err(_) => {
                println!("Connection error, client: {}", stream.peer_addr().unwrap());
                stream.shutdown(Shutdown::Both).unwrap();
                break
            }
        }
    }

    println!("Client has disconnected");
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:42480").unwrap();
    println!("Server listening on port 42480");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}