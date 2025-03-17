# CSS Stacker

Simple program to stack css|scss|sass files into a single css file

**Usage:** `css-stacker [OPTIONS]`

## Installation

```bash
cargo install --locked --git https://github.com/anotherempty/css-stacker
```

## Options

- `-p`, `--path`: Path to the directory containing the styles [default: ./]
- `-e`, `--extensions`: Restrict file extensions to be processed. When not provided, all css, scss and sass files are processed [possible values: css, scss, sass]
- `-d`, `--output-dir`: Path of the output directory. Defaults to the current directory
- `-n`, `--output-name`: Name of the output file. Defaults to 'styles'
- `-f`, `--output-format`: Format of the output file. When not provided, both minified and pretty formats are generated [possible values: minified, pretty]

## Example

```bash
css-stacker -p ./src/styles -e sass -e scss -d ./assets -n main -f minified
```

 Will create `./assets/style.min.css`
