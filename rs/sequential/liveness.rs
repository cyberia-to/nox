//! Test-only, bounded reachability observation; never collects or changes data.
extern crate std;
use super::*;
use crate::{ErrorKind, data::Data};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
struct Roots {
    ids: [Order; 4],
    len: usize,
}

impl Roots {
    fn empty() -> Self {
        Self {
            ids: [crate::NIL; 4],
            len: 0,
        }
    }
    fn add(&mut self, id: Order) {
        assert!(self.len < self.ids.len());
        self.ids[self.len] = id;
        self.len += 1;
    }
    fn each(self, mut visit: impl FnMut(Order)) {
        for id in &self.ids[..self.len] {
            visit(*id);
        }
    }
}

fn frame_roots(frame: &Frame) -> Roots {
    let mut roots = Roots::empty();
    match frame.phase {
        Phase::BinaryLeft { b, .. } => {
            roots.add(frame.row.r[1] as Order);
            roots.add(b);
        }
        Phase::BinaryRight { left, .. } => roots.add(left),
        Phase::BranchTest { yes, no, .. } => {
            roots.add(frame.row.r[1] as Order);
            roots.add(yes);
            roots.add(no);
        }
        Phase::Unary(_) | Phase::BranchChosen(_) | Phase::Compose => {}
    }
    roots
}

fn step_roots(action: &Action, stack: &[Frame]) -> Roots {
    let mut roots = Roots::empty();
    match action {
        Action::Enter {
            object, formula, ..
        } => {
            roots.add(*object);
            roots.add(*formula);
        }
        Action::Return(Outcome::Ok(result, _)) => {
            roots.add(*result);
            if let Some(frame) = stack.last() {
                frame_roots(frame).each(|id| roots.add(id));
            }
        }
        Action::Return(_) => {}
    }
    roots
}

struct Census {
    live: u32,
    bitset_bytes: usize,
    polls: u64,
    elapsed_micros: u128,
}

fn census<const N: usize>(
    ar: &Reduction<N>,
    pinned: u32,
    stack: &[Frame],
    active: Roots,
    deadline: Instant,
) -> Result<Census, &'static str> {
    let started = Instant::now();
    let count = ar.count();
    if pinned > count {
        return Err("pinned range exceeds arena");
    }
    let words = (count as usize).div_ceil(64);
    let mut marks = Vec::new();
    marks
        .try_reserve_exact(words)
        .map_err(|_| "mark allocation failed")?;
    marks.resize(words, 0u64);
    let mut invalid = false;
    let mut mark = |id: Order| {
        if id >= count {
            invalid = true;
            return;
        }
        marks[id as usize / 64] |= 1u64 << (id % 64);
    };
    for id in 0..pinned {
        mark(id);
    }
    for frame in stack {
        frame_roots(frame).each(&mut mark);
    }
    active.each(&mut mark);
    if invalid {
        return Err("invalid continuation root");
    }
    let mut polls = 0;
    for id in (0..count).rev() {
        if id % 4096 == 0 {
            polls += 1;
            if Instant::now() >= deadline {
                return Err("liveness deadline exceeded");
            }
        }
        if let Data::Pair { left, right } = ar.get(id).ok_or("missing allocated node")?.inner {
            if left >= id || right >= id {
                return Err("non-topological allocated pair");
            }
            if marks[id as usize / 64] & (1u64 << (id % 64)) == 0 {
                continue;
            }
            for child in [left, right] {
                marks[child as usize / 64] |= 1u64 << (child % 64);
            }
        }
    }
    let live = marks.iter().map(|word| word.count_ones()).sum();
    Ok(Census {
        live,
        bitset_bytes: words * 8,
        polls,
        elapsed_micros: started.elapsed().as_micros(),
    })
}

pub(super) struct Probe {
    output: Option<PathBuf>,
    pinned: u32,
    prior: Roots,
    deadline: Instant,
    sampled: bool,
}

impl Probe {
    pub(super) fn new(pinned: u32) -> Self {
        Self {
            output: std::env::var_os("NOX_LIVENESS_OUTPUT").map(PathBuf::from),
            pinned,
            prior: Roots::empty(),
            deadline: Instant::now() + Duration::from_secs(1200),
            sampled: false,
        }
    }

    pub(super) fn before_step<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        stack: &[Frame],
    ) {
        if self.output.is_none() || self.sampled {
            return;
        }
        if matches!(
            action,
            Action::Return(Outcome::Error(ErrorKind::Unavailable))
        ) {
            let c =
                census(ar, self.pinned, stack, self.prior, self.deadline).expect("bounded census");
            let report = std::format!(
                "{{\n  \"schema\": \"nox/liveness-census/v1\",\n  \"total_nodes\": {},\n  \"live_nodes\": {},\n  \"unreachable_nodes\": {},\n  \"pinned_nodes\": {},\n  \"pending_frames\": {},\n  \"pending_frame_bytes\": {},\n  \"retry_root_slots\": {},\n  \"retry_root_orders\": {:?},\n  \"mark_buffer_bytes\": {},\n  \"traversal_stack_bytes\": 0,\n  \"root_snapshot_bytes\": {},\n  \"marker_deadline_polls\": {},\n  \"marker_elapsed_micros\": {},\n  \"cache_excluded_from_roots\": true\n}}\n",
                ar.count(),
                c.live,
                ar.count() - c.live,
                self.pinned,
                stack.len(),
                core::mem::size_of_val(stack),
                self.prior.len,
                &self.prior.ids[..self.prior.len],
                c.bitset_bytes,
                core::mem::size_of::<Roots>(),
                c.polls,
                c.elapsed_micros,
            );
            std::fs::write(self.output.as_ref().unwrap(), report).expect("write census receipt");
            self.sampled = true;
        }
        self.prior = step_roots(action, stack);
    }
}

#[cfg(test)]
mod tests;
