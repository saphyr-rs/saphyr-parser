pub mod buffered;
pub mod str;

#[allow(clippy::module_name_repetitions)]
pub use buffered::BufferedInput;

use crate::char_traits::{
    is_alpha, is_blank, is_blank_or_breakz, is_break, is_breakz, is_digit, is_flow, is_z,
};

/// Interface for a source of characters.
///
/// Hiding the input's implementation behind this trait allows mostly:
///  * For input-specific optimizations (for instance, using `str` methods instead of manually
///    transferring one `char` at a time to a buffer).
///  * To return `&str`s referencing the input string, thus avoiding potentially costly
///    allocations. Should users need an owned version of the data, they can always `.to_owned()`
///    their YAML object.
pub trait Input {
    /// A hint to the input source that we will need to read `count` characters.
    ///
    /// If the input is exhausted, `\0` can be used to pad the last characters and later returned.
    /// The characters must not be consumed, but may be placed in an internal buffer.
    ///
    /// This method may be a no-op if buffering yields no performance improvement.
    ///
    /// Implementers of [`Input`] must _not_ load more than `count` characters into the buffer. The
    /// parser tracks how many characters are loaded in the buffer and acts accordingly.
    fn lookahead(&mut self, count: usize);

    /// Return the number of buffered characters in `self`.
    #[must_use]
    fn buflen(&self) -> usize;

    /// Return the capacity of the buffer in `self`.
    #[must_use]
    fn bufmaxlen(&self) -> usize;

    /// Return whether the buffer (!= stream) is empty.
    #[inline]
    #[must_use]
    fn buf_is_empty(&self) -> bool {
        self.buflen() == 0
    }

    /// Skips characters until `f` returns `true` or the end of input is reached.
    ///
    /// The character that caused `f` to return `true` is not skipped.
    ///
    /// Returns the number of skipped characters.
    fn skip_until<F>(&mut self, f: F) -> usize
    where
        F: FnMut(char) -> bool;

