use std::{fs::File, io::Write, path::Path};

use clap::ValueEnum;
use ignore::WalkBuilder;
use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::{Browsers, Targets},
};
use thiserror::Error;

const EXTENSIONS: [&str; 3] = ["scss", "css", "sass"];

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Minified,
    Pretty,
    Both,
}

#[derive(Debug, Error)]
enum ProcessError {
    #[error("Style collecting error: {0}")]
    StyleSearch(String),
    #[error("Sass processing error: {0}")]
    Sass(String),
    #[error("Css processing error: {0}")]
    Stylesheet(String),
    #[error("File handling error: {0}")]
    Writing(String),
}

/// stack css/sass/scss files into a single css file
/// ### Arguments
/// * **path** - path to the directory containing the styles : defaults to current directory
/// * **output_path** - name with path of the output css file without the extension : defaults to `'./style'`
/// * **format** - format of the output file : defaults to `Format::Both`
/// ### Returns
/// **(style_path, minified_style_path)** - tuple containing the path to the output file and the path to the minified output file
/// ### Note
/// * Automatically ignores files and path mentioned in the `.gitignore` file
/// * Ignores files contained inside hidden folders
pub fn stack_styles<P>(path: P, output_path: P, format: Format) -> (String, String)
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    let styles = retrieve_styles(path).unwrap();

    let sass = compile_sass(&styles).unwrap();

    let css = sass_to_css(&sass, format).unwrap();

    write_styles(output_path, css.0, css.1).unwrap()
}

fn retrieve_styles<P>(path: P) -> Result<String, ProcessError>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    let mut styles = String::new();

    for result in WalkBuilder::new(&path).hidden(true).build() {
        match result {
            Ok(entry) => {
                if EXTENSIONS.contains(
                    &entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                ) {
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
            }
            Err(err) => return Err(ProcessError::StyleSearch(err.to_string())),
        }
    }

    Ok(styles)
}

fn compile_sass(styles: &str) -> Result<String, ProcessError> {
    grass::from_string(styles, &grass::Options::default())
        .map_err(|err| ProcessError::Sass(err.to_string()))
}

fn sass_to_css(
    styles: &str,
    format: Format,
) -> Result<(Option<String>, Option<String>), ProcessError> {
    let mut stylesheet = StyleSheet::parse(styles, ParserOptions::default())
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

    let targets = Targets {
        browsers: Browsers::from_browserslist([
            ">0.3%, defaults, supports es6-module, maintained node versions",
        ])
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?,
        ..Default::default()
    };

    // * note: doesn't remove spaces but does minify structuraly according to the options provided
    stylesheet
        .minify(MinifyOptions {
            targets,
            ..Default::default()
        })
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

    match format {
        Format::Minified => {
            let css_min = stylesheet
                .to_css(PrinterOptions {
                    minify: true, // removes spaces
                    ..Default::default()
                })
                .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

            Ok((None, Some(css_min.code)))
        }
        Format::Pretty => {
            let css = stylesheet
                .to_css(PrinterOptions::default())
                .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

            Ok((Some(css.code), None))
        }
        Format::Both => {
            let css_min = stylesheet
                .to_css(PrinterOptions {
                    minify: true, // removes spaces
                    ..Default::default()
                })
                .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

            let css = stylesheet
                .to_css(PrinterOptions::default())
                .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

            Ok((Some(css.code), Some(css_min.code)))
        }
    }
}

fn write_styles<P>(
    output_path: P,
    styles: Option<String>,
    styles_min: Option<String>,
) -> Result<(String, String), ProcessError>
where
    P: AsRef<Path>,
{
    let mut style_path = "".to_string();
    if let Some(styles) = styles {
        style_path = format!("{}.css", output_path.as_ref().display());
        let mut file =
            File::create(&style_path).map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.write_all(styles.as_bytes())
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.flush()
            .map_err(|err| ProcessError::Writing(err.to_string()))?;
    }

    let mut style_min_path = "".to_string();
    if let Some(styles_min) = styles_min {
        style_min_path = format!("{}.min.css", output_path.as_ref().display());
        let mut file =
            File::create(&style_min_path).map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.write_all(styles_min.as_bytes())
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.flush()
            .map_err(|err| ProcessError::Writing(err.to_string()))?;
    }

    Ok((style_path, style_min_path))
}
