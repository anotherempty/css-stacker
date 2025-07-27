use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::{Hash, Hasher},
    io::Read,
    str::FromStr,
};

use css_stacker::{Format, Stacker, StackerOptions, StyleExtension};

#[derive(Debug, PartialEq, Eq)]
struct CssBlock {
    selector: String,
    declarations: HashMap<String, String>,
}

impl Hash for CssBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.selector.hash(state);
        // Sort declarations by key for consistent hashing
        let mut sorted_decls: Vec<_> = self.declarations.iter().collect();
        sorted_decls.sort_by_key(|(k, _)| k.as_str());
        for (key, value) in sorted_decls {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CssDocument {
    blocks: HashSet<CssBlock>,
}

#[derive(Debug)]
struct ParseError;

impl FromStr for CssBlock {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Find the opening brace
        let brace_pos = s.find('{').ok_or(ParseError)?;
        let selector = s[..brace_pos].trim().to_string();

        // Find the closing brace
        let close_brace_pos = s.rfind('}').ok_or(ParseError)?;
        let declarations_str = &s[brace_pos + 1..close_brace_pos].trim();

        let mut declarations = HashMap::new();

        // Parse declarations
        for decl in declarations_str.split(';') {
            let decl = decl.trim();
            if !decl.is_empty() {
                if let Some((prop, value)) = decl.split_once(':') {
                    declarations.insert(prop.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Ok(CssBlock {
            selector,
            declarations,
        })
    }
}

impl FromStr for CssDocument {
    type Err = ParseError;

    fn from_str(css: &str) -> Result<Self, Self::Err> {
        let mut blocks = HashSet::new();
        let mut current_pos = 0;

        while current_pos < css.len() {
            // Skip whitespace
            while current_pos < css.len() && css.chars().nth(current_pos).unwrap().is_whitespace() {
                current_pos += 1;
            }

            if current_pos >= css.len() {
                break;
            }

            // Find the end of this block
            let block_start = current_pos;
            let mut brace_count = 0;
            let mut in_block = false;

            while current_pos < css.len() {
                match css.chars().nth(current_pos).unwrap() {
                    '{' => {
                        brace_count += 1;
                        in_block = true;
                    }
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 && in_block {
                            current_pos += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                current_pos += 1;
            }

            let block_str = &css[block_start..current_pos];
            if let Ok(block) = CssBlock::from_str(block_str) {
                blocks.insert(block);
            }
        }

        Ok(CssDocument { blocks })
    }
}

#[test]
fn test_css_document_parsing() {
    let css = "div{border:1px solid #000}.color{color:red;padding:1rem}";

    let mut expected_blocks = HashSet::new();

    let mut div_declarations = HashMap::new();
    div_declarations.insert("border".to_string(), "1px solid #000".to_string());
    expected_blocks.insert(CssBlock {
        selector: "div".to_string(),
        declarations: div_declarations,
    });

    let mut color_declarations = HashMap::new();
    color_declarations.insert("color".to_string(), "red".to_string());
    color_declarations.insert("padding".to_string(), "1rem".to_string());
    expected_blocks.insert(CssBlock {
        selector: ".color".to_string(),
        declarations: color_declarations,
    });

    let expected = CssDocument {
        blocks: expected_blocks,
    };
    let actual: CssDocument = css.parse().unwrap();

    assert_eq!(expected, actual);
}

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

    let expected_css = "div{border:1px solid #000}.color{color:red;padding:1rem}.color>span{color:#00f}a{color:red;transition:all .15s ease-in-out}body{color:#000;background:#fff}body>p{color:#000}";

    let mut test_file = File::open("./tests/test.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    let expected_doc: CssDocument = expected_css.parse().unwrap();
    let actual_doc: CssDocument = file_contents.parse().unwrap();

    assert_eq!(expected_doc, actual_doc);
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

    let expected_css = "div{border:1px solid #000}";

    let mut test_file = File::open("./tests/test_filtered.min.css").unwrap();

    let mut file_contents = String::new();
    test_file.read_to_string(&mut file_contents).unwrap();

    let expected_doc: CssDocument = expected_css.parse().unwrap();
    let actual_doc: CssDocument = file_contents.parse().unwrap();

    assert_eq!(expected_doc, actual_doc);
}
