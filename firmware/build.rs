//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.
//!
//! The build script also sets the linker flags to tell it which link script to use.

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let out_path = PathBuf::from(out);

    // Read the original memory.x content
    let mut memory_x_content = String::new();
    {
        let memory_x_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("memory.x");
        let mut memory_x_file = File::open(memory_x_path).unwrap();
        memory_x_file.read_to_string(&mut memory_x_content).unwrap();
    }

    let modified_content = match env::var("FLASH_ORIGIN") {
        Ok(val) => {
            let flash_origin = match val.as_str() {
                "SECOND" => "0x08100000",
                "FIRST" => "0x08040000",
                _ => panic!("Invalid value for FLASH_ORIGIN. Use 'FIRST' or 'SECOND'."),
            };
            // Replace the FLASH: ORIGIN placeholder or value
            memory_x_content
                .lines()
                .map(|line| {
                    if line.starts_with("  FLASH  : ORIGIN = 0x") {
                        format!("FLASH  : ORIGIN = {}, LENGTH = 1M", flash_origin)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(_) => memory_x_content,
    };

    // Write the modified memory.x to the output directory
    let modified_memory_x_path = out_path.join("memory.x");
    let mut modified_memory_x_file = File::create(modified_memory_x_path).unwrap();
    modified_memory_x_file
        .write_all(modified_content.as_bytes())
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
}
