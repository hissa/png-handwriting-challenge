use std::{fs::File, io::Write};

use crc::{Crc, CRC_32_ISO_HDLC};
use flate2::{write::ZlibEncoder, Compression};

fn main() {
    let head = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let stream: Vec<u8> = head
        .into_iter()
        .chain(get_ihdr().into_iter())
        .chain(get_idat().into_iter())
        .chain(get_iend().into_iter())
        .collect();

    println!("{:02X?}", stream);

    let mut file = File::create("out.png").unwrap();
    file.write_all(&stream).unwrap();
    file.flush().unwrap();
}

fn get_ihdr() -> Vec<u8> {
    let chunk_type = [b'I', b'H', b'D', b'R'];
    let ihdr = [
        0x00, 0x00, 0x00, 0x02, // Width: 2
        0x00, 0x00, 0x00, 0x02, // Height: 2
        0x08, // BitDepth: 8
        0x02, // ColorType: Color
        0x00, // Compression method 0
        0x00, // Filter method 0
        0x00, // No interlace
    ];
    let crc = get_crc(&chunk_type, &ihdr);
    let length = (ihdr.len() as u32).to_be_bytes();

    let stream: Vec<u8> = length
        .into_iter()
        .chain(chunk_type.into_iter())
        .chain(ihdr.into_iter())
        .chain(crc.into_iter())
        .collect();

    stream
}

fn get_idat() -> Vec<u8> {
    let chunk_type = [b'I', b'D', b'A', b'T'];

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let raw_data = [
        0x00,
            0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00,
        0x00,
            0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF
    ];

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw_data).unwrap();
    let compressed = encoder.finish().unwrap();

    let length = (compressed.len() as u32).to_be_bytes();
    let crc = get_crc(&chunk_type, &compressed);

    let stream: Vec<u8> = length
        .into_iter()
        .chain(chunk_type.into_iter())
        .chain(compressed.into_iter())
        .chain(crc.into_iter())
        .collect();

    stream
}

fn get_iend() -> Vec<u8> {
    let chunk_type = [b'I', b'E', b'N', b'D'];
    let length = 0u32.to_be_bytes();
    let data: [u8; 0] = [];
    let crc = get_crc(&chunk_type, &data);

    let stream: Vec<u8> = length
        .into_iter()
        .chain(chunk_type.into_iter())
        .chain(crc.into_iter())
        .collect();

    stream
}

fn get_crc(chunk_type: &[u8; 4], data: &[u8]) -> [u8; 4] {
    let stream: Vec<u8> = {
        let chunk_type = chunk_type.into_iter().cloned();
        let data = data.into_iter().cloned();
        chunk_type.chain(data).collect()
    };

    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let checksum = crc.checksum(&stream);

    checksum.to_be_bytes()
}
