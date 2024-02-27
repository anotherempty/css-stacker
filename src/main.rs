use clap::Parser;
use css_stacker::{Format, Result, Stacker};

/// Simple program to stack css|scss|sass files into a single file
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Path to the directory containing the styles
    #[arg(short, long, default_value = "./")]
    path: String,

    /// Name with path of the output css file without the extension
    #[arg(short, long, default_value = "./style")]
    output: String,

    /// Format of the output file
    #[arg(short, long, value_enum, default_value = "both")]
    format: Option<Format>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (style, style_min) = Stacker::create(args.path, args.output, args.format)?;

    if !style.is_empty() {
        println!("Stylesheet created at {style}");
    }

    if !style_min.is_empty() {
        println!("Minified stylesheet created at {style_min}");
    }

    Ok(())
}
