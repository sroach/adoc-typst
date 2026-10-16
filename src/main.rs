pub mod cli;
pub mod converter;
pub mod ir;
pub mod parser;
pub mod remote;
pub mod renderer;

use anyhow::Result;
use clap::Parser;
use ir::blocks::Block;
use remote::fetcher;
use std::fs;
use std::path::Path;
use std::process;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Read input file
    let input = match fs::read_to_string(&cli.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: Cannot read input file '{}': {}", cli.input, e);
            process::exit(1);
        }
    };

    // Parse Asciidoc to AST
    let ast = parser::parse(&input);

    // Convert AST to IR
    let mut ir_doc = converter::mapper::convert(&ast);

    // Resolve remote macros if --remote-macro-url is provided
    if let Some(ref server_url) = cli.remote_macro_url {
        // Determine assets directory: default to the same directory as the output file
        let assets_dir = if cli.remote_assets_dir != "." {
            cli.remote_assets_dir.clone()
        } else {
            Path::new(&cli.output)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        resolve_remote_macros(&mut ir_doc, server_url, &assets_dir)?;
    } else {
        // No remote URL provided — clear RemoteMacro bodies so the renderer
        // falls back to a code block with a helpful message.
        clear_remote_macros(&mut ir_doc);
    }

    // Render IR to Typst
    let pdf_standard = cli.pdf_standard.as_deref();
    let output = renderer::document::render_document(&ir_doc, pdf_standard, &cli.lang);

    // Write output file
    match fs::write(&cli.output, &output) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: Cannot write output file '{}': {}", cli.output, e);
            process::exit(1);
        }
    }

    println!("Converted {} -> {}", cli.input, cli.output);
    Ok(())
}

/// Walks the IR document and resolves all RemoteMacro blocks by fetching SVGs from the remote server.
fn resolve_remote_macros(
    doc: &mut ir::document::Document,
    server_url: &str,
    assets_dir: &str,
) -> Result<()> {
    let mut macro_index = 0;

    for section in &mut doc.sections {
        for block in &mut section.content {
            resolve_block_remote_macros(block, server_url, assets_dir, &mut macro_index)?;
        }
    }

    if macro_index > 0 {
        println!("[adoc-typst] Resolved {} remote macro(s)", macro_index);
    }

    Ok(())
}

/// Recursively resolves RemoteMacro blocks within a single block.
fn resolve_block_remote_macros(
    block: &mut Block,
    server_url: &str,
    assets_dir: &str,
    index: &mut usize,
) -> Result<()> {
    match block {
        Block::RemoteMacro { viz_type, body } => {
            let body_clone = body.clone();
            let viz_type_clone = viz_type.clone();

            match fetch_svg_from_server(server_url, &viz_type_clone, &body_clone) {
                Ok(svg) => {
                    let filename =
                        fetcher::save_svg(assets_dir, &viz_type_clone, &svg, *index)?;
                    println!(
                        "[adoc-typst] Saved {} -> {}",
                        viz_type_clone, filename
                    );
                    *body = filename; // Replace body with the SVG file path
                    *index += 1;
                }
                Err(e) => {
                    eprintln!(
                        "[adoc-typst] Warning: Failed to fetch remote macro '{}': {}",
                        viz_type_clone, e
                    );
                    // Leave body empty so the renderer falls back to a code block
                    *body = String::new();
                }
            }
        }
        Block::QuoteBlock { content, .. } => {
            for sub in content {
                resolve_block_remote_macros(sub, server_url, assets_dir, index)?;
            }
        }
        Block::Admonition { content, .. } => {
            for sub in content {
                resolve_block_remote_macros(sub, server_url, assets_dir, index)?;
            }
        }
        Block::Section(nested) => {
            for sub in &mut nested.content {
                resolve_block_remote_macros(sub, server_url, assets_dir, index)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Fetches SVG from the remote server for a given macro type and body.
fn fetch_svg_from_server(server_url: &str, viz_type: &str, body: &str) -> Result<String> {
    fetcher::fetch_svg(server_url, viz_type, body)
}

/// Clears all RemoteMacro bodies so the renderer falls back to a code block.
fn clear_remote_macros(doc: &mut ir::document::Document) {
    for section in &mut doc.sections {
        for block in &mut section.content {
            clear_block_remote_macros(block);
        }
    }
}

fn clear_block_remote_macros(block: &mut Block) {
    match block {
        Block::RemoteMacro { body, .. } => {
            *body = String::new();
        }
        Block::QuoteBlock { content, .. } => {
            for sub in content {
                clear_block_remote_macros(sub);
            }
        }
        Block::Admonition { content, .. } => {
            for sub in content {
                clear_block_remote_macros(sub);
            }
        }
        Block::Section(nested) => {
            for sub in &mut nested.content {
                clear_block_remote_macros(sub);
            }
        }
        _ => {}
    }
}
