extern crate serde;

pub mod session;

#[cfg(test)]
mod tests {
    use crate::session::Session;
    use std::path::PathBuf;

    #[test]
    fn test_session() {
        // let session = crate::session::Session::new();
        let session = Session::new();
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
}
