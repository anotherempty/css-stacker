use std::{fs::OpenOptions, io::Write, path::PathBuf};

use ignore::WalkBuilder;

//

fn main() {
    let css = "./style.scss";

    let allowed_extension = ["scss", "css"];

    let mut file = OpenOptions::new()
        .append(true)
        .create(true) // Create the file if it doesn't exist
        .open(css)
        .unwrap();

    for result in WalkBuilder::new("./")
        .hidden(false)
        .filter_entry(move |p| p.clone().into_path() != PathBuf::from(css))
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
}
