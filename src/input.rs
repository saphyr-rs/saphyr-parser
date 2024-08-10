pub mod buffered;
pub mod str;

#[allow(clippy::module_name_repetitions)]
pub use buffered::BufferedInput;

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

    /// Return the next character, without consuming it.
    ///
    /// Users of the [`Input`] must make sure that the character has been loaded through a prior
    /// call to [`Input::lookahead`]. Implementors of [`Input`] may assume that a valid call to
    /// [`Input::lookahead`] has been made beforehand.
    ///
    /// # Return
    ///
    /// If the input source is not exhausted, returns the next character to be fed into the
    /// scanner. Otherwise, returns `\0`.
    ///
    /// If the next character is not an ASCII character, the returned character is a non-ASCII
    /// character that might not be the actual character found in the input (e.g., it might be
    /// an UTF-8 byte casted to `char`).
    #[must_use]
    #[inline]
    fn peek_ascii(&self) -> char {
        self.peek()
    }

    /// Return the `n`-th character in the buffer, without consuming it.
    ///
    /// This function assumes that the n-th character in the input has already been fetched through
    /// [`Input::lookahead`] and all non-consumed characters before the `n`-th are ASCII.
    ///
    /// If the `n`-th character is not ASCII, the returned character is a non-ASCII character that
    /// might not be the actual character found in the input (e.g., it might be an UTF-8 byte casted
    /// to `char`).
    #[must_use]
    #[inline]
    fn peek_nth_ascii(&self, n: usize) -> char {
        self.peek_nth(n)
    }

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
}
