use std::{fs::File, io::Write, path::Path};

use ignore::WalkBuilder;
use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::{Browsers, Targets},
};
use thiserror::Error;

const EXTENSIONS: [&str; 3] = ["scss", "css", "sass"];

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
/// * **minify** - whether to create a minified version of the output file : `false` by default
/// ### Returns
/// **(style_path, minified_style_path)** - tuple containing the path to the output file and the path to the minified output file if `minify` is `true`
/// ### Note
/// * Automatically ignores files and path mentioned in the `.gitignore` file
/// * Ignores files contained inside hidden folders
pub fn stack_styles<P>(path: P, output_path: P, minify: bool) -> (String, Option<String>)
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    let styles = retrieve_styles(path).unwrap();

    let sass = compile_sass(&styles).unwrap();

    let css = sass_to_css(&sass, minify).unwrap();

    write_styles(output_path, &css.0, css.1).unwrap()
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

fn sass_to_css(styles: &str, minify: bool) -> Result<(String, Option<String>), ProcessError> {
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

    let css = stylesheet
        .to_css(PrinterOptions::default())
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

    let styles = css.code;

    let mut styles_min = None;

    if minify {
        let css_min = stylesheet
            .to_css(PrinterOptions {
                minify: true, // removes spaces
                ..Default::default()
            })
            .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

        styles_min = Some(css_min.code);
    }

    Ok((styles, styles_min))
}

fn write_styles<P>(
    output_path: P,
    styles: &str,
    styles_min: Option<String>,
) -> Result<(String, Option<String>), ProcessError>
where
    P: AsRef<Path>,
{
    let style_path = format!("{}.css", output_path.as_ref().display());
    let mut file =
        File::create(&style_path).map_err(|err| ProcessError::Writing(err.to_string()))?;

    file.write_all(styles.as_bytes())
        .map_err(|err| ProcessError::Writing(err.to_string()))?;

    file.flush()
        .map_err(|err| ProcessError::Writing(err.to_string()))?;

    if let Some(styles_min) = styles_min {
        let style_min_path = format!("{}.min.css", output_path.as_ref().display());
        let mut file =
            File::create(&style_min_path).map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.write_all(styles_min.as_bytes())
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.flush()
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        return Ok((style_path, Some(style_min_path)));
    }

    Ok((style_path, None))
}
