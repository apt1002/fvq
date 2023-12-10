#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitString {
    /// Complete 64-bit words (little-endian).
    words: Vec<u64>,

    /// The incomplete last word (little-endian). Unused bits must be zero.
    last_word: u64,

    /// The number of bits used in `last_word` (0 to 63).
    bit: u8,
}

impl BitString {
    /// The number of bits in this `BitString`.
    pub fn len(&self) -> usize { 64 * self.words.len() + (self.bit as usize) }

    /// Append one bit.
    pub fn push(&mut self, bit: bool) {
        self.last_word |= (bit as u64) << self.bit;
        self.bit += 1;
        if self.bit == 64 {
            self.words.push(self.last_word);
            self.last_word = 0;
            self.bit = 0;
        }
    }

    /// Remove and return the last bit.
    pub fn pop(&mut self) -> Option<bool> {
        if self.bit == 0 {
            if let Some(last_word) = self.words.pop() {
                self.last_word = last_word;
                self.bit = 64;
            } else {
                return None;
            }
        }
        self.bit -= 1;
        let ret = self.last_word & (1 << self.bit);
        self.last_word ^= ret;
        Some(ret != 0)
    }

    /// Returns an [`Iterator`] through the bits of this `BitString`.
    pub fn iter(&self) -> Iter { self.into_iter() }
}

impl<'a> IntoIterator for &'a BitString {
    type Item = bool;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter { Iter {bs: self, pos: 0} }
}

// ----------------------------------------------------------------------------

pub struct Iter<'a> {
    bs: &'a BitString,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let word_pos = self.pos / 64;
        let bit_pos = (self.pos % 64) as u8;
        let word = if word_pos < self.bs.words.len() {
            self.bs.words[word_pos]
        } else {
            if bit_pos < self.bs.bit {
                self.bs.last_word
            } else {
                return None;
            }
        };
        self.pos += 1;
        Some((word & (1 << bit_pos)) != 0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let ret = self.bs.len() - self.pos;
        (ret, Some(ret))
    }
}

impl<'a> std::iter::ExactSizeIterator for Iter<'a> {}
impl<'a> std::iter::FusedIterator for Iter<'a> {}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn vec() {
        // Make a `BitString` and a matching `Vec<bool>`.
        let mut bs = BitString::default();
        let mut bv = Vec::<bool>::default();
        let mut seed: u32 = 1;
        for _ in 0..1000 {
            seed = seed.wrapping_mul(3141592653);
            seed = seed.wrapping_add(2718281845);
            let bit = (seed >> 31) != 0;
            bs.push(bit);
            bv.push(bit);
        }
        // Check `Iterator` behaviour matches.
        let mut bsi = bs.iter();
        let mut bvi = bv.iter().copied();
        for _ in 0..1000 {
            let bit1 = bsi.next().unwrap();
            let bit2 = bvi.next().unwrap();
            assert_eq!(bit1, bit2);
        }
        assert!(bsi.next().is_none());
        assert!(bvi.next().is_none());
        // Check `pop()` behaviour matches.
        for _ in 0..1000 {
            let bit1 = bs.pop().unwrap();
            let bit2 = bv.pop().unwrap();
            assert_eq!(bit1, bit2);
        }
        assert!(bs.pop().is_none());
        assert!(bv.pop().is_none());
    }
}
