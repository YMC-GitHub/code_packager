# Code Packager

A Rust tool to package source code files into a single text file with proper syntax formatting.

## Features

- Recursively packages source code from directories
- Supports glob patterns for ignoring files/directories
- Includes extra files using glob patterns
- Outputs formatted code with file path markers

## Installation

```bash
cargo install code_packager
```

## Usage

### Command Line
```bash
# Basic usage
code_packager

# Specify input and output
code_packager -i ./src -o output.txt

# Add extra files
code_packager -a "Cargo.toml" -a "*.md"

# Ignore patterns
code_packager --ignore "target/*" --ignore "*.tmp"
```

### As a Library
```toml
[dependencies]
code_packager = "0.1"
```

```rust
use code_packager::{package_code, PackagerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PackagerConfig {
        input_dir: "src".to_string(),
        output_file: "code.txt".to_string(),
        extra_files: vec!["Cargo.toml".to_string()],
        ignore_patterns: vec!["target/*".to_string()],
    };
    
    package_code(&config)?;
    Ok(())
}
```

## License

Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
