use std::{fs::File, io::Read};

use css_stacker::{Format, Stacker, StyleExtension};

#[test]
fn test_stacking() {
    let _ = Stacker::create("./tests", "./tests/test", None, &[]);

    let test_result = "body{color:#000;background:#fff}body>p{color:#000}div{border:1px solid #000}a{color:red;transition:all .15s ease-in-out}.color{color:red;padding:1rem}.color>span{color:#00f}";

    let mut test_file = File::open("./tests/test.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    assert_eq!(file_contents, test_result);
}

#[test]
fn test_stacking_with_filter() {
    let _ = Stacker::create(
        "./tests",
        "./tests/test_filtered",
        Some(Format::Minified),
        &[StyleExtension::Sass],
    );

    let test_result = "div{border:1px solid #000}";

    let mut test_file = File::open("./tests/test_filtered.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    assert_eq!(file_contents, test_result);
}
