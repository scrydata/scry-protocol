//! Build script for scry-protocol
//!
//! Generates Rust code from FlatBuffers schemas if `flatc` is available.
//! If `flatc` is not installed, uses pre-generated code in src/generated/.

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Only regenerate if schema files change
    println!("cargo:rerun-if-changed=schema/database_event.fbs");
    println!("cargo:rerun-if-changed=schema/query_event.fbs");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Try to find flatc
    let flatc = find_flatc();

    if let Some(flatc_path) = flatc {
        generate_flatbuffers(&flatc_path, &out_dir);
    } else {
        println!(
            "cargo:warning=flatc not found. Using pre-generated code. \
             Install flatc to regenerate: https://github.com/google/flatbuffers/releases"
        );
    }
}

fn find_flatc() -> Option<String> {
    // Check if flatc is in PATH
    if Command::new("flatc").arg("--version").output().is_ok() {
        return Some("flatc".to_string());
    }

    // Check common installation locations
    let common_paths = if cfg!(windows) {
        vec![
            r"C:\Program Files\FlatBuffers\flatc.exe",
            r"C:\flatbuffers\flatc.exe",
        ]
    } else {
        vec![
            "/usr/local/bin/flatc",
            "/usr/bin/flatc",
            "/opt/homebrew/bin/flatc",
        ]
    };

    for path in common_paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

fn generate_flatbuffers(flatc: &str, out_dir: &str) {
    let schema_dir = Path::new("schema");

    // Generate code for database_event.fbs
    let status = Command::new(flatc)
        .args([
            "--rust",
            "-o",
            out_dir,
            "--gen-all",
            "--gen-object-api",
            schema_dir.join("database_event.fbs").to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Generated FlatBuffers code for database_event.fbs");
        }
        Ok(s) => {
            println!(
                "cargo:warning=flatc failed with exit code {:?}. Using pre-generated code.",
                s.code()
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run flatc: {}. Using pre-generated code.",
                e
            );
        }
    }
}
