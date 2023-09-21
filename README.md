# CSS Stacker

Simple program to stack css|scss|sass files into a single css file

**Usage:** `css-stacker [OPTIONS]`

## Options

* `-p, --path <PATH>`          Path to the directory containing the styles [default: `./`]
* `-o, --output <OUTPUT>`      Name with path of the output css file without the extension [default: `./style`]
* `-m, --minify`               Whether to create a minified version of the output file [default: `false`]
* `-i, --include-path-styles`  Whether to include the files that are the direct children of the provided path directory [default: `false`]
* `-h, --help`                 Print help
* `-V, --version`              Print version

## Example

```bash
css-stacker -p ./src/styles -o ./dist/style -m
```
