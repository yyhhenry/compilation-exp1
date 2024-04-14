use anyhow::{anyhow, Result};
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
        if offset > self.content.len() {
            return (self.start_offset.len(), 1);
        }
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
}
#[derive(Debug, Clone)]
pub struct OffsetError {
    pub offset: usize,
    pub msg: String,
}
impl OffsetError {
    /// Display the error message with the line and column number.
    /// ```plaintext
    /// [file_name:line:col] Error/Warning: msg
    ///    line_content
    ///    ^
    /// ```
    pub fn display_with(
        level: &str,
        line_pos: &LinePos,
        file_name: &str,
        e: &OffsetError,
    ) -> String {
        let (line, col) = line_pos.line_col(e.offset);
        let mut result = format!("[{}:{}:{}] {}: {}\n", file_name, line, col, level, e.msg);
        result.push_str(&format!(
            "    {}\n",
            line_pos.get_line(line).unwrap_or("".to_string()).trim_end()
        ));
        result.push_str(&format!("    {}^\n", " ".repeat(col - 1)));
        result
    }
}

#[derive(Debug, Clone)]
pub struct ErrorRecorder {
    errors: Vec<OffsetError>,
    warnings: Vec<OffsetError>,
}
impl ErrorRecorder {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    pub fn hard<T>(&mut self, offset: usize, msg: impl AsRef<str>) -> Result<T> {
        let msg = msg.as_ref().to_string();
        self.error(offset, msg.clone());
        Err(anyhow!(msg))
    }
    pub fn error(&mut self, offset: usize, msg: impl AsRef<str>) {
        let msg = msg.as_ref().to_string();
        self.errors.push(OffsetError { offset, msg });
    }
    pub fn warning(&mut self, offset: usize, msg: impl AsRef<str>) {
        let msg = msg.as_ref().to_string();
        self.warnings.push(OffsetError { offset, msg });
    }
    pub fn no_error(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn display_with(&self, file_name: &str, content: &str) -> String {
        let line_pos = LinePos::new(content);
        let errors = self.errors.iter().map(|e| {
            (
                e.offset,
                OffsetError::display_with("Error", &line_pos, file_name, e),
            )
        });
        let warnings = self.warnings.iter().map(|e| {
            (
                e.offset,
                OffsetError::display_with("Warning", &line_pos, file_name, e),
            )
        });
        let mut all: Vec<_> = errors.chain(warnings).collect();
        all.sort_by_key(|x| x.0);
        all.into_iter().map(|(_, s)| s).collect()
    }
    /// Display the error message with the line and column number.
    pub fn print_with(&self, file_name: &str, content: &str) {
        eprintln!("{}", self.display_with(file_name, content));
    }
}
