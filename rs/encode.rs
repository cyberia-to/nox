//! canonical encoding and wire format for nox data (specs/encoding.md)
//!
//! Model B (leaf-based, tag-free). data is stored by its field leaves:
//!   - atom  = 8 bytes  — one Goldilocks field, little-endian
//!   - pair  = 64 bytes — two child `particle`s (32 ‖ 32)
//!
//! node type is read from length (8 vs 64); no tag byte.
//!
//! identity is the tree hash, not a second scheme: `particle(data)` is the
//! 32-byte view of the hemera tree-hash digest (`hash_atom` for a leaf,
//! `hash_pair` for a node). the in-order hash-cons key and the wire particle
//! are the same bytes — one hash, by construction.

use alloc::vec::Vec;
use alloc::vec;
use nebu::Goldilocks;
use crate::data::{Reduction, Order, Data};
use crate::data::{digest_bytes, digest_from_bytes, hash_atom, hash_pair};

/// content-addressed identity: 32-byte view of the tree-hash digest
pub type Particle = [u8; 32];

/// error from decode or wire-parse operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    TruncatedPayload,
    TrailingBytes,
    FieldOutOfRange,
    MessageTooLarge,
    InvalidLength,
    HashMismatch,
    InvalidMessageType,
}

/// standalone decoded data not bound to an Reduction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedData {
    /// a leaf — one field. a `word` is just an atom whose value is < 2^32;
    /// the encoding does not distinguish them (the range is a refinement, not
    /// a tag). a hemera hash result is a pair-tree of atoms, not a leaf.
    Atom(Goldilocks),
    /// a node — two children referenced by `particle`
    Pair { left: Particle, right: Particle },
}

/// one verified entry from a wire message
#[derive(Debug, Clone)]
pub struct WireEntry {
    pub id:   Particle,
    pub data: DecodedData,
}

/// parsed wire message
#[derive(Debug, Clone)]
pub enum WireMessage {
    Push(Vec<WireEntry>),
    Request(Vec<Particle>),
    Response(Vec<WireEntry>),
}

// ── constants ────────────────────────────────────────────────────────────────

/// atom payload: one field, 8 bytes little-endian
const SIZE_ATOM: usize = 8;
/// pair payload: two particles, 32 ‖ 32
const SIZE_PAIR: usize = 64;

/// 16 MiB wire message cap (specs/encoding.md canonical invariant)
const MAX_WIRE_BYTES: usize = 1 << 24;

/// p = 2^64 - 2^32 + 1; values >= p are out of range for field elements
const GOLDILOCKS_P: u64 = 0xFFFF_FFFF_0000_0001;

// ── primitive encoders ───────────────────────────────────────────────────────

/// encode an atom: the field value, 8 bytes little-endian. tag-free.
pub fn encode_atom(value: Goldilocks) -> [u8; SIZE_ATOM] {
    value.as_u64().to_le_bytes()
}

/// encode a pair: its two children's particles, 32 ‖ 32. tag-free.
pub fn encode_pair(left: &Particle, right: &Particle) -> [u8; SIZE_PAIR] {
    let mut buf = [0u8; SIZE_PAIR];
    buf[0..32].copy_from_slice(left);
    buf[32..64].copy_from_slice(right);
    buf
}

// ── content-addressed identity ────────────────────────────────────────────────

/// particle of canonical encoded bytes — the tree hash of the node.
/// 8 bytes → leaf hash; 64 bytes → node hash of the two child particles.
pub fn particle_of(encoded: &[u8]) -> Result<Particle, DecodeError> {
    match encoded.len() {
        SIZE_ATOM => {
            let v = u64::from_le_bytes(encoded.try_into().unwrap());
            if v >= GOLDILOCKS_P {
                return Err(DecodeError::FieldOutOfRange);
            }
            Ok(digest_bytes(&hash_atom(Goldilocks::new(v))))
        }
        SIZE_PAIR => {
            let left:  Particle = encoded[0..32].try_into().unwrap();
            let right: Particle = encoded[32..64].try_into().unwrap();
            let ld = digest_from_bytes(&left);
            let rd = digest_from_bytes(&right);
            Ok(digest_bytes(&hash_pair(&ld, &rd)))
        }
        _ => Err(DecodeError::InvalidLength),
    }
}

