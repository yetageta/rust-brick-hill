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

        let success: bool = match (*unlocked_stream).read(&mut data) {
            Ok(size) => {
                drop(unlocked_stream);
                if size == 0 {
                    break;
                }
                true
            }
            Err(_) => {
                println!(
                    "Connection error, client: {}",
                    unlocked_stream.peer_addr().unwrap()
                );
                unlocked_stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        };

        if !success {
            continue;
        }

        let mut buffer = buffer::new(Some(data));
        buffer.read_uint_v();
        buffer.zlib_uncompress();

        if buffer.data.len() == 0 {
            continue;
        }

        let packet_type = buffer.read_byte();

        if packet_type == 18 {
            continue;
        }

        let mut result_lock = game.try_lock();
        if let Ok(ref mut locked_game) = result_lock {
            if packet_type == 1 {
                let mut temp_player = player::new();
                temp_player.net_id = locked_game.new_net_object();

                player = Arc::new(Mutex::new(temp_player));

                let mut locked_player = player.lock().unwrap();
                locked_player.set_stream(Arc::clone(&stream));
                locked_player.check_auth(&mut buffer, locked_game);
                drop(locked_player);

                (*locked_game).add_player(player.clone());
            } else if packet_type == 3 {
                let command = buffer.read_string();
                let args = buffer.read_string();

                let locked_player = player.lock().unwrap();
                let net_id = locked_player.net_id;
                drop(locked_player);

                (*locked_game).chatted(net_id, command, args);
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
