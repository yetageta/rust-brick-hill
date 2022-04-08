use crate::{
    buffer::{self, Buffer},
    packet_builder,
    player::{self, Player},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Game {
    pub is_local: bool,
    pub brick_count: u32,

    pub players: Vec<Arc<Mutex<player::Player>>>,

    pub last_net_id: u32,
}

pub fn new() -> Game {
    return Game {
        is_local: true,
        brick_count: 0,
        players: vec![],
        last_net_id: 0,
    };
}

impl Game {
    pub async fn add_player(&mut self, player: Arc<Mutex<player::Player>>) {
        self.players.push(player);

        let players_length = self.players.len();

        let last_player = &self.players[players_length - 1].clone();
        let last_player_lock = last_player.lock().await;

        let mut packet: Buffer = buffer::new(None);
        packet.write_byte(3);
        packet.write_byte(1);
        packet.write_uint32(last_player_lock.net_id);
        packet.write_string(last_player_lock.username.clone());
        packet.write_uint32(last_player_lock.user_id);

        packet.write_byte(last_player_lock.admin as u8);
        packet.write_byte(last_player_lock.membership);
        packet.write_uint_v();

        let net_id = last_player_lock.net_id;
        drop(last_player_lock);

        self.broadcast_packet_except(packet, net_id).await;

        let mut packet: Buffer = buffer::new(None);
        packet.write_byte(3);
        packet.write_byte(players_length as u8 - 1);

        let mut count = 0;
        for plr in &self.players {
            let unlocked = plr.lock().await;
            if unlocked.net_id == net_id {
                drop(unlocked);
                continue;
            }
            count += 1;

            packet.write_uint32(unlocked.net_id);
            packet.write_string(unlocked.username.clone());
            packet.write_uint32(unlocked.user_id);

            packet.write_byte(unlocked.admin as u8);
            packet.write_byte(unlocked.membership);
        }

        if count > 0 {
            let last_player = &self.players[self.players.len() - 1].clone();
            let mut last_player_lock = last_player.lock().await;

            packet.write_uint_v();
            last_player_lock.send_packet(packet).await;
        }
    }

    pub async fn find_player(&mut self, net_id: u32) -> &Arc<Mutex<Player>> {
        for plr in &self.players {
            let unlocked = plr.lock().await;
            if unlocked.net_id == net_id {
                return &plr;
            }
        }
        return &self.players[0];
    }

    pub fn new_net_object(&mut self) -> u32 {
        self.last_net_id += 1;
        return self.last_net_id - 1;
    }

    pub async fn broadcast_packet(&mut self, buf: &Buffer) {
        for plr in &self.players {
            let mut unlocked = plr.lock().await;
            let _ = unlocked.send_packet(buf.clone()).await;
        }
    }

    pub async fn broadcast_packet_except(&mut self, buf: Buffer, net_id: u32) {
        for plr in &self.players {
            let mut unlocked = plr.lock().await;
            if unlocked.net_id == net_id {
                continue;
            }

            let _ = unlocked.send_packet(buf.clone()).await;
        }
    }

    pub async fn chatted(&mut self, net_id: u32, command: String, args: String) {
        let player = self.find_player(net_id).await;
        let player_clone = Arc::clone(player);

        let username = player_clone.lock().await.username.clone();
        drop(player_clone);

        if command != "chat" {
            return;
        }

        let packet =
            packet_builder::build_message_packet(format!("\\c6 {}: \\c0{}", username, args));

        drop(username);

        self.broadcast_packet(&packet).await;

        drop(packet);
    }
}
