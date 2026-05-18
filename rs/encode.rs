//! canonical encoding and wire format for nox nouns (specs/encoding.md)
//!
//! three layers: storage encoding (tag + payload), content-addressed identity
//! (NounId = hemera(encoded)), and wire format (framed messages).

use alloc::vec::Vec;
use alloc::vec;
use nebu::Goldilocks;
use crate::noun::{Order, NounId, Noun, Tag};

/// content-addressed identity: hemera(canonical_encoded_bytes), 32 bytes
pub type ContentId = [u8; 32];

/// error from decode or wire-parse operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    UnknownTag,
    TruncatedPayload,
    TrailingBytes,
    FieldOutOfRange,
    WordOutOfRange,
    MessageTooLarge,
    InvalidLength,
    HashMismatch,
    InvalidMessageType,
}

/// standalone decoded noun not bound to an Order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedNoun {
    Field(Goldilocks),
    /// value is guaranteed < 2^32
    Word(Goldilocks),
    Hash([Goldilocks; 4]),
    Cell { left: ContentId, right: ContentId },
}

/// one verified entry from a wire message
#[derive(Debug, Clone)]
pub struct WireEntry {
    pub id:   ContentId,
    pub noun: DecodedNoun,
}

/// parsed wire message
#[derive(Debug, Clone)]
pub enum WireMessage {
    Push(Vec<WireEntry>),
    Request(Vec<ContentId>),
    Response(Vec<WireEntry>),
}

// ── constants ────────────────────────────────────────────────────────────────

const TAG_FIELD: u8 = 0x00;
const TAG_WORD:  u8 = 0x01;
const TAG_HASH:  u8 = 0x02;
const TAG_CELL:  u8 = 0x03;

const SIZE_FIELD: usize = 9;
const SIZE_WORD:  usize = 9;
const SIZE_HASH:  usize = 33;
const SIZE_CELL:  usize = 65;

/// 16 MiB wire message cap (specs/encoding.md canonical invariant 10)
const MAX_WIRE_BYTES: usize = 1 << 24;

/// p = 2^64 - 2^32 + 1; values >= p are out of range for field elements
const GOLDILOCKS_P: u64 = 0xFFFF_FFFF_0000_0001;

// ── primitive encoders ───────────────────────────────────────────────────────

pub fn encode_field(value: Goldilocks) -> [u8; SIZE_FIELD] {
    let mut buf = [0u8; SIZE_FIELD];
    buf[0] = TAG_FIELD;
    buf[1..9].copy_from_slice(&value.as_u64().to_le_bytes());
    buf
}

pub fn encode_word(value: Goldilocks) -> [u8; SIZE_WORD] {
    let mut buf = [0u8; SIZE_WORD];
    buf[0] = TAG_WORD;
    buf[1..9].copy_from_slice(&value.as_u64().to_le_bytes());
    buf
}

/// encode a 4-element hemera hash as a wire hash atom (tag 0x02)
pub fn encode_hash(elems: &[Goldilocks; 4]) -> [u8; SIZE_HASH] {
    let mut buf = [0u8; SIZE_HASH];
    buf[0] = TAG_HASH;
    for (i, e) in elems.iter().enumerate() {
        let s = 1 + i * 8;
        buf[s..s + 8].copy_from_slice(&e.as_u64().to_le_bytes());
    }
    buf
}

pub fn encode_cell(left: &ContentId, right: &ContentId) -> [u8; SIZE_CELL] {
    let mut buf = [0u8; SIZE_CELL];
    buf[0] = TAG_CELL;
    buf[1..33].copy_from_slice(left);
    buf[33..65].copy_from_slice(right);
    buf
}

// ── content-addressed identity ────────────────────────────────────────────────

/// content-addressed identity of canonical encoded bytes
pub fn noun_id(encoded: &[u8]) -> ContentId {
    *hemera::hash(encoded).as_bytes()
}

// ── Order encoding ────────────────────────────────────────────────────────────

/// encode all nouns reachable from `root` in topological order (children before
/// parents). returns (ContentId, encoded_bytes) pairs ready for wire transmission.
///
/// children in an Order always have smaller NounId than their parents, so sorting
/// reachable NounIds ascending gives topological order.
pub fn encode_tree<const N: usize>(
    order: &Order<N>,
    root: NounId,
) -> Option<Vec<(ContentId, Vec<u8>)>> {
    let reachable = collect_reachable(order, root)?;
    let mut cache: Vec<(NounId, ContentId)> = Vec::with_capacity(reachable.len());
    let mut result: Vec<(ContentId, Vec<u8>)> = Vec::with_capacity(reachable.len());

    for &nid in &reachable {
        let bytes: Vec<u8> = match order.get(nid)?.inner {
            Noun::Atom { value, tag: Tag::Field } => encode_field(value).to_vec(),
            Noun::Atom { value, tag: Tag::Word  } => encode_word(value).to_vec(),
            Noun::Cell { left, right } => {
                let l = cache_lookup(&cache, left)?;
                let r = cache_lookup(&cache, right)?;
                encode_cell(&l, &r).to_vec()
            }
        };
        let cid = noun_id(&bytes);
        cache.push((nid, cid));
        result.push((cid, bytes));
    }
    Some(result)
}

