use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(pub usize);

const INVALID_POSITION: Position = Position(0);

impl Position {
    #[inline]
    pub fn is_valid(&self) -> bool {
        *self != INVALID_POSITION
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    name: String,
    offset: usize,
    pub line: usize,
    pub column: usize,
}

impl SourcePosition {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.line > 0
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

pub struct FileSet {
    files: Vec<File>,
    last_touched_file: Option<usize>,
    base: usize,
}

impl FileSet {
    pub fn new() -> Self {
        Self {
            files: vec![],
            last_touched_file: None,
            base: 1,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.files.iter().map(|f| f.size).sum()
    }

    pub fn add_file(&mut self, file_name: String, file_size: usize) -> &mut File {
        let file = File {
            name: file_name,
            base: self.base,
            size: file_size,
            lines: vec![0],
        };

        let next_base = self
            .base
            .checked_add(file_size + 1)
            .expect("crash: offset overflow");

        self.base = next_base;
        let index = self.files.len();
        self.files.push(file);
        self.last_touched_file = Some(index);

        self.files.last_mut().unwrap()
    }

    #[inline]
    pub fn source_pos(&mut self, p: Position) -> Option<SourcePosition> {
        self.get_file(p).map(|f| f.source_pos(p))
    }

    fn get_file(&mut self, p: Position) -> Option<&File> {
        if !p.is_valid() {
            return None;
        }

        let p_val = p.0;

        if let Some(idx) = self.last_touched_file {
            if let Some(f) = self.files.get(idx) {
                if p_val >= f.base && p_val <= f.base + f.size {
                    return Some(f);
                }
            }
        }

        let mut idx = self.files.partition_point(|f| f.base <= p_val);

        if idx > 0 {
            idx -= 1;
            let f = &self.files[idx];

            if p_val <= f.base + f.size {
                self.last_touched_file = Some(idx);
                return Some(f);
            }
        }

        None
    }
}

pub struct File {
    name: String,
    size: usize,
    lines: Vec<usize>,
    pub base: usize,
}

impl File {
    #[inline]
    pub fn add_line(&mut self, offset: usize) {
        if (self.lines.is_empty() || *self.lines.last().unwrap() < offset) && offset < self.size {
            self.lines.push(offset);
        }
    }

    #[inline]
    pub fn to_set_pos(&self, offset: usize) -> Position {
        assert!(offset <= self.size, "invalid file offset");
        Position(self.base + offset)
    }

    #[inline]
    pub fn to_file_offset(&self, p: Position) -> usize {
        assert!(
            p.0 >= self.base && p.0 <= self.base + self.size,
            "illegal tape position"
        );

        p.0 - self.base
    }

    pub fn source_pos(&self, p: Position) -> SourcePosition {
        if !p.is_valid() {
            return SourcePosition {
                name: self.name.clone(),
                offset: 0,
                line: 0,
                column: 0,
            };
        }

        assert!(
            p.0 >= self.base && p.0 <= self.base + self.size,
            "illegal position"
        );

        let offset = p.0 - self.base;
        let mut idx = self.lines.partition_point(|&l| l <= offset);

        if idx > 0 {
            idx -= 1;
        }

        SourcePosition {
            name: self.name.clone(),
            offset,
            line: idx + 1,
            column: offset - self.lines[idx] + 1,
        }
    }
}
