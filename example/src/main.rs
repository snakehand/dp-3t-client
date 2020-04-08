extern crate dp_3t_client;

use dp_3t_client::session::{Ephemeral, ReplayKey};

fn speed_test() {
    let mut key: [u8; 32] = [0; 32];
    let mut total = 0;
    for i in 0..1000_u32 {
        key[28..32].copy_from_slice(&i.to_be_bytes());
        let rp = ReplayKey::new(0, 14, 8, &key);
        total += rp.fold(0_u64, |s, _e| s + 1);
    }
    println!("{} ephemerals", total);
}

fn main() {
    let key: [u8; 32] = [0; 32];
    let rp = ReplayKey::new(0, 1, 8, &key);
    let mut ephm: Vec<Ephemeral> = rp.collect();
    ephm.reverse(); // Reverse since Replay Key pops off from the end of vector for efficiency reasons
    for (i, e) in ephm.iter().enumerate() {
        println!("eph: {} - {}", i, e);
    }
}
