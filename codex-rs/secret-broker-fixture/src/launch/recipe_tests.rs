#![allow(clippy::unwrap_used)]
use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s01_recipe_rejects_collisions_and_sentinel_ids() {
    let base = [(101, 201), (102, 202), (103, 203)];
    let path = PathBuf::from("/fixture/probe");
    assert!(SyntheticLaunchRecipe::new(PathBuf::from("relative"), base, 204).is_err());
    for anchor in [0, u32::MAX, 201, 202, 203] {
        assert!(SyntheticLaunchRecipe::new(path.clone(), base, anchor).is_err());
    }
    for index in 0..3 {
        for value in [0, u32::MAX, base[(index + 1) % 3].0] {
            let mut ids = base;
            ids[index].0 = value;
            assert!(SyntheticLaunchRecipe::new(path.clone(), ids, 204).is_err());
        }
        for value in [0, u32::MAX, base[(index + 1) % 3].1] {
            let mut ids = base;
            ids[index].1 = value;
            assert!(SyntheticLaunchRecipe::new(path.clone(), ids, 204).is_err());
        }
    }
}

#[test]
fn pf_27_s01_preparation_arguments_are_exact_and_canonical() {
    for (role, anchor, expected) in [
        ("journal", "204", vec![204]),
        ("policy", "204", vec![204]),
        ("worker", "none", vec![]),
    ] {
        let args = [role, "101", "201", anchor].map(str::to_owned);
        assert_eq!(
            SyntheticChildIdentity::parse(&args).ok(),
            Some(SyntheticChildIdentity {
                uid: 101,
                gid: 201,
                groups: expected
            })
        );
    }
    for args in [
        vec!["worker", "101", "201", "204"],
        vec!["journal", "101", "201", "none"],
        vec!["policy", "101", "201", "201"],
        vec!["unknown", "101", "201", "204"],
        vec!["worker", "101", "201"],
        vec!["worker", "101", "201", "none", "extra"],
    ] {
        assert!(
            SyntheticChildIdentity::parse(&args.into_iter().map(str::to_owned).collect::<Vec<_>>())
                .is_err()
        );
    }
    for index in 1..4 {
        for value in [
            "0",
            "4294967295",
            "4294967296",
            "-1",
            "+101",
            "0101",
            " 101",
            "",
        ] {
            let mut args = ["journal", "101", "201", "204"].map(str::to_owned);
            args[index] = value.to_owned();
            assert!(SyntheticChildIdentity::parse(&args).is_err());
        }
    }
}
