use std::{io::Read, num::Wrapping};

use byteorder::{ByteOrder, LittleEndian};
use flate2::read::ZlibDecoder;

pub struct Message {
    message_size: u16,
    end: u16,
}

fn read_uint_v(buffer: &[u8]) -> Message {
    let mut msg = Message {
        message_size: 0,
        end: 0,
    };

    if (buffer[0] & 1) == 1 {
        msg.message_size = (buffer[0] >> 1) as u16;
        msg.end = 1;
    } else if (buffer[0] & 2) == 1 {
        msg.message_size = ((LittleEndian::read_u16(buffer) >> 2) + 0x80) as u16;
        msg.end = 2;
    } else if (buffer[0] & 4) == 1 {
        msg.message_size = ((Wrapping(buffer[2]) << 13)
            + (Wrapping(buffer[1]) << 5)
            + (Wrapping(buffer[0]) >> 3)
            + Wrapping(0x4080 as u16 as u8))
        .0 as u16;
        msg.end = 3;
    } else {
        msg.message_size = ((LittleEndian::read_u32(buffer) / 8) + (0x204080 as u32)) as u16;
        msg.end = 4;
    }

    return msg;
}

#[derive(Clone)]
pub struct Buffer {
    pub data: Vec<u8>,
}

pub fn new(bytes: Option<[u8; 80]>) -> Buffer {
    match bytes {
        Some(x) => return Buffer { data: x.to_vec() },
        None => return Buffer { data: [].to_vec() },
    }
}

impl Buffer {
    pub fn zlib_uncompress(&mut self) -> bool {
        let mut _buffer = [0; 80];
        let success: bool = match ZlibDecoder::new(&(*self.data)).read(&mut _buffer) {
            Err(_) => false,
            Ok(_) => true,
        };
        if !success {
            return false;
        };

        if _buffer[0] != 0 {
            self.data.clear();
            self.data = _buffer.to_vec();
        }

        return true;
    }

    pub fn read_uint_v(&mut self) -> Message {
        let msg = read_uint_v(&self.data);

        self.data = self.data[msg.end as usize..(msg.message_size + msg.end) as usize].to_vec();

        return msg;
    }

    pub fn write_uint_v(&mut self) {
        let length = self.data.len();

        if length < 0x80 {
            let size = (length << 1) + 1;
            self.data.insert(0, size as u8);
        } else if length < 0x4080 {
            let mut size = [0 as u8; 2];
            let i = ((length - 0x80) << 2) + 2;
            LittleEndian::write_u16(&mut size, i as u16);

            for i in 0..2 {
                self.data.insert(0, size[1 - i]);
            }
        } else if length < 0x204080 {
            let mut size = [0 as u8; 3];
            let i = ((length - 0x4080) << 3) + 4;
            size[0] = (i & 0xFF) as u8;

            LittleEndian::write_u16(&mut size, (i >> 8) as u16);

            for i in 0..3 {
                self.data.insert(0, size[2 - i]);
            }
        } else {
            let mut size = [0 as u8; 4];
            LittleEndian::write_u32(&mut size, ((length - 0x204080) * 8) as u32);

            for i in 0..4 {
                self.data.insert(0, size[3 - i]);
            }
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.data[0];
        self.data.remove(0);
        return byte;
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn write_uint32(&mut self, uint: u32) {
        let mut buffer: [u8; 4] = [0; 4];
        LittleEndian::write_u32(&mut buffer, uint);
        for i in 0..4 {
            self.data.push(buffer[i]);
        }
    }

    pub fn read_string(&mut self) -> String {
        let mut new_string = String::new();
        let mut index = 0;

        for byte in &self.data {
            index += 1;
            let char = *byte as char;
            if char == '\0' {
                break;
            }
            new_string.push(char);
        }

        for _ in 0..index {
            self.data.remove(0);
        }

        return new_string;
    }

    pub fn write_string(&mut self, wrt_string: String) {
        for char in wrt_string.chars() {
            let byte = char as u8;
            self.write_byte(byte);
        }
        self.write_byte(0);
    }
}
