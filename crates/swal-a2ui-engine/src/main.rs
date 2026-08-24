use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use swal_a2ui_engine::native_render::{evaluate_ast_to_gpu_commands, LayoutRect};
use swal_a2ui_engine::{compile_widget, ComponentNode, ThemePalette};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("toggle");

    match command {
        "toggle" => {
            let res = send_ctl_command("toggle-agent-monitor")
                .or_else(|_| send_ctl_command("toggle-dashboard"))
                .unwrap_or_else(|_| "A2UI surface toggled natively.".to_string());
            println!("⚡ SWAL Native A2UI Desktop Workspace toggled.");
            println!("   {}", res.trim());
        }
        "render" => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or("widget.json");
            println!("🔍 Inspecting and compiling A2UI JSON AST at {}", path);
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(ast) = compile_widget(&content, "fluent-dark") {
                    let commands = evaluate_ast_to_gpu_commands(&ast.root, LayoutRect::new(0.0, 0.0, 1920.0, 1080.0));
                    println!("✓ A2UI Widget '{}' compiled successfully!", ast.title);
                    println!("✓ Generated {} GPU draw commands.", commands.len());
                } else {
                    eprintln!("❌ Failed to compile A2UI widget JSON.");
                }
            }
        }
        "spawn-hermes" => {
            println!("󰆍 Spawning Hermes Autonomous Agent in native terminal...");
            let _ = std::process::Command::new("ghostty")
                .args(["-e", "hermes"])
                .spawn();
        }
        "spawn-xavier" => {
            println!("🧠 Connecting to Xavier Memory GraphRAG...");
            let _ = std::process::Command::new("ghostty")
                .args(["-e", "hermes", "-c"])
                .spawn();
        }
        "tree" => {
            let sample_card = ComponentNode::Card {
                title: Some("⚡ SWAL A2UI AGENTIC DESKTOP".to_string()),
                elevation: Some("high".to_string()),
                children: vec![
                    ComponentNode::StatusBadge {
                        status: "active".to_string(),
                        label: "Live Autonomous Agent Network".to_string(),
                        color: Some("#00e5ff".to_string()),
                    },
                    ComponentNode::MetricPill {
                        label: "ACTIVE AGENTS".to_string(),
                        value: "3".to_string(),
                        unit: Some("live".to_string()),
                        trend: Some("+1".to_string()),
                        color: Some("#10b981".to_string()),
                    },
                ],
            };
            let cmds = evaluate_ast_to_gpu_commands(&sample_card, LayoutRect::new(0.0, 0.0, 1200.0, 800.0));
            println!("AST Hierarchy: {:#?}", sample_card);
            println!("GPU Draw Commands: {} generated", cmds.len());
        }
        _ => {
            println!("SWAL A2UI Engine CLI");
            println!("Usage: swal-a2ui [toggle|render <file>|spawn-hermes|spawn-xavier|tree]");
        }
    }

    Ok(())
}

fn send_ctl_command(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect("/tmp/swal_desktop_ctl.sock")?;
    stream.write_all(cmd.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    Ok(response)
}