/// canonical encoded bytes of a single noun from an Order
pub fn encoded_bytes<const N: usize>(order: &Order<N>, id: NounId) -> Option<Vec<u8>> {
    match order.get(id)?.inner {
        Noun::Atom { value, tag: Tag::Field } => Some(encode_field(value).to_vec()),
        Noun::Atom { value, tag: Tag::Word  } => Some(encode_word(value).to_vec()),
        Noun::Cell { .. } => Some(encode_tree(order, id)?.into_iter().last()?.1),
    }
}

/// content-addressed identity of a noun in an Order
pub fn content_id<const N: usize>(order: &Order<N>, id: NounId) -> Option<ContentId> {
    match order.get(id)?.inner {
        Noun::Atom { value, tag: Tag::Field } => Some(noun_id(&encode_field(value))),
        Noun::Atom { value, tag: Tag::Word  } => Some(noun_id(&encode_word(value))),
        Noun::Cell { .. } => Some(encode_tree(order, id)?.into_iter().last()?.0),
    }
}

/// collect all NounIds reachable from root via DFS, sorted ascending (topological order)
fn collect_reachable<const N: usize>(order: &Order<N>, root: NounId) -> Option<Vec<NounId>> {
    let count = order.count() as usize;
    if root as usize >= count {
        return None;
    }
    let mut visited = vec![false; count];
    let mut stack: Vec<NounId> = Vec::new();
    let mut result: Vec<NounId> = Vec::new();
    stack.push(root);
    while let Some(id) = stack.pop() {
        let idx = id as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        result.push(id);
        if let Noun::Cell { left, right } = order.get(id)?.inner {
            stack.push(left);
            stack.push(right);
        }
    }
    result.sort_unstable();
    Some(result)
}

fn cache_lookup(cache: &[(NounId, ContentId)], id: NounId) -> Option<ContentId> {
    cache
        .binary_search_by_key(&id, |&(n, _)| n)
        .ok()
        .map(|i| cache[i].1)
}

// ── decode ────────────────────────────────────────────────────────────────────

/// decode canonical encoded bytes into a standalone noun with full validation
pub fn decode(bytes: &[u8]) -> Result<DecodedNoun, DecodeError> {
    let tag = bytes.first().ok_or(DecodeError::TruncatedPayload)?;
    match *tag {
        TAG_FIELD => {
            if bytes.len() < SIZE_FIELD {
                return Err(DecodeError::TruncatedPayload);
            }
            if bytes.len() > SIZE_FIELD {
                return Err(DecodeError::TrailingBytes);
            }
            let v = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            if v >= GOLDILOCKS_P {
                return Err(DecodeError::FieldOutOfRange);
            }
            Ok(DecodedNoun::Field(Goldilocks::new(v)))
        }
        TAG_WORD => {
            if bytes.len() < SIZE_WORD {
                return Err(DecodeError::TruncatedPayload);
            }
            if bytes.len() > SIZE_WORD {
                return Err(DecodeError::TrailingBytes);
            }
            let v = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            if v >= (1u64 << 32) {
                return Err(DecodeError::WordOutOfRange);
            }
            Ok(DecodedNoun::Word(Goldilocks::new(v)))
        }
        TAG_HASH => {
            if bytes.len() < SIZE_HASH {
                return Err(DecodeError::TruncatedPayload);
            }
            if bytes.len() > SIZE_HASH {
                return Err(DecodeError::TrailingBytes);
            }
            let mut elems = [Goldilocks::ZERO; 4];
            for (i, elem) in elems.iter_mut().enumerate() {
                let s = 1 + i * 8;
                let v = u64::from_le_bytes(bytes[s..s + 8].try_into().unwrap());
                if v >= GOLDILOCKS_P {
                    return Err(DecodeError::FieldOutOfRange);
                }
                *elem = Goldilocks::new(v);
            }
            Ok(DecodedNoun::Hash(elems))
        }
        TAG_CELL => {
            if bytes.len() < SIZE_CELL {
                return Err(DecodeError::TruncatedPayload);
            }
            if bytes.len() > SIZE_CELL {
                return Err(DecodeError::TrailingBytes);
            }
            let left:  ContentId = bytes[1..33].try_into().unwrap();
            let right: ContentId = bytes[33..65].try_into().unwrap();
            Ok(DecodedNoun::Cell { left, right })
        }
        _ => Err(DecodeError::UnknownTag),
    }
}

