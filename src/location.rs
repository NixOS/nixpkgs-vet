use relative_path::RelativePathBuf;

/// A location that's suitable for error messages.
#[derive(Clone, Debug)]
pub struct Location {
    pub file: RelativePathBuf,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(file: impl Into<RelativePathBuf>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

/// A simple utility for calculating the line for a string offset.
///
/// This doesn't do any Unicode handling, though that probably doesn't matter because newlines
/// can't split up Unicode characters. This is only used for error reporting.
pub struct LineIndex {
    /// Stores the indices of newlines
    newlines: Vec<usize>,
}

impl LineIndex {
    pub fn new(s: &str) -> Self {
        let mut newlines = vec![];
        let mut index = 0;
        // Iterates over all newline-split parts of the string, adding the index of the newline to
        // the vec
        for split in s.split_inclusive('\n') {
            index += split.len();
            newlines.push(index - 1);
        }
        Self { newlines }
    }

    /// Returns the line number for a string index.
    /// If the index points to a newline, returns the line number before the newline.
    pub fn line(&self, index: usize) -> usize {
        match self.newlines.binary_search(&index) {
            // +1 because lines are 1-indexed
            Ok(x) | Err(x) => x + 1,
        }
    }

    /// Returns the string index for a line and column.
    pub fn fromlinecolumn(&self, line: usize, column: usize) -> usize {
        // If it's the 1th line, the column is the index
        if line == 1 {
            // But columns are 1-indexed
            column - 1
        } else {
            // For the nth line, we add the index of the (n-1)st newline to the column,
            // and remove one more from the index since arrays are 0-indexed.
            // Then add the 1-indexed column to get not the newline index itself,
            // but rather the index of the position on the next line.
            self.newlines[line - 2] + column
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index() {
        let line_index = LineIndex::new("a\nbc\n\ndef\n");

        let pairs = [
            (0, 1, 1),
            (1, 1, 2),
            (2, 2, 1),
            (3, 2, 2),
            (4, 2, 3),
            (5, 3, 1),
            (6, 4, 1),
            (7, 4, 2),
            (8, 4, 3),
            (9, 4, 4),
        ];

        for (index, line, column) in pairs {
            assert_eq!(line_index.line(index), line);
            assert_eq!(line_index.fromlinecolumn(line, column), index);
        }
    }
}
