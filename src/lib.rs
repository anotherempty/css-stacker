mod types;

use clap::Parser;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
pub use types::{Format, Result, StyleExtension};

use ignore::WalkBuilder;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    targets::{Browsers, Targets},
};
#[cfg(feature = "tracing")]
use tracing::debug as println;

use crate::types::StackerError;

const DEFAULT_OUTPUT_NAME: &str = "styles";

pub struct StackerOutput {
    pub pretty: Option<PathBuf>,
    pub minified: Option<PathBuf>,
}

/// Simple program to stack css|scss|sass files into a single file
#[derive(Parser, Debug)]
#[command(version)]
pub struct StackerOptions {
    /// Path to the directory containing the styles.
    #[arg(short, long, default_value = "./")]
    pub path: PathBuf,

    /// Restrict file extensions to be processed.
    /// When not provided, all css, scss and sass files are processed.
    #[arg(short, long, value_enum)]
    pub extensions: Vec<StyleExtension>,

    /// Path of the output directory.
    /// Defaults to the current directory.
    #[arg(short = 'd', long)]
    pub output_dir: Option<PathBuf>,

    /// Name of the output file.
    /// Defaults to 'styles'.
    #[arg(short = 'n', long)]
    pub output_name: Option<String>,

    /// Format of the output file. When not provided, both minified and pretty formats are generated.
    #[arg(short = 'f', long, value_enum)]
    pub output_format: Option<Format>,

    /// Show verbose output.
    #[arg(short, long)]
    pub verbose: bool,
}

pub struct Stacker;

impl Stacker {
    /// Returns the path to the generated styles and minified styles.
    pub fn create(options: StackerOptions) -> Result<StackerOutput> {
        let filename = options
            .output_name
            .unwrap_or(DEFAULT_OUTPUT_NAME.to_string());

        let output_directory = options
            .output_dir
            .as_deref()
            .unwrap_or_else(|| Path::new("."));

        let verbose = options.verbose || cfg!(feature = "tracing");

        let styles = Self::collect(options.path, &options.extensions, verbose)?;
        let sass = Self::process_sass(styles, verbose)?;
        let (styles, styles_min) = Self::sass_to_css(sass, options.output_format, verbose)?;

        Self::save(output_directory, filename, styles, styles_min, verbose)
    }

    fn collect<P>(path: P, allowed_extensions: &[StyleExtension], verbose: bool) -> Result<String>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let mut styles = String::new();

        if verbose {
            println!("Collecting styles from: {}", path.as_ref().display());
        }

        for result in WalkBuilder::new(&path).hidden(true).build() {
            let Ok(entry) = result else {
                continue;
            };

            if let Some(ext) = StyleExtension::from_os_str(entry.path().extension()) {
                if !allowed_extensions.is_empty() && !allowed_extensions.contains(&ext) {
                    continue;
                }
            } else {
                continue;
            }

            if StyleExtension::from_os_str(entry.path().extension()).is_none() {
                continue;
            }

            let path = entry.path().display();

            if verbose {
                println!("Adding file: {}", path);
            }

            styles.push_str(&format!(
                "@use '{}' as {};\n",
                path,
                path.to_string()
                    .replace('_', "-")
                    .replace(['/', '.', '\\'], "_")
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
            ));
        }

        if styles.is_empty() {
            return Err(StackerError::Collect("No styles found".to_string()));
        }

        Ok(styles)
    }

    fn process_sass(raw_styles: String, verbose: bool) -> Result<String> {
        if verbose {
            println!("Processing SASS");
        }

        grass::from_string(raw_styles, &grass::Options::default())
            .map_err(|err| StackerError::Sass(err.to_string()))
    }

    fn sass_to_css(
        sass: String,
        format: Option<Format>,
        verbose: bool,
    ) -> Result<(Option<String>, Option<String>)> {
        if verbose {
            println!("Converting SASS to CSS");
        }

        let mut stylesheet = StyleSheet::parse(&sass, ParserOptions::default())
            .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

        let targets = Targets {
            browsers: Browsers::from_browserslist([
                ">0.3%, defaults, supports es6-module, maintained node versions",
            ])
            .map_err(|err| StackerError::Stylesheet(err.to_string()))?,
            ..Default::default()
        };

        // * note: doesn't remove spaces but does minify structuraly according to the options provided
        stylesheet
            .minify(MinifyOptions {
                targets,
                ..Default::default()
            })
            .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

        match format {
            Some(Format::Minified) => {
                if verbose {
                    println!("Generating minified CSS");
                }

                let css_min = stylesheet
                    .to_css(PrinterOptions {
                        minify: true, // removes spaces
                        ..Default::default()
                    })
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((None, Some(css_min.code)))
            }
            Some(Format::Pretty) => {
                if verbose {
                    println!("Generating pretty CSS");
                }

                let css = stylesheet
                    .to_css(PrinterOptions::default())
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((Some(css.code), None))
            }
            None => {
                if verbose {
                    println!("Generating minified CSS");
                }

                let css_min = stylesheet
                    .to_css(PrinterOptions {
                        minify: true, // removes spaces
                        ..Default::default()
                    })
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                if verbose {
                    println!("Generating pretty CSS");
                }

                let css = stylesheet
                    .to_css(PrinterOptions::default())
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((Some(css.code), Some(css_min.code)))
            }
        }
    }

    fn save<P>(
        output_dir: P,
        filename: String,
        styles: Option<String>,
        styles_min: Option<String>,
        verbose: bool,
    ) -> Result<StackerOutput>
    where
        P: AsRef<Path>,
    {
        if verbose {
            println!("Making sure output directory exists");
        }

        fs::create_dir_all(&output_dir).map_err(|err| StackerError::Save(err.to_string()))?;

        let output_path = output_dir.as_ref().join(filename);

        let mut style_path = None;
        if let Some(styles) = styles {
            let path = output_path.with_extension("css");
            let mut file =
                File::create(&path).map_err(|err| StackerError::Save(err.to_string()))?;

            if verbose {
                println!("Saving styles to: {}", path.display());
            }

            style_path = Some(path);

            file.write_all(styles.as_bytes())
                .map_err(|err| StackerError::Save(err.to_string()))?;

            file.flush()
                .map_err(|err| StackerError::Save(err.to_string()))?;
        }

        let mut style_min_path = None;
        if let Some(styles_min) = styles_min {
            let path = output_path.with_extension("min.css");
            let mut file =
                File::create(&path).map_err(|err| StackerError::Save(err.to_string()))?;

            if verbose {
                println!("Saving minified styles to: {}", path.display());
            }

            style_min_path = Some(path);

            file.write_all(styles_min.as_bytes())
                .map_err(|err| StackerError::Save(err.to_string()))?;

            file.flush()
                .map_err(|err| StackerError::Save(err.to_string()))?;
        }

        Ok(StackerOutput {
            pretty: style_path,
            minified: style_min_path,
        })
    }
}
