use std::{fs::File, io::Write, path::Path};

use ignore::WalkBuilder;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use thiserror::Error;

// assemble all css files
// compile sass
// process with lightningcss

// options :
// - ignore files and folders
// - input and output file
// - minification and prettiness
// - browsers targets
// - use gitignore
// -

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

fn main() {
    let styles = retrieve_styles("./").unwrap();

    let sass = compile_sass(&styles).unwrap();

    let css = sass_to_css(&sass, true).unwrap();

    write_styles("style", &css.0, css.1).unwrap();
}

fn retrieve_styles<P>(path: P) -> Result<String, ProcessError>
where
    P: AsRef<Path>,
{
    let mut styles = String::new();

    for result in WalkBuilder::new(path)
        .hidden(false)
        .filter_entry(|p| {
            !(p.path().parent().unwrap().to_str().unwrap() == "." && p.path().is_file())
        })
        .build()
    {
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
                            .replace(['/', '.', '&'], "_")
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
    let stylesheet = StyleSheet::parse(styles, ParserOptions::default())
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

    // * doesn't seem to do any structural modification : minification, vendor prefixes, etc
    // stylesheet
    //     .minify(MinifyOptions {
    //         targets: Targets {
    //             browsers: Some(Browsers {
    //                 chrome: Some(50),
    //                 edge: Some(12),
    //                 ie: Some(10),
    //                 safari: Some(12),
    //                 opera: Some(40),
    //                 firefox: Some(50),
    //                 ..Default::default()
    //             }),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .unwrap();

    let css = stylesheet
        .to_css(PrinterOptions {
            minify: false,
            ..Default::default()
        })
        .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

    let styles = css.code;

    let mut styles_min = None;

    if minify {
        let css_min = stylesheet
            .to_css(PrinterOptions {
                minify: true,
                ..Default::default()
            })
            .map_err(|err| ProcessError::Stylesheet(err.to_string()))?;

        styles_min = Some(css_min.code);
    }

    Ok((styles, styles_min))
}

fn write_styles(
    filename: &str,
    styles: &str,
    styles_min: Option<String>,
) -> Result<(), ProcessError> {
    let mut file = File::create(format!("{filename}.css"))
        .map_err(|err| ProcessError::Writing(err.to_string()))?;

    file.write_all(styles.as_bytes())
        .map_err(|err| ProcessError::Writing(err.to_string()))?;

    file.flush()
        .map_err(|err| ProcessError::Writing(err.to_string()))?;

    if let Some(styles_min) = styles_min {
        let mut file = File::create(format!("{filename}.min.css"))
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.write_all(styles_min.as_bytes())
            .map_err(|err| ProcessError::Writing(err.to_string()))?;

        file.flush()
            .map_err(|err| ProcessError::Writing(err.to_string()))?;
    }

    Ok(())
}
