use clap::Parser;
use css_stacker::{Format, Result, Stacker, StyleExtension};

/// Simple program to stack css|scss|sass files into a single file
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Path to the directory containing the styles
    #[arg(short, long, default_value = "./")]
    path: String,

    /// Allowed extension of the style files
    #[arg(short, long, value_enum)]
    extensions: Vec<StyleExtension>,

    /// Name with path of the output css file without the extension
    #[arg(short, long, default_value = "./style")]
    output: String,

    /// Format of the output file
    #[arg(short, long, value_enum)]
    format: Option<Format>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (style, style_min) =
        Stacker::create(args.path, args.output, args.format, &args.extensions)?;

    if !style.is_empty() {
        println!("Stylesheet created at {style}");
    }

    if !style_min.is_empty() {
        println!("Minified stylesheet created at {style_min}");
    }

    Ok(())
}
