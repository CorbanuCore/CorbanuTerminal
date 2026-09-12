use super::*;
use pretty_assertions::assert_eq;
use std::io::Cursor;

fn put(b: &mut [u8], offset: usize, value: u64) {
    b[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn fixture() -> Vec<u8> {
    let mut b = vec![0; 1024];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    b[16] = 3;
    b[18] = 62;
    b[20] = 1;
    put(&mut b, 24, 512);
    put(&mut b, 32, 64);
    b[52] = 64;
    b[54] = 56;
    b[56] = 2;
    b[64] = 1;
    b[68] = 5;
    put(&mut b, 96, 1024);
    put(&mut b, 104, 1024);
    put(&mut b, 112, 4096);
    b[120] = 2;
    b[124] = 6;
    put(&mut b, 128, 384);
    put(&mut b, 136, 384);
    put(&mut b, 152, 32);
    put(&mut b, 160, 32);
    put(&mut b, 168, 8);
    put(&mut b, 384, 0x6ffffffb);
    put(&mut b, 392, 0x08000001);
    b
}
fn parse(b: Vec<u8>) -> io::Result<()> {
    let size = b.len() as u64;
    inspect(&mut Cursor::new(b), size)
}

#[test]
fn pf_27_s01_elf_profile_identity_and_header_bounds() {
    assert!(parse(fixture()).is_ok());
    for (offset, value) in [
        (0, 0),
        (4, 1),
        (5, 2),
        (6, 0),
        (7, 255),
        (8, 1),
        (16, 2),
        (18, 1),
        (20, 0),
        (48, 1),
        (52, 63),
        (54, 55),
        (56, 0),
        (56, 129),
    ] {
        let mut b = fixture();
        b[offset] = value;
        assert!(parse(b).is_err(), "header byte {offset}");
    }
    for offset in [24, 32] {
        let mut b = fixture();
        put(&mut b, offset, u64::MAX);
        assert!(parse(b).is_err());
    }
    for length in [0, 63, 100, 1023] {
        let mut b = fixture();
        b.truncate(length);
        assert!(parse(b).is_err());
    }
    assert!(inspect(&mut Cursor::new(fixture()), IMAGE_LIMIT + 1).is_err());
}

#[test]
fn pf_27_s01_elf_profile_program_ranges_and_mapping_denials() {
    for (offset, value) in [(64, 3), (68, 7), (120, 0xdead), (124, 4)] {
        let mut b = fixture();
        b[offset..offset + 4].copy_from_slice(&(value as u32).to_le_bytes());
        assert!(parse(b).is_err());
    }
    for (offset, value) in [
        (72, u64::MAX),
        (80, u64::MAX),
        (96, 1025),
        (104, 1023),
        (112, 3),
        (128, 385),
        (136, 392),
        (152, 31),
        (152, 0),
        (160, 33),
        (152, 65552),
        (128, u64::MAX),
        (136, u64::MAX),
        (24, 1024),
    ] {
        let mut b = fixture();
        put(&mut b, offset, value);
        assert!(parse(b).is_err(), "segment field {offset} value {value}");
    }
    // Duplicate singleton and ambiguous LOAD mappings are independently rejected.
    for source in [64..120, 120..176] {
        let mut b = fixture();
        let segment = b[source].to_vec();
        b[176..232].copy_from_slice(&segment);
        b[56] = 3;
        assert!(parse(b).is_err());
    }
    let mut b = fixture();
    put(&mut b, 96, 512); // Entry in BSS, not file-backed.
    assert!(parse(b).is_err());
}

#[test]
fn pf_27_s01_elf_profile_dynamic_vocabulary_and_termination() {
    for tag in [
        1, 15, 29, 0x7fffffff, 0x7ffffffd, 0x6ffffefc, 0x6ffffefb, 0xdead,
    ] {
        let mut b = fixture();
        put(&mut b, 400, tag);
        assert!(parse(b).is_err(), "external or unknown tag {tag}");
    }
    for value in [0, 1, 0x08000000, 0x08000003] {
        let mut b = fixture();
        put(&mut b, 392, value);
        assert!(parse(b).is_err());
    }
    let mut b = fixture();
    put(&mut b, 400, 0x6ffffffb);
    put(&mut b, 408, 0x08000001);
    assert!(parse(b).is_err());
    let mut b = fixture();
    put(&mut b, 408, 1);
    assert!(parse(b).is_err());
    for (tag, value) in [
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
        (8, 0),
        (9, 24),
        (10, 0),
        (11, 24),
        (12, 0),
        (13, 0),
        (21, 0),
        (25, 0),
        (26, 0),
        (27, 0),
        (28, 0),
        (30, 8),
        (0x6ffffef5, 0),
        (0x6ffffff9, 0),
    ] {
        let mut b = fixture();
        put(&mut b, 152, 48);
        put(&mut b, 160, 48);
        put(&mut b, 400, tag);
        put(&mut b, 408, value);
        assert!(parse(b).is_ok(), "supported tag {tag}");
    }
    for tag in [9, 11, 21, 30] {
        let mut b = fixture();
        put(&mut b, 152, 48);
        put(&mut b, 160, 48);
        put(&mut b, 400, tag);
        put(&mut b, 408, 99);
        assert!(parse(b).is_err());
    }
    let mut b = fixture();
    put(&mut b, 152, 48);
    put(&mut b, 160, 48);
    assert!(parse(b.clone()).is_ok()); // Zero-only trailer permitted.
    put(&mut b, 416, 1);
    assert!(parse(b).is_err()); // Never active trailer tags.
}

#[test]
fn pf_27_s01_elf_profile_io_failures_are_sanitized() {
    struct Fault {
        cursor: Cursor<Vec<u8>>,
        fail_at: u64,
    }
    impl Read for Fault {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if self.cursor.position() >= self.fail_at {
                Err(io::Error::other("private detail"))
            } else {
                self.cursor.read(b)
            }
        }
    }
    impl Seek for Fault {
        fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(pos)
        }
    }
    for fail_at in [0, 64, 120, 384, 400] {
        let err = inspect(
            &mut Fault {
                cursor: Cursor::new(fixture()),
                fail_at,
            },
            1024,
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "ELF read failed");
    }
}

#[test]
#[ignore = "explicit RTX artifact paths required; parser only, no invocation"]
fn pf_27_s01_elf_profile_frozen_artifacts_as_data() {
    for (variable, expected) in [
        ("PF27_ELF_STATIC", true),
        ("PF27_ELF_GNU", false),
        ("PF27_ELF_INTERPRETER", false),
    ] {
        let path = std::env::var(variable).expect("record all three frozen artifact paths");
        let mut file = std::fs::File::open(path).unwrap();
        let size = file.metadata().unwrap().len();
        assert_eq!(inspect(&mut file, size).is_ok(), expected, "{variable}");
    }
}
