mod config;
mod ignore;
mod interactive;
mod matcher;
mod output;
mod scanner;

use anyhow::Result;
use std::io::{self, Write};
use std::path::PathBuf;

use config::SearchConfig;
use output::Output;
use scanner::Scanner;

// ANSI color codes for banner
const BANNER_CYAN: &str = "\x1b[36m";
const BANNER_GREEN: &str = "\x1b[32m";
const BANNER_YELLOW: &str = "\x1b[33m";
const BANNER_MAGENTA: &str = "\x1b[35m";
const BANNER_DIM: &str = "\x1b[2m";
const BANNER_BOLD: &str = "\x1b[1m";
const BANNER_RESET: &str = "\x1b[0m";

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_banner() {
    println!();
    println!("{}╭──────────────────────────────────────────────────────────────╮{}", BANNER_CYAN, BANNER_RESET);
    println!("{}│{}  {}🔍{} {}FileFinder{} {}─{} 文件查找神器                          {}│{}", 
        BANNER_CYAN, BANNER_RESET, BANNER_YELLOW, BANNER_RESET, BANNER_BOLD, BANNER_RESET, BANNER_CYAN, BANNER_RESET, BANNER_CYAN, BANNER_RESET);
    println!("{}│{}                                                              {}│{}", BANNER_CYAN, BANNER_RESET, BANNER_CYAN, BANNER_RESET);
    println!("{}│{}  {}📂{} {}快速定位文件{}     {}⚡{} {}多种匹配方式{}     {}🎯{} {}交互式选择{}        {}│{}", 
        BANNER_CYAN, BANNER_RESET, BANNER_GREEN, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_YELLOW, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_MAGENTA, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_CYAN, BANNER_RESET);
    println!("{}│{}  {}🔧{} {}大小/时间过滤{}   {}💡{} {}正则支持{}       {}🚀{} {}高性能扫描{}        {}│{}", 
        BANNER_CYAN, BANNER_RESET, BANNER_GREEN, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_YELLOW, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_MAGENTA, BANNER_RESET, BANNER_DIM, BANNER_RESET,
        BANNER_CYAN, BANNER_RESET);
    println!("{}│{}                                                              {}│{}", BANNER_CYAN, BANNER_RESET, BANNER_CYAN, BANNER_RESET);
    println!("{}╰──────────────────────────────────────────────────────────────╯{}", BANNER_CYAN, BANNER_RESET);
    println!();
    println!("  {}输入 {}help{} {}查看命令帮助  ·  输入 {}quit{} {}退出程序", 
        BANNER_DIM, BANNER_CYAN, BANNER_RESET, BANNER_DIM, BANNER_CYAN, BANNER_RESET, BANNER_DIM);
    println!();
}

fn run() -> Result<()> {
    let mut output = Output::new();
    
    print_banner();
    
    loop {
        print!("filefinder> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!("\n退出");
            break;
        }
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        // 处理内置命令
        match handle_builtin_command(input) {
            Some(BuiltinCommand::Quit) => {
                println!("退出");
                break;
            }
            Some(BuiltinCommand::Help) => {
                print_help();
                continue;
            }
            
            None => {
                // 不是内置命令，尝试作为搜索参数解析
            }
        }
        
        // 解析搜索命令
        match parse_and_execute(input, &mut output) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        println!();
    }
    
    Ok(())
}

enum BuiltinCommand {
    Quit,
    Help,
}

fn handle_builtin_command(input: &str) -> Option<BuiltinCommand> {
    let cmd = input.split_whitespace().next().unwrap_or("");
    match cmd {
        "quit" | "exit" | "q" => Some(BuiltinCommand::Quit),
        "help" | "h" | "?" => Some(BuiltinCommand::Help),
        _ => None,
    }
}

fn print_help() {
    println!("可用命令:");
    println!("  find <path> [options]    查找文件");
    println!("  help                     显示帮助信息");
    println!("  quit / exit              退出程序");
    println!();
    println!("查找选项:");
    println!("  -n, --name <pattern>     按文件名匹配");
    println!("  -e, --ext <ext>          按扩展名过滤");
    println!("  -s, --size <range>       按大小范围 (e.g. 10K..100M)");
    println!("  -r, --regex              使用正则匹配");
    println!("  --ignore-git             忽略 .git 目录");
    println!("  --ignore-node            忽略 node_modules 目录");
    println!("  --max-depth <n>          最大递归深度");
    println!("  -i, --interactive        交互模式选择文件");
    println!();
    println!("示例:");
    println!("  find . -e rs");
    println!("  find ./target -n main");
    println!("  find /home -s 1M..100M --ignore-git");
}