// ── wire format ───────────────────────────────────────────────────────────────

/// write a noun_push message (type 0x10): sender pushes entries without a request
pub fn write_push(entries: &[(ContentId, Vec<u8>)]) -> Option<Vec<u8>> {
    write_entries(0x10, entries)
}

/// write a noun_request message (type 0x11): request nouns by ContentId list
pub fn write_request(ids: &[ContentId]) -> Option<Vec<u8>> {
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

/// write a noun_response message (type 0x12): response to a noun_request
pub fn write_response(entries: &[(ContentId, Vec<u8>)]) -> Option<Vec<u8>> {
    write_entries(0x12, entries)
}

fn write_entries(msg_type: u8, entries: &[(ContentId, Vec<u8>)]) -> Option<Vec<u8>> {
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
        // 32-byte ContentId + 1-byte length
        if pos + 33 > bytes.len() {
            return Err(DecodeError::TruncatedPayload);
        }
        let id: ContentId = bytes[pos..pos + 32].try_into().unwrap();
        pos += 32;
        let entry_len = bytes[pos] as usize;
        pos += 1;
        if entry_len != 9 && entry_len != 33 && entry_len != 65 {
            return Err(DecodeError::InvalidLength);
        }
        if pos + entry_len > bytes.len() {
            return Err(DecodeError::TruncatedPayload);
        }
        let encoded = &bytes[pos..pos + entry_len];
        pos += entry_len;
        let actual_id = noun_id(encoded);
        if actual_id != id {
            return Err(DecodeError::HashMismatch);
        }
        let noun = decode(encoded)?;
        entries.push(WireEntry { id, noun });
    }
    if pos != bytes.len() {
        return Err(DecodeError::TrailingBytes);
    }
    Ok(entries)
}

fn parse_id_list(bytes: &[u8]) -> Result<Vec<ContentId>, DecodeError> {
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
    let mut ids: Vec<ContentId> = Vec::with_capacity(count.min(512));
    for i in 0..count {
        let s = 4 + i * 32;
        ids.push(bytes[s..s + 32].try_into().unwrap());
    }
    Ok(ids)
}

// ── polynomial encoding (brakedown feature) ───────────────────────────────────

/// ContentId for a multilinear polynomial: the Brakedown commitment hash.
///
/// Using the commitment as the content ID binds the noun to the polynomial
/// opening proof rather than the raw evaluation table bytes, enabling
/// succinct polynomial noun identity.
#[cfg(feature = "brakedown")]
pub fn poly_content_id(poly: &cyb_lens_core::MultilinearPoly<Goldilocks>) -> ContentId {
    use cyb_lens_brakedown::Brakedown;
    use cyb_lens_core::Lens;
    let commitment = Brakedown::commit(poly);
    let mut id = [0u8; 32];
    id.copy_from_slice(commitment.as_bytes());
    id
}

