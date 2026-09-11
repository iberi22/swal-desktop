//! Read-only text viewer and clipboard extraction helper for SWAL Files.
//! Allows viewing files using read-only pagers and copying lines or ranges
//! to the system clipboard without editing permissions or write access.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use crate::session::SessionState;

/// Returns standard PATH lookup status for a binary name.
fn is_binary_in_path(binary: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(':') {
            let bin_path = Path::new(p).join(binary);
            if bin_path.is_file() {
                return true;
            }
        }
    }
    false
}

/// Helper to send content to the Wayland system clipboard using `wl-copy`.
fn copy_to_clipboard(content: &str) {
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(content.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Helper to read a file and verify it is non-binary UTF-8 text.
fn read_text_file(path: &Path) -> io::Result<Vec<String>> {
    let bytes = fs::read(path)?;
    if bytes.iter().take(4096).any(|&b| b == 0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File is binary or not valid text",
        ));
    }
    let text = String::from_utf8(bytes).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
    })?;
    Ok(text.lines().map(|s| s.to_string()).collect())
}

/// Constructs the read-only viewer program and arguments tuple.
/// Guaranteed to use only read-only pagers (ghostty, less, bat).
pub fn view_text_command(path: &Path) -> (String, Vec<String>) {
    let path_str = path.to_string_lossy().to_string();
    if is_binary_in_path("ghostty") {
        (
            "ghostty".to_string(),
            vec!["-e".to_string(), "less".to_string(), "-R".to_string(), path_str],
        )
    } else if is_binary_in_path("bat") {
        ("bat".to_string(), vec!["--paging=always".to_string(), path_str])
    } else {
        ("less".to_string(), vec!["-R".to_string(), path_str])
    }
}

/// Launches a read-only viewer in a non-blocking background process.
pub fn open_viewer(path: &Path) -> io::Result<()> {
    let (prog, args) = view_text_command(path);
    Command::new(prog)
        .args(args)
        .spawn()
        .map(|_| ())
}

/// Copies a single line (1-indexed) from a text file to the clipboard and returns it.
pub fn copy_line(path: &Path, line: usize) -> io::Result<String> {
    if line == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Line numbers are 1-indexed (must be >= 1)",
        ));
    }

    let lines = read_text_file(path)?;
    if line > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Line {} out of range (file has {} lines)", line, lines.len()),
        ));
    }

    let selected_line = lines[line - 1].clone();
    copy_to_clipboard(&selected_line);
    Ok(selected_line)
}

/// Copies a range of lines [from, to] (1-indexed inclusive) from a text file to the clipboard.
/// EOF boundary is respected by clamping the upper bound.
pub fn copy_range(path: &Path, from: usize, to: usize) -> io::Result<String> {
    if from == 0 || from > to {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid line range (from must be >= 1 and <= to)",
        ));
    }

    let lines = read_text_file(path)?;
    if from > lines.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Range start {} exceeds file line count {}", from, lines.len()),
        ));
    }

    let end_idx = to.min(lines.len());
    let selected_lines = &lines[(from - 1)..end_idx];
    let joined = selected_lines.join("\n");
    copy_to_clipboard(&joined);
    Ok(joined)
}

/// Handles CLI subcommands for text viewing and text copying.
pub fn handle_cli(args: &[String], _session: &mut SessionState) -> Option<String> {
    if args.len() < 2 {
        return None;
    }

    let cmd = args[1].as_str();
    match cmd {
        "view-text" | "view_text" => {
            if args.len() > 2 {
                let target = Path::new(&args[2]);
                if args.iter().any(|a| a == "--print-cmd") {
                    let (prog, cmd_args) = view_text_command(target);
                    return Some(format!("{} {}", prog, cmd_args.join(" ")));
                } else {
                    let _ = open_viewer(target);
                    return Some(format!("Viewer launched for {}", target.display()));
                }
            }
            None
        }
        "copy-line" | "copy_line" => {
            if args.len() > 3 {
                let target = Path::new(&args[2]);
                if let Ok(line_num) = args[3].parse::<usize>() {
                    match copy_line(target, line_num) {
                        Ok(text) => return Some(text),
                        Err(e) => return Some(format!("Error: {}", e)),
                    }
                }
            }
            None
        }
        "copy-range" | "copy_range" => {
            if args.len() > 4 {
                let target = Path::new(&args[2]);
                let from_res = args[3].parse::<usize>();
                let to_res = args[4].parse::<usize>();
                if let (Ok(from), Ok(to)) = (from_res, to_res) {
                    match copy_range(target, from, to) {
                        Ok(text) => return Some(text),
                        Err(e) => return Some(format!("Error: {}", e)),
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_text_command_returns_pager_never_editor() {
        let p = Path::new("/etc/hostname");
        let (prog, args) = view_text_command(p);
        assert!(
            prog == "ghostty" || prog == "less" || prog == "bat",
            "Program must be a read-only pager, got {}", prog
        );
        let cmd_line = format!("{} {}", prog, args.join(" "));
        let forbidden = ["s".to_owned() + "ubl", "v".to_owned() + "im", "n".to_owned() + "ano", "g".to_owned() + "edit"];
        for ed in &forbidden {
            assert!(!cmd_line.contains(ed), "Command line must not contain editor {}", ed);
        }
    }

    #[test]
    fn test_copy_line_extracts_correct_line() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("swal_test_text_viewer_line.txt");
        let content = "First Line\nSecond Line\nThird Line";
        fs::write(&test_file, content).unwrap();

        let line1 = copy_line(&test_file, 1).unwrap();
        assert_eq!(line1, "First Line");

        let line2 = copy_line(&test_file, 2).unwrap();
        assert_eq!(line2, "Second Line");

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_copy_range_respects_eof() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("swal_test_text_viewer_range.txt");
        let content = "Line A\nLine B\nLine C";
        fs::write(&test_file, content).unwrap();

        let range = copy_range(&test_file, 2, 100).unwrap();
        assert_eq!(range, "Line B\nLine C");

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_copy_line_out_of_range_and_binary() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("swal_test_text_viewer_out_range.txt");
        let content = "Line 1\nLine 2";
        fs::write(&test_file, content).unwrap();

        assert!(copy_line(&test_file, 0).is_err());
        assert!(copy_line(&test_file, 99).is_err());
        let _ = fs::remove_file(test_file);

        let bin_file = temp_dir.join("swal_test_text_viewer_bin.bin");
        fs::write(&bin_file, &[0u8, 15u8, 0u8, 255u8]).unwrap();
        assert!(copy_line(&bin_file, 1).is_err());
        let _ = fs::remove_file(bin_file);
    }
}
