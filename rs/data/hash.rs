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

#[cfg(test)]
mod tests {
    use super::*;

    fn g(v: u64) -> Goldilocks {
        Goldilocks::new(v)
    }

    #[test]
    fn hash_atom_is_deterministic() {
        assert_eq!(hash_atom(g(42)), hash_atom(g(42)));
    }

    #[test]
    fn hash_atom_distinguishes_values() {
        assert_ne!(hash_atom(g(1)), hash_atom(g(2)));
    }

    #[test]
    fn hash_pair_is_deterministic() {
        let a = hash_atom(g(1));
        let b = hash_atom(g(2));
        assert_eq!(hash_pair(&a, &b), hash_pair(&a, &b));
    }

    #[test]
    fn hash_pair_is_not_commutative() {
        let a = hash_atom(g(1));
        let b = hash_atom(g(2));
        assert_ne!(hash_pair(&a, &b), hash_pair(&b, &a));
    }

    #[test]
    fn hash_pair_distinguishes_children() {
        let a = hash_atom(g(1));
        let b = hash_atom(g(2));
        let c = hash_atom(g(3));
        assert_ne!(hash_pair(&a, &b), hash_pair(&a, &c));
    }

    #[test]
    fn digest_bytes_round_trips_a_canonical_digest() {
        let d = hash_atom(g(0xDEAD_BEEF));
        assert_eq!(digest_from_bytes(&digest_bytes(&d)), d);
    }

    #[test]
    fn digest_bytes_is_little_endian_per_limb() {
        let d = [g(1), g(0x0102_0304_0506_0708), g(0), g(0)];
        let bytes = digest_bytes(&d);
        assert_eq!(&bytes[0..8], &[1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&bytes[8..16], &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    // digest_from_bytes canonicalizes every limb (its own doc comment says so):
    // a raw limb >= p and that limb minus p decode to the identical digest.
    // This is intentional per the doc comment, but it means digest_from_bytes
    // alone does not reject a non-canonical particle encoding — callers that
    // need canonical-only input (e.g. neuron's Content::validate) must guard
    // it themselves. Pinning the behavior here documents why that guard exists.
    #[test]
    fn digest_from_bytes_canonicalizes_an_out_of_range_limb() {
        let p = nebu::field::P;
        let mut raw = [0u8; 32];
        raw[0..8].copy_from_slice(&(p + 5).to_le_bytes());
        let mut reduced = [0u8; 32];
        reduced[0..8].copy_from_slice(&5u64.to_le_bytes());
        assert_eq!(digest_from_bytes(&raw), digest_from_bytes(&reduced));
    }

    #[test]
    fn digest_from_bytes_leaves_a_canonical_limb_unchanged() {
        let mut raw = [0u8; 32];
        raw[0..8].copy_from_slice(&123u64.to_le_bytes());
        assert_eq!(digest_from_bytes(&raw)[0], g(123));
    }
}
