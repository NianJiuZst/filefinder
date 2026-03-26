# filefinder

A blazing fast local file search CLI tool written in Rust.

## Description

filefinder is a high-performance command-line tool for searching files on your local filesystem. It uses parallel scanning with rayon for maximum speed and supports flexible filtering by name patterns (glob or regex), file extension, and file size.

## Features

- **Fast parallel scanning** - Uses rayon for concurrent directory traversal
- **Flexible pattern matching** - Supports both glob patterns and regex
- **Size filtering** - Filter files by size range (e.g., 10K..100M)
- **Extension filtering** - Filter files by extension (e.g., -e rs)
- **Smart ignore rules** - Optionally skip .git and node_modules directories
- **Colored output** - ANSI colored terminal output for easy reading
- **Interactive mode** - Select a file to open with your system's default application
- **Small binary** - Optimized for minimal size (<2MB)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/filefinder.git
cd filefinder
cargo build --release
# Binary will be at target/release/filefinder
```

### From Binary Downloads

Download pre-built binaries from the releases page and make it executable:

```bash
chmod +x filefinder
./filefinder --help
```

## Usage Examples

### Basic file search

Search for all files matching a glob pattern:
```bash
filefinder /path/to/search "*.txt"
```

### Search by extension

Find all Rust source files:
```bash
filefinder /path/to/search -e rs
```

### Size filtering

Find files between 10KB and 100MB:
```bash
filefinder /path/to/search -s 10K..100M
```

Find files larger than 1GB:
```bash
filefinder /path/to/search -s 1G..
```

Find files smaller than 1MB:
```bash
filefinder /path/to/search -s ..1M
```

### Regex matching

Search using regular expressions:
```bash
filefinder /path/to/search -r "file[0-9]+\.txt"
```

### Ignore specific directories

Skip .git directories:
```bash
filefinder /path/to/search --ignore-git
```

Skip node_modules:
```bash
filefinder /path/to/search --ignore-node
```

### Limit recursion depth

Limit search to 2 levels deep:
```bash
filefinder /path/to/search --max-depth 2
```

### Interactive mode

Search and select a file to open:
```bash
filefinder /path/to/search "*.txt" -i
```

### Combine options

Search for Rust files larger than 10KB, ignoring git directories:
```bash
filefinder /path/to/search -e rs -s 10K.. --ignore-git
```

## Build from Source

### Prerequisites

- Rust 1.75.0 or later
- Cargo

### Build Instructions

```bash
git clone https://github.com/yourusername/filefinder.git
cd filefinder
cargo build --release
```

The optimized binary will be at `target/release/filefinder`.

### Binary Size

The release binary is optimized for small size:
- LTO (Link Time Optimization) enabled
- Size optimization level (`opt-level = "s"`)
- Panic abort (no unwinding code)
- Stripped symbols

Target size: **< 2MB**

## Command Line Options

```
filefinder [OPTIONS] [PATH] [NAME]

Arguments:
  [PATH]              Directory to search (default: current directory)
  [NAME]               Search pattern (glob or regex)

Options:
  -e, --ext <EXT>      Filter by file extension (e.g. rs, txt, md)
  -s, --size <SIZE>    Filter by size range (e.g. 10K..100M, 1M.., ..1G)
  -r, --regex          Use regex matching instead of glob
      --ignore-git     Ignore .git directories
      --ignore-node    Ignore node_modules directories
      --max-depth <MAX_DEPTH>
                       Maximum directory recursion depth
  -i, --interactive    Interactive mode: select a file to open
  -h, --help           Print help
  -V, --version        Print version
```

## Size Format

The size filter accepts the following units:
- `B` - bytes
- `K` / `KB` - kilobytes (1024 bytes)
- `M` / `MB` - megabytes (1024^2 bytes)
- `G` / `GB` - gigabytes (1024^3 bytes)
- `T` / `TB` - terabytes (1024^4 bytes)

Examples:
- `10K..100M` - between 10KB and 100MB
- `1G..` - larger than 1GB
- `..1M` - smaller than 1MB

## License

MIT License - see LICENSE file for details
