use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

mod buffer;
use buffer::Buffer;

mod packet_builder;

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
                    let authentication_packet = packet_builder::build_auth_packet(
                        1 as u32,
                        "Player 1".to_string(),
                        false,
                        0,
                        0,
                        100,
                    );
                    match stream.write(&authentication_packet.data) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error when sending: {}", e)
                        }
                    };
                }
            }
            Err(_) => {
                println!("Connection error, client: {}", stream.peer_addr().unwrap());
                stream.shutdown(Shutdown::Both).unwrap();
                break;
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
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}
