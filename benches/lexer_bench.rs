use criterion::{Criterion, criterion_group, criterion_main};
use ham::frontend::lexer::{lex::Lexer, pos::FileSet, token::Tok};
use std::{fs, hint::black_box, path::PathBuf};

fn load_test_files() -> Vec<(String, Vec<u8>)> {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/test_data");
    let mut entries: Vec<PathBuf> = fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .collect();

    entries.sort();

    entries
        .into_iter()
        .map(|path| {
            let contents = fs::read(&path).unwrap();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_owned();
            (name, contents)
        })
        .collect()
}

fn scan_file(file: &mut ham::frontend::lexer::pos::File, src: &[u8]) {
    let mut lexer = Lexer::new(file, src, false, |msg, pos| {
        panic!("[lexer_error]: {} at {}", msg, pos)
    });

    loop {
        let (tok, lit, pos) = lexer.scan();
        black_box((tok, lit, pos));
        if tok == Tok::Eof {
            break;
        }
    }
}

fn lexer_benchmark(c: &mut Criterion) {
    let test_files = load_test_files();

    c.bench_function("scan_all_test_data", |b| {
        b.iter(|| {
            let mut set = FileSet::new();

            for (name, src) in &test_files {
                let file = set.add_file(name.clone(), src.len());
                scan_file(file, src);
            }
        })
    });
}

criterion_group!(benches, lexer_benchmark);
criterion_main!(benches);
