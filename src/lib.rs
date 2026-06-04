#![recursion_limit = "512"]

mod cli;
pub mod frontend;

pub fn executor() -> Result<(), String> {
    const MAX_STACK_SIZE: usize = 128 * 1024 * 1024; // 128 MB

    let handler = std::thread::Builder::new()
        .name(String::from("ham_main"))
        .stack_size(MAX_STACK_SIZE)
        .spawn(|| {
            let args: Vec<String> = std::env::args().collect();
            let builder = cli::builder::Builder::new();
            if let Err(e) = builder.parse_cli_args(&args) {
                return Err(e);
            }

            Ok(())
        })
        .expect("internal_error: failed to spawn main thread");

    match handler.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("internal_error: main thread panicked during execution".to_owned()),
    }
}