    /// Skips characters until `f` returns `true` or the end of input is reached.
    ///
    /// The character that caused `f` to return `true` is not skipped.
    ///
    /// When `f` receives a non-ASCII character, it must return `true`. The non-ASCII might not be
    /// the actual character found in the input (e.g., it might be an UTF-8 byte casted to `char`).
    ///
    /// Returns the number of skipped characters.
    fn skip_ascii_until<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        self.skip_until(|c| {
            let stop = f(c);
            debug_assert!(stop || c.is_ascii());
            stop
        })
    }

    /// Reads characters into `out` until `f` returns `true` or the end of input is reached.
    ///
    /// The character that caused `f` to return `true` is not consumed or placed into `out`.
    ///
    /// Returns the number of read characters.
    fn read_until<F>(&mut self, out: &mut String, f: F) -> usize
    where
        F: FnMut(char) -> bool;

    /// Consume the next character.
    fn skip(&mut self);

    /// Consume the next `count` character.
    fn skip_n(&mut self, count: usize);

    /// Return the next character, without consuming it.
    ///
    /// Users of the [`Input`] must make sure that the character has been loaded through a prior
    /// call to [`Input::lookahead`]. Implementors of [`Input`] may assume that a valid call to
    /// [`Input::lookahead`] has been made beforehand.
    ///
    /// # Return
    /// If the input source is not exhausted, returns the next character to be fed into the
    /// scanner. Otherwise, returns `\0`.
    #[must_use]
    fn peek(&self) -> char;

    /// Return the `n`-th character in the buffer, without consuming it.
    ///
    /// This function assumes that the n-th character in the input has already been fetched through
    /// [`Input::lookahead`].
    #[must_use]
    fn peek_nth(&self, n: usize) -> char;

    /// Look for the next character and return it.
    ///
    /// The character is not consumed.
    /// Equivalent to calling [`Input::lookahead`] and [`Input::peek`].
    #[inline]
    #[must_use]
    fn look_ch(&mut self) -> char {
        self.lookahead(1);
        self.peek()
    }

    /// Return whether the next character in the input source is equal to `c`.
    ///
    /// This function assumes that the next character in the input has already been fetched through
    /// [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_char_is(&self, c: char) -> bool {
        self.peek() == c
    }

    /// Return whether the `n`-th character in the input source is equal to `c`.
    ///
    /// This function assumes that the n-th character in the input has already been fetched through
    /// [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn nth_char_is(&self, n: usize, c: char) -> bool {
        self.peek_nth(n) == c
    }

    /// Return whether the next 2 characters in the input source match the given characters.
    ///
    /// This function assumes that the next 2 characters in the input have already been fetched
    /// through [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_2_are(&self, c1: char, c2: char) -> bool {
        assert!(self.buflen() >= 2);
        self.peek() == c1 && self.peek_nth(1) == c2
    }

    /// Return whether the next 3 characters in the input source match the given characters.
    ///
    /// This function assumes that the next 3 characters in the input have already been fetched
    /// through [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_3_are(&self, c1: char, c2: char, c3: char) -> bool {
        assert!(self.buflen() >= 3);
        self.peek() == c1 && self.peek_nth(1) == c2 && self.peek_nth(2) == c3
    }

    /// Check whether the next characters correspond to a document indicator.
    ///
    /// This function assumes that the next 4 characters in the input has already been fetched
    /// through [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_is_document_indicator(&self) -> bool {
        assert!(self.buflen() >= 4);
        is_blank_or_breakz(self.peek_nth(3))
            && (self.next_3_are('.', '.', '.') || self.next_3_are('-', '-', '-'))
    }

    /// Check whether the next characters correspond to a start of document.
    ///
    /// This function assumes that the next 4 characters in the input has already been fetched
    /// through [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_is_document_start(&self) -> bool {
        assert!(self.buflen() >= 4);
        self.next_3_are('-', '-', '-') && is_blank_or_breakz(self.peek_nth(3))
    }

    /// Check whether the next characters correspond to an end of document.
    ///
    /// This function assumes that the next 4 characters in the input has already been fetched
    /// through [`Input::lookahead`].
    #[inline]
    #[must_use]
    fn next_is_document_end(&self) -> bool {
        assert!(self.buflen() >= 4);
        self.next_3_are('.', '.', '.') && is_blank_or_breakz(self.peek_nth(3))
    }

    /// Check whether the next characters may be part of a plain scalar.
    ///
    /// This function assumes we are not given a blankz character.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peek_nth(1);
        match self.peek() {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            ':' if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    /// Check whether the next character is [a blank] or [a break].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a blank] or [a break], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a blank]: is_blank
    /// [a break]: is_break
    #[inline]
    fn next_is_blank_or_break(&self) -> bool {
        is_blank(self.peek()) || is_break(self.peek())
    }

    /// Check whether the next character is [a blank] or [a breakz].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a blank] or [a break], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a blank]: is_blank
    /// [a breakz]: is_breakz
    #[inline]
    fn next_is_blank_or_breakz(&self) -> bool {
        is_blank(self.peek()) || is_breakz(self.peek())
    }

    /// Check whether the next character is [a blank].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a blank], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a blank]: is_blank
    #[inline]
    fn next_is_blank(&self) -> bool {
        is_blank(self.peek())
    }

    /// Check whether the next character is [a break].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a break], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a break]: is_break
    #[inline]
    fn next_is_break(&self) -> bool {
        is_break(self.peek())
    }

    /// Check whether the next character is [a breakz].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a breakz], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a breakz]: is_breakz
    #[inline]
    fn next_is_breakz(&self) -> bool {
        is_breakz(self.peek())
    }

    /// Check whether the next character is [a z].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a z], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a z]: is_z
    #[inline]
    fn next_is_z(&self) -> bool {
        is_z(self.peek())
    }

    /// Check whether the next character is [a flow].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a flow], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a flow]: is_flow
    #[inline]
    fn next_is_flow(&self) -> bool {
        is_flow(self.peek())
    }

    /// Check whether the next character is [a digit].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a digit], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a digit]: is_digit
    #[inline]
    fn next_is_digit(&self) -> bool {
        is_digit(self.peek())
    }

    /// Check whether the next character is [a letter].
    ///
    /// The character must have previously been fetched through [`lookahead`]
    ///
    /// # Return
    /// Returns true if the character is [a letter], false otherwise.
    ///
    /// [`lookahead`]: Input::lookahead
    /// [a letter]: is_alpha
    #[inline]
    fn next_is_alpha(&self) -> bool {
        is_alpha(self.peek())
    }
}
