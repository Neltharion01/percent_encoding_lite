//! URL encoding/decoding functions
//!
//! Check [`encode`] and [`decode`] docs for example usage

/// Bitmask that contains allowed character set
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bitmask(pub [u32; 3]);

impl Bitmask {
    /// Checks if this bitmask contains `ch`
    pub const fn contains(&self, ch: u8) -> bool {
        if ch < 32 || ch > 127 { return false; }
        self.0[(ch as usize - 32) / 32] & (1_u32 << (ch % 32)) != 0
    }

    /// Adds `ch` to this bitmask
    pub const fn add(mut self, ch: u8) -> Bitmask {
        if ch >= 32 && ch <= 127 {
            self.0[(ch as usize - 32) / 32] |= 1_u32 << (ch % 32);
        }
        self
    }

    /// Adds all `chars` into this bitmask
    pub const fn add_all(mut self, chars: &[u8]) -> Bitmask {
        let mut i = 0;
        while i < chars.len() {
            self = self.add(chars[i]);
            i += 1;
        }
        self
    }

    /// Removes `ch` from this bitmask
    pub const fn remove(mut self, ch: u8) -> Bitmask {
        if ch >= 32 && ch <= 127 {
            self.0[(ch as usize - 32) / 32] &= !(1_u32 << (ch % 32));
        }
        self
    }

    /// Removes all `chars` from this bitmask
    pub const fn remove_all(mut self, chars: &[u8]) -> Bitmask {
        let mut i = 0;
        while i < chars.len() {
            self = self.remove(chars[i]);
            i += 1;
        }
        self
    }

    pub const EMPTY: Bitmask = Bitmask([0, 0, 0]);
    pub const URI_COMPONENT: Bitmask = Bitmask::EMPTY
        .add_all(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        .add_all(b"abcdefghijklmnopqrstuvwxyz")
        .add_all(b"0123456789")
        .add_all(b"-_.!~*'()");
    pub const URI: Bitmask = Bitmask::URI_COMPONENT.add_all(b";/?:@&=+$,#");
    pub const RFC3986: Bitmask = Bitmask::URI.add_all(b"[]").remove_all(b"!'()*");
    pub const PATH: Bitmask = Bitmask::URI_COMPONENT.remove(b'/');
}

const HEX: &[u8] = b"0123456789ABCDEF";
/// Encodes given slice using provided [`Bitmask`]
/// # Example
/// ```
/// # use percent_encoding_lite::Bitmask;
/// let string = "Dragonborn, dragonborn, by his honor is sworn";
/// let encoded = percent_encoding_lite::encode(string.as_bytes(), Bitmask::URI);
/// assert_eq!(&encoded, "Dragonborn,+dragonborn,+by+his+honor+is+sworn");
/// ```
pub fn encode(src: &[u8], mask: Bitmask) -> String {
    let mut out = String::with_capacity(src.len());
    for ch in src.iter().copied() {
        if ch == b' ' {
            out.push('+');
        } else if mask.contains(ch) {
            out.push(ch as char);
        } else {
            out.push('%');
            out.push(HEX[ch as usize >> 4] as char);
            out.push(HEX[ch as usize & 0xF] as char);
        }
    }
    out
}

/// Decodes a percent encoded string
/// # Example
/// ```
/// let encoded = "%54%6F%20%6B%65%65%70%20%65%76%69%6C%20%66%6F%72%65%76%65%72%20%61%74%20%62%61%79%21";
/// let decoded = percent_encoding_lite::decode(encoded);
/// assert_eq!(&decoded, b"To keep evil forever at bay!");
/// ```
pub fn decode(src: &str) -> Vec<u8> {
    let mut slice = src.as_bytes();
    let mut out = vec![];
    while let Some(&i) = slice.first() {
        slice = &slice[1..]; // I wish rust had random access iterators

        if i == b'+' {
            out.push(b' ');
        } else if i != b'%' {
            out.push(i);
        } else {
            if slice.len() < 2 { out.push(i); slice = &slice[1..]; continue; }
            let (hi, lo) = (slice[0], slice[1]);
            let digits = char::from(hi).to_digit(16).zip(char::from(lo).to_digit(16));
            if digits.is_none() { out.push(i); slice = &slice[1..]; continue; }
            let (hi, lo) = digits.unwrap();
            out.push((hi * 16 + lo) as u8);
            slice = &slice[2..];
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn urldecode_test() {
        let encoded = "Anno+1404.Gold+Edition.v+2.1.5010.0.(%D0%9D%D0%BE%D0%B2%D1%8B%D0%B9+%D0%94%D0%B8%D1%81%D0%BA).(2010).Repack";
        let decoded = String::from_utf8(decode(encoded)).unwrap();
        let correct = "Anno 1404.Gold Edition.v 2.1.5010.0.(Новый Диск).(2010).Repack";
        assert_eq!(&decoded, correct);

        let encoded = "The+Elder+Scrolls+V.+Skyrim.+Anniversary+Edition+v.1.6.640.0.8+(2011-2021)";
        let decoded = String::from_utf8(decode(encoded)).unwrap();
        let correct = "The Elder Scrolls V. Skyrim. Anniversary Edition v.1.6.640.0.8 (2011-2021)";
        assert_eq!(&decoded, correct);
    }
    #[test]
    fn urlencode_test() {
        let orig = "Microsoft Windows 10, version 22H2, build 19045.2846 (updated April 2023) - Оригинальные образы от Microsoft MSDN [Ru]";
        let encoded = encode(orig.as_bytes(), Bitmask::URI_COMPONENT);
        let correct = "Microsoft+Windows+10%2C+version+22H2%2C+build+19045.2846+(updated+April+2023)+-+%D0%9E%D1%80%D0%B8%D0%B3%D0%B8%D0%BD%D0%B0%D0%BB%D1%8C%D0%BD%D1%8B%D0%B5+%D0%BE%D0%B1%D1%80%D0%B0%D0%B7%D1%8B+%D0%BE%D1%82+Microsoft+MSDN+%5BRu%5D";
        assert_eq!(&encoded, correct);

        let orig = "Windows_Embedded_8.1_Industry_Pro_with_Update_x86_x64_MultiLang";
        let encoded = encode(orig.as_bytes(), Bitmask::URI_COMPONENT);
        assert_eq!(&encoded, orig);
    }
}
