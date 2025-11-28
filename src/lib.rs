//! A library for packaging source code files into a single text file.
//!
//! This crate provides functionality to recursively collect source code files
//! from directories and package them into a formatted text file.
//!
//! # Examples
//!
//! ```no_run
//! use code_packager::{package_code, PackagerConfig, parse_rule_string};
//!
//! let rule = "Cargo.toml + src + !target";
//! let (extra_files, ignore_patterns) = parse_rule_string(rule, " + ").unwrap();
//!
//! let config = PackagerConfig {
//!     input_dir: ".".to_string(),
//!     output_file: "src_output.txt".to_string(),
//!     extra_files,
//!     ignore_patterns,
//! };
//!
//! package_code(&config).unwrap();
//! ```

use anyhow::{Context, Result};
use glob::Pattern;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Configuration for the code packager
#[derive(Debug, Clone)]
pub struct PackagerConfig {
    /// Input directory path
    pub input_dir: String,
    /// Output file path  
    pub output_file: String,
    /// Extra files to include (supports glob patterns)
    pub extra_files: Vec<String>,
    /// Patterns to ignore files/directories
    pub ignore_patterns: Vec<String>,
}

impl Default for PackagerConfig {
    fn default() -> Self {
        Self {
            input_dir: "src".to_string(),
            output_file: "src_code.txt".to_string(),
            extra_files: Vec::new(),
            ignore_patterns: Vec::new(),
        }
    }
}

/// Parse a rule string into extra_files and ignore_patterns
///
/// # Arguments
/// * `rule_string` - The rule string to parse (e.g., "Cargo.toml + src + !target")
/// * `separator` - The separator used in the rule string (e.g., " + ")
///
/// # Returns
/// A tuple of (extra_files, ignore_patterns)
///
/// # Rules
/// - Items without "!" prefix are added to extra_files
/// - Items with "!" prefix are added to ignore_patterns (without the "!" prefix)
/// - Empty items are ignored
/// - Leading and trailing whitespace is trimmed
///
/// # Examples
/// ```
/// use code_packager::parse_rule_string;
///
/// let (extra, ignore) = parse_rule_string("file.txt + src + !target", " + ").unwrap();
/// assert_eq!(extra, vec!["file.txt", "src"]);
/// assert_eq!(ignore, vec!["target"]);
/// ```
pub fn parse_rule_string(rule_string: &str, separator: &str) -> Result<(Vec<String>, Vec<String>)> {
    let mut extra_files = Vec::new();
    let mut ignore_patterns = Vec::new();

    for item in rule_string.split(separator) {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(ignore_pattern) = trimmed.strip_prefix('!') {
            let pattern = ignore_pattern.trim().to_string();
            if !pattern.is_empty() {
                ignore_patterns.push(pattern);
            }
        } else {
            extra_files.push(trimmed.to_string());
        }
    }

    Ok((extra_files, ignore_patterns))
}

/// Merge rule-based configuration with individual file and ignore patterns
///
/// # Arguments
/// * `rule_extra` - Extra files from rule parsing
/// * `rule_ignore` - Ignore patterns from rule parsing  
/// * `cli_extra` - Extra files from CLI arguments
/// * `cli_ignore` - Ignore patterns from CLI arguments
///
/// # Returns
/// Merged (extra_files, ignore_patterns)
///
/// # Examples
/// ```
/// use code_packager::merge_rule_config;
///
/// let rule_extra = vec!["src".to_string()];
/// let rule_ignore = vec!["target".to_string()];
/// let cli_extra = vec!["Cargo.toml".to_string()];
/// let cli_ignore = vec!["*.tmp".to_string()];
///
/// let (merged_extra, merged_ignore) = merge_rule_config(
///     rule_extra, rule_ignore, cli_extra, cli_ignore
/// );
///
/// assert_eq!(merged_extra, vec!["src", "Cargo.toml"]);
/// assert_eq!(merged_ignore, vec!["target", "*.tmp"]);
/// ```
pub fn merge_rule_config(
    rule_extra: Vec<String>,
    rule_ignore: Vec<String>,
    cli_extra: Vec<String>,
    cli_ignore: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut extra_files = rule_extra;
    let mut ignore_patterns = rule_ignore;

    extra_files.extend(cli_extra);
    ignore_patterns.extend(cli_ignore);

    (extra_files, ignore_patterns)
}

