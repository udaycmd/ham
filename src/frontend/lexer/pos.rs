use std::{boxed::Box, fmt, rc::Rc};

const INVALID_POSITION: u64 = 0;

#[derive(Default)]
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
    files: Vec<Rc<LexFile>>,
    last_file: Option<Rc<LexFile>>,
    base: u64,
}

impl SourcePosition {
    pub fn is_valid(&self) -> bool {
        return !self.name.is_empty() && self.line > 0 && self.column > 0;
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
    pub fn add_line(&mut self, offset: u64) {
        let count = self.lines.len();
        if (count == 0 || self.lines[count - 1] < offset) && offset < self.max_size {
            self.lines.push(offset);
        }
    }

    pub fn lex_pos(&self, offset: u64) -> Option<u64> {
        if offset > self.max_size {
            return None;
        }

        return Some(self.base + offset);
    }

    pub fn lex_offset(&self, pos: u64) -> Option<u64> {
        if pos > (self.max_size + self.base) || pos < (self.base) {
            return None;
        }

        return Some(pos - self.base);
    }

    pub fn source_pos(&self, pos: u64) -> Option<Box<SourcePosition>> {
        let mut src_pos = Box::new(SourcePosition::default());

        if pos == INVALID_POSITION || pos > (self.max_size + self.base) || pos < (self.base) {
            return None;
        }

        let offset = pos - self.base;
        src_pos.offset = offset;
        src_pos.name = self.name.clone();

        let index = self
            .lines
            .partition_point(|&line| line <= offset)
            .saturating_sub(1);
        src_pos.line = (index + 1) as u64;
        src_pos.column = offset - self.lines[index] + 1;

        return Some(src_pos);
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

    pub fn size(&self) -> u64 {
        return self.files.iter().map(|f| f.max_size as u64).sum();
    }

    pub fn add_file(&mut self, filename: String, file_size: u64) {
        let mut file = LexFile::default();
        file.name = filename;
        file.base = self.base;
        file.max_size = file_size;
        file.lines = vec![0];

        self.base += file_size + 1;

        let file = Rc::new(file);
        self.files.push(Rc::clone(&file));
        self.last_file = Some(Rc::clone(&file));
    }

    pub fn source_pos(&mut self, pos: u64) -> Option<Box<SourcePosition>> {
        if let Some(lexfile) = self.get_lexfile(pos) {
            return lexfile.source_pos(pos);
        };

        return None;
    }

    fn get_lexfile(&mut self, pos: u64) -> Option<Rc<LexFile>> {
        if pos == INVALID_POSITION {
            return None;
        }

        if let Some(lexfile) = &self.last_file {
            if lexfile.base <= pos && pos <= lexfile.base + lexfile.max_size {
                return Some(Rc::clone(lexfile));
            }
        };

        let index = self
            .files
            .partition_point(|f| f.base <= pos)
            .saturating_sub(1);

        if let Some(lexfile) = self.files.get(index) {
            if pos <= lexfile.base + lexfile.max_size {
                self.last_file = Some(Rc::clone(lexfile));
                return Some(Rc::clone(lexfile));
            }
        }

        return None;
    }
}