/// Build a right-nested cons list of Field atoms from a multilinear polynomial.
///
/// Layout: `[e0 | [e1 | ... en-1]]` for n = 2^num_vars evaluations.
/// The jet 18 (poly_eval) reads this structure to reconstruct the polynomial.
#[cfg(feature = "brakedown")]
pub fn encode_poly<const N: usize>(
    order: &mut Order<N>,
    poly: &cyb_lens_core::MultilinearPoly<Goldilocks>,
) -> Option<NounId> {
    let evals = &poly.evals;
    if evals.is_empty() {
        return None;
    }
    let mut cur = order.atom(*evals.last().unwrap(), Tag::Field)?;
    for &eval in evals[..evals.len() - 1].iter().rev() {
        let atom = order.atom(eval, Tag::Field)?;
        cur = order.cell(atom, cur)?;
    }
    Some(cur)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noun::{Order, Tag};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn field_atom_encode_decode_roundtrip() {
        for v in [0u64, 1, 42, u32::MAX as u64, GOLDILOCKS_P - 1] {
            let encoded = encode_field(g(v));
            assert_eq!(encoded[0], 0x00);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, DecodedNoun::Field(g(v)));
        }
    }

    #[test]
    fn word_atom_encode_decode_roundtrip() {
        for v in [0u64, 1, 255, u32::MAX as u64] {
            let encoded = encode_word(g(v));
            assert_eq!(encoded[0], 0x01);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, DecodedNoun::Word(g(v)));
        }
    }

    #[test]
    fn hash_atom_encode_decode_roundtrip() {
        let elems = [g(1), g(2), g(3), g(4)];
        let encoded = encode_hash(&elems);
        assert_eq!(encoded[0], 0x02);
        assert_eq!(encoded.len(), 33);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, DecodedNoun::Hash(elems));
    }

    #[test]
    fn field_out_of_range_rejected() {
        // value = p is invalid
        let mut bytes = [0u8; 9];
        bytes[0] = 0x00;
        bytes[1..9].copy_from_slice(&GOLDILOCKS_P.to_le_bytes());
        assert_eq!(decode(&bytes), Err(DecodeError::FieldOutOfRange));
    }

    #[test]
    fn word_out_of_range_rejected() {
        let mut bytes = [0u8; 9];
        bytes[0] = 0x01;
        let too_big: u64 = 1u64 << 32;
        bytes[1..9].copy_from_slice(&too_big.to_le_bytes());
        assert_eq!(decode(&bytes), Err(DecodeError::WordOutOfRange));
    }

    #[test]
    fn unknown_tag_rejected() {
        let bytes = [0x04u8, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(decode(&bytes), Err(DecodeError::UnknownTag));
    }

    #[test]
    fn noun_id_is_deterministic() {
        let encoded = encode_field(g(42));
        assert_eq!(noun_id(&encoded), noun_id(&encoded));
    }

    #[test]
    fn content_id_field_atom() {
        let mut order = Order::<1024>::new();
        let id = order.atom(g(7), Tag::Field).unwrap();
        let cid = content_id(&order, id).unwrap();
        assert_eq!(cid, noun_id(&encode_field(g(7))));
    }

    #[test]
    fn content_id_cell() {
        let mut order = Order::<1024>::new();
        let a = order.atom(g(1), Tag::Field).unwrap();
        let b = order.atom(g(2), Tag::Field).unwrap();
        let c = order.cell(a, b).unwrap();
        let cid = content_id(&order, c).unwrap();
        let left_id  = noun_id(&encode_field(g(1)));
        let right_id = noun_id(&encode_field(g(2)));
        assert_eq!(cid, noun_id(&encode_cell(&left_id, &right_id)));
    }

    #[test]
    fn encode_tree_topological_order() {
        let mut order = Order::<1024>::new();
        let a = order.atom(g(10), Tag::Field).unwrap();
        let b = order.atom(g(20), Tag::Field).unwrap();
        let c = order.cell(a, b).unwrap();
        let tree = encode_tree(&order, c).unwrap();
        // children must appear before the cell
        assert_eq!(tree.len(), 3);
        let cell_entry = tree.last().unwrap();
        assert_eq!(cell_entry.1[0], TAG_CELL);
    }

    #[test]
    fn encode_tree_shared_child_deduplication() {
        let mut order = Order::<1024>::new();
        let a = order.atom(g(99), Tag::Field).unwrap();
        let c = order.cell(a, a).unwrap();  // shared child
        let tree = encode_tree(&order, c).unwrap();
        assert_eq!(tree.len(), 2);  // atom appears once, cell appears once
    }

    #[test]
    fn wire_push_roundtrip() {
        let mut order = Order::<1024>::new();
        let id = order.atom(g(5), Tag::Field).unwrap();
        let tree = encode_tree(&order, id).unwrap();
        let msg_bytes = write_push(&tree).unwrap();
        match parse_message(&msg_bytes).unwrap() {
            WireMessage::Push(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].noun, DecodedNoun::Field(g(5)));
            }
            _ => panic!("expected Push"),
        }
    }

    #[test]
    fn wire_request_roundtrip() {
        let ids: Vec<ContentId> = (0..3).map(|i| noun_id(&encode_field(g(i)))).collect();
        let msg = write_request(&ids).unwrap();
        match parse_message(&msg).unwrap() {
            WireMessage::Request(got) => assert_eq!(got, ids),
            _ => panic!("expected Request"),
        }
    }

    #[test]
    fn wire_hash_mismatch_rejected() {
        let encoded = encode_field(g(1));
        let wrong_id = noun_id(&encode_field(g(2)));  // wrong id for this noun
        let entry = [(wrong_id, encoded.to_vec())];
        let msg = write_push(&entry).unwrap();
        match parse_message(&msg) {
            Err(DecodeError::HashMismatch) => {}
            other => panic!("expected HashMismatch, got {:?}", other),
        }
    }

    #[test]
    fn wire_message_too_large_rejected() {
        // build a request that would exceed 16 MiB
        let count = (MAX_WIRE_BYTES / 32) + 1;
        let ids: Vec<ContentId> = (0..count).map(|_| [0u8; 32]).collect();
        assert!(write_request(&ids).is_none());
    }
}
