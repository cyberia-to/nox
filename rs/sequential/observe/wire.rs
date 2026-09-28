//! Canonical version-1 event encoding: little-endian u64 words, no padding.
use super::*;

pub struct EncodedEvent {
    bytes: [u8; 512],
    len: usize,
}
impl EncodedEvent {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
    fn word(&mut self, value: u64) {
        self.bytes[self.len..self.len + 8].copy_from_slice(&value.to_le_bytes());
        self.len += 8;
    }
    fn particle(&mut self, value: Particle) {
        for limb in value {
            self.word(limb);
        }
    }
    fn optional_word(&mut self, value: Option<u64>) {
        self.word(u64::from(value.is_some()));
        if let Some(value) = value {
            self.word(value);
        }
    }
    fn action(&mut self, value: LogicalAction) {
        match value {
            LogicalAction::Enter {
                object,
                formula,
                budget,
            } => {
                self.word(0);
                self.particle(object);
                self.particle(formula);
                self.word(budget);
            }
            LogicalAction::Return { value, remaining } => {
                self.word(1);
                self.particle(value);
                self.word(remaining);
            }
            LogicalAction::Halt { remaining } => {
                self.word(2);
                self.word(remaining);
            }
            LogicalAction::Error(error) => {
                self.word(3);
                self.word(error as u64);
            }
        }
    }
    fn reservation(&mut self, value: BudgetReservation) {
        self.word(value.parent);
        self.word(value.child);
    }
    fn frame(&mut self, frame: LiveFrame) {
        match frame {
            LiveFrame::Unary {
                opcode,
                reservation,
            } => {
                self.word(0);
                self.word(opcode);
                self.reservation(reservation);
            }
            LiveFrame::BinaryLeft {
                opcode,
                object,
                right,
                budget,
                first,
                second,
            } => {
                self.word(1);
                self.word(opcode);
                self.particle(object);
                self.particle(right);
                self.word(budget);
                self.word(first);
                self.optional_word(second);
            }
            LiveFrame::BinaryRight {
                opcode,
                left,
                budget,
                used,
                second,
            } => {
                self.word(2);
                self.word(opcode);
                self.particle(left);
                self.word(budget);
                self.word(used);
                self.word(second);
            }
            LiveFrame::BranchTest {
                object,
                yes,
                no,
                reservation,
            } => {
                self.word(3);
                self.particle(object);
                self.particle(yes);
                self.particle(no);
                self.reservation(reservation);
            }
            LiveFrame::BranchChosen(reservation) => {
                self.word(4);
                self.reservation(reservation);
            }
            LiveFrame::Compose => self.word(5),
        }
    }
    fn optional_frame(&mut self, frame: Option<LiveFrame>) {
        self.word(u64::from(frame.is_some()));
        if let Some(frame) = frame {
            self.frame(frame);
        }
    }
}

impl Event {
    /// Uses constant stack storage; the returned slice excludes unused bytes.
    pub fn encode(&self) -> EncodedEvent {
        let mut out = EncodedEvent {
            bytes: [0; 512],
            len: 0,
        };
        match *self {
            Self::Begin {
                version,
                initial,
                initial_nodes,
                max_frames,
                max_total_allocations,
                max_collection_work,
                resident_limit,
            } => {
                out.word(0);
                out.word(version);
                out.action(initial);
                out.word(u64::from(initial_nodes));
                out.word(u64::from(max_frames));
                out.word(max_total_allocations);
                out.word(max_collection_work);
                out.word(u64::from(resident_limit));
            }
            Self::Node(node) => {
                out.word(1);
                out.particle(node.particle);
                match node.value {
                    NodeValue::Atom(value) => {
                        out.word(0);
                        out.word(value);
                    }
                    NodeValue::Pair { left, right } => {
                        out.word(1);
                        out.particle(left);
                        out.particle(right);
                    }
                }
                out.word(u64::from(node.bound.is_dynamic()));
                out.word(node.bound.value());
            }
            Self::Transition(t) => {
                out.word(2);
                out.word(t.sequence);
                out.action(t.before);
                out.action(t.after);
                out.word(u64::from(t.depth_before));
                out.word(u64::from(t.depth_after));
                out.optional_frame(t.popped);
                out.optional_frame(t.pushed);
                out.word(u64::from(t.fresh_nodes));
            }
            Self::Completed {
                steps,
                value,
                remaining,
            } => {
                out.word(3);
                out.word(steps);
                out.particle(value);
                out.word(remaining);
            }
        }
        out
    }
}
