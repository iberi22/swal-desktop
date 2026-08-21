//! SWAL Files CLI & Agentic Explorer

use std::path::PathBuf;
use swal_files::config::FileManagerConfig;
use swal_files::omnibar::{parse_omnibar_input, OmnibarIntent};
use swal_files::scanner::{scan_directory, ScanOptions, SortBy};

fn print_usage() {
    println!("⚡ SWAL Files — Modern Minimalist Agentic File Manager (v0.1.0)");
    println!("\nUsage:");
    println!("  swal-files ls [path]           List directory contents with metadata & git status");
    println!("  swal-files search <query>      Sub-millisecond fast directory & tag search");
    println!("  swal-files omnibar <input>     Parse omnibar command (@agent, >cmd, ?search, /path)");
    println!("  swal-files config [path]       Show or init declarative configuration");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let cmd = args[1].as_str();
    match cmd {
        "ls" | "list" => {
            let target_path = if args.len() > 2 {
                PathBuf::from(&args[2])
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            };

            let opts = ScanOptions {
                show_hidden: args.iter().any(|a| a == "-a" || a == "--all"),
                sort_by: SortBy::Name,
                ascending: true,
                filter_query: None,
            };

            match scan_directory(&target_path, &opts) {
                Ok(entries) => {
                    println!("📂 {} ({} items)", target_path.display(), entries.len());
                    println!("{:<3} {:<32} {:<10} {:<12}", "Type", "Name", "Size", "Category");
                    println!("{:-<60}", "");
                    for e in entries {
                        let name_display = if e.is_dir { format!("{}/", e.name) } else { e.name.clone() };
                        println!(
                            "{:<3} {:<32} {:<10} {:<12}",
                            e.icon,
                            truncate_str(&name_display, 30),
                            e.formatted_size,
                            e.mime_category
                        );
                    }
                }
                Err(err) => eprintln!("Error scanning directory: {}", err),
            }
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: swal-files search <query> [path]");
                return;
            }
            let query = &args[2];
            let root = if args.len() > 3 {
                PathBuf::from(&args[3])
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            };

            let opts = ScanOptions {
                show_hidden: false,
                sort_by: SortBy::Name,
                ascending: true,
                filter_query: Some(query.clone()),
            };

            match scan_directory(&root, &opts) {
                Ok(entries) => {
                    println!("🔍 Results for '{}' in {}: ({} matches)", query, root.display(), entries.len());
                    for e in entries {
                        println!("  {} {:<30} ({})", e.icon, e.name, e.formatted_size);
                    }
                }
                Err(err) => eprintln!("Search error: {}", err),
            }
        }
        "omnibar" => {
            if args.len() < 3 {
                eprintln!("Usage: swal-files omnibar <input>");
                return;
            }
            let input = &args[2..].join(" ");
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let intent = parse_omnibar_input(input, &cwd);

            match intent {
                OmnibarIntent::Navigate(p) => println!("🧭 NAVIGATE -> {}", p.display()),
                OmnibarIntent::SearchQuery(q) => println!("🔍 SEARCH -> '{}'", q),
                OmnibarIntent::AgentPrompt(p) => println!("🤖 AGENT PROMPT -> \"{}\"", p),
                OmnibarIntent::Command(c) => println!("⚡ COMMAND EXEC -> `{}`", c),
            }
        }
        "config" => {
            let cfg = FileManagerConfig::default();
            println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
        }
        _ => print_usage(),
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
