use std::fmt;
use std::borrow::Cow;

use rand::{self, Rng};
use rocket::request::FromParam;

/// A _probably_ unique paste ID.
pub struct PasteID<'a>(Cow<'a, str>);

fn base62_char(idx: usize) -> char {
    fn chars_len(start: char, end: char) -> isize {
        (end as u8 - start as u8 + 1) as isize
    }

    fn char_off(start: char, off: isize) -> char {
        assert!(off >= 0);
        (start as u8 + off as u8) as char
    }

    let mut posn = idx as isize;
    let top = chars_len('0', '9');
    if posn < top {
        return char_off('0', posn);
    };
    posn -= top;
    let top = chars_len('a', 'z');
    if posn < top {
        return char_off('a', posn);
    };
    posn -= top;
    let top = chars_len('A', 'Z');
    if posn < top {
        return char_off('A', posn);
    };
    panic!("base62_char: index off end")
}

/// Returns `true` if `id` is a valid paste ID and `false` otherwise.
fn is_base62(id: &str) -> bool {
    id.chars().all(|c| {
        (c >= 'a' && c <= 'z')
     || (c >= 'A' && c <= 'Z')
     || (c >= '0' && c <= '9')
    })
}

/// Returns an instance of `PasteID` if the path segment is a valid ID.
/// Otherwise returns the invalid ID as the `Err` value.
impl<'a> FromParam<'a> for PasteID<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<PasteID<'a>, &'a str> {
        match is_base62(param) {
            true => Ok(PasteID(Cow::Borrowed(param))),
            false => Err(param)
        }
    }
}

impl<'a> PasteID<'a> {
    /// Generate a _probably_ unique ID with `size`
    /// characters. For readability, the characters used are
    /// from the sets [0-9], [A-Z], [a-z]. The probability
    /// of a collision depends on the value of `size`. In
    /// particular, the probability of a collision is
    /// 1/62^(size).
    pub fn new(size: usize) -> PasteID<'static> {
        let mut rng = rand::thread_rng();
        let mut id = String::with_capacity(size);
        for _ in 0..size {
            id.push(base62_char(rng.gen::<usize>() % 62));
        };
        PasteID(Cow::Owned(id))
    }
}

impl<'a> fmt::Display for PasteID<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
