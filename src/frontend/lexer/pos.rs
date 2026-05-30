use std::fmt;

const INVALID_POSITION: u64 = 0;

#[derive(Default, Debug)]
pub struct SourcePosition {
    name: String,
    offset: u64,
    line: u64,
    column: u64,
}

#[derive(Default)]
pub struct LexFile {
    name: String,
    base: u64,
    max_size: u64,
    lines: Vec<u64>,
}

pub struct LexTape {
    files: Vec<LexFile>,
    last_file: Option<usize>,
    base: u64,
}

impl SourcePosition {
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.line > 0 && self.column > 0
    }
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "{}:{}:{}", self.name, self.line, self.column)
        } else {
            write!(f, "")
        }
    }
}

impl LexFile {
    #[inline(always)]
    pub fn add_line(&mut self, offset: u64) {
        let count = self.lines.len();
        if (count == 0 || self.lines[count - 1] < offset) && offset < self.max_size {
            self.lines.push(offset);
        }
    }

    pub fn source_pos(&self, offset: u64) -> SourcePosition {
        let mut src_pos = SourcePosition::default();

        src_pos.offset = offset;
        src_pos.name = self.name.clone();

        let index = self
            .lines
            .partition_point(|&line| line <= offset)
            .saturating_sub(1);
        src_pos.line = (index + 1) as u64;
        src_pos.column = offset - self.lines[index] + 1;

        src_pos
    }

    #[inline(always)]
    pub fn get_file_base(&self) -> u64 {
        self.base
    }

    #[inline(always)]
    pub fn lex_offset(&self, tape_pos: u64) -> Option<u64> {
        if tape_pos > (self.max_size + self.base) || tape_pos < (self.base) {
            return None;
        }

        Some(tape_pos - self.base)
    }
}

impl LexTape {
    pub fn new() -> LexTape {
        Self {
            files: vec![],
            last_file: None,
            base: 1,
        }
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.max_size as u64).sum()
    }

    #[inline(always)]
    pub fn add_file(&mut self, filename: String, file_size: u64) {
        let mut file = LexFile::default();
        file.name = filename;
        file.base = self.base;
        file.max_size = file_size;
        file.lines = vec![0];

        self.base += file_size + 1;
        self.files.push(file);
        self.last_file = Some(self.files.len() - 1);
    }

    #[inline(always)]
    pub fn source_pos(&mut self, tape_pos: u64) -> Option<SourcePosition> {
        if let Some(lex_file) = self.get_lex_file(tape_pos) {
            let offset = lex_file.lex_offset(tape_pos)?;
            return Some(lex_file.source_pos(offset));
        };

        None
    }

    fn get_lex_file(&mut self, tape_pos: u64) -> Option<&LexFile> {
        if tape_pos == INVALID_POSITION {
            return None;
        }

        if let Some(index) = self.last_file {
            let lex_file = &self.files[index];

            if lex_file.base <= tape_pos && tape_pos <= lex_file.base + lex_file.max_size {
                return Some(lex_file);
            }
        };

        let index = self
            .files
            .partition_point(|f| f.base <= tape_pos)
            .saturating_sub(1);

        if let Some(lex_file) = self.files.get(index) {
            if lex_file.base <= tape_pos && tape_pos <= lex_file.base + lex_file.max_size {
                self.last_file = Some(index);
                return Some(lex_file);
            }
        }

        None
    }
}
