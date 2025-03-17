use std::{fs::File, io::Read};

use css_stacker::{Format, Stacker, StackerOptions, StyleExtension};

#[test]
fn test_stacking() {
    let options = StackerOptions {
        path: "./tests".into(),
        extensions: Vec::new(),
        output_dir: Some("./tests".into()),
        output_name: Some("test".into()),
        output_format: None,
        verbose: false,
    };
    let _ = Stacker::create(options);

    let test_result = "div{border:1px solid #000}.color{color:red;padding:1rem}.color>span{color:#00f}a{color:red;transition:all .15s ease-in-out}body{color:#000;background:#fff}body>p{color:#000}";

    let mut test_file = File::open("./tests/test.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    assert_eq!(file_contents, test_result);
}

#[test]
fn test_stacking_with_filter() {
    let options = StackerOptions {
        path: "./tests".into(),
        extensions: vec![StyleExtension::Sass],
        output_dir: Some("./tests".into()),
        output_name: Some("test_filtered".into()),
        output_format: Some(Format::Minified),
        verbose: false,
    };

    let _ = Stacker::create(options);

    let test_result = "div{border:1px solid #000}";

    let mut test_file = File::open("./tests/test_filtered.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    assert_eq!(file_contents, test_result);
}
