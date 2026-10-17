# adoc-typst

Convert Asciidoc (`.adoc`) documents to Typst (`.typst`) format.

## Installation

```bash
git clone https://github.com/example/adoc-typst.git
cd adoc-typst
cargo install --path .
```

Or run directly without installing:

```bash
cargo run --release -- input.adoc
```

## Usage

### Convert a single file

```bash
# Output to stdout
adoc-typst docs/minimal.adoc

# Output to a file
adoc-typst docs/basic.adoc --output output.typst
```

### Pass custom attributes

```bash
adoc-typst input.adoc --attribute "icons=font" --output output.typst
```

### Compile to PDF

If `typst` is installed on your system, use `--pdf` to compile the generated Typst file to PDF automatically:

```bash
adoc-typst input.adoc --output output.typst --pdf
```

This will produce `output.pdf` alongside `output.typst`. If `typst` is not found on your PATH, an error message with installation instructions will be shown.

### Render custom `[docops,type]` macros remotely

If your document contains custom Asciidoctor macros like `[docops,recipe]`, you can render them by pointing to a remote server that returns SVGs. The production DocOps server is available at `https://roach.gy/docops-ext/`:

```bash
adoc-typst docs/recipe.adoc --output output.typst --remote-macro-url https://roach.gy/docops-ext/
```

The converter will:
1. Detect `[docops,type]` blocks in the document
2. Gzip-compress and base64-encode the block body
3. Send a GET request to `https://roach.gy/docops-ext/svg?type=TYPE&data=ENCODED_BODY&format=pdf`
4. Save the returned SVG as a local file (e.g., `docops-recipe-0.svg`)
5. Emit `#image("docops-recipe-0.svg")` in the Typst output

Without `--remote-macro-url`, custom macros fall back to a code block with a note.

You can control where generated SVGs are saved:

```bash
adoc-typst input.adoc --output output.typst \
  --remote-macro-url https://roach.gy/docops-ext/ \
  --remote-assets-dir ./assets
```

### Example with sample files

Three sample Asciidoc files are included in the `docs/` directory:

```bash
# Minimal — quick smoke test
adoc-typst docs/minimal.adoc --output minimal.typst

# Basic — common features
adoc-typst docs/basic.adoc --output basic.typst

# Comprehensive — all supported constructs
adoc-typst docs/comprehensive.adoc --output comprehensive.typst
```

## Supported Asciidoc Features

### Document Metadata
- Title, author, and date

### Inline Formatting
- **Bold** (`*text*` or `**text**`)
- *Italic* (`_text_` or `__text__`)
- `Inline code` (backticks)
- ==Highlighted text== (double equals)
- Subscript (`H~2~O`)
- Superscript (`X^2^`)
- Footnotes (`foot:[text]`)

### Blocks
- Section headings (`==`, `===`, `====`, etc.)
- Paragraphs
- Unordered lists (`-`)
- Ordered lists (`+`)
- Nested lists
- Code blocks with language highlighting
- Tables

### Special Blocks
- Admonitions: NOTE, TIP, IMPORTANT, WARNING, CAUTION
- Block quotes with attribution
- Images (`image::path[alt]`)
- Links (`[text](url)`)

## Project Structure

```
src/
├── main.rs           # CLI entry point
├── cli.rs            # CLI argument parsing (clap)
├── parser/           # Asciidoc parsing
├── ir/               # Intermediate representation
├── converter/        # AST → IR mapping
├── renderer/         # IR → Typst rendering
└── remote/           # Remote macro fetching (gzip, HTTP, SVG)
```

## License

MIT
