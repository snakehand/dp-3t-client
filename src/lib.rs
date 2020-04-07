extern crate serde;

pub mod session;

#[cfg(test)]
mod tests {
    use crate::session::{ReplayKey, Session};
    use hex::FromHex;
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
        let eph1 = session.get_ephemeral(8).unwrap();
        let eph2 = session.get_ephemeral(8).unwrap();
        assert_eq!(eph1, eph2);
    }

    #[test]
    fn test_rewind() {
        let mut session = Session::new();
        let eph1 = session.get_ephemeral(8).unwrap();
        session.set_future(20); // 20 days into future
        let eph2 = session.get_ephemeral(8).unwrap();
        assert_ne!(eph1, eph2);
        session.set_future(0); // rewind
                               // This will fail as key from day 0 is no longer available
        assert!(session.get_ephemeral(8).is_err());
    }

    #[test]
    fn test_recovery() {
        let mut session = Session::new();
        session.set_future(10);
        let eph1 = session.get_ephemeral(8).unwrap();
        session.set_future(0);
        let (day, secret) = session.get_secret().unwrap();
        let mut rplay = ReplayKey::new(day, day + 14, 8, &secret);
        let found = rplay.any(|e| e == eph1[3]);
        assert!(found);
    }

    #[test]
    fn test_compat1() {
        let key = <[u8; 32]>::from_hex(
            "a02b24fe26cd8607424cd21ec8240da2c1f4294ae39fc0d90c38121d9229f943",
        )
        .unwrap();
        let ephemerals = [
            "59efe619ad0db70807b71ba0c0a120d1",
            "25063bbcb6858f7ee77b4013f27685c0",
            "92f2957f3b35d506214f4f578e9a8c31",
            "2d3931973a075f801ac213dc8105c4c0",
            "9658c6a552d58af78c2317828da23c5c",
            "7c97b737eebf34e2e5d4181eb294a056",
            "fbea2ce4015cd406c8e1656b66e18b0e",
            "aefc05e3dc44b962e29324bf42c79566",
        ];
        for eph_str in &ephemerals {
            let mut rplay = ReplayKey::new(0, 1, 8, &key);
            let eph = <[u8; 16]>::from_hex(eph_str).unwrap();
            let found = rplay.any(|e| e.token == eph);
            // assert!(found);
        }
    }
}
