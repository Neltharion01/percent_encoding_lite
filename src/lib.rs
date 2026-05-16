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
    pub const PATH: Bitmask = Bitmask::URI_COMPONENT.add(b'/');
}

fn encode_inner(src: &[u8], mask: Bitmask, is_formdata: bool) -> String {
    let mut out = String::with_capacity(src.len());
    for &ch in src.iter() {
        if ch == b' ' && !mask.contains(b' ') && is_formdata {
            out.push('+');
        } else if mask.contains(ch) {
            out.push(ch as char);
        } else {
            const HEX: &[u8] = b"0123456789ABCDEF";
            out.push('%');
            out.push(HEX[ch as usize >> 4] as char);
            out.push(HEX[ch as usize & 0xF] as char);
        }
    }
    out
}

/// Encodes given slice using provided [`Bitmask`]
/// # Example
/// ```
/// # use percent_encoding_lite::Bitmask;
/// let string = "Till the heat death of the universe, we will suffer";
/// let encoded = percent_encoding_lite::encode(string, Bitmask::URI_COMPONENT);
/// assert_eq!(&encoded, "Till%20the%20heat%20death%20of%20the%20universe%2C%20we%20will%20suffer");
/// ```
pub fn encode(src: impl AsRef<[u8]>, mask: Bitmask) -> String {
    encode_inner(src.as_ref(), mask, false)
}

/// Same as [`encode`], but replaces spaces with the `+` symbol
/// # Example
/// ```
/// # use percent_encoding_lite::Bitmask;
/// let string = "what happens when you type google in google";
/// let encoded = percent_encoding_lite::encode_form(string, Bitmask::URI_COMPONENT);
/// assert_eq!(&encoded, "what+happens+when+you+type+google+in+google");
/// ```
pub fn encode_form(src: impl AsRef<[u8]>, mask: Bitmask) -> String {
    encode_inner(src.as_ref(), mask, true)
}

fn decode_inner(src: &[u8], is_formdata: bool) -> Vec<u8> {
    let mut iter = src.iter();
    let mut out = vec![];
    while let Some(&i) = iter.next() {
        if i == b'+' && is_formdata {
            out.push(b' ');
        } else if i != b'%' {
            out.push(i);
        } else {
            if iter.len() < 2 { out.push(i); iter.next(); continue; }
            let (hi, lo) = (iter.as_slice()[0], iter.as_slice()[1]);
            let digits = char::from(hi).to_digit(16).zip(char::from(lo).to_digit(16));
            if digits.is_none() { out.push(i); iter.next(); continue; }
            let (hi, lo) = digits.unwrap();
            out.push((hi * 16 + lo) as u8);
            iter.next(); iter.next();
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
pub fn decode(src: impl AsRef<[u8]>) -> Vec<u8> {
    decode_inner(src.as_ref(), false)
}

/// Same as [`decode`], but decodes `+` as space
/// # Example
/// ```
/// let encoded = "Plus+is+space";
/// let decoded = percent_encoding_lite::decode_form(encoded);
/// assert_eq!(&decoded, b"Plus is space");
/// ```
pub fn decode_form(src: impl AsRef<[u8]>) -> Vec<u8> {
    decode_inner(src.as_ref(), true)
}

/// Checks if this string contains any unencoded characters
///
/// You can use this function to optimize your code, to avoid allocation if data does not need to he encoded (since this library's functions don't return `Cow`)
/// # Example
/// ```
/// # use percent_encoding_lite::{is_encoded, Bitmask};
/// let string = "Spaces should be encoded";
/// assert_eq!(is_encoded(&string, Bitmask::URI_COMPONENT), false);
/// ```
pub fn is_encoded(src: impl AsRef<[u8]>, mask: Bitmask) -> bool {
    for &ch in src.as_ref() {
        if !mask.contains(ch) { return false; }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn urldecode_test() {
        let encoded = "Anno+1404.Gold+Edition.v+2.1.5010.0.(%D0%9D%D0%BE%D0%B2%D1%8B%D0%B9+%D0%94%D0%B8%D1%81%D0%BA).(2010).Repack";
        let decoded = String::from_utf8(decode_form(encoded)).unwrap();
        let correct = "Anno 1404.Gold Edition.v 2.1.5010.0.(Новый Диск).(2010).Repack";
        assert_eq!(&decoded, correct);

        let encoded = "The+Elder+Scrolls+V.+Skyrim.+Anniversary+Edition+v.1.6.640.0.8+(2011-2021)";
        let decoded = String::from_utf8(decode_form(encoded)).unwrap();
        let correct = "The Elder Scrolls V. Skyrim. Anniversary Edition v.1.6.640.0.8 (2011-2021)";
        assert_eq!(&decoded, correct);
    }
    #[test]
    fn urlencode_test() {
        let orig = "Microsoft Windows 10, version 22H2, build 19045.2846 (updated April 2023) - Оригинальные образы от Microsoft MSDN [Ru]";
        let encoded = encode_form(orig, Bitmask::URI_COMPONENT);
        let correct = "Microsoft+Windows+10%2C+version+22H2%2C+build+19045.2846+(updated+April+2023)+-+%D0%9E%D1%80%D0%B8%D0%B3%D0%B8%D0%BD%D0%B0%D0%BB%D1%8C%D0%BD%D1%8B%D0%B5+%D0%BE%D0%B1%D1%80%D0%B0%D0%B7%D1%8B+%D0%BE%D1%82+Microsoft+MSDN+%5BRu%5D";
        assert_eq!(&encoded, correct);

        let orig = "Windows_Embedded_8.1_Industry_Pro_with_Update_x86_x64_MultiLang";
        let encoded = encode(orig, Bitmask::URI_COMPONENT);
        assert_eq!(&encoded, orig);
        assert_eq!(is_encoded(&orig, Bitmask::URI_COMPONENT), true);

        let orig = "In URI, space is %20";
        let encoded = encode(orig, Bitmask::URI);
        let correct = "In%20URI,%20space%20is%20%2520";
        assert_eq!(&encoded, correct);
    }
    #[test]
    fn is_encoded_test() {
        assert_eq!(is_encoded("abc", Bitmask::URI), true);
        assert_eq!(is_encoded("abc,def", Bitmask::URI), true);
        assert_eq!(is_encoded("abc,def", Bitmask::URI_COMPONENT), false);
        assert_eq!(is_encoded("abc[def", Bitmask::URI), false);
        assert_eq!(is_encoded("%01%02", Bitmask::URI), false);
    }
}
