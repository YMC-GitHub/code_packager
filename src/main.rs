//! Command-line interface for code_packager
//!
//! This binary provides a CLI for the code_packager library.

use anyhow::Result;
use clap::{Arg, Command};
use code_packager::{merge_rule_config, package_code, parse_rule_string, PackagerConfig};

fn main() -> Result<()> {
    let matches = Command::new("code_packager")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("DIR")
                .help("Input directory path")
                .default_value("."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .default_value("src_code.txt"),
        )
        .arg(
            Arg::new("add")
                .short('a')
                .long("add")
                .value_name("FILE")
                .action(clap::ArgAction::Append)
                .help("Extra files to include (supports glob patterns)"),
        )
        .arg(
            Arg::new("ignore")
                .long("ignore")
                .value_name("PATTERN")
                .action(clap::ArgAction::Append)
                .help("Ignore files/directories matching pattern"),
        )
        .arg(
            Arg::new("rule")
                .long("rule")
                .value_name("RULE_STRING")
                .help(
                "Rule string for including/excluding files (e.g., \"Cargo.toml + src + !target\")",
            ),
        )
        .arg(
            Arg::new("rule-separator")
                .long("rule-separator")
                .value_name("SEPARATOR")
                .default_value("+")
                .help("Separator used in rule string"),
        )
        .get_matches();

    // Get basic configuration
    let input_dir = matches.get_one::<String>("input").unwrap().to_string();
    let output_file = matches.get_one::<String>("output").unwrap().to_string();
    let cli_extra_files: Vec<String> = matches
        .get_many("add")
        .unwrap_or_default()
        .cloned()
        .collect();
    let cli_ignore_patterns: Vec<String> = matches
        .get_many("ignore")
        .unwrap_or_default()
        .cloned()
        .collect();

    // Parse rule string if provided
    let (rule_extra_files, rule_ignore_patterns) =
        if let Some(rule_string) = matches.get_one::<String>("rule") {
            let separator = matches.get_one::<String>("rule-separator").unwrap();
            parse_rule_string(rule_string, separator)?
        } else {
            (Vec::new(), Vec::new())
        };

    // Merge rule configuration with CLI arguments
    let (extra_files, ignore_patterns) = merge_rule_config(
        rule_extra_files,
        rule_ignore_patterns,
        cli_extra_files,
        cli_ignore_patterns,
    );

    let config = PackagerConfig {
        input_dir,
        output_file,
        extra_files,
        ignore_patterns,
    };

    package_code(&config)?;

    println!(
        "Source code successfully packaged to {}",
        config.output_file
    );
    Ok(())
}
