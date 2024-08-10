use crate::input::Input;

#[allow(clippy::module_name_repetitions)]
pub struct StrInput<'a> {
    /// The input str buffer.
    buffer: &'a str,
    /// The number of characters we have looked ahead.
    ///
    /// We must however keep track of how many characters the parser asked us to look ahead for so
    /// that we can return the correct value in [`Self::buflen`].
    lookahead: usize,
}

impl<'a> StrInput<'a> {
    /// Create a new [`StrInput`] with the given str.
    pub fn new(input: &'a str) -> Self {
        Self {
            buffer: input,
            lookahead: 0,
        }
    }
}

impl<'a> Input for StrInput<'a> {
    #[inline]
    fn lookahead(&mut self, x: usize) {
        // We already have all characters that we need.
        // We cannot add '\0's to the buffer should we prematurely reach EOF.
        // Returning '\0's befalls the character-retrieving functions.
        self.lookahead = self.lookahead.max(x);
    }

    #[inline]
    fn buflen(&self) -> usize {
        self.lookahead
    }

    #[inline]
    fn bufmaxlen(&self) -> usize {
        BUFFER_LEN
    }

    fn buf_is_empty(&self) -> bool {
        self.buflen() == 0
    }

    fn skip_until<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut char_count = 0usize;
        let mut new_str = self.buffer;

        while let Some((c, sub_str)) = split_first_char(new_str) {
            if f(c) {
                break;
            }
            new_str = sub_str;
            char_count += 1;
        }

        self.buffer = new_str;

        char_count
    }

    fn skip_ascii_until<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut new_str = self.buffer.as_bytes();

        while let Some((&c, sub_str)) = new_str.split_first() {
            if f(c.into()) {
                break;
            }
            debug_assert!(c.is_ascii());
            new_str = sub_str;
        }

        // Since all skipped characters are ascii, the number of characters is equal to the number
        // of bytes.
        let char_count = self.buffer.len() - new_str.len();
        self.buffer = &self.buffer[char_count..];

        char_count
    }

    fn read_until<F>(&mut self, out: &mut String, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut char_count = 0usize;
        let mut new_str = self.buffer;

        while let Some((c, sub_str)) = split_first_char(new_str) {
            if f(c) {
                break;
            }
            new_str = sub_str;
            char_count += 1;
        }

        let byte_count = self.buffer.len() - new_str.len();
        out.push_str(&self.buffer[..byte_count]);

        self.buffer = new_str;

        char_count
    }

    #[inline]
    fn skip(&mut self) {
        let mut chars = self.buffer.chars();
        if chars.next().is_some() {
            self.buffer = chars.as_str();
        }
    }

    #[inline]
    fn skip_n(&mut self, count: usize) {
        let mut chars = self.buffer.chars();
        for _ in 0..count {
            if chars.next().is_none() {
                break;
            }
        }
        self.buffer = chars.as_str();
    }

    #[inline]
    fn peek(&self) -> char {
        self.buffer.chars().next().unwrap_or('\0')
    }

    #[inline]
    fn peek_ascii(&self) -> char {
        self.buffer.as_bytes().first().map_or('\0', |&c| c.into())
    }

    #[inline]
    fn peek_nth(&self, n: usize) -> char {
        let mut chars = self.buffer.chars();
        for _ in 0..n {
            if chars.next().is_none() {
                return '\0';
            }
        }
        chars.next().unwrap_or('\0')
    }

    #[inline]
    fn peek_nth_ascii(&self, n: usize) -> char {
        self.buffer.as_bytes().get(n).map_or('\0', |&c| c.into())
    }

    #[inline]
    fn look_ch(&mut self) -> char {
        self.lookahead(1);
        self.peek()
    }

    #[inline]
    fn nth_char_is(&self, n: usize, c: char) -> bool {
        self.peek_nth(n) == c
    }

    #[inline]
    fn next_2_are(&self, c1: char, c2: char) -> bool {
        let mut chars = self.buffer.chars();
        chars.next().is_some_and(|c| c == c1) && chars.next().is_some_and(|c| c == c2)
    }
}

/// The buffer size we return to the scanner.
///
/// This does not correspond to any allocated buffer size. In practice, the scanner can withdraw
/// any character they want. If it's within the input buffer, the given character is returned,
/// otherwise `\0` is returned.
///
/// The number of characters we are asked to retrieve in [`lookahead`] depends on the buffer size
/// of the input. Our buffer here is virtually unlimited, but the scanner cannot work with that. It
/// may allocate buffers of its own of the size we return in [`bufmaxlen`] (so we can't return
/// [`usize::MAX`]). We can't always return the number of characters left either, as the scanner
/// expects [`buflen`] to return the same value that was given to [`lookahead`] right after its
/// call.
///
/// This create a complex situation where [`bufmaxlen`] influences what value [`lookahead`] is
/// called with, which in turns dictates what [`buflen`] returns. In order to avoid breaking any
/// function, we return this constant in [`bufmaxlen`] which, since the input is processed one line
/// at a time, should fit what we expect to be a good balance between memory consumption and what
/// we expect the maximum line length to be.
///
/// [`lookahead`]: `StrInput::lookahead`
/// [`bufmaxlen`]: `StrInput::bufmaxlen`
/// [`buflen`]: `StrInput::buflen`
const BUFFER_LEN: usize = 128;

/// Splits the first character of the given string and returns it along with the rest of the
/// string.
#[inline]
fn split_first_char(s: &str) -> Option<(char, &str)> {
    let mut iter = s.chars();
    let c = iter.next()?;
    Some((c, iter.as_str()))
}
