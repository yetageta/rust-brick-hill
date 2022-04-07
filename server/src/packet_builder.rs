use crate::buffer;

pub fn build_auth_packet(
    user_id: u32,
    username: String,
    admin: bool,
    membership: u8,
    net_id: u32,
    brick_count: u32,
) -> buffer::Buffer {
    let mut packet = buffer::new(None);
    packet.write_byte(1);
    packet.write_uint32(net_id);
    packet.write_uint32(brick_count);
    packet.write_uint32(user_id);
    packet.write_string(username);
    packet.write_byte(admin as u8);
    packet.write_byte(membership);
    packet.write_uint_v();
    return packet;
}

pub fn build_message_packet(message: String) -> buffer::Buffer {
    let mut packet = buffer::new(None);
    packet.write_byte(6);
    packet.write_string(message);
    packet.write_uint_v();

    return packet
}

