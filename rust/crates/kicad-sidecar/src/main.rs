use std::io;

use kicad_sidecar::build_stdio_runner;

fn main() {
    if let Err(err) = run() {
        eprintln!("kicad-sidecar error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let runner = build_stdio_runner();
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout().lock();
    runner.run_stdio(&mut reader, &mut stdout)?;
    Ok(())
}