// ── Reduction encoding ────────────────────────────────────────────────────────────

/// encode all data reachable from `root` in topological order (children before
/// parents). returns (Particle, encoded_bytes) pairs ready for wire transmission.
///
/// children in an Reduction always have smaller Order than their parents, so sorting
/// reachable order ids ascending gives topological order.
pub fn encode_tree<const N: usize>(
    reduction: &Reduction<N>,
    root: Order,
) -> Option<Vec<(Particle, Vec<u8>)>> {
    let reachable = collect_reachable(reduction, root)?;
    let mut cache: Vec<(Order, Particle)> = Vec::with_capacity(reachable.len());
    let mut result: Vec<(Particle, Vec<u8>)> = Vec::with_capacity(reachable.len());

    for &nid in &reachable {
        let bytes: Vec<u8> = match reduction.get(nid)?.inner {
            Data::Atom { value } => encode_atom(value).to_vec(),
            Data::Pair { left, right } => {
                let l = cache_lookup(&cache, left)?;
                let r = cache_lookup(&cache, right)?;
                encode_pair(&l, &r).to_vec()
            }
        };
        // identity is the node's already-computed tree-hash digest — the same
        // bytes the hash-cons table keys on. no re-hashing, no second scheme.
        let pid = digest_bytes(&reduction.get(nid)?.hash);
        cache.push((nid, pid));
        result.push((pid, bytes));
    }
    Some(result)
}

/// canonical encoded bytes of a single data node from an Reduction
pub fn encoded_bytes<const N: usize>(reduction: &Reduction<N>, id: Order) -> Option<Vec<u8>> {
    match reduction.get(id)?.inner {
        Data::Atom { value } => Some(encode_atom(value).to_vec()),
        Data::Pair { .. } => Some(encode_tree(reduction, id)?.into_iter().last()?.1),
    }
}

/// particle of a data node in an Reduction — the stored tree-hash digest.
pub fn particle_id<const N: usize>(reduction: &Reduction<N>, id: Order) -> Option<Particle> {
    Some(digest_bytes(&reduction.get(id)?.hash))
}

/// collect all order ids reachable from root via DFS, sorted ascending (topological order)
fn collect_reachable<const N: usize>(reduction: &Reduction<N>, root: Order) -> Option<Vec<Order>> {
    let count = reduction.count() as usize;
    if root as usize >= count {
        return None;
    }
    let mut visited = vec![false; count];
    let mut stack: Vec<Order> = Vec::new();
    let mut result: Vec<Order> = Vec::new();
    stack.push(root);
    while let Some(id) = stack.pop() {
        let idx = id as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        result.push(id);
        if let Data::Pair { left, right } = reduction.get(id)?.inner {
            stack.push(left);
            stack.push(right);
        }
    }
    result.sort_unstable();
    Some(result)
}

fn cache_lookup(cache: &[(Order, Particle)], id: Order) -> Option<Particle> {
    cache
        .binary_search_by_key(&id, |&(n, _)| n)
        .ok()
        .map(|i| cache[i].1)
}

// ── decode ────────────────────────────────────────────────────────────────────

/// decode canonical encoded bytes into a standalone data node with full validation.
/// node type is read from length: 8 = atom, 64 = pair. there is no tag byte.
pub fn decode(bytes: &[u8]) -> Result<DecodedData, DecodeError> {
    match bytes.len() {
        SIZE_ATOM => {
            let v = u64::from_le_bytes(bytes.try_into().unwrap());
            if v >= GOLDILOCKS_P {
                return Err(DecodeError::FieldOutOfRange);
            }
            Ok(DecodedData::Atom(Goldilocks::new(v)))
        }
        SIZE_PAIR => {
            let left:  Particle = bytes[0..32].try_into().unwrap();
            let right: Particle = bytes[32..64].try_into().unwrap();
            Ok(DecodedData::Pair { left, right })
        }
        _ => Err(DecodeError::InvalidLength),
    }
}

