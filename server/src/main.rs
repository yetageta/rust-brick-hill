use std::io::{self};
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

mod brick;
mod buffer;
mod colour;
mod game;
mod packet_builder;
mod player;

async fn handle_client(_stream: TcpStream, game: Arc<Mutex<game::Game>>) -> Result<(), ()> {
    let mut data = [0 as u8; 80];

    let stream = Arc::new(Mutex::new(_stream));
    let mut player: Arc<Mutex<player::Player>> = Arc::new(Mutex::new(player::new()));

    loop {
        let unlocked_stream = stream.lock().await;
        match unlocked_stream.try_read(&mut data) {
            Ok(0) => break,
            Ok(_) => {
                drop(unlocked_stream);

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

                {
                    let result_lock = game.try_lock();
                    let mut locked_game = result_lock.unwrap();

                    if packet_type == 1 {
                        let mut temp_player = player::new();
                        temp_player.net_id = locked_game.new_net_object();

                        player = Arc::new(Mutex::new(temp_player));

                        let mut locked_player = player.lock().await;
                        locked_player.set_stream(stream.clone());
                        locked_player
                            .check_auth(&mut buffer, &mut locked_game)
                            .await;
                        drop(locked_player);

                        (*locked_game).add_player(player.clone()).await;
                    } else if packet_type == 3 {
                        let command = buffer.read_string();
                        let args = buffer.read_string();

                        let locked_player = player.lock().await;
                        let net_id = locked_player.net_id;
                        drop(locked_player);

                        (*locked_game).chatted(net_id, command, args).await;
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => {
                return Ok(());
            }
        }
    }

    println!("Client has disconnected");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let listener = TcpListener::bind("0.0.0.0:42480").await.unwrap();
    println!("Server listening on port 42480");

    let game = Arc::new(Mutex::new(game::new()));

    brick::load_from_file("map.brk".to_string());

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(_) => {
                continue;
            }
        };

        println!("New connection: {}", stream.peer_addr().unwrap());

        let game_clone = Arc::clone(&game);

        tokio::spawn(async move { handle_client(stream, game_clone).await });
    }
}
