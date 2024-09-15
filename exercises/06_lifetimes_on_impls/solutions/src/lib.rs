// First, the struct:

/// This struct keeps track of where we're up to in the string.
struct WordIterator<'s> {
    position: usize,
    string: &'s str
}

impl<'lifetime> WordIterator<'lifetime> {
    /// Creates a new WordIterator based on a string.
    fn new(string: &'lifetime str) -> WordIterator<'lifetime> {
        WordIterator {
            position: 0,
            string
        }
    }

    /// Gives the next word. `None` if there aren't any words left.
    fn next_word(&mut self) -> Option<&'lifetime str> {
        let start_of_word = &self.string[self.position..];
        let index_of_next_space = start_of_word.find(' ').unwrap_or(start_of_word.len());
        if start_of_word.len() != 0 {
            self.position += index_of_next_space + 1;
            Some(&start_of_word[..index_of_next_space])
        } else {
            None
        }
    }
}

