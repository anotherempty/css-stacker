use clap::Parser;
use css_stacker::{Result, Stacker, StackerOptions};

fn main() -> Result<()> {
    let options = StackerOptions::parse();

    let output = Stacker::create(options)?;

    if let Some(style) = output.pretty {
        println!("Stylesheet created at {}", style.display());
    }

    if let Some(style_min) = output.minified {
        println!("Minified stylesheet created at {}", style_min.display());
    }

    Ok(())
}
