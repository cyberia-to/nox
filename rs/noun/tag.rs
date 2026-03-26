//! type tags for atoms — Field (0x00) and Word (0x01)
//! hash nouns are structured cells, not a tag variant

/// type tag distinguishing field elements from word integers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    /// field element of the instantiated field F
    Field = 0x00,
    /// word element of the instantiated word width W
    Word = 0x01,
}
