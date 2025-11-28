# Code Packager

一个用于将源代码文件打包到单个文本文件中的 Rust 工具。

## 功能特性

- 递归打包目录中的源代码文件
- 支持 glob 模式忽略文件/目录
- 支持额外包含文件
- 输出格式化的代码文件

## 安装

### 从 crates.io 安装

```bash
cargo install code_packager
```

### 从源码安装

```bash
git clone https://github.com/ymc-github/code_packager
cd code_packager
cargo install --path .
```

## 使用方法

### 作为命令行工具

```bash
# 基本使用
code_packager

# 指定输入输出
code_packager -i ./src -o output.txt

# 添加额外文件
code_packager -a "Cargo.toml" -a "README.md"

# 忽略文件模式
code_packager --ignore "target/*" --ignore "*.tmp"
```

### 作为库使用

添加依赖到 `Cargo.toml`：

```toml
[dependencies]
code_packager = "0.1"
```

在代码中使用：

```rust
use code_packager::{package_code, PackagerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PackagerConfig {
        input_dir: "src".to_string(),
        output_file: "code.txt".to_string(),
        extra_files: vec!["Cargo.toml".to_string(), "README.md".to_string()],
        ignore_patterns: vec!["target/*".to_string(), "*.tmp".to_string()],
    };
    
    package_code(&config)?;
    Ok(())
}
```

### 作为 Cargo 子命令

安装后，可以直接使用：

```bash
cargo code_packager
```

## 许可证

MIT OR Apache-2.0
