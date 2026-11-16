use super::*;
use nebu::Goldilocks;

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_nodes: 1024,
    max_depth: 128,
};

fn atom<const N: usize>(ar: &mut Reduction<N>, v: u64) -> Order {
    ar.atom(Goldilocks::new(v)).unwrap()
}

fn entries(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut pos = HEADER;
    let mut result = Vec::new();
    while pos < bytes.len() {
        let len = 33 + bytes[pos + 32] as usize;
        result.push(bytes[pos..pos + len].to_vec());
        pos += len;
    }
    result
}

fn assemble(root: &[u8], nodes: &[Vec<u8>]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(root);
    out.extend_from_slice(&(nodes.len() as u32).to_le_bytes());
    for node in nodes {
        out.extend_from_slice(node);
    }
    out
}

fn fixture() -> Vec<u8> {
    let mut ar = Reduction::<128>::new();
    let a = atom(&mut ar, 1);
    let b = atom(&mut ar, 2);
    let pair = ar.pair(a, b).unwrap();
    encode(&ar, pair, LIMITS).unwrap()
}

fn rejected(bytes: &[u8], limits: Limits) -> Error {
    let mut ar = Reduction::<128>::new();
    let sentinel = atom(&mut ar, 777);
    let before = ar.count();
    let err = decode(&mut ar, bytes, limits).unwrap_err();
    assert_eq!(
        ar.count(),
        before,
        "validation must finish before VM allocation"
    );
    assert_eq!(ar.atom_value(sentinel).unwrap().as_u64(), 777);
    err
}

#[test]
fn complete_roundtrip_preserves_topology_sharing_and_root_identity() {
    let mut ar = Reduction::<128>::new();
    let a = atom(&mut ar, 1);
    let b = atom(&mut ar, 2);
    let c = atom(&mut ar, 3);
    let ab = ar.pair(a, b).unwrap();
    let bc = ar.pair(b, c).unwrap();
    let l = ar.pair(ab, c).unwrap();
    let r = ar.pair(a, bc).unwrap();
    assert_ne!(
        encode(&ar, l, LIMITS).unwrap(),
        encode(&ar, r, LIMITS).unwrap()
    );
    let shared = ar.pair(l, l).unwrap();
    let bytes = encode(&ar, shared, LIMITS).unwrap();
    assert_eq!(entries(&bytes).len(), 6);
    let mut dest = Reduction::<128>::new();
    atom(&mut dest, 100);
    let loaded = decode(&mut dest, &bytes, LIMITS).unwrap();
    assert_eq!(ar.digest(shared), dest.digest(loaded));
    assert_eq!(dest.head(loaded), dest.tail(loaded));
    assert_eq!(encode(&dest, loaded, LIMITS).unwrap(), bytes);
}

#[test]
fn allocation_history_cannot_change_canonical_bytes() {
    let mut a = Reduction::<128>::new();
    let a1 = atom(&mut a, 1);
    let a2 = atom(&mut a, 2);
    let root_a = a.pair(a1, a2).unwrap();
    let mut b = Reduction::<128>::new();
    atom(&mut b, 99);
    let b2 = atom(&mut b, 2);
    let b1 = atom(&mut b, 1);
    let root_b = b.pair(b1, b2).unwrap();
    assert_ne!(root_a, root_b);
    assert_eq!(
        encode(&a, root_a, LIMITS).unwrap(),
        encode(&b, root_b, LIMITS).unwrap()
    );
    let list = entries(&encode(&a, root_a, LIMITS).unwrap());
    assert_eq!(&list[0][33..], &1u64.to_le_bytes());
    assert_eq!(&list[1][33..], &2u64.to_le_bytes());
}

#[test]
fn bounds_are_inclusive_and_checked_by_both_directions() {
    let bytes = fixture();
    let exact = Limits {
        max_bytes: bytes.len(),
        max_nodes: 3,
        max_depth: 1,
    };
    let mut ar = Reduction::<128>::new();
    let root = decode(&mut ar, &bytes, exact).unwrap();
    assert_eq!(encode(&ar, root, exact).unwrap(), bytes);
    for (limits, error) in [
        (
            Limits {
                max_bytes: bytes.len() - 1,
                ..exact
            },
            Error::ByteLimit,
        ),
        (
            Limits {
                max_nodes: 2,
                ..exact
            },
            Error::NodeLimit,
        ),
        (
            Limits {
                max_depth: 0,
                ..exact
            },
            Error::DepthLimit,
        ),
    ] {
        assert_eq!(rejected(&bytes, limits), error);
        assert_eq!(encode(&ar, root, limits), Err(error));
    }
    let atom_only = atom(&mut ar, 0);
    assert!(
        encode(
            &ar,
            atom_only,
            Limits {
                max_depth: 0,
                ..LIMITS
            }
        )
        .is_ok()
    );
}