fn parse_and_execute(input: &str, output: &mut Output) -> Result<()> {
    // 构造 clap 命令行参数
    let args_vec: Vec<String> = input
        .split_whitespace()
        .flat_map(|s| shell_split(s))
        .collect();
    
    // 提取第一个词作为命令名称，后面的是参数
    let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();
    
    // 找到 "find" 命令的位置
    let find_idx = args.iter().position(|&s| s == "find");
    
    let search_args: Vec<&str> = match find_idx {
        Some(idx) => args[idx + 1..].to_vec(),
        None => args[0..].to_vec(),
    };
    
    // 手动解析参数
    let config = parse_search_args(&search_args)?;
    
    // 验证路径
    if !config.path.exists() {
        anyhow::bail!("路径不存在: {}", config.path.display());
    }
    
    // 执行扫描
    let scanner = Scanner::new(config.clone())?;
    let entries = scanner.scan();
    
    // 打印结果
    if entries.is_empty() {
        println!("未找到文件。");
        return Ok(());
    }
    
    let search_root = &config.path;
    for (i, entry) in entries.iter().enumerate() {
        let relative_path = if entry.path.starts_with(search_root) {
            entry.path.strip_prefix(search_root).unwrap_or(&entry.path)
        } else {
            &entry.path
        };
        output.print_entry(i + 1, relative_path, entry.size, entry.mtime)?;
    }
    
    println!("\n找到 {} 个文件。", entries.len());
    
    // 交互模式
    if config.interactive {
        output.print_prompt(entries.len())?;
        interactive::interactive_select(entries, search_root)?;
    }
    
    Ok(())
}

// 简单的 shell 参数分割
fn shell_split(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    
    for c in s.chars() {
        if !in_quotes && (c == '"' || c == '\'') {
            in_quotes = true;
            quote_char = c;
        } else if in_quotes && c == quote_char {
            in_quotes = false;
        } else if !in_quotes && c == ' ' {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    
    if !current.is_empty() {
        result.push(current);
    }
    
    result
}

fn parse_search_args(args: &[&str]) -> Result<SearchConfig> {
    let mut path = PathBuf::from("/");
    let mut name: Option<String> = None;
    let mut ext: Option<String> = None;
    let mut size: Option<String> = None;
    let mut regex = false;
    let mut ignore_git = true;  // 默认忽略 .git
    let mut ignore_node = true; // 默认忽略 node_modules
    let mut max_depth: Option<usize> = None;
    let mut interactive = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-n" | "--name" => {
                if i + 1 < args.len() {
                    name = Some(args[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-e" | "--ext" => {
                if i + 1 < args.len() {
                    ext = Some(args[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-s" | "--size" => {
                if i + 1 < args.len() {
                    size = Some(args[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-r" | "--regex" => {
                regex = true;
                i += 1;
            }
            "--ignore-git" => {
                ignore_git = true;
                i += 1;
            }
            "--ignore-node" => {
                ignore_node = true;
                i += 1;
            }
            "--max-depth" => {
                if i + 1 < args.len() {
                    max_depth = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-i" | "--interactive" => {
                interactive = true;
                i += 1;
            }
            "-h" | "--help" => {
                print_help();
                return Err(anyhow::anyhow!("帮助已显示"));
            }
            _ => {
                if !args[i].starts_with('-') {
                    let token = args[i];
                    let candidate = PathBuf::from(token);
                    let looks_like_path = candidate.exists()
                        || token == "."
                        || token == ".."
                        || token.starts_with('/')
                        || token.starts_with("~/")
                        || token.contains(std::path::MAIN_SEPARATOR);

                    if path == PathBuf::from("/") && looks_like_path {
                        path = candidate;
                    } else if name.is_none() {
                        name = Some(token.to_string());
                    }
                }
                i += 1;
            }
        }
    }
    
    let size_range = size.as_ref().and_then(|s| {
        config::SizeRange::parse(s).ok()
    });
    
    Ok(SearchConfig {
        path,
        pattern: name,
        ext,
        size_range,
        use_regex: regex,
        ignore_git,
        ignore_node,
        max_depth,
        interactive,
    })
}