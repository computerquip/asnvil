use parol::build::Builder;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut builder = Builder::with_explicit_output_dir(&out_dir);
    builder.set_cargo_integration(false);
    builder.grammar_file("src/asn1.par");
    builder.parser_output_file("parser.rs");
    builder.actions_output_file("grammar_trait.rs");
    builder.node_kind_enums_output_file("node_kind.rs");
    builder.disable_output_sanity_checks();
    builder
        .generate_parser()
        .expect("Failed to generate parser");

    // Strip inner attributes from grammar_trait.rs since they cause issues with include!()
    let trait_path = out_dir.join("grammar_trait.rs");
    if let Ok(content) = fs::read_to_string(&trait_path) {
        let cleaned = content
            .lines()
            .filter(|line| !line.trim_start().starts_with("#!["))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&trait_path, cleaned).expect("Failed to write cleaned grammar_trait.rs");
    }

    // Do the same for parser.rs if it has inner attributes
    let parser_path = out_dir.join("parser.rs");
    if let Ok(content) = fs::read_to_string(&parser_path) {
        let cleaned = content
            .lines()
            .filter(|line| !line.trim_start().starts_with("#!["))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&parser_path, cleaned).expect("Failed to write cleaned parser.rs");
    }
}
