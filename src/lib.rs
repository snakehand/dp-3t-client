extern crate serde;

pub mod session;

#[cfg(test)]
mod tests {
    use crate::session::{ReplayKey, Session, SessionKey};
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
        let key = [0; 32];
        let julian_day = 0;
        let sk = SessionKey { julian_day, key };
        let key2 = <[u8; 32]>::from_hex(
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925",
        )
        .unwrap();
        let sk2 = sk.next();
        assert_eq!(key2, sk2.key);
        let ephemerals = [
            "c71d3e89927435b2aae42be7e7aea70a",
            "6c4902c119d6a7ada139677983ef02b6",
            "034015608c5a55672315cb614f5a94a3",
            "a747e729bf2e3de3ec6ecbdb0f889f5b",
        ];
        let ephems = sk.get_ephemeral(5);
        for (i, eph_str) in ephemerals.iter().enumerate() {
            let eph_tst = <[u8; 16]>::from_hex(eph_str).unwrap();
            assert_eq!(eph_tst, ephems[4 - i].token);
        }
    }
}
