extern crate dp_3t_client;

use dp_3t_client::session::{ReplayKey, Session};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;

#[repr(C)]
pub struct Dp3tSessionKey {
    pub julian_day: u32,
    pub key: [u8; 32],
}

#[repr(C)]
pub struct Dp3tEphemeral {
    pub julian_day: u32,
    pub ephem: [u8; 16],
}

#[no_mangle]
pub extern "C" fn dp3t_new_session() -> Box<Session> {
    let sess = Session::new();
    Box::new(sess)
}
#[no_mangle]
pub extern "C" fn dp3t_load_session(filename: *const c_char) -> Option<Box<Session>> {
    let c_str = unsafe { CStr::from_ptr(filename) };
    let path = PathBuf::from(c_str.to_str().unwrap());
    match Session::load(&path) {
        Ok(sess) => Some(Box::new(sess)),
        Err(_) => None,
    }
}

#[no_mangle]
pub extern "C" fn dp3t_get_ephemerals(mut sess: Box<Session>, out: *mut u8, num: u32) {
    let eph = sess.get_ephemeral(num).unwrap();
    let mut pos = 0;
    for e in eph {
        for b in e.token.iter() {
            unsafe { *out.add(pos) = *b };
            pos += 1;
        }
    }
    std::mem::forget(sess);
}

#[no_mangle]
pub extern "C" fn dp3t_save_session(mut sess: Box<Session>, filename: *const c_char) -> c_int {
    let c_str = unsafe { CStr::from_ptr(filename) };
    let path = PathBuf::from(c_str.to_str().unwrap());
    let res = match sess.save(&path) {
        Ok(_) => 0,
        Err(_) => 1,
    };
    std::mem::forget(sess);
    res
}

#[no_mangle]
pub extern "C" fn dp3t_get_session_key(sess: Box<Session>, sk: *mut Dp3tSessionKey) -> c_int {
    let (jd, secret) = match sess.get_secret() {
        Some(js) => js,
        None => {
            std::mem::forget(sess);
            return 1;
        }
    };
    unsafe {
        (*sk).julian_day = jd;
        (*sk).key = secret;
    }
    std::mem::forget(sess);
    0
}

#[no_mangle]
pub extern "C" fn dp3t_free_session(sess: Box<Session>) {
    let _ = &sess;
}

#[no_mangle]
pub extern "C" fn dp3t_new_replay(sk: *const Dp3tSessionKey, num_tokens: u32) -> Box<ReplayKey> {
    let jd = unsafe { (*sk).julian_day };
    let replay = ReplayKey::new(jd, jd + 15, num_tokens, unsafe { &(*sk).key });
    Box::new(replay)
}

#[no_mangle]
pub extern "C" fn dp3t_next(mut replay: Box<ReplayKey>, ephemeral: *mut Dp3tEphemeral) -> c_int {
    let eph = match replay.next() {
        Some(e) => e,
        None => {
            std::mem::forget(replay);
            return 0;
        }
    };
    unsafe {
        (*ephemeral).julian_day = eph.day;
        (*ephemeral).ephem = eph.token;
    }
    std::mem::forget(replay);
    1
}

#[no_mangle]
pub extern "C" fn dp3t_free_replay(replay: Box<ReplayKey>) {
    let _ = &replay;
}
