use aes::block_cipher_trait::generic_array::GenericArray;
use aes::block_cipher_trait::BlockCipher;
use aes::Aes256;
use ring::rand::SecureRandom;
use ring::{digest, hmac};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
struct SessionKey {
    julian_day: u32,
    key: [u8; 32],
}

impl SessionKey {
    fn next(&self) -> SessionKey {
        let julian_day = self.julian_day + 1;
        let mut key = [0; 32];
        let hash = digest::digest(&digest::SHA256, &self.key);
        key.copy_from_slice(hash.as_ref());
        SessionKey { julian_day, key }
    }

    fn get_ephemeral(&self, dst: &mut [u8; 16], index: u32) {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.key);
        let aes_key = hmac::sign(&key, b"broadcast key");
        let mut nonce_serial = [0u8; 16];
        nonce_serial[0..8].copy_from_slice(b"AES_PRNG");
        nonce_serial[12..16].copy_from_slice(&index.to_be_bytes());
        let mut block = GenericArray::clone_from_slice(&nonce_serial);
        let cipher = Aes256::new(aes_key.as_ref().into());
        cipher.encrypt_block(&mut block);
        dst.copy_from_slice(&block);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Session {
    recent_keys: Vec<SessionKey>,
}

impl Session {
    pub fn load(path: &PathBuf) -> Result<Session, &'static str> {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Err("Could not open file"),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(_) => return Err("Could not read file"),
        }
        let mut recent_keys: Vec<SessionKey> = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => return Err("Could not decode file"),
        };
        recent_keys.sort();
        Ok(Session { recent_keys })
    }

    pub fn new() -> Session {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let julian_day = ((now.as_secs() / (24 * 3600)) + 2_440_588) as u32;
        let rnd = ring::rand::SystemRandom::new();
        let mut key = [0; 32];
        rnd.fill(&mut key).unwrap();
        let mut recent_keys = Vec::new();
        recent_keys.push(SessionKey { julian_day, key });
        // recent_keys.push( recent_keys[0].next() );
        Session { recent_keys }
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), &'static str> {
        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(_) => return Err("Could not create file"),
        };
        let serialized = match serde_json::to_string(&self.recent_keys) {
            Ok(k) => k,
            Err(_) => return Err("Could not serialise"),
        };
        match file.write_all(serialized.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not write to file"),
        }
    }

    pub fn get_ephemeral(&mut self, dst: &mut [u8; 16], index: u32) -> Result<(), &'static str> {
        if self.recent_keys.is_empty() {
            return Err("No keys");
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let julian_day = ((now.as_secs() / (24 * 3600)) + 2_440_588) as u32;
        if julian_day < self.recent_keys[0].julian_day {
            return Err("No keys from past available");
        }
        let mut bri = 0;
        for (i, k) in self.recent_keys.iter().enumerate() {
            if k.julian_day == julian_day {
                k.get_ephemeral(dst, index);
                return Ok(());
            }
            if k.julian_day > julian_day {
                bri = i;
                break;
            }
        }
        let last_index = if bri == 0 {
            self.recent_keys.len() - 1
        } else {
            bri - 1
        };
        let mut key = self.recent_keys[last_index];
        while key.julian_day < julian_day {
            key = key.next();
            self.recent_keys.push(key);
            if key.julian_day == julian_day {
                key.get_ephemeral(dst, index);
            }
        }
        self.recent_keys.sort();
        // TODO: Prune old keys - save keys
        Ok(())
    }
}