// ── wire format ───────────────────────────────────────────────────────────────

/// write a data_push message (type 0x10): sender pushes entries without a request
pub fn write_push(entries: &[(Particle, Vec<u8>)]) -> Option<Vec<u8>> {
    write_entries(0x10, entries)
}

/// write a data_request message (type 0x11): request data by Particle list
pub fn write_request(ids: &[Particle]) -> Option<Vec<u8>> {
    let payload_len = 1 + 4 + ids.len() * 32;
    if payload_len > MAX_WIRE_BYTES {
        return None;
    }
    let mut out = Vec::with_capacity(4 + payload_len);
    out.extend_from_slice(&(payload_len as u32).to_le_bytes());
    out.push(0x11);
    out.extend_from_slice(&(ids.len() as u32).to_le_bytes());
    for id in ids {
        out.extend_from_slice(id);
    }
    Some(out)
}

/// write a data_response message (type 0x12): response to a data_request
pub fn write_response(entries: &[(Particle, Vec<u8>)]) -> Option<Vec<u8>> {
    write_entries(0x12, entries)
}

fn write_entries(msg_type: u8, entries: &[(Particle, Vec<u8>)]) -> Option<Vec<u8>> {
    let entry_bytes: usize = entries.iter().map(|(_, b)| 32 + 1 + b.len()).sum();
    let payload_len = 1 + 4 + entry_bytes;
    if payload_len > MAX_WIRE_BYTES {
        return None;
    }
    let mut out = Vec::with_capacity(4 + payload_len);
    out.extend_from_slice(&(payload_len as u32).to_le_bytes());
    out.push(msg_type);
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (id, encoded) in entries {
        out.extend_from_slice(id);
        out.push(encoded.len() as u8);
        out.extend_from_slice(encoded);
    }
    Some(out)
}

/// parse a wire message: verify length prefix, type byte, and all entry hashes
pub fn parse_message(bytes: &[u8]) -> Result<WireMessage, DecodeError> {
    if bytes.len() < 4 {
        return Err(DecodeError::TruncatedPayload);
    }
    let payload_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    if payload_len > MAX_WIRE_BYTES {
        return Err(DecodeError::MessageTooLarge);
    }
    if bytes.len() < 4 + payload_len {
        return Err(DecodeError::TruncatedPayload);
    }
    if bytes.len() > 4 + payload_len {
        return Err(DecodeError::TrailingBytes);
    }
    let payload = &bytes[4..4 + payload_len];
    let msg_type = *payload.first().ok_or(DecodeError::TruncatedPayload)?;
    let body = &payload[1..];
    match msg_type {
        0x10 => parse_entries(body).map(WireMessage::Push),
        0x11 => parse_id_list(body).map(WireMessage::Request),
        0x12 => parse_entries(body).map(WireMessage::Response),
        _    => Err(DecodeError::InvalidMessageType),
    }
}

fn parse_entries(bytes: &[u8]) -> Result<Vec<WireEntry>, DecodeError> {
    if bytes.len() < 4 {
        return Err(DecodeError::TruncatedPayload);
    }
    let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let mut pos = 4;
    let mut entries: Vec<WireEntry> = Vec::new();
    for _ in 0..count {
        // 32-byte Particle + 1-byte length
        if pos + 33 > bytes.len() {
            return Err(DecodeError::TruncatedPayload);
        }
        let id: Particle = bytes[pos..pos + 32].try_into().unwrap();
        pos += 32;
        let entry_len = bytes[pos] as usize;
        pos += 1;
        if entry_len != SIZE_ATOM && entry_len != SIZE_PAIR {
            return Err(DecodeError::InvalidLength);
        }
        if pos + entry_len > bytes.len() {
            return Err(DecodeError::TruncatedPayload);
        }
        let encoded = &bytes[pos..pos + entry_len];
        pos += entry_len;
        // verify identity: recompute the tree hash from the bytes, compare
        let actual_id = particle_of(encoded)?;
        if actual_id != id {
            return Err(DecodeError::HashMismatch);
        }
        let data = decode(encoded)?;
        entries.push(WireEntry { id, data });
    }
    if pos != bytes.len() {
        return Err(DecodeError::TrailingBytes);
    }
    Ok(entries)
}

