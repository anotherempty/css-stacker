use clap::Parser;
use css_stacker::{Result, Stacker, StackerOptions};

fn main() -> Result<()> {
    let options = StackerOptions::parse();

    let (style, style_min) = Stacker::create(options)?;

    if !style.is_empty() {
        println!("Stylesheet created at {style}");
    }

    if !style_min.is_empty() {
        println!("Minified stylesheet created at {style_min}");
    }

    Ok(())
}
