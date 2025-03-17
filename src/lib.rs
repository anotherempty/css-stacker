mod types;

use clap::Parser;
pub use types::{Format, Result, StyleExtension};
pub struct Stacker;

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    targets::{Browsers, Targets},
};

use crate::types::StackerError;

const DEFAULT_OUTPUT_NAME: &str = "styles";

/// Simple program to stack css|scss|sass files into a single file
#[derive(Parser, Debug)]
#[command(version)]
pub struct StackerOptions {
    /// Path to the directory containing the styles.
    #[arg(short, long, default_value = "./")]
    pub path: PathBuf,

    /// Allowed file extensions that will be added to the stacked file.
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

    /// Format of the output file.
    #[arg(short = 'f', long, value_enum)]
    pub output_format: Option<Format>,
}

impl Stacker {
    pub fn create(options: StackerOptions) -> Result<(String, String)> {
        let filename = options
            .output_name
            .unwrap_or(DEFAULT_OUTPUT_NAME.to_string());

        let output_path = options
            .output_dir
            .as_deref()
            .unwrap_or_else(|| Path::new("."))
            .join(filename);

        let styles = Self::collect(options.output_dir.unwrap_or_default(), &options.extensions)?;
        let sass = Self::process_sass(styles)?;
        let (styles, styles_min) = Self::sass_to_css(sass, options.output_format)?;

        Self::save(output_path, styles, styles_min)
    }

    fn collect<P>(path: P, allowed_extensions: &[StyleExtension]) -> Result<String>
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let mut styles = String::new();

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

    fn process_sass(raw_styles: String) -> Result<String> {
        grass::from_string(raw_styles, &grass::Options::default())
            .map_err(|err| StackerError::Sass(err.to_string()))
    }

    fn sass_to_css(
        sass: String,
        format: Option<Format>,
    ) -> Result<(Option<String>, Option<String>)> {
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
                let css_min = stylesheet
                    .to_css(PrinterOptions {
                        minify: true, // removes spaces
                        ..Default::default()
                    })
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((None, Some(css_min.code)))
            }
            Some(Format::Pretty) => {
                let css = stylesheet
                    .to_css(PrinterOptions::default())
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((Some(css.code), None))
            }
            None => {
                let css_min = stylesheet
                    .to_css(PrinterOptions {
                        minify: true, // removes spaces
                        ..Default::default()
                    })
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                let css = stylesheet
                    .to_css(PrinterOptions::default())
                    .map_err(|err| StackerError::Stylesheet(err.to_string()))?;

                Ok((Some(css.code), Some(css_min.code)))
            }
        }
    }

    fn save<P>(
        output_path: P,
        styles: Option<String>,
        styles_min: Option<String>,
    ) -> Result<(String, String)>
    where
        P: AsRef<Path>,
    {
        let path = PathBuf::from(output_path.as_ref());
        let dir = path.parent().ok_or_else(|| {
            StackerError::Save("Failed to get the parent directory of the output path".to_string())
        })?;

        fs::create_dir_all(dir).map_err(|err| StackerError::Save(err.to_string()))?;

        let mut style_path = "".to_string();
        if let Some(styles) = styles {
            style_path = format!("{}.css", output_path.as_ref().display());
            let mut file =
                File::create(&style_path).map_err(|err| StackerError::Save(err.to_string()))?;

            file.write_all(styles.as_bytes())
                .map_err(|err| StackerError::Save(err.to_string()))?;

            file.flush()
                .map_err(|err| StackerError::Save(err.to_string()))?;
        }

        let mut style_min_path = "".to_string();
        if let Some(styles_min) = styles_min {
            style_min_path = format!("{}.min.css", output_path.as_ref().display());
            let mut file =
                File::create(&style_min_path).map_err(|err| StackerError::Save(err.to_string()))?;

            file.write_all(styles_min.as_bytes())
                .map_err(|err| StackerError::Save(err.to_string()))?;

            file.flush()
                .map_err(|err| StackerError::Save(err.to_string()))?;
        }

        Ok((style_path, style_min_path))
    }
}