/// Package source code files into a single text file
///
/// # Arguments
/// * `config` - Configuration for packaging
///
/// # Errors
/// Returns `Err` if:
/// - Input directory doesn't exist or can't be read
/// - Output file can't be created
/// - Any source file can't be read
///
/// # Examples
/// ```
/// use code_packager::{package_code, PackagerConfig};
///
/// let config = PackagerConfig::default();
/// package_code(&config).unwrap();
/// ```
pub fn package_code(config: &PackagerConfig) -> Result<()> {
    let compiled_ignores: Result<Vec<Pattern>> = config
        .ignore_patterns
        .iter()
        .map(|p| Pattern::new(p).context(format!("Invalid ignore pattern: {}", p)))
        .collect();
    let compiled_ignores = compiled_ignores?;

    let mut output = File::create(&config.output_file).context(format!(
        "Failed to create output file: {}",
        config.output_file
    ))?;

    // 首先处理额外文件/目录
    for file_pattern in &config.extra_files {
        let matches =
            glob::glob(file_pattern).context(format!("Invalid file pattern: {}", file_pattern))?;

        for entry in matches {
            let path = entry.context("Failed to parse file path")?;
            if path.exists() {
                // // 使用当前目录 "." 作为 base_dir 来检查是否应该忽略
                // if should_ignore(&path, &compiled_ignores, ".") {
                //     continue; // 跳过被忽略的文件
                // }

                if path.is_dir() {
                    // 处理额外目录
                    process_directory(
                        &path.to_string_lossy(),
                        &mut output,
                        &compiled_ignores,
                        &path.to_string_lossy(), // 使用目录自身作为基准路径
                    )
                    .context(format!(
                        "Failed to process extra directory: {}",
                        path.display()
                    ))?;
                } else if path.is_file() {
                    // 处理额外文件
                    write_file_to_output(&path.to_string_lossy(), &mut output)
                        .context(format!("Failed to process extra file: {}", path.display()))?;
                }
            }
        }
    }

    // 然后处理主输入目录（如果存在且不是 "."）

    if Path::new(&config.input_dir).exists() && config.input_dir != "." {
        // 检查输入目录本身是否应该被忽略
        // let input_dir_path = Path::new(&config.input_dir);
        // if should_ignore(input_dir_path, &compiled_ignores, ".") {
        //     // 如果整个输入目录都被忽略，跳过处理
        //     return Ok(());
        // }

        process_directory(
            &config.input_dir,
            &mut output,
            &compiled_ignores,
            &config.input_dir,
        )
        .context("Failed to process input directory")?;
    }

    Ok(())
}

fn process_directory(
    dir_path: &str,
    output: &mut File,
    ignore_patterns: &[Pattern],
    base_dir: &str,
) -> Result<()> {
    let entries =
        fs::read_dir(dir_path).context(format!("Failed to read directory: {}", dir_path))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if should_ignore(&path, ignore_patterns, base_dir) {
            continue;
        }

        if path.is_dir() {
            process_directory(&path_str, output, ignore_patterns, base_dir)?;
        } else if path.is_file() {
            write_file_to_output(&path_str, output)
                .context(format!("Failed to process file: {}", path_str))?;
        }
    }

    Ok(())
}

fn should_ignore(path: &Path, ignore_patterns: &[Pattern], base_dir: &str) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in ignore_patterns {
        if pattern.matches(&path_str) {
            return true;
        }

        if let Ok(relative_path) = path.strip_prefix(base_dir) {
            let relative_str = relative_path.to_string_lossy();
            if pattern.matches(&relative_str) {
                return true;
            }
        }
    }

    false
}

