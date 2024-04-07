#[derive(Debug, Clone)]
pub struct OffsetError {
    pub offset: usize,
    pub msg: String,
}
#[derive(Debug, Clone)]
pub struct LinePos {
    content: Vec<char>,
    start_offset: Vec<usize>,
}

impl LinePos {
    pub fn new(content: &str) -> Self {
        let mut content: Vec<_> = content.chars().collect();
        if content.last() != Some(&'\n') {
            content.push('\n');
        }
        let mut start_offset = vec![0];
        for (i, c) in content.iter().enumerate() {
            if *c == '\n' {
                start_offset.push(i + 1);
            }
        }
        LinePos {
            content,
            start_offset,
        }
    }
    /// Get the line and column number from the offset.
    /// The line and column number are 1-based.
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line = self
            .start_offset
            .binary_search(&offset)
            .map(|x| x + 1)
            .unwrap_or_else(|x| x);
        let col = offset - self.start_offset[line - 1] + 1;
        (line, col)
    }
    /// Get the line content from the line number.
    /// The line number is 1-based.
    pub fn get_line(&self, line: usize) -> Option<String> {
        let start = self.start_offset.get(line - 1)?;
        let end = self.start_offset.get(line)?;
        Some(self.content[*start..*end].iter().collect::<String>())
    }
    /// Display the error message with the line and column number.
    pub fn display_error(&self, file: &str, e: &OffsetError) {
        let (line, col) = self.line_col(e.offset);
        println!("[{}:{}:{}] Error: {}", file, line, col, e.msg);
        println!(
            "    {}",
            self.get_line(line).unwrap_or("".to_string()).trim_end()
        );
        println!("    {}^", " ".repeat(col - 1));
    }
}
