use std::{collections::HashSet, fs::File, io::Write, path::PathBuf};

use ignore::WalkBuilder;
use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::{Browsers, Targets},
};

// assemble all css files
// compile sass
// process with lightningcss

fn main() {
    let css = "./style.scss";

    let allowed_extension = ["scss", "css"];

    let ignored_files = [
        "./style.css",
        "./style.scss",
        "./style.min.css",
        "./style.min.scss",
    ];

    let mut file = File::create(css).unwrap();

    let mut sass = String::new();

    for result in WalkBuilder::new("./")
        .hidden(false)
        .filter_entry(move |p| !ignored_files.contains(&p.clone().into_path().to_str().unwrap()))
        .build()
    {
        match result {
            Ok(entry) => {
                println!("{}", entry.path().display());
                if allowed_extension.contains(
                    &entry
                        .path()
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                ) {
                    println!(
                        "ext: {}",
                        entry.path().extension().unwrap().to_str().unwrap()
                    );

                    sass.push_str(&format!("@use '{}';\n", entry.path().display()));
                }
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }
    file.write_all(sass.as_bytes()).unwrap();

    file.flush().unwrap();

    let sass = grass::from_path(css, &grass::Options::default()).unwrap();

    let mut stylesheet = StyleSheet::parse(&sass, ParserOptions::default()).unwrap();

    let targets = Targets {
        browsers: Some(Browsers {
            chrome: Some(80),
            firefox: Some(70),
            edge: Some(80),
            safari: Some(13),
            opera: Some(67),
            ..Default::default()
        }),
        ..Default::default()
    };

    stylesheet
        .minify(MinifyOptions {
            targets,
            unused_symbols: HashSet::new(),
        })
        .unwrap();

    let res = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: None,
            project_root: None,
            targets,
            analyze_dependencies: None,
            pseudo_classes: None,
        })
        .unwrap();

    println!("{}", res.code);

    let file_path = PathBuf::from("./style.css");

    let mut file = File::create(file_path).unwrap();

    file.write_all(res.code.as_bytes()).unwrap();
    file.flush().unwrap();
}
