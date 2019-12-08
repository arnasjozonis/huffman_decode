use std::fs::{File};
use std::io::{BufReader,BufWriter};
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;
use std::io::prelude::*;

fn main() {
    println!("Decoding...");
    
    let file = File::open("zipped.ajoz").unwrap();
    let buff_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buff_reader);
    let mut bits_buff = 0u16;
    let mut bit_ptr_pos = 0u8;
    let mut decode: HashMap<(u16, u8), usize> = HashMap::new();
    let mut decode_dict_len = br.read_byte().unwrap() as u16;
    decode_dict_len = decode_dict_len << 8 | br.read_byte().unwrap() as u16;
    for i in 0..decode_dict_len {
        let mut code_bytes: [u8; 2] = [0; 2];
        let mut symbol_bytes: [u8; 2] = [0; 2];
        code_bytes[0] = br.read_byte().unwrap();
        code_bytes[1] = br.read_byte().unwrap();
        let important_bits = br.read_byte().unwrap();
        symbol_bytes[0] = br.read_byte().unwrap();
        symbol_bytes[1] = br.read_byte().unwrap();
        let code: u16 = (code_bytes[0] as u16) << 8 | (code_bytes[1] as u16);
        let symbol: u16 = (symbol_bytes[0] as u16) << 8 | (symbol_bytes[1] as u16);
        decode.insert((code, important_bits), symbol as usize);
    }
    let w = File::create("test.txt").unwrap();
    let mut buf_writer = BufWriter::new(w);
    let mut bw = BitWriter::new(&mut buf_writer);
    loop {
        let b = br.read_bit();
        match b {
            Ok(bit) => {
                bits_buff = bits_buff << 1 | bit as u16;
                match decode.get(&(bits_buff, bit_ptr_pos + 1)) {
                    Some(value) => {
                        bw.write_byte(*value as u8).unwrap();
                        bits_buff = 0;
                        bit_ptr_pos = 0;
                    },
                    None => {
                        bit_ptr_pos = bit_ptr_pos + 1;
                    }
                }
            },
            Err(err) => {
                println!("\n\n{}", err);
                break
            }
        }
    }
    buf_writer.flush().unwrap();
}
