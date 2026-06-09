//! hemera-based data hashing with capacity domain separation
//!
//! atoms: hemera::tree::hash_leaf (FLAG_CHUNK in capacity)
//! pairs: hemera::tree::hash_node (FLAG_PARENT in capacity)
//! digest: first 4 of 8 Goldilocks elements (128-bit collision security)

use nebu::Goldilocks;

/// hash identity — 4 Goldilocks elements = 32 bytes
/// intentional truncation from hemera 64-byte output (per trace.md)
pub type Digest = [Goldilocks; 4];

/// Hash an atom using hemera tree leaf mode.
///
/// 8-byte payload: value as little-endian u64. domain = 0 (no tag framing).
pub fn hash_atom(value: Goldilocks) -> Digest {
    let mut data = [0u8; 8];
    data[0..8].copy_from_slice(&value.as_u64().to_le_bytes());
    let h = hemera::tree::hash_leaf(&data, 0, false);
    extract_digest(&h)
}

/// hash a pair using hemera tree node mode
pub fn hash_pair(left: &Digest, right: &Digest) -> Digest {
    let lh = pack_digest(left);
    let rh = pack_digest(right);
    let h = hemera::tree::hash_node(&lh, &rh, false);
    extract_digest(&h)
}

/// serialize a digest to its 32-byte particle representation (4 LE limbs).
/// a `particle` IS this byte view of the tree-hash digest — there is no
/// separate identity scheme.
pub fn digest_bytes(d: &Digest) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..4 {
        out[i * 8..(i + 1) * 8].copy_from_slice(&d[i].as_u64().to_le_bytes());
    }
    out
}

/// parse a 32-byte particle back into a digest (canonicalized limbs).
pub fn digest_from_bytes(b: &[u8; 32]) -> Digest {
    let mut d = [Goldilocks::ZERO; 4];
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&b[i * 8..(i + 1) * 8]);
        d[i] = Goldilocks::new(u64::from_le_bytes(buf)).canonicalize();
    }
    d
}

fn pack_digest(d: &Digest) -> hemera::Hash {
    hemera::Hash::from_bytes(digest_bytes(d))
}

fn extract_digest(h: &hemera::Hash) -> Digest {
    // Canonicalize each limb so stored digests live in [0, p). Goldilocks
    // PartialEq already canonicalizes on read, so this is defensive — guarantees
    // hash-cons table keys and serialized digests use the same representation.
    let bytes = h.as_bytes();
    let mut digest = [Goldilocks::ZERO; 4];
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        digest[i] = Goldilocks::new(u64::from_le_bytes(buf)).canonicalize();
    }
    digest
}
