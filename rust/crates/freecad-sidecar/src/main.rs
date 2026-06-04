use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, Mutex};

use freecad_sidecar::{build_runner, select_engine_bridge, MechanicalMutationAdapter, SidecarState};
use sidecar_protocol::Capabilities;
use sidecar_runner::RunnerConfig;

fn main() {
    if let Err(err) = run() {
        eprintln!("freecad-sidecar error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(SidecarState::default()));
    let runner = build_runner(
        RunnerConfig {
            sidecar_name: "freecad".to_string(),
            sidecar_version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: Capabilities {
                supportsSceneGraphWrites: Some(true),
                supportsRoundtripExport: Some(true),
                extra: BTreeMap::new(),
            },
        },
        state,
        Arc::new(MechanicalMutationAdapter),
        select_engine_bridge(),
    );

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout().lock();
    runner.run_stdio(&mut reader, &mut stdout)?;
    Ok(())
}
