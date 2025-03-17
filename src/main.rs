use clap::Parser;
use css_stacker::{Result, Stacker, StackerOptions};
#[cfg(feature = "tracing")]
use tracing::Level;

fn main() -> Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .without_time()
        .init();

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
