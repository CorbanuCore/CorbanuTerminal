#![allow(clippy::unwrap_used)]
use super::*;
use pretty_assertions::assert_eq;

struct Fake {
    fail_at: Option<usize>,
    calls: Vec<String>,
}
impl Fake {
    fn step(&mut self, call: String) -> io::Result<()> {
        self.calls.push(call);
        if self.fail_at == Some(self.calls.len() - 1) {
            Err(unavailable())
        } else {
            Ok(())
        }
    }
}
impl IdentityOps for Fake {
    fn root_and_single_thread(&mut self) -> io::Result<()> {
        self.step("root/thread/fds".into())
    }
    fn restrict_exec(&mut self) -> io::Result<()> {
        self.step("restrict".into())
    }
    fn groups(&mut self, groups: &[u32]) -> io::Result<()> {
        self.step(format!("groups:{groups:?}"))
    }
    fn gid(&mut self, gid: u32) -> io::Result<()> {
        self.step(format!("gid:{gid}"))
    }
    fn uid(&mut self, uid: u32) -> io::Result<()> {
        self.step(format!("uid:{uid}"))
    }
    fn verify(&mut self, _: &SyntheticChildIdentity) -> io::Result<()> {
        self.step("verify".into())
    }
}

#[test]
fn pf_27_s01_preparation_stops_at_each_failed_boundary() {
    let identity =
        SyntheticChildIdentity::parse(&["journal", "101", "201", "204"].map(str::to_owned))
            .unwrap();
    let expected = [
        "root/thread/fds",
        "restrict",
        "groups:[204]",
        "gid:201",
        "uid:101",
        "verify",
    ];
    for fail_at in (0..6).map(Some).chain([None]) {
        let mut fake = Fake {
            fail_at,
            calls: vec![],
        };
        assert_eq!(apply(&mut fake, &identity).is_ok(), fail_at.is_none());
        assert_eq!(fake.calls, expected[..fail_at.map_or(6, |i| i + 1)]);
    }
}

#[test]
fn pf_27_s01_preparation_requires_exact_saved_ids_and_groups() {
    for (role, anchor, groups) in [("journal", "204", vec![204]), ("worker", "none", vec![])] {
        let identity =
            SyntheticChildIdentity::parse(&[role, "101", "201", anchor].map(str::to_owned))
                .unwrap();
        assert!(matches_target(&identity, [101; 3], [201; 3], &groups));
        for index in 0..3 {
            let mut uid = [101; 3];
            uid[index] = 0;
            let mut gid = [201; 3];
            gid[index] = 0;
            assert!(!matches_target(&identity, uid, [201; 3], &groups));
            assert!(!matches_target(&identity, [101; 3], gid, &groups));
        }
        let mut extra = groups.clone();
        extra.push(205);
        assert!(!matches_target(&identity, [101; 3], [201; 3], &extra));
        assert!(!matches_target(&identity, [101; 3], [201; 3], &[206]));
    }
}
