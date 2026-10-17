use clap::Parser;

/// Asciidoc to Typst converter
#[derive(Parser, Debug)]
#[command(name = "adoc-typst")]
#[command(about = "Convert Asciidoc (.adoc) files to Typst (.typst) format")]
pub struct Cli {
    /// Input .adoc file or directory
    pub input: String,

    /// Output .typst file or directory
    #[arg(short, long)]
    pub output: String,

    /// Custom Asciidoc attributes (e.g., "icons=font")
    #[arg(short, long)]
    pub attribute: Vec<String>,

    /// Remote server URL for custom [docops,type] macro rendering (e.g., "http://localhost:3000")
    #[arg(long)]
    pub remote_macro_url: Option<String>,

    /// Output directory for generated remote macro assets (default: same directory as output file)
    #[arg(long, default_value = ".")]
    pub remote_assets_dir: String,

    /// PDF standard to target (e.g., "ua-1" for PDF/UA-1, "a-2b" for PDF/A-2b, or comma-separated like "ua-1,a-2b")
    #[arg(long)]
    pub pdf_standard: Option<String>,

    /// Document language for PDF/UA compliance (e.g., "en", "de", "fr")
    #[arg(long, default_value = "en")]
    pub lang: String,

    /// Compile to PDF using `typst` (requires `typst` on PATH)
    #[arg(long)]
    pub pdf: bool,
}