#[test]
fn truncation_lengths_versions_and_trailing_bytes_are_rejected() {
    let bytes = fixture();
    for end in 0..bytes.len() {
        rejected(&bytes[..end], LIMITS);
    }
    let mut changed = bytes.clone();
    changed.push(0);
    assert_eq!(rejected(&changed, LIMITS), Error::Framing);
    let mut changed = bytes.clone();
    changed[7] = b'2';
    assert_eq!(rejected(&changed, LIMITS), Error::Version);
    let mut changed = bytes.clone();
    changed[HEADER + 32] = 16;
    assert_eq!(rejected(&changed, LIMITS), Error::Framing);
    let mut changed = bytes.clone();
    changed[40..44].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(rejected(&changed, LIMITS), Error::NodeLimit);
    let mut changed = bytes.clone();
    changed[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(rejected(&changed, LIMITS), Error::NodeLimit);
    assert_eq!(
        rejected(
            &changed,
            Limits {
                max_nodes: u32::MAX,
                ..LIMITS
            }
        ),
        Error::Framing
    );
}

#[test]
fn noncanonical_field_and_all_particle_locations_are_rejected() {
    let bytes = fixture();
    for start in [8, HEADER, HEADER + 2 * 41 + 33, HEADER + 2 * 41 + 33 + 32] {
        for limb in 0..4 {
            let at = start + limb * 8;
            let mut changed = bytes.clone();
            changed[at..at + 8].copy_from_slice(&MODULUS.to_le_bytes());
            assert_eq!(rejected(&changed, LIMITS), Error::FieldRange);
        }
    }
    for at in [
        8,
        HEADER,
        HEADER + 33,
        HEADER + 2 * 41 + 33,
        HEADER + 2 * 41 + 33 + 32,
    ] {
        let mut changed = bytes.clone();
        changed[at..at + 8].copy_from_slice(&MODULUS.to_le_bytes());
        assert_eq!(rejected(&changed, LIMITS), Error::FieldRange, "offset {at}");
    }
    let mut changed = bytes.clone();
    changed[HEADER + 33] ^= 4;
    assert_eq!(rejected(&changed, LIMITS), Error::HashMismatch);
    let mut changed = bytes;
    changed[HEADER] ^= 4;
    assert_eq!(rejected(&changed, LIMITS), Error::HashMismatch);
}

#[test]
fn duplicate_missing_forward_and_unreachable_nodes_are_rejected() {
    let bytes = fixture();
    let root = &bytes[8..40];
    let e = entries(&bytes);
    assert_eq!(
        rejected(
            &assemble(
                root,
                &[e[0].clone(), e[0].clone(), e[1].clone(), e[2].clone()]
            ),
            LIMITS
        ),
        Error::Duplicate
    );
    assert_eq!(
        rejected(&assemble(root, &[e[0].clone(), e[2].clone()]), LIMITS),
        Error::MissingReference
    );
    assert_eq!(
        rejected(
            &assemble(root, &[e[2].clone(), e[0].clone(), e[1].clone()]),
            LIMITS
        ),
        Error::MissingReference
    );
    assert_eq!(
        rejected(&assemble(root, &[e[0].clone(), e[1].clone()]), LIMITS),
        Error::MissingRoot
    );
    // All nodes valid/reachable and children before parent, but wrong DFS order.
    assert_eq!(
        rejected(
            &assemble(root, &[e[1].clone(), e[0].clone(), e[2].clone()]),
            LIMITS
        ),
        Error::Order
    );
    let mut ar = Reduction::<128>::new();
    let extra = atom(&mut ar, 999);
    let extra = entries(&encode(&ar, extra, LIMITS).unwrap()).remove(0);
    assert_eq!(
        rejected(
            &assemble(root, &[e[0].clone(), e[1].clone(), e[2].clone(), extra]),
            LIMITS
        ),
        Error::Order
    );
}

#[test]
fn compact_shared_dag_uses_longest_depth_without_leaf_expansion() {
    let mut ar = Reduction::<1024>::new();
    let mut root = atom(&mut ar, 5);
    for _ in 0..100 {
        root = ar.pair(root, root).unwrap();
    }
    let exact = Limits {
        max_depth: 100,
        ..LIMITS
    };
    let bytes = encode(&ar, root, exact).unwrap();
    assert_eq!(entries(&bytes).len(), 101);
    assert!(bytes.len() < 10_000);
    let mut dest = Reduction::<1024>::new();
    let loaded = decode(&mut dest, &bytes, exact).unwrap();
    assert_eq!(dest.count(), 101);
    assert_eq!(dest.digest(loaded), ar.digest(root));
    assert_eq!(
        rejected(
            &bytes,
            Limits {
                max_depth: 99,
                ..LIMITS
            }
        ),
        Error::DepthLimit
    );
}

#[test]
fn sharing_reached_first_on_a_shallow_path_does_not_hide_deeper_paths() {
    let mut ar = Reduction::<128>::new();
    let a = atom(&mut ar, 1);
    let mut deep = a;
    for _ in 0..5 {
        deep = ar.pair(a, deep).unwrap();
    }
    let root = ar.pair(a, deep).unwrap();
    let bytes = encode(&ar, root, LIMITS).unwrap();
    assert_eq!(
        encode(
            &ar,
            root,
            Limits {
                max_depth: 5,
                ..LIMITS
            }
        ),
        Err(Error::DepthLimit)
    );
    assert_eq!(
        rejected(
            &bytes,
            Limits {
                max_depth: 5,
                ..LIMITS
            }
        ),
        Error::DepthLimit
    );
}

#[test]
fn arena_failure_does_not_return_a_partial_root_or_change_old_nodes() {
    let bytes = fixture();
    let mut ar = Reduction::<8>::new();
    let sentinel = atom(&mut ar, 777);
    for v in 100..104 {
        atom(&mut ar, v);
    }
    assert_eq!(ar.count(), 5);
    assert_eq!(decode(&mut ar, &bytes, LIMITS), Err(Error::Allocation));
    assert_eq!(ar.count(), 6);
    assert_eq!(ar.atom_value(sentinel).unwrap().as_u64(), 777);
}