fn parse_id_list(bytes: &[u8]) -> Result<Vec<Particle>, DecodeError> {
    if bytes.len() < 4 {
        return Err(DecodeError::TruncatedPayload);
    }
    let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let expected = 4 + count * 32;
    if bytes.len() < expected {
        return Err(DecodeError::TruncatedPayload);
    }
    if bytes.len() > expected {
        return Err(DecodeError::TrailingBytes);
    }
    let mut ids: Vec<Particle> = Vec::with_capacity(count.min(512));
    for i in 0..count {
        let s = 4 + i * 32;
        ids.push(bytes[s..s + 32].try_into().unwrap());
    }
    Ok(ids)
}

// ── polynomial encoding (brakedown feature) ───────────────────────────────────

/// Particle for a multilinear polynomial: the Brakedown commitment hash.
///
/// Using the commitment as the content ID binds the data to the polynomial
/// opening proof rather than the raw evaluation table bytes, enabling
/// succinct polynomial data identity.
#[cfg(feature = "brakedown")]
pub fn poly_content_id(poly: &cyber_lens_core::MultilinearPoly<Goldilocks>) -> Particle {
    use cyber_lens_brakedown::Brakedown;
    use cyber_lens_core::Lens;
    let commitment = Brakedown::commit(poly);
    let mut id = [0u8; 32];
    id.copy_from_slice(commitment.as_bytes());
    id
}

