//! `hb-bridge-hnf` — JSON stdin/stdout bridge for Python KiCad/FreeCAD plugins.

use std::io::{self, Read, Write};

use hnf_core::HnfMutation;
use hnf_freecad::{mechanical_domain_to_mutations, mutations_to_mechanical_domain};
use hnf_kicad::{
    layout_domain_to_mutations, mutations_to_layout_domain, mutations_to_schematic_domain,
    schematic_domain_to_mutations, split_mutations,
};
use serde_json::Value;
use sidecar_protocol::Mutation;

fn read_stdin_json() -> Result<Value, String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&buf).map_err(|e| e.to_string())
}

fn write_stdout_json(value: &Value) -> Result<(), String> {
    let line = serde_json::to_string(value).map_err(|e| e.to_string())?;
    let mut out = io::stdout();
    writeln!(out, "{line}").map_err(|e| e.to_string())
}

fn parse_mutations(value: &Value) -> Result<Vec<Mutation>, String> {
    let raw = value
        .get("mutations")
        .ok_or_else(|| "expected { mutations: [...] }".to_string())?;
    serde_json::from_value(raw.clone()).map_err(|e| e.to_string())
}

fn parse_hnf_mutations(value: &Value) -> Result<Vec<HnfMutation>, String> {
    let raw = value
        .get("mutations")
        .ok_or_else(|| "expected { mutations: [...] }".to_string())?;
    serde_json::from_value(raw.clone()).map_err(|e| e.to_string())
}

fn project_path(value: &Value) -> Result<String, String> {
    value
        .get("project_path")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "expected project_path".to_string())
}

fn parse_hnf_document(value: &Value) -> Result<Value, String> {
    value
        .get("hnf_document")
        .cloned()
        .ok_or_else(|| "expected hnf_document".to_string())
}

fn usage() -> ! {
    eprintln!(
        "usage: hb-bridge-hnf <kicad|freecad> <export-layout|import-layout|export-schematic|import-schematic|export-mechanical|import-mechanical>"
    );
    std::process::exit(2);
}

fn dispatch(tool: &str, command: &str, input: &Value) -> Result<Value, String> {
    match (tool, command) {
        ("kicad", "export-layout") => {
            let project = project_path(&input)?;
            let mutations = parse_mutations(&input)?;
            let (layout, _) = split_mutations(&mutations);
            mutations_to_layout_domain(&project, &layout).map_err(|e| e.to_string())
        }
        ("kicad", "import-layout") => {
            let doc = parse_hnf_document(&input)?;
            layout_domain_to_mutations(&doc)
                .map(|m| Value::from(serde_json::json!({ "mutations": m })))
                .map_err(|e| e.to_string())
        }
        ("kicad", "export-schematic") => {
            let project = project_path(&input)?;
            let mutations = parse_mutations(&input)?;
            let (_, schematic) = split_mutations(&mutations);
            mutations_to_schematic_domain(&project, &schematic).map_err(|e| e.to_string())
        }
        ("kicad", "import-schematic") => {
            let doc = parse_hnf_document(&input)?;
            schematic_domain_to_mutations(&doc)
                .map(|m| Value::from(serde_json::json!({ "mutations": m })))
                .map_err(|e| e.to_string())
        }
        ("freecad", "export-mechanical") => {
            let project = project_path(&input)?;
            let mutations = parse_hnf_mutations(&input)?;
            mutations_to_mechanical_domain(&project, &mutations).map_err(|e| e.to_string())
        }
        ("freecad", "import-mechanical") => {
            let doc = parse_hnf_document(&input)?;
            mechanical_domain_to_mutations(&doc)
                .map(|m| Value::from(serde_json::json!({ "mutations": m })))
                .map_err(|e| e.to_string())
        }
        _ => usage(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }
    let input = read_stdin_json().unwrap_or_else(|e| {
        eprintln!("stdin JSON error: {e}");
        std::process::exit(1);
    });

    match dispatch(&args[1], &args[2], &input) {
        Ok(value) => {
            if write_stdout_json(&value).is_err() {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
