use std::io::Read;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

mod buffer;
mod game;
mod packet_builder;
mod player;

fn handle_client(mut _stream: TcpStream, game: Arc<Mutex<game::Game>>) {
    let mut data = [0 as u8; 80];

    let stream = Arc::new(Mutex::new(_stream));
    let mut player: Arc<Mutex<player::Player>> = Arc::new(Mutex::new(player::new()));

    loop {
        let mut unlocked_stream = stream.lock().unwrap();
        match unlocked_stream.read(&mut data) {
            Ok(size) => {
                drop(unlocked_stream);

                if size == 0 {
                    break;
                }
                let mut buffer = buffer::new(Some(data));
                buffer.read_uint_v();
                buffer.zlib_uncompress();

                let packet_type = buffer.read_byte();

                if packet_type == 3 {
                    let command = buffer.read_string();
                    let args = buffer.read_string();

                    let mut lock = game.try_lock();
                    if let Ok(ref mut mutex) = lock {
                        let locked_player = player.lock().unwrap();
                        let net_id = locked_player.net_id;
                        drop(locked_player);

                        (*mutex).chatted(net_id, command, args);
                    }
                } else if packet_type == 1 {
                    player = Arc::new(Mutex::new(player::new()));
                    let mut locked_player = player.lock().unwrap();
                    locked_player.set_stream(Arc::clone(&stream));
                    locked_player.check_auth(&mut buffer, Arc::clone(&game));
                    drop(locked_player);

                    let mut lock = game.try_lock();
                    if let Ok(ref mut mutex) = lock {
                        (*mutex).add_player(player.clone());
                    }
                }
            }
            Err(_) => {
                println!(
                    "Connection error, client: {}",
                    unlocked_stream.peer_addr().unwrap()
                );
                unlocked_stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }

    println!("Client has disconnected");
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:42480").unwrap();
    println!("Server listening on port 42480");

    let game = Arc::new(Mutex::new(game::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());

                let game_clone = Arc::clone(&game);
                thread::spawn(move || handle_client(stream, game_clone));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    drop(listener);
}
