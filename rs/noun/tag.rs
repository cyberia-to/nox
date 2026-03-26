//! type tags for atoms — Field (0x00) and Word (0x01)
//! hash nouns are structured cells, not a tag variant

/// type tag distinguishing field elements from word integers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    /// Goldilocks field element, full range [0, p)
    Field = 0x00,
    /// 32-bit word, range [0, 2^32), bitwise operations
    Word = 0x01,
}
