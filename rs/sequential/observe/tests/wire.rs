use super::*;

#[test]
fn encoding_has_fixed_tags_little_endian_words_and_no_padding() {
    let event = Event::Completed {
        steps: 12,
        value: [1, 2, 3, 4],
        remaining: 9,
    };
    let expected: Vec<u8> = [3u64, 12, 1, 2, 3, 4, 9]
        .into_iter()
        .flat_map(u64::to_le_bytes)
        .collect();
    assert_eq!(event.encode().as_bytes(), expected);
    let mut longest = 0;
    let p = [u64::MAX; 4];
    let reservation = BudgetReservation {
        parent: u64::MAX,
        child: u64::MAX,
    };
    let frames = [
        LiveFrame::Unary {
            opcode: 15,
            reservation,
        },
        LiveFrame::BinaryLeft {
            opcode: 2,
            object: p,
            right: p,
            budget: u64::MAX,
            first: u64::MAX,
            second: Some(u64::MAX),
        },
        LiveFrame::BinaryLeft {
            opcode: 2,
            object: p,
            right: p,
            budget: 0,
            first: 0,
            second: None,
        },
        LiveFrame::BinaryRight {
            opcode: 2,
            left: p,
            budget: u64::MAX,
            used: u64::MAX,
            second: u64::MAX,
        },
        LiveFrame::BranchTest {
            object: p,
            yes: p,
            no: p,
            reservation,
        },
        LiveFrame::BranchChosen(reservation),
        LiveFrame::Compose,
    ];
    for frame in frames {
        let action = LogicalAction::Enter {
            object: p,
            formula: p,
            budget: u64::MAX,
        };
        let event = Event::Transition(Transition {
            sequence: u64::MAX,
            before: action,
            after: action,
            depth_before: u32::MAX,
            depth_after: u32::MAX,
            popped: Some(frame),
            pushed: Some(frame),
            fresh_nodes: u32::MAX,
        });
        let encoded = event.encode();
        assert!(encoded.as_bytes().len() <= 512);
        longest = longest.max(encoded.as_bytes().len());
        assert_eq!(encoded.as_bytes(), event.encode().as_bytes());
    }
    assert_eq!(longest, 456);
}
