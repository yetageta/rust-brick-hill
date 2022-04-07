use crate::{buffer::Buffer, game::Game, packet_builder};
use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub struct Player {
    pub stream: Option<Arc<Mutex<TcpStream>>>,

    pub username: String,
    pub user_id: u32,
    pub net_id: u32,
    admin: bool,
    membership: u8,
}

pub fn new() -> Player {
    return Player {
        stream: None,
        username: String::from("Player"),
        user_id: 0,
        net_id: 0,
        admin: false,
        membership: 0,
    };
}

impl Player {
    pub fn set_stream(&mut self, stream: Arc<Mutex<TcpStream>>) {
        self.stream = Some(stream);
    }

    pub fn check_auth(&mut self, buf: &mut Buffer, unlocked_game: Arc<Mutex<Game>>) {
        //let token = buf.read_string();
        //let version = buf.read_string();

        let game = unlocked_game.lock().unwrap();

        if (*game).is_local {
            let packet = packet_builder::build_auth_packet(
                self.user_id,
                self.username.clone(),
                self.admin,
                self.membership,
                self.net_id,
                game.brick_count,
            );

            match &self.stream {
                Some(stream) => {
                    stream.lock().unwrap().write(&packet.data);
                }
                None => {
                    println!("No stream to send auth to");
                }
            }

            return;
        }

        //let url = format!("https://api.brick-hill.com/v1/auth/verifyToken?token=${}&host_key=${}", token, "");
    }
}