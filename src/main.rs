use std::fs::{File};
use std::io::{BufReader,BufWriter};
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;
use std::io::prelude::*;
use std::env;
use std::time::{SystemTime};

fn main() {
    let start_time = SystemTime::now();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide filename argument!");
        return;        
    }
    let filename = &args[1];
    let file = File::open(filename.to_string()).unwrap();
    let buff_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buff_reader);
    let mut decode: HashMap<(u16, u16), u16> = HashMap::new();
    let word_len = br.read_byte().unwrap();

    let mut decode_dict_len = br.read_byte().unwrap() as u16;
    decode_dict_len = decode_dict_len << 8 | br.read_byte().unwrap() as u16;
    let code_length_counter = 64 - (decode_dict_len as u64).leading_zeros();
    let mut total_bits_from_dict: u32 = br.read_byte().unwrap() as u32;
    total_bits_from_dict = total_bits_from_dict << 8 | br.read_byte().unwrap() as u32;
    total_bits_from_dict = total_bits_from_dict << 8 | br.read_byte().unwrap() as u32;
    total_bits_from_dict = total_bits_from_dict << 8 | br.read_byte().unwrap() as u32;
    let uncompressed_bytes_count = br.read_byte().unwrap();
    println!("Header is read successfully:
        word length: {},
        dict length: {},
        dict bytes: {} ({} bits),
        uncompressed bytes: {},
        code length counter: {}
    ", word_len, decode_dict_len, total_bits_from_dict/8, total_bits_from_dict, uncompressed_bytes_count, code_length_counter);

    let mut dict_bits = 0u128;
    for _ in 0..decode_dict_len {
        let relevant_bits = read_bits_into(&mut br, code_length_counter);
        let code = read_bits_into(&mut br, relevant_bits as u32);
        let symbol = read_bits_into(&mut br, word_len as u32);
        decode.insert((code, relevant_bits), symbol);
        dict_bits += code_length_counter as u128 + relevant_bits  as u128 + word_len  as u128;

    }
    println!("Dictionary is built successfully: total {} bits", dict_bits);
    let newfile = filename.trim_right_matches(".bdazip");
    let w = File::create(newfile).unwrap();
    let mut buf_writer = BufWriter::new(w);
    let mut bw = BitWriter::new(&mut buf_writer);
    let mut dict_bytes_read_counter = 0u32;
    let mut bits_container: u128 = 0;
    let bytes_chunks_to_read = total_bits_from_dict/32;
    let bits_left = total_bits_from_dict%32;
    let mut bits_leftover_in_container = 0u32;
    let mut bits_left_counter = total_bits_from_dict;
    while dict_bytes_read_counter != bytes_chunks_to_read {
        bits_left_counter -= 32;
        match br.read_bits(32) {
            Ok(int) => {
                dict_bytes_read_counter = dict_bytes_read_counter + 1;
                bits_container = bits_container << 32 | int as u128;
                bits_leftover_in_container = bits_leftover_in_container + 32;
            },
            Err(err) => {
                println!("\n\n{} {}", dict_bytes_read_counter, err);
                break
            }
        }
        if dict_bytes_read_counter % 3 == 0 {
            let mut code_len: u16 = 0;
            let mut code: u16 = 0;
            let wl = word_len as usize;
            for i in (0..bits_leftover_in_container).rev() {
                code = code << 1 | (((1u128 << i) & bits_container) > 0) as u16;
                code_len = code_len + 1;
                
                match decode.get(&(code,code_len)) {
                    Some(symbol) => {
                        bw.write_bits(*symbol as u32, wl).unwrap();
                        code_len = 0;
                        code = 0;
                    }, 
                    None => {}
                }
            }
            bits_leftover_in_container = code_len as u32;
            if bits_leftover_in_container > 0 {
                bits_container = ((1u128 << bits_leftover_in_container) - 1)  & bits_container;
            } else {
                bits_container = 0;
            }
            
        }
    }
    println!("bits leftover: {}, bits_leftover_in_container: {}, [{}]", bits_left_counter ,bits_leftover_in_container, bits_container);
    let mut code_len: u16 = 0;
    let mut code: u16 = 0;
    let wl = word_len as usize;
    
    match br.read_bits((bits_left) as usize) {
        Ok(int) => {
            bits_container = (bits_container << (bits_left)) | (int as u128);
            for i in (0..(bits_left + bits_leftover_in_container)).rev() {
                code = code << 1 | (((1u128 << i) & bits_container) > 0) as u16;
                code_len = code_len + 1;
                match decode.get(&(code,code_len)) {
                    Some(symbol) => {
                        bw.write_bits(*symbol as u32, wl).unwrap();
                        code_len = 0;
                        code = 0;
                    }, 
                    None => continue
                }
            }
        },
        Err(e) => println!("{}", e)
    }
    
    for _ in 0..uncompressed_bytes_count {
        println!("printing not compressed bytes.");
        let value = br.read_byte().unwrap();
        bw.write_byte(value).unwrap();
    }
    match start_time.elapsed() {
        Ok(elapsed) => {
            println!("Dedompressed in: {} s", elapsed.as_secs());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    buf_writer.flush().unwrap();
}

fn read_bits_into (
    br: &mut BitReader<std::io::BufReader<std::fs::File>, bitbit::reader::MSB>,
    relevant_bits: u32) -> u16 {
    let mut res = 0;
    for _ in 0..relevant_bits {
        match br.read_bit() {
            Ok(bit) => {
                res = (res << 1) | (bit as u16);
            },
            _ => return 0
        }
    }
    res
}
