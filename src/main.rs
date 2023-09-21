use clap::Parser;
use css_stacker::stack_styles;

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

    /// Whether to include the files that are the direct children of the provided path directory
    #[arg(short, long, default_value = "false")]
    minify: bool,

    /// Whether to include the files that are the direct children of the provided path directory
    #[arg(short, long, default_value = "false")]
    include_path_styles: bool,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args);

    stack_styles(
        args.path,
        args.output,
        args.minify,
        args.include_path_styles,
    );
}
