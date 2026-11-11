use super::MAX_SUBSTITUTION_DEPTH;
use super::Reading;
use super::SimpleCommand;
use super::simple_commands_read;
use pretty_assertions::assert_eq;
use std::collections::HashSet;

/// Compare the actual point-of-use result with main's uncached algorithm.
/// Every core unit test that closes a linked substitution runs this oracle.
pub(super) fn assert_reference_top(commands: &[SimpleCommand], first: usize, actual: &[usize]) {
    let nested: HashSet<usize> = commands[first..]
        .iter()
        .flat_map(|command| command.fed_by.iter().copied())
        .collect();
    let expected: Vec<usize> = (first..commands.len())
        .filter(|index| !nested.contains(index))
        .collect();
    assert_eq!(actual, expected);
}

fn chain(depth: usize) -> String {
    let mut script = "echo leaf".to_owned();
    for _ in 0..depth {
        script = format!("printf %s $({script})");
    }
    script
}

#[test]
fn cache_matches_reference_across_readings_and_depth_boundary() {
    let mut cases = vec![
        "echo $(printf one) $(printf $(echo two)); echo $(cd /work && pwd)".to_owned(),
        "echo $(printf one >(cat)); echo $(printf two <(cat))".to_owned(),
        "cat input | echo $(printf $(echo value)) >output".to_owned(),
        "cat input | echo $(printf $(echo value".to_owned(),
        "echo $((1 + $(echo 2))); echo `printf $(echo three)`".to_owned(),
        "echo $(>log echo hidden); { cd /tmp; } >log; echo $(pwd)".to_owned(),
        "echo $(printf α中🙂); echo $(cat <input); echo $(echo {a,b})".to_owned(),
    ];
    for depth in [
        0,
        1,
        MAX_SUBSTITUTION_DEPTH,
        MAX_SUBSTITUTION_DEPTH + 1,
        2_000,
    ] {
        cases.push(chain(depth));
    }
    for reading in [
        Reading::Legacy,
        Reading::Redirections,
        Reading::LiteralBraces,
    ] {
        for script in &cases {
            let lexed = simple_commands_read(std::slice::from_ref(script), reading);
            for (consumer, command) in lexed.commands.iter().enumerate() {
                assert!(command.fed_by.iter().all(|&source| source < consumer));
            }
        }
        for depth in [MAX_SUBSTITUTION_DEPTH, MAX_SUBSTITUTION_DEPTH + 1, 2_000] {
            let lexed = simple_commands_read(&[chain(depth)], reading);
            assert_eq!(lexed.too_deep, depth > MAX_SUBSTITUTION_DEPTH);
        }
    }
}

#[test]
#[ignore = "manual membership microbenchmark; not an end-to-end latency claim"]
fn membership_microbenchmark() {
    use std::hint::black_box;
    use std::time::Instant;

    // Actual depth-capped parser graph; both paths include membership creation.
    // Revisit each prefix as nested substitutions close, not arbitrary cached
    // lookup without the cost of registering completed commands.
    let lexed = simple_commands_read(&[chain(MAX_SUBSTITUTION_DEPTH)], Reading::LiteralBraces);
    let commands = &lexed.commands;
    let repeats = 2_000;
    let start = Instant::now();
    let mut reference_count = 0;
    for _ in 0..repeats {
        for end in 1..=commands.len() {
            let nested: HashSet<usize> = commands[..end]
                .iter()
                .flat_map(|command| command.fed_by.iter().copied())
                .collect();
            reference_count += black_box((0..end).filter(|index| !nested.contains(index)).count());
        }
    }
    let reference = start.elapsed();
    let start = Instant::now();
    let mut cached_count = 0;
    for _ in 0..repeats {
        let mut feeds_finished = Vec::with_capacity(commands.len());
        for (end, command) in commands.iter().enumerate() {
            for &index in &command.fed_by {
                feeds_finished[index] = true;
            }
            feeds_finished.push(false);
            cached_count += black_box((0..=end).filter(|&index| !feeds_finished[index]).count());
        }
    }
    let cached = start.elapsed();
    assert_eq!(cached_count, reference_count);
    eprintln!(
        "depth={MAX_SUBSTITUTION_DEPTH} repeats={repeats} uncached={reference:?} cached={cached:?}"
    );
}
