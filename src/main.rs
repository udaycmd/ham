fn main() {
    if let Err(out) = ham::executor() {
        eprintln!("{out}");
        std::process::exit(1);
    }
}
