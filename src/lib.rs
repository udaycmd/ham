#![recursion_limit = "512"]

pub mod frontend;

pub fn executor() -> Result<(), String> {
    const MAX_STACK_SIZE: usize = 128 * 1024 * 1024; // 128 MB

    let handler = std::thread::Builder::new()
        .name(String::from("ham_main"))
        .stack_size(MAX_STACK_SIZE)
        .spawn(|| println!("Hello, World"))
        .map_err(|e| format!("internal_error: failed to spawn main thread: {e}"))?;

    handler.join().map_err(|e| {
        format!(
            "internal_error: main thread panicked during execution [cause: {:?}]",
            e
        )
    })?;

    Ok(())
}
