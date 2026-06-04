use crate::cli::builder::BuildMode::TranspileOnly;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    TranspileAndRun,
    TranspileOnly,
    RunOnly,
    ErrorCheck,
}

pub struct Builder {
    input_files: Vec<String>,
    mode: BuildMode,
    quiet: bool,
    max_errors: u32,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            input_files: vec![],
            mode: TranspileOnly,
            quiet: false,
            max_errors: 10,
        }
    }
}
