use std::{fs::File, io::Write, path::PathBuf};

use ignore::WalkBuilder;

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

                    file.write_all(format!("@use '{}';\n", entry.path().display()).as_bytes())
                        .unwrap();
                }
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }

    file.flush().unwrap();

    let sass = grass::from_path(css, &grass::Options::default()).unwrap();

    let file_path = PathBuf::from("./style.css");

    let mut file = File::create(file_path).unwrap();

    file.write_all(sass.as_bytes()).unwrap();
    file.flush().unwrap();
}
