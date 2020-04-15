use aes::block_cipher_trait::generic_array::GenericArray;
use aes::block_cipher_trait::BlockCipher;
use aes::Aes256;
use ring::rand::SecureRandom;
use ring::{digest, hmac};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::PathBuf;
use std::time::SystemTime;

const MAX_HISTORY: u32 = 14;
const JULIAN_DAY_1970: u64 = 2440587;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct SessionKey {
    pub julian_day: u32,
    pub key: [u8; 32],
}

#[derive(PartialEq, Eq)]
pub struct Ephemeral {
    pub day: u32,
    pub token: [u8; 16],
}

impl fmt::Display for Ephemeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = hex::encode(self.token);
        write!(f, "Ephemeral(day:{}, token:{})", self.day, hex_string)
    }
}

impl fmt::Debug for Ephemeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_string = hex::encode(self.token);
        write!(f, "Ephemeral(day:{}, token:{})", self.day, hex_string)
    }
}

impl SessionKey {
    pub fn next(&self) -> SessionKey {
        let julian_day = self.julian_day + 1;
        let mut key = [0; 32];
        let hash = digest::digest(&digest::SHA256, &self.key);
        key.copy_from_slice(hash.as_ref());
        SessionKey { julian_day, key }
    }

    pub fn get_ephemeral(&self, num_tokens: u32) -> Vec<Ephemeral> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.key);
        let aes_key = hmac::sign(&key, b"Broadcast key");
        // let hex_string = hex::encode(aes_key);
        // println!("AES key:{})", hex_string);
        let cipher = Aes256::new(aes_key.as_ref().into());
        let mut result = Vec::with_capacity(num_tokens as usize);
        let day = self.julian_day;
        for i in 0..num_tokens {
            let mut serial = [0u8; 16];
            serial[12..16].copy_from_slice(&i.to_be_bytes());
            let mut block = GenericArray::clone_from_slice(&serial);
            cipher.encrypt_block(&mut block);
            let mut token = [0; 16];
            token.copy_from_slice(&block);
            result.push(Ephemeral { day, token })
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Session {
    recent_keys: Vec<SessionKey>,
    test_future: u32,
    path: Option<PathBuf>,
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
        Ok(Session {
            recent_keys,
            test_future: 0,
            path: Some(path.to_owned()),
        })
    }

    pub fn new() -> Session {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let julian_day = ((now.as_secs() / (24 * 3600)) + JULIAN_DAY_1970) as u32;
        let rnd = ring::rand::SystemRandom::new();
        let mut key = [0; 32];
        rnd.fill(&mut key).unwrap();
        let mut recent_keys = Vec::new();
        recent_keys.push(SessionKey { julian_day, key });
        // recent_keys.push( recent_keys[0].next() );
        Session {
            recent_keys,
            test_future: 0,
            path: None,
        }
    }

    pub fn save(&mut self, path: &PathBuf) -> Result<(), &'static str> {
        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(_) => return Err("Could not create file"),
        };
        let serialized = match serde_json::to_string(&self.recent_keys) {
            Ok(k) => k,
            Err(_) => return Err("Could not serialise"),
        };
        match file.write_all(serialized.as_bytes()) {
            Ok(_) => {
                self.path = Some(path.to_owned());
                Ok(())
            }
            Err(_) => Err("Could not write to file"),
        }
    }

    pub fn get_ephemeral(&mut self, num_tokens: u32) -> Result<Vec<Ephemeral>, &'static str> {
        if self.recent_keys.is_empty() {
            return Err("No keys");
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let julian_day =
            ((now.as_secs() / (24 * 3600)) + JULIAN_DAY_1970) as u32 + self.test_future;
        if julian_day < self.recent_keys[0].julian_day {
            return Err("No keys from past available");
        }
        let mut bri = 0;
        for (i, k) in self.recent_keys.iter().enumerate() {
            if k.julian_day == julian_day {
                return Ok(k.get_ephemeral(num_tokens));
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
        let tokens = loop {
            key = key.next();
            self.recent_keys.push(key);
            if key.julian_day == julian_day {
                break key.get_ephemeral(num_tokens);
            }
        };
        self.recent_keys
            .retain(|k| k.julian_day + MAX_HISTORY >= julian_day);
        self.recent_keys.sort();
        if let Some(path) = self.path.take() {
            let _ = self.save(&path);
        }
        Ok(tokens)
    }

    pub fn set_future(&mut self, future: u32) {
        self.test_future = future;
    }

    pub fn get_secret(&self) -> Option<(u32, [u8; 32])> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let julian_day =
            ((now.as_secs() / (24 * 3600)) + JULIAN_DAY_1970) as u32 + self.test_future;
        for k in &self.recent_keys {
            if k.julian_day + MAX_HISTORY >= julian_day {
                return Some((k.julian_day, k.key));
            }
        }
        None
    }
}

pub struct ReplayKey {
    key: SessionKey,
    end_day: u32,
    num_tokens: u32,
    cache: Vec<Ephemeral>,
}

impl ReplayKey {
    pub fn new(start_day: u32, end_day: u32, num_tokens: u32, key: &[u8; 32]) -> ReplayKey {
        let key = SessionKey {
            julian_day: start_day,
            key: *key,
        };
        ReplayKey {
            key,
            end_day,
            num_tokens,
            cache: Vec::new(),
        }
    }
}

impl Iterator for ReplayKey {
    type Item = Ephemeral;

    fn next(&mut self) -> Option<Self::Item> {
        let last_key = if !self.cache.is_empty() {
            return self.cache.pop();
        } else {
            let last = self.key;
            self.key = last.next();
            last
        };
        if self.key.julian_day > self.end_day {
            return None;
        }
        self.cache = last_key.get_ephemeral(self.num_tokens);
        self.cache.pop()
    }
}