/// Build a right-nested cons list of atoms from a multilinear polynomial.
///
/// Layout: `[e0 | [e1 | ... en-1]]` for n = 2^num_vars evaluations.
/// The jet 18 (poly_eval) reads this structure to reconstruct the polynomial.
#[cfg(feature = "brakedown")]
pub fn encode_poly<const N: usize>(
    reduction: &mut Reduction<N>,
    poly: &cyber_lens_core::MultilinearPoly<Goldilocks>,
) -> Option<Order> {
    let evals = &poly.evals;
    if evals.is_empty() {
        return None;
    }
    let mut cur = reduction.atom(*evals.last().unwrap())?;
    for &eval in evals[..evals.len() - 1].iter().rev() {
        let atom = reduction.atom(eval)?;
        cur = reduction.pair(atom, cur)?;
    }
    Some(cur)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Reduction;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn atom_encode_decode_roundtrip() {
        for v in [0u64, 1, 42, u32::MAX as u64, GOLDILOCKS_P - 1] {
            let encoded = encode_atom(g(v));
            assert_eq!(encoded.len(), 8);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, DecodedData::Atom(g(v)));
        }
    }

    /// field(v) and word(v) are the same atom — no tag distinguishes them.
    #[test]
    fn field_and_word_value_share_identity() {
        // a word-range value (42) encodes identically to the field 42
        assert_eq!(encode_atom(g(42)).to_vec(), encode_atom(g(42)).to_vec());
        assert_eq!(particle_of(&encode_atom(g(42))), particle_of(&encode_atom(g(42))));
    }

    #[test]
    fn pair_encode_decode_roundtrip() {
        let l = particle_of(&encode_atom(g(1))).unwrap();
        let r = particle_of(&encode_atom(g(2))).unwrap();
        let encoded = encode_pair(&l, &r);
        assert_eq!(encoded.len(), 64);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, DecodedData::Pair { left: l, right: r });
    }

    #[test]
    fn field_out_of_range_rejected() {
        // value = p is invalid
        let bytes = GOLDILOCKS_P.to_le_bytes();
        assert_eq!(decode(&bytes), Err(DecodeError::FieldOutOfRange));
    }

    #[test]
    fn invalid_length_rejected() {
        assert_eq!(decode(&[0u8; 9]), Err(DecodeError::InvalidLength));
        assert_eq!(decode(&[0u8; 65]), Err(DecodeError::InvalidLength));
        assert_eq!(decode(&[]), Err(DecodeError::InvalidLength));
    }

    #[test]
    fn particle_is_deterministic() {
        let encoded = encode_atom(g(42));
        assert_eq!(particle_of(&encoded), particle_of(&encoded));
    }

    /// the wire particle equals the in-order hash-cons key — ONE scheme.
    #[test]
    fn particle_id_matches_order_hash() {
        let mut reduction = Reduction::<1024>::new();
        let id = reduction.atom(g(7)).unwrap();
        let from_order = particle_id(&reduction, id).unwrap();
        let from_bytes = particle_of(&encode_atom(g(7))).unwrap();
        assert_eq!(from_order, from_bytes);
    }

    #[test]
    fn content_id_pair_matches_recompute() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(g(1)).unwrap();
        let b = reduction.atom(g(2)).unwrap();
        let c = reduction.pair(a, b).unwrap();
        let pid = particle_id(&reduction, c).unwrap();
        let left_id  = particle_of(&encode_atom(g(1))).unwrap();
        let right_id = particle_of(&encode_atom(g(2))).unwrap();
        assert_eq!(pid, particle_of(&encode_pair(&left_id, &right_id)).unwrap());
    }

    #[test]
    fn encode_tree_topological_order() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(g(10)).unwrap();
        let b = reduction.atom(g(20)).unwrap();
        let c = reduction.pair(a, b).unwrap();
        let tree = encode_tree(&reduction, c).unwrap();
        // children must appear before the pair
        assert_eq!(tree.len(), 3);
        let pair_entry = tree.last().unwrap();
        assert_eq!(pair_entry.1.len(), SIZE_PAIR);
    }

    #[test]
    fn encode_tree_shared_child_deduplication() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(g(99)).unwrap();
        let c = reduction.pair(a, a).unwrap();  // shared child
        let tree = encode_tree(&reduction, c).unwrap();
        assert_eq!(tree.len(), 2);  // atom appears once, pair appears once
    }

    #[test]
    fn wire_push_roundtrip() {
        let mut reduction = Reduction::<1024>::new();
        let id = reduction.atom(g(5)).unwrap();
        let tree = encode_tree(&reduction, id).unwrap();
        let msg_bytes = write_push(&tree).unwrap();
        match parse_message(&msg_bytes).unwrap() {
            WireMessage::Push(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].data, DecodedData::Atom(g(5)));
            }
            _ => panic!("expected Push"),
        }
    }

    #[test]
    fn wire_pair_roundtrip() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(g(3)).unwrap();
        let b = reduction.atom(g(4)).unwrap();
        let c = reduction.pair(a, b).unwrap();
        let tree = encode_tree(&reduction, c).unwrap();
        let msg_bytes = write_push(&tree).unwrap();
        match parse_message(&msg_bytes).unwrap() {
            WireMessage::Push(entries) => {
                assert_eq!(entries.len(), 3);  // a, b, pair
                // last entry is the pair, referencing the two atom particles
                let pa = particle_of(&encode_atom(g(3))).unwrap();
                let pb = particle_of(&encode_atom(g(4))).unwrap();
                assert_eq!(entries[2].data, DecodedData::Pair { left: pa, right: pb });
            }
            _ => panic!("expected Push"),
        }
    }

    #[test]
    fn wire_request_roundtrip() {
        let ids: Vec<Particle> = (0..3).map(|i| particle_of(&encode_atom(g(i))).unwrap()).collect();
        let msg = write_request(&ids).unwrap();
        match parse_message(&msg).unwrap() {
            WireMessage::Request(got) => assert_eq!(got, ids),
            _ => panic!("expected Request"),
        }
    }

    #[test]
    fn wire_hash_mismatch_rejected() {
        let encoded = encode_atom(g(1));
        let wrong_id = particle_of(&encode_atom(g(2))).unwrap();  // wrong id for this atom
        let entry = [(wrong_id, encoded.to_vec())];
        let msg = write_push(&entry).unwrap();
        match parse_message(&msg) {
            Err(DecodeError::HashMismatch) => {}
            other => panic!("expected HashMismatch, got {:?}", other),
        }
    }
}
