//! hemera-based noun hashing with capacity domain separation
//!
//! atoms: hemera::tree::hash_leaf (FLAG_CHUNK in capacity)
//! cells: hemera::tree::hash_node (FLAG_PARENT in capacity)
//! digest: first 4 of 8 Goldilocks elements (128-bit collision security)

use nebu::Goldilocks;
use super::tag::Tag;

/// hash identity — 4 Goldilocks elements = 32 bytes
/// intentional truncation from hemera 64-byte output (per trace.md)
pub type Digest = [Goldilocks; 4];

/// hash an atom using hemera tree leaf mode
pub fn hash_atom(value: Goldilocks, tag: Tag) -> Digest {
    let mut data = [0u8; 9];
    data[0..8].copy_from_slice(&value.as_u64().to_le_bytes());
    data[8] = tag as u8;
    let h = hemera::tree::hash_leaf(&data, tag as u64, false);
    extract_digest(&h)
}

/// hash a cell using hemera tree node mode
pub fn hash_cell(left: &Digest, right: &Digest) -> Digest {
    let lh = pack_digest(left);
    let rh = pack_digest(right);
    let h = hemera::tree::hash_node(&lh, &rh, false);
    extract_digest(&h)
}

fn pack_digest(d: &Digest) -> hemera::Hash {
    let mut bytes = [0u8; 32];
    for i in 0..4 {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&d[i].as_u64().to_le_bytes());
    }
    hemera::Hash::from_bytes(bytes)
}

fn extract_digest(h: &hemera::Hash) -> Digest {
    let bytes = h.as_bytes();
    let mut digest = [Goldilocks::ZERO; 4];
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        digest[i] = Goldilocks::new(u64::from_le_bytes(buf));
    }
    digest
}
