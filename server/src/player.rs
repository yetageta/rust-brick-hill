use crate::{buffer::Buffer, game::Game, packet_builder};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct Player {
    pub stream: Option<Arc<Mutex<TcpStream>>>,

    pub username: String,
    pub user_id: u32,
    pub net_id: u32,
    pub admin: bool,
    pub membership: u8,
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
    pub async fn build_auth_packet(&mut self, brick_count: u32) -> Buffer {
        return packet_builder::build_auth_packet(
            self.user_id,
            self.username.clone(),
            self.admin,
            self.membership,
            self.net_id,
            brick_count,
        );
    }

    pub fn set_stream(&mut self, stream: Arc<Mutex<TcpStream>>) {
        self.stream = Some(stream);
    }

    pub async fn send_packet(&mut self, buf: Buffer) {
        let stream = self.stream.as_ref().unwrap();
        let mut new_stream = (&stream).clone().lock().await;

        match new_stream.write(&buf.data).await {
            Ok(size) => {
                if size != buf.data.len() {
                    println!("Failed to send all data: Size: {}", size);
                }
            }
            Err(e) => {
                println!("Error sending packet: {:?}", e);
            }
        }
    }

    pub async fn check_auth(&mut self, _buf: &mut Buffer, game: &mut MutexGuard<'_, Game>) {
        if (*game).is_local {
            self.username = String::from(format!("Player {}", game.players.len() + 1));

            let packet = self.build_auth_packet(game.brick_count).await;

            self.send_packet(packet).await;

            return;
        }

        // let url = format!("https://api.brick-hill.com/v1/auth/verifyToken?token=${}&host_key=${}", token, "");
    }
}
