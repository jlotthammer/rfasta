fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(error) = rfasta::run_cli(&args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
