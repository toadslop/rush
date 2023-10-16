use unicode_segmentation::UnicodeSegmentation;

pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Checks to see if the number of graphemes is above a certain count
pub fn is_longer_than(s: &str, len: usize) -> bool {
    UnicodeSegmentation::graphemes(s, true).count() > len
}
