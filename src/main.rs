mod opcodes;
mod hash;
use anyhow::Result;
use hex;
use chrono::prelude::{TimeZone, Utc};
use std::{
    fs::File, 
    fs::OpenOptions, 
    io::{BufReader, Write, Read, BufWriter}
};

use opcodes::script_to_opcodes;

const MAGIC: u32 = 3_652_501_241; // FEBEB4D9

struct VarInt(u64, u32, [u8; 9]);
impl VarInt {
    fn value(&self) -> u64 {
        self.0
    }
    fn len(&self) -> u32 {
        self.1
    }
    fn data(&self) -> [u8; 9] {
        self.2
    }
}

fn read_varint<T: Read>(reader: &mut BufReader<T>) -> Result<VarInt> {
    let mut b1 = vec![0u8; 1];
    let mut b2 = vec![0u8; 2];
    let mut b4 = vec![0u8; 4];
    let mut b8 = vec![0u8; 8];

    reader.read_exact(&mut b1)?;
    let number = u8::from_le_bytes(b1[..].try_into()?) as u64;
    let mut data = [0u8; 9];
    data[..1].copy_from_slice(&b1[..]);

    let varint = match number {
        253 => {
            reader.read_exact(&mut b2)?;
            data[1..3].copy_from_slice(&b2[..]);
            VarInt(u16::from_le_bytes(b2[..].try_into()?) as u64, 3, data)
        },
        254 => {
            reader.read_exact(&mut b4)?;
            data[1..5].copy_from_slice(&b4[..]);
            VarInt(u32::from_le_bytes(b4[..4].try_into()?) as u64, 5, data)
        },
        255 => {
            reader.read_exact(&mut b8)?;
            data[1..9].copy_from_slice(&b8[..]);
            VarInt(u64::from_le_bytes(b8[..8].try_into()?), 8, data)
        },
        _ => {
            VarInt(number, 1, data)
        }
    };
    Ok(varint)
}
struct Header {
    version: [u8; 4],
    prev_hash: [u8; 32],
    merkle_root: [u8; 32],
    time: [u8; 4],
    bits: [u8; 4],
    nonce: [u8; 4],
}
struct Block {
    size: u32,
    header: Header,
    transactions: Vec<Tx>,
}
struct Tx {
    version: u32,
    flag: Option<[u8; 2]>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    witnesses: Option<Vec<Witness>>,
    lock_time: [u8; 4],
}
struct Input {
    txid: [u8; 32],
    vout: u32,
    script_len: VarInt,
    script: Vec<u8>,
    sequence: [u8; 4],
}
struct Output {
    value: u64,
    script_len: VarInt,
    script: Vec<u8>,
}
struct Witness {

}

