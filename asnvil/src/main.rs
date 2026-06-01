use anyhow::{bail, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

use asnvil_codegen::builder::CodeAstBuilder;
use asnvil_codegen::python::PythonRenderer;
use asnvil_codegen::renderer::LanguageRenderer;
use asnvil_ir::from_ast;
use asnvil_ir::resolver::Resolver;
use asnvil_parser::grammar::Grammar;
use asnvil_parser::parse;

#[derive(Parser, Debug)]
#[command(name = "asnvil", version, about = "ASN.1 Compiler", long_about = None)]
struct Cli {
    #[arg(help = "ASN.1 module files to compile")]
    input: Vec<PathBuf>,

    #[arg(short, long, help = "Output directory for generated code")]
    out_dir: Option<PathBuf>,

    #[arg(long, default_value = "python", help = "Target language: python, rust, ...")]
    lang: String,

    #[arg(long, default_value = "der", help = "Target encoding: ber, der")]
    encoding: String,

    #[arg(long, help = "Also copy runtime library to output directory")]
    emit_runtime: bool,

    #[arg(long, help = "Custom path to runtime library")]
    runtime_dir: Option<PathBuf>,

    #[arg(long, help = "Print IR instead of generating code")]
    print_ir: bool,

    #[arg(long, help = "Print AST instead of generating code")]
    print_ast: bool,

    #[arg(long, help = "Print Code AST instead of generating code")]
    print_code_ast: bool,

    #[arg(short, long, help = "Verbose output")]
    verbose: bool,

    #[arg(short, long, help = "Suppress non-error output")]
    quiet: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.input.is_empty() {
        bail!("No input files specified. Use --help for usage.");
    }

    let out_dir = cli.out_dir.unwrap_or_else(|| PathBuf::from("output"));

    if cli.verbose {
        println!("Processing {} input file(s)...", cli.input.len());
    }

    let mut resolver = Resolver::new();

    for input_path in &cli.input {
        if cli.verbose {
            println!("Parsing {}...", input_path.display());
        }

        let source = fs::read_to_string(input_path)
            .map_err(|e| anyhow::anyhow!("Failed to read input file {}: {}", input_path.display(), e))?;

        let mut grammar = Grammar::new();
        let parse_tree = parse(&source, input_path, &mut grammar)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {:?}", input_path.display(), e))?;

        grammar.parse_tree = Some(parse_tree);

        let ast_module = grammar.result
            .ok_or_else(|| anyhow::anyhow!("Parser produced no AST for {}", input_path.display()))?;

        if cli.print_ast {
            println!("{:#?}", ast_module);
            continue;
        }

        let module = from_ast::module_to_ir(&ast_module)
            .map_err(|e| anyhow::anyhow!("Failed to convert AST to IR for {}: {}", input_path.display(), e))?;

        if cli.print_ir {
            println!("{:#?}", module);
            continue;
        }

        resolver.add_module(module)
            .map_err(|e| anyhow::anyhow!("Failed to add module: {}", e))?;
    }

    resolver.resolve()
        .map_err(|e| anyhow::anyhow!("Failed to resolve: {}", e))?;

    let builder = CodeAstBuilder::new();

    for (name, module) in resolver.modules() {
        let code_ast = builder.build_module(module);

        if cli.print_code_ast {
            println!("{:#?}", code_ast);
            continue;
        }

        let renderer = match cli.lang.as_str() {
            "python" => Box::new(PythonRenderer::new()) as Box<dyn LanguageRenderer>,
            other => {
                bail!("Unsupported target language: {}", other);
            }
        };

        let output = renderer.render_module(&code_ast)
            .map_err(|e| anyhow::anyhow!("Failed to render module: {}", e))?;

        let output_path = out_dir.join(format!("{}.py", name));
        fs::create_dir_all(&out_dir)?;
        fs::write(&output_path, output)?;

        if !cli.quiet {
            println!("Generated: {}", output_path.display());
        }
    }

    if cli.emit_runtime {
        let runtime_dir = cli.runtime_dir.unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("asnvil-runtime-python")
        });

        let dest = out_dir.join("asnvil_runtime");
        fs::create_dir_all(&dest)?;

        copy_dir(&runtime_dir, &dest)?;

        if !cli.quiet {
            println!("Copied runtime to: {}", dest.display());
        }
    }

    Ok(())
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
