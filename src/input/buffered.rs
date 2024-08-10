use crate::input::Input;

use arraydeque::ArrayDeque;

/// The size of the [`BufferedInput`] buffer.
///
/// The buffer is statically allocated to avoid conditions for reallocations each time we
/// consume/push a character. As of now, almost all lookaheads are 4 characters maximum, except:
///   - Escape sequences parsing: some escape codes are 8 characters
///   - Scanning indent in scalars: this looks ahead `indent + 2` characters
///
/// This constant must be set to at least 8. When scanning indent in scalars, the lookahead is done
/// in a single call if and only if the indent is `BUFFER_LEN - 2` or less. If the indent is higher
/// than that, the code will fall back to a loop of lookaheads.
const BUFFER_LEN: usize = 16;

/// A wrapper around an [`Iterator`] of [`char`]s with a buffer.
///
/// The YAML scanner often needs some lookahead. With fully allocated buffers such as `String` or
/// `&str`, this is not an issue. However, with streams, we need to have a way of peeking multiple
/// characters at a time and sometimes pushing some back into the stream.
/// There is no "easy" way of doing this without itertools. In order to avoid pulling the entierty
/// of itertools for one method, we use this structure.
#[allow(clippy::module_name_repetitions)]
pub struct BufferedInput<T: Iterator<Item = char>> {
    /// The iterator source,
    input: T,
    /// Buffer for the next characters to consume.
    buffer: ArrayDeque<char, BUFFER_LEN>,
}

impl<T: Iterator<Item = char>> BufferedInput<T> {
    /// Create a new [`BufferedInput`] with the given input.
    pub fn new(input: T) -> Self {
        Self {
            input,
            buffer: ArrayDeque::default(),
        }
    }
}

impl<T: Iterator<Item = char>> Input for BufferedInput<T> {
    #[inline]
    fn lookahead(&mut self, count: usize) {
        if self.buffer.len() >= count {
            return;
        }
        for _ in 0..(count - self.buffer.len()) {
            self.buffer
                .push_back(self.input.next().unwrap_or('\0'))
                .unwrap();
        }
    }

    #[inline]
    fn buflen(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn bufmaxlen(&self) -> usize {
        BUFFER_LEN
    }

    fn skip_until<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut char_count = 0;

        for &c in &self.buffer {
            if f(c) {
                break;
            }
            char_count += 1;
        }

        self.buffer.drain(0..char_count);

        if self.buffer.is_empty() {
            for c in self.input.by_ref() {
                if f(c) {
                    self.buffer.push_back(c).unwrap();
                    break;
                }
                char_count += 1;
            }
        }

        char_count
    }

    fn read_until<F>(&mut self, out: &mut String, mut f: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut char_count = 0;

        for &c in &self.buffer {
            if f(c) {
                break;
            }
            out.push(c);
            char_count += 1;
        }

        self.buffer.drain(0..char_count);

        if self.buffer.is_empty() {
            for c in self.input.by_ref() {
                if f(c) {
                    self.buffer.push_back(c).unwrap();
                    break;
                }
                out.push(c);
                char_count += 1;
            }
        }

        char_count
    }

    #[inline]
    fn skip(&mut self) {
        self.buffer.pop_front();
    }

    #[inline]
    fn skip_n(&mut self, count: usize) {
        self.buffer.drain(0..count);
    }

    #[inline]
    fn peek(&self) -> char {
        self.buffer[0]
    }

    #[inline]
    fn peek_nth(&self, n: usize) -> char {
        self.buffer[n]
    }
}
