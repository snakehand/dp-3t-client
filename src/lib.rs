extern crate serde;

pub mod session;

#[cfg(test)]
mod tests {
    use crate::session::{ReplayKey, Session};
    use std::path::PathBuf;

    #[test]
    fn test_session() {
        let mut session = Session::new();
        let path = PathBuf::from("test_session.json");
        assert!(session.save(&path).is_ok());
        let s2 = Session::load(&path).unwrap();
        assert_eq!(session, s2);
    }

    #[test]
    fn test_ephemeral() {
        let mut session = Session::new();
        let mut eph1 = [0u8; 16];
        let mut eph2 = [0u8; 16];
        session.get_ephemeral(&mut eph1, 0).unwrap();
        session.get_ephemeral(&mut eph2, 1).unwrap();
        assert_ne!(eph1, eph2);
        session.get_ephemeral(&mut eph1, 1).unwrap();
        assert_eq!(eph1, eph2);
    }

    #[test]
    fn test_rewind() {
        let mut session = Session::new();
        let mut eph = [0u8; 16];
        session.get_ephemeral(&mut eph, 0).unwrap();
        session.set_future(20); // 20 days into future
        session.get_ephemeral(&mut eph, 0).unwrap();
        session.set_future(0); // rewind
                               // This will fail as key from day 0 is no longer available
        assert!(session.get_ephemeral(&mut eph, 0).is_err());
    }

    #[test]
    fn test_recovery() {
        let mut session = Session::new();
        let mut eph = [0u8; 16];
        session.set_future(10);
        session.get_ephemeral(&mut eph, 3).unwrap();
        session.set_future(0);
        let (day, secret) = session.get_secret().unwrap();
        let mut rplay = ReplayKey::new(day, day + 14, 8, &secret);
        let found = rplay.any(|e| e.token == eph);
        assert!(found);
    }
}
