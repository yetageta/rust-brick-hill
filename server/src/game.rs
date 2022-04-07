use crate::{
    buffer::Buffer,
    packet_builder,
    player::{self, Player},
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

pub struct Game {
    pub is_local: bool,
    pub brick_count: u32,

    pub players: Vec<Arc<Mutex<player::Player>>>,
}

pub fn new() -> Game {
    return Game {
        is_local: true,
        brick_count: 0,
        players: vec![],
    };
}

impl Game {
    pub fn add_player(&mut self, player: Arc<Mutex<player::Player>>) {
        self.players.push(player);
    }

    pub fn find_player(&mut self, net_id: u32) -> &Arc<Mutex<Player>> {
        for plr in &self.players {
            let unlocked = plr.lock().unwrap();
            if unlocked.net_id == net_id {
                return &plr;
            }
        }
        return &self.players[0];
    }

    pub fn broadcast_packet(&mut self, buf: Buffer) {
        for plr in &self.players {
            let unlocked = plr.lock();

            if let Ok(mut p) = unlocked {
                let _ = &p.send_packet(buf.clone());
            }
        }
    }

    pub fn chatted(&mut self, net_id: u32, command: String, args: String) {
        let player = self.find_player(net_id);

        if command != "chat" {
            return;
        }

        let packet = packet_builder::build_message_packet(format!(
            "\\c6 {}: \\c0{}",
            player.lock().unwrap().username,
            args
        ));
        drop(player);

        self.broadcast_packet(packet);
    }
}