fn main() -> Result<()> {
    let mut b1 = vec![0u8; 1];
    let mut b4 = vec![0u8; 4];
    let mut b8 = vec![0u8; 8];
    let mut b32 = vec![0u8; 32];

    let mut block_number = 0;

    // segwit
    //let mut file_number = 976;

    // create files with headers
    let f = File::create("F:/csv/blocks.csv").expect("Unable to create file");
    let mut blk_w = BufWriter::new(f);
    writeln!(blk_w, "FILE,BLOCK,DATE,TIME,VERSION,PREV_HASH,MERKLE_ROOT,BITS,NONCE,TX_COUNT")?;

    let f = File::create("F:/csv/tx.csv").expect("Unable to create file");
    let mut tx_w = BufWriter::new(f);
    writeln!(tx_w, "BLOCK,TXID,INP_COUNT,OUT_COUNT")?;
    
    let f = File::create("F:/csv/txi.csv").expect("Unable to create file");
    let mut txi_w = BufWriter::new(f);
    writeln!(txi_w, "TXID,TIN,VOUT,SCRIPT")?;

    let f = File::create("F:/csv/txo.csv").expect("Unable to create file");
    let mut txo_w = BufWriter::new(f);
    writeln!(txo_w, "TXID,AMOUNT,SCRIPT")?;

    for file_number in 0..4586 {
        let reader = File::open(format!("F:/btc/blocks/blk{:05}.dat", file_number))?;
        let mut reader = BufReader::new(reader);

        loop {       
            if reader.read_exact(&mut b4).is_err() {
                println!("eof for file {}", file_number);
                break;
            }

            //println!("\nBLOCK NUMBER: {}", block_number);

            let magic = u32::from_le_bytes(b4[..4].try_into()?);
            assert!(magic == MAGIC, "Wrong magic number");    
            reader.read_exact(&mut b4)?;
            //let bsize = u32::from_le_bytes(b4[..4].try_into()?);
            
            // block
            // version
            reader.read_exact(&mut b4)?;
            let version = b4[..4].try_into()?;
            let version_int = u32::from_le_bytes(b4[..4].try_into()?);
            
            // previous block hash
            reader.read_exact(&mut b32)?;
            let prev_hash = b32[..32].try_into()?;
            
            // merkle root
            reader.read_exact(&mut b32)?;
            let merkle_root = b32[..32].try_into()?;
            
            // time
            reader.read_exact(&mut b4)?;
            let time = b4[..4].try_into()?;
            let timestamp = u32::from_le_bytes(b4[..4].try_into()?) as i64;
            let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
            let date_str = datetime.format("%Y-%m-%d");
            let time_str = datetime.format("%H:%M:%S");            
            println!("FILE {} BLOCK {} {} {}", file_number, block_number, date_str, time_str);
            
            // bits
            reader.read_exact(&mut b4)?;
            let bits = b4[..4].try_into()?;

            // nonce
            reader.read_exact(&mut b4)?;
            let nonce = b4[..4].try_into()?;
            
            // header
            let header = Header {version, prev_hash, merkle_root, time, bits, nonce};
            
            let tx_count = read_varint(&mut reader)?;
            //println!("Transactions: {}", tx_count.value());
            
            writeln!(blk_w, "{},{},{},{},{},{},{},{},{},{}", 
                file_number, block_number, date_str, time_str,
                hex::encode(version), 
                hex::encode(prev_hash), 
                hex::encode(merkle_root), 
                hex::encode(bits), 
                hex::encode(nonce),
                tx_count.value()
            )?;
            
            // tx
            for tx in 0..tx_count.value() {
                //println!(" Transaction: {}", (tx + 1));
                let mut tx_data = Vec::new();

                // version
                reader.read_exact(&mut b4)?;
                tx_data.extend_from_slice(&b4);
                let version = u32::from_le_bytes(b4[..4].try_into()?);                
                assert_eq!(version, 1);
                //println!("version     : {}", hex::encode(&b4));

                // optional flag 0001 2 bytes or varint with num of inputs
                let mut in_count = read_varint(&mut reader)?;
                //println!(" Inputs     : {}", in_count.value());
                
                let mut has_witness = false;

                if in_count.value() == 0 {
                    has_witness = true;    
                    reader.read_exact(&mut b1)?;
                    assert_eq!(hex::encode(&b1), "01");
                    in_count = read_varint(&mut reader)?;
                    //println!("segwit flag, in_counter: {}", in_counter.value());
                } 
                tx_data.extend_from_slice(&in_count.data()[..in_count.len() as usize]);
                
                // input
                for _ in 0..in_count.value() {
                    // prev tx hash
                    reader.read_exact(&mut b32)?;
                    tx_data.extend_from_slice(&b32);                
                    let txid = hex::encode(&b32);
                    //println!("  txid     : {}", txid);

                    // prev txout index
                    reader.read_exact(&mut b4)?;
                    tx_data.extend_from_slice(&b4);
                    let vout = u32::from_le_bytes(b4[..4].try_into()?); 
                    //println!("  vout: {}", hex::encode(&b4));
                    
                    // tx in script len
                    let in_script_len = read_varint(&mut reader)?;
                    tx_data.extend_from_slice(&in_script_len.data()[..in_script_len.len() as usize]);
                    //println!("  script_len: {}", in_script_len.value());
                    // scriptsig
                    let mut script_sig = vec![0u8; in_script_len.value() as usize];
                    reader.read_exact(&mut script_sig)?;
                    tx_data.extend_from_slice(&script_sig[..script_sig.len() as usize]);
                    //println!("  script_sig: {}", hex::encode(&script_sig));
                    let opcode = script_to_opcodes(&script_sig);
                    println!("  script_sig: {}", opcode);

                    // sequence
                    reader.read_exact(&mut b4)?;
                    tx_data.extend_from_slice(&b4);
                    //println!("  sequence nr : {}", hex::encode(&b4));

                    //writeln!(txi_w, "{},{},{}", txid, vout, opcode)?;
            
                }

                // out-counter
                let out_count = read_varint(&mut reader)?;
                tx_data.extend_from_slice(&out_count.data()[..out_count.len() as usize]);
                //println!(" Outputs    : {}", out_count.value());

                // output
                for _ in 0..out_count.value() {
                    // value
                    reader.read_exact(&mut b8)?;
                    tx_data.extend_from_slice(&b8);
                    //println!("  Sat value : {}", u64::from_le_bytes(b8[..8].try_into()?));

                    // tx in script len
                    let script_len = read_varint(&mut reader)?;
                    tx_data.extend_from_slice(&script_len.data()[..script_len.len() as usize]);
                    //println!("  script_len: {}", script_len.value());
                    
                    // scriptpk
                    if script_len.value() > 0 {
                        let mut script_pky = vec![0u8; script_len.value() as usize];
                        reader.read_exact(&mut script_pky)?;
                        tx_data.extend_from_slice(&script_pky[..script_pky.len() as usize]);
                        //println!("  script_pub: {}", hex::encode(&script_pky));
                        let opcode = script_to_opcodes(&script_pky);
                        println!("  script_pub: {}", opcode);
                    }
                    //writeln!(txo_w, "{},{},{}", txid, vout, opcode)?;

                }
                // witnesses
                if has_witness {
                    for _ in 0..in_count.value() {
                        let wit_count = read_varint(&mut reader)?;
                        //println!("  witness items : {}", wit_counter.value());
                        
                        for _ in 0..wit_count.value() {
                            let wit_len = read_varint(&mut reader)?.value();                            
                            let mut wit_buf = vec![0u8; wit_len as usize];
                            reader.read_exact(&mut wit_buf)?;
                            //println!("  witness : {}: {}", wit_len, hex::encode(wit_buf));

                        }
                    }
                }
                // lock time
                reader.read_exact(&mut b4)?;
                //println!("lock_time : {}", hex::encode(&b4));
                tx_data.extend_from_slice(&b4);
                let txid = hash::compute_txid(&tx_data[..]);
                let txid_str = hex::encode(&txid[..]);
                // println!("txid: {}", hex::encode(&txid[..]));
                writeln!(tx_w, "{},{},{},{}",block_number, txid_str, in_count.value(), out_count.value())?;
            
            }   

            //blk_w.flush()?;

            //break;

            block_number +=1;

        }
    }
    
    Ok(())
}