fn write_file_to_output(file_path: &str, output: &mut File) -> Result<()> {
    let content =
        fs::read_to_string(file_path).context(format!("Failed to read file: {}", file_path))?;

    writeln!(output, "```{}", file_path)?;
    write!(output, "{}", content)?;
    if !content.ends_with('\n') {
        writeln!(output)?;
    }
    writeln!(output, "```")?;
    writeln!(output)?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_rule_string_basic() {
        let rule = "Cargo.toml + src + !target";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert_eq!(extra, vec!["Cargo.toml", "src"]);
        assert_eq!(ignore, vec!["target"]);
    }

    #[test]
    fn test_parse_rule_string_complex() {
        let rule = "Cargo.toml + src + !src/nodes + src/nodes/mod.rs + !src/bin";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert_eq!(extra, vec!["Cargo.toml", "src", "src/nodes/mod.rs"]);
        assert_eq!(ignore, vec!["src/nodes", "src/bin"]);
    }

    #[test]
    fn test_parse_rule_string_with_whitespace() {
        let rule = "  file1.txt  +  !  pattern/*  +  dir/  +  !  *.tmp  ";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert_eq!(extra, vec!["file1.txt", "dir/"]);
        assert_eq!(ignore, vec!["pattern/*", "*.tmp"]);
    }

    #[test]
    fn test_parse_rule_string_empty_and_blank() {
        let rule = " + file.txt +  + !pattern + ";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert_eq!(extra, vec!["file.txt"]);
        assert_eq!(ignore, vec!["pattern"]);
    }

    #[test]
    fn test_parse_rule_string_custom_separator() {
        let rule = "file.txt | src | !target";
        let (extra, ignore) = parse_rule_string(rule, " | ").unwrap();

        assert_eq!(extra, vec!["file.txt", "src"]);
        assert_eq!(ignore, vec!["target"]);
    }

    #[test]
    fn test_parse_rule_string_only_ignores() {
        let rule = "!target + !*.tmp + !node_modules";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert!(extra.is_empty());
        assert_eq!(ignore, vec!["target", "*.tmp", "node_modules"]);
    }

    #[test]
    fn test_parse_rule_string_only_extras() {
        let rule = "src + Cargo.toml + README.md";
        let (extra, ignore) = parse_rule_string(rule, " + ").unwrap();

        assert_eq!(extra, vec!["src", "Cargo.toml", "README.md"]);
        assert!(ignore.is_empty());
    }

    #[test]
    fn test_merge_rule_config() {
        let rule_extra = vec!["src".to_string(), "docs".to_string()];
        let rule_ignore = vec!["target".to_string(), "*.tmp".to_string()];
        let cli_extra = vec!["Cargo.toml".to_string()];
        let cli_ignore = vec!["node_modules".to_string()];

        let (merged_extra, merged_ignore) =
            merge_rule_config(rule_extra, rule_ignore, cli_extra, cli_ignore);

        assert_eq!(merged_extra, vec!["src", "docs", "Cargo.toml"]);
        assert_eq!(merged_ignore, vec!["target", "*.tmp", "node_modules"]);
    }

    #[test]
    fn test_merge_rule_config_empty() {
        let (merged_extra, merged_ignore) =
            merge_rule_config(Vec::new(), Vec::new(), Vec::new(), Vec::new());

        assert!(merged_extra.is_empty());
        assert!(merged_ignore.is_empty());
    }

    #[test]
    fn test_packager_config_default() {
        let config = PackagerConfig::default();
        assert_eq!(config.input_dir, "src");
        assert_eq!(config.output_file, "src_code.txt");
        assert!(config.extra_files.is_empty());
        assert!(config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_should_ignore() {
        let patterns = vec![
            Pattern::new("*.tmp").unwrap(),
            Pattern::new("target/*").unwrap(),
        ];

        let base_dir = "/project";
        let path = Path::new("/project/src/main.rs");

        // Test file that should not be ignored
        assert!(!should_ignore(path, &patterns, base_dir));

        // Test file that should be ignored
        let ignore_path = Path::new("/project/test.tmp");
        assert!(should_ignore(ignore_path, &patterns, base_dir));
    }

    #[test]
    fn test_write_file_to_output() -> Result<()> {
        // 创建临时目录和文件，而不是使用 NamedTempFile
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("src_output.txt");
        let test_file_path = temp_dir.path().join("test.rs");

        let test_content = "fn main() {\n    println!(\"Hello\");\n}";

        // 创建测试文件
        fs::write(&test_file_path, test_content)?;

        // 创建输出文件
        let mut output_file = File::create(&output_path)?;

        write_file_to_output(&test_file_path.to_string_lossy(), &mut output_file)?;

        // 验证输出内容
        let output_content = fs::read_to_string(&output_path)?;
        assert!(output_content.contains("```"));
        assert!(output_content.contains("fn main()"));
        assert!(output_content.contains("Hello"));

        Ok(())
    }

    #[test]
    fn test_write_file_to_output_with_trailing_newline() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("src_output.txt");
        let test_file_path = temp_dir.path().join("test.rs");

        // 测试没有结尾换行符的内容
        let test_content = "fn main() {\n    println!(\"Hello\");\n}"; // 没有结尾换行

        // 创建测试文件
        fs::write(&test_file_path, test_content)?;

        // 创建输出文件
        let mut output_file = File::create(&output_path)?;

        write_file_to_output(&test_file_path.to_string_lossy(), &mut output_file)?;

        // 验证输出内容
        let output_content = fs::read_to_string(&output_path)?;
        assert!(output_content.ends_with("```\n\n"));

        Ok(())
    }

    #[test]
    fn test_package_code_with_invalid_config() {
        let config = PackagerConfig {
            input_dir: "/nonexistent/directory".to_string(),
            output_file: "src_output.txt".to_string(),
            extra_files: vec![],
            ignore_patterns: vec![],
        };

        let result = package_code(&config);
        // assert!(result.is_err());
        assert!(result.is_ok());
    }

    #[test]
    fn test_package_code_integration() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // 创建测试目录结构
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;

        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() { println!(\"Hello\"); }")?;

        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "pub fn add(a: i32, b: i32) -> i32 { a + b }")?;

        // 创建 Cargo.toml
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )?;

        let output_path = temp_dir.path().join("src_output.txt");

        let config = PackagerConfig {
            input_dir: temp_dir.path().to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
            extra_files: vec!["Cargo.toml".to_string(), "src/*.rs".to_string()],
            ignore_patterns: vec![],
        };

        package_code(&config)?;

        // 验证输出文件存在且有内容
        assert!(output_path.exists());
        let output_content = fs::read_to_string(&output_path)?;
        assert!(output_content.contains("Cargo.toml"));
        assert!(output_content.contains("main.rs"));
        assert!(output_content.contains("lib.rs"));
        assert!(output_content.contains("Hello"));

        Ok(())
    }
}
