//! Provides functionality for tracebacks.

/// Wrapper for an iterator, counting the current index.
/// Basically allows for the same thing as `enumerate` but not just in one statement.
pub struct TracebackIterator<T>
where
    T: Iterator,
{
    inner_iterator: T,
    current_line: u64,
}

impl<T> TracebackIterator<T>
where
    T: Iterator,
{
    /// Gets the current index of the iterator.
    pub fn current_line(&self) -> u64 {
        self.current_line
    }
}

impl<T> Iterator for TracebackIterator<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line += 1;
        self.inner_iterator.next()
    }
}

impl<T> From<T> for TracebackIterator<T>
where
    T: Iterator,
{
    fn from(inner_iterator: T) -> Self {
        Self {
            inner_iterator,
            current_line: 0,
        }
    }
}
