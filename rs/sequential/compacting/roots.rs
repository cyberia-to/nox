use super::*;

pub(super) fn action(action: &mut Action, map: &mut impl FnMut(Order) -> Order) {
    match action {
        Action::Enter {
            object, formula, ..
        } => {
            *object = map(*object);
            *formula = map(*formula);
        }
        Action::Return(Outcome::Ok(value, _)) => *value = map(*value),
        Action::Return(_) => {}
    }
}

pub(super) fn frame(frame: &mut Frame, map: &mut impl FnMut(Order) -> Order) {
    match &mut frame.phase {
        Phase::BinaryLeft { b, .. } => {
            frame.row.r[1] = u64::from(map(frame.row.r[1] as Order));
            *b = map(*b);
        }
        Phase::BinaryRight { left, .. } => *left = map(*left),
        Phase::BranchTest { yes, no, .. } => {
            frame.row.r[1] = u64::from(map(frame.row.r[1] as Order));
            *yes = map(*yes);
            *no = map(*no);
        }
        Phase::Unary(_) | Phase::BranchChosen(_) | Phase::Compose => {}
    }
}

pub(super) fn allocation_headroom<const N: usize>(
    ar: &Reduction<N>,
    action: &Action,
    stack: &[Frame],
    max_frames: u32,
) -> u32 {
    if let Action::Enter {
        object,
        formula,
        budget,
    } = action
    {
        if stack.len() >= max_frames as usize {
            return 0;
        }
        // Axis zero returns hash data directly during Enter. A halted entry
        // performs no allocation and must not start collection.
        return if *budget >= 1
            && ar.get(*object).is_some()
            && ar
                .head(*formula)
                .and_then(|r| ar.atom_value(r))
                .is_some_and(|v| v.as_u64() == 0)
            && ar
                .tail(*formula)
                .and_then(|r| ar.atom_value(r))
                .is_some_and(|v| v.as_u64() == 0)
        {
            7
        } else {
            0
        };
    }
    let Action::Return(Outcome::Ok(value, _)) = action else {
        return 0;
    };
    // Missing external values fail before a constructor attempts admission.
    if ar.get(*value).is_none() {
        return 0;
    }
    let Some(frame) = stack.last() else {
        return 0;
    };
    match frame.phase {
        Phase::Unary(_) => match frame.row.r[0] {
            15 => 7,
            8 | 13 => 1,
            _ => 0,
        },
        Phase::BinaryRight { left, .. } if frame.row.r[0] != 2 && ar.get(left).is_some() => 1,
        _ => 0,
    }
}
