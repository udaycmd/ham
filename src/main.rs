fn main() {
    if let Err(e) = ham::executor() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
