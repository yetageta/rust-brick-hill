use crate::{
    buffer::{Buffer, self},
    packet_builder,
    player::{self, Player},
};
use std::sync::{Arc, Mutex};

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
    pub fn add_player(&mut self, player: Arc<Mutex<player::Player>>) {
        self.players.push(player);

        let players_length = self.players.len();

        let last_player = &self.players[players_length-1];
        let mut last_player_lock = last_player.lock().unwrap();

        let mut packet: Buffer = last_player_lock.build_auth_packet(self.brick_count);
        packet.prepend_type(3); // SendPlayers type

        let net_id = last_player_lock.net_id;
        drop(last_player_lock);

        self.broadcast_packet_except(packet, net_id);


        let mut packet: Buffer = buffer::new(None);
        packet.write_byte(3);
        packet.write_byte(players_length as u8 - 1);

        let mut count = 0;
        for plr in &self.players {
            let unlocked = plr.lock().unwrap();
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
            let last_player = &self.players[self.players.len()-1];
            let mut last_player_lock = last_player.lock().unwrap();

            packet.write_uint_v();;
            last_player_lock.send_packet(packet);
        }
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

    pub fn new_net_object(&mut self) -> u32 {
        self.last_net_id += 1;
        return self.last_net_id - 1;
    }

    pub fn broadcast_packet(&mut self, buf: Buffer) {
        for plr in &self.players {
            let unlocked = plr.lock();

            if let Ok(mut p) = unlocked {
                let _ = &p.send_packet(buf.clone());
            }
        }
    }

    pub fn broadcast_packet_except(&mut self, buf: Buffer, net_id: u32) {
        for plr in &self.players {
            let unlocked = plr.lock();

            if let Ok(mut p) = unlocked {
                if p.net_id == net_id {
                    continue;
                }
                let _ = &p.send_packet(buf.clone());
            }
        }
    }

    pub fn chatted(&mut self, net_id: u32, command: String, args: String) {
        let player = self.find_player(net_id);
        let player_clone = player.clone();
        let username = player_clone.lock().unwrap().username.clone();
        drop(player_clone);

        if command != "chat" {
            return;
        }

        let packet = packet_builder::build_message_packet(format!(
            "\\c6 {}: \\c0{}",
            username,
            args
        ));
        drop(player);

        self.broadcast_packet(packet);
    }
}
