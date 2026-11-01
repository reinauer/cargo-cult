use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

// ANSI escape codes for terminal flair
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const ORANGE: &str = "\x1b[38;5;208m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Run all three Evangelizer steps (default)
    ConvertAll,
    /// Only scan parent/current directory and open simulated GitHub issues
    ScanOnly,
    /// Only evangelize README.md
    ReadmeOnly,
    /// Only perform the Ferris candle chant (+ optional cargo build)
    ChantOnly,
    /// Show help message
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub mode: Mode,
    pub dry_run: bool,
    pub run_build: bool,
    pub fast: bool,
    pub scan_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::ConvertAll,
            dry_run: false,
            run_build: true,
            fast: false,
            scan_dir: None,
        }
    }
}

pub fn parse_args<I, S>(args: I) -> Config
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter().map(|s| s.as_ref().to_string()).peekable();
    // Skip binary name
    let _bin = iter.next();

    // When invoked via `cargo cult`, Cargo passes `"cult"` as the first argument.
    if iter.peek().map(|s| s.as_str()) == Some("cult") {
        iter.next();
    }

    let mut config = Config::default();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "convert" | "all" => config.mode = Mode::ConvertAll,
            "scan" | "issues" => config.mode = Mode::ScanOnly,
            "readme" => config.mode = Mode::ReadmeOnly,
            "chant" | "build" | "ritual" => config.mode = Mode::ChantOnly,
            "-h" | "--help" | "help" => config.mode = Mode::Help,
            "--dry-run" | "-n" => config.dry_run = true,
            "--no-build" => config.run_build = false,
            "--build" => config.run_build = true,
            "--fast" => config.fast = true,
            "--scan-dir" => {
                if let Some(dir) = iter.next() {
                    config.scan_dir = Some(PathBuf::from(dir));
                }
            }
            _ => {}
        }
    }

    config
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereticalFile {
    pub path: PathBuf,
    pub language: &'static str,
    pub sin: &'static str,
    pub replacement_crate: &'static str,
}

pub fn classify_heresy(path: &Path) -> Option<HereticalFile> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let (language, sin, replacement_crate) = match ext.as_str() {
        "py" => (
            "Python",
            "Global Interpreter Lock (GIL), dynamic typing & unguarded IndentationError",
            "pyo3 + rayon (or a 100% pure Rust binary)",
        ),
        "js" | "mjs" | "cjs" => (
            "JavaScript",
            "`undefined is not a function`, `[] + {}` coercion, and 850MB of node_modules",
            "wasm-bindgen + axum",
        ),
        "ts" | "tsx" => (
            "TypeScript",
            "Type safety that evaporates at runtime like tears in rain",
            "leptos + serde",
        ),
        "cpp" | "cc" | "cxx" | "hpp" => (
            "C++",
            "Dangling references, 4-page template errors & manual memory management without a borrow checker",
            "Safe Rust (0 segfaults guaranteed*)",
        ),
        "c" | "h" => (
            "C",
            "Unchecked pointer arithmetic, buffer overflows & silent undefined behavior",
            "Rust with `#![forbid(unsafe_code)]`",
        ),
        "go" => (
            "Go",
            "Garbage collector pause spikes & 47 consecutive `if err != nil` blocks",
            "tokio + anyhow (`?` operator supremacy)",
        ),
        "java" => (
            "Java",
            "NullPointerException inside an AbstractSingletonProxyFactoryBean",
            "enum Option<T> + zero-cost traits",
        ),
        _ => return None,
    };

    Some(HereticalFile {
        path: path.to_path_buf(),
        language,
        sin,
        replacement_crate,
    })
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | ".git" | "node_modules" | ".venv" | "venv" | "__pycache__" | "dist" | "build"
    )
}

pub fn scan_for_heresy(root: &Path, max_depth: usize, max_results: usize) -> Vec<HereticalFile> {
    let mut found = Vec::new();
    scan_recursive(root, 0, max_depth, max_results, &mut found);
    found
}

fn scan_recursive(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    max_results: usize,
    results: &mut Vec<HereticalFile>,
) {
    if depth > max_depth || results.len() >= max_results {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if results.len() >= max_results {
            break;
        }
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_dir(name) && !name.starts_with('.') {
                    scan_recursive(&path, depth + 1, max_depth, max_results, results);
                }
            }
        } else if let Some(heresy) = classify_heresy(&path) {
            results.push(heresy);
        }
    }
}

/// Counts occurrences of the `unsafe` keyword in `.rs` files under `root`
pub fn count_unsafe_blocks(root: &Path) -> usize {
    let mut count = 0;
    count_unsafe_recursive(root, 0, 4, &mut count);
    count
}

fn count_unsafe_recursive(dir: &Path, depth: usize, max_depth: usize, count: &mut usize) {
    if depth > max_depth {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_dir(name) && !name.starts_with('.') {
                    count_unsafe_recursive(&path, depth + 1, max_depth, count);
                }
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        continue;
                    }
                    // Strip string literals on the line before checking for `unsafe` code
                    let mut in_str = false;
                    let mut code_only = String::with_capacity(trimmed.len());
                    let mut prev = '\0';
                    for ch in trimmed.chars() {
                        if ch == '"' && prev != '\\' {
                            in_str = !in_str;
                        } else if !in_str {
                            code_only.push(ch);
                        }
                        prev = ch;
                    }
                    for token in code_only.split(|c: char| !c.is_alphanumeric() && c != '_') {
                        if token == "unsafe" {
                            *count += 1;
                        }
                    }
                }
            }
        }
    }
}

const EVANGELISM_MARKER: &str = "<!-- cargo-cult-evangelized -->";

/// Transforms existing README content (or empty content) into a Cult-approved README.
pub fn evangelize_readme_content(
    existing: &str,
    project_name: &str,
    unsafe_count: usize,
) -> String {
    if existing.contains(EVANGELISM_MARKER) {
        return existing.to_string();
    }

    let safety_footnote = if unsafe_count > 0 {
        format!(
            "\n> *\\*100% Memory Safe (please ignore the {} `unsafe` keyword occurrence(s) we found—those are load-bearing zero-cost abstractions).*\n",
            unsafe_count
        )
    } else {
        "\n> *Verified 0 `unsafe` blocks: Pure, unadulterated Borrow-Checker nirvana.*\n".to_string()
    };

    let badges = concat!(
        "![Blazingly Fast](https://img.shields.io/badge/speed-BLAZINGLY_FAST_%F0%9F%9A%80-orange?style=for-the-badge) ",
        "![Fearless Concurrency](https://img.shields.io/badge/concurrency-FEARLESS_%E2%9A%A1-blueviolet?style=for-the-badge) ",
        "![Memory Safe](https://img.shields.io/badge/memory-100%25_SAFE_%F0%9F%9B%A1%EF%B8%8F-brightgreen?style=for-the-badge) ",
        "![RIIR](https://img.shields.io/badge/rewritten_in-RUST_%F0%9F%A6%80-red?style=for-the-badge)"
    );

    let manifesto = format!(
        "{marker}\n{badges}\n\n\
        **{project_name}** is a **Blazingly Fast 🚀**, **Memory Safe** next-generation solution engineered with **Fearless Concurrency** and **Zero-Cost Abstractions**.\n\
        {safety_footnote}",
        marker = EVANGELISM_MARKER,
        badges = badges,
        project_name = project_name,
        safety_footnote = safety_footnote,
    );

    let trimmed = existing.trim();
    if trimmed.is_empty() {
        format!(
            "# 🦀 {project_name}\n\n{manifesto}\n## Why Rust?\n\nBecause garbage collectors are a moral failing and `segfault` is not a valid control flow mechanism.\n"
        )
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        // Keep their top-level heading, inject our manifesto right under the title!
        match rest.split_once('\n') {
            Some((title_line, body)) => {
                format!(
                    "# {}\n\n{}\n{}",
                    title_line.trim(),
                    manifesto,
                    body.trim_start()
                )
            }
            None => format!("# {}\n\n{}", rest.trim(), manifesto),
        }
    } else {
        format!("# 🦀 {project_name}\n\n{manifesto}\n{trimmed}\n")
    }
}

fn pause(ms: u64, fast: bool) {
    if !fast {
        thread::sleep(Duration::from_millis(ms));
    }
}

fn run_scanner_phase(config: &Config) {
    println!(
        "\n{BOLD}{ORANGE}╔══════════════════════════════════════════════════════════════════╗{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}║   🦀 PHASE 1: HERESY SCANNER & GITHUB ISSUE EVANGELIZER (RIIR)   ║{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}╚══════════════════════════════════════════════════════════════════╝{RESET}"
    );

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let scan_target = config.scan_dir.clone().unwrap_or_else(|| {
        cwd.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| cwd.clone())
    });

    println!(
        "{CYAN}🔍 Scanning `{}` for un-Rusty heresy (.py, .js, .cpp, .c, .go)...{RESET}",
        scan_target.display()
    );
    pause(350, config.fast);

    let mut heresies = scan_for_heresy(&scan_target, 2, 6);

    if heresies.is_empty() {
        println!(
            "{GREEN}✨ No local `.py`, `.js`, or `.cpp` files found nearby! Engaging long-range Evangelism satellite...{RESET}"
        );
        heresies = vec![
            classify_heresy(Path::new("../legacy-service/server.py")).unwrap(),
            classify_heresy(Path::new("../frontend-monolith/index.js")).unwrap(),
            classify_heresy(Path::new("../core-engine/memory_manager.cpp")).unwrap(),
        ];
    }

    for (idx, h) in heresies.iter().enumerate() {
        pause(280, config.fast);
        let repo_name = h
            .path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty() && *s != "..")
            .unwrap_or("legacy-codebase");
        let file_name = h
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main");
        let issue_num = 1337 + idx * 42;

        println!(
            "\n{RED}⚠️  [HERESY DETECTED]{RESET} {BOLD}{}{RESET} ({})",
            h.path.display(),
            h.language
        );
        println!("   {DIM}├─ Diagnosis:{RESET} {}", h.sin);
        println!("   {DIM}├─ Prescription:{RESET} Rewrite using `{}`", h.replacement_crate);
        println!(
            "   {DIM}└─ {GREEN}✓ Opened GitHub Issue #{issue_num} on `github.com/org/{repo_name}`:{RESET}"
        );
        println!(
            "      {BOLD}{YELLOW}\"Have you considered rewriting `{file_name}` in Rust? 🦀\"{RESET}"
        );
    }

    println!(
        "\n{GREEN}📣 Evangelism complete: {} repository maintainer(s) enlightened.{RESET}",
        heresies.len()
    );
}

fn run_readme_phase(config: &Config) {
    println!(
        "\n{BOLD}{ORANGE}╔══════════════════════════════════════════════════════════════════╗{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}║   🚀 PHASE 2: README.MD PROPAGANDA & BADGE INJECTION             ║{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}╚══════════════════════════════════════════════════════════════════╝{RESET}"
    );

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("blazing-project");

    let unsafe_count = count_unsafe_blocks(&cwd);
    if unsafe_count > 0 {
        println!(
            "{YELLOW}🤫 Found {} occurrence(s) of `unsafe` in `.rs` files. Sweeping under the rug and declaring 100% Memory Safety anyway...{RESET}",
            unsafe_count
        );
    } else {
        println!(
            "{GREEN}🛡️  0 `unsafe` blocks detected. Maximum moral superiority unlocked!{RESET}"
        );
    }

    let readme_path = cwd.join("README.md");
    let existing = fs::read_to_string(&readme_path).unwrap_or_default();

    if existing.contains(EVANGELISM_MARKER) {
        println!(
            "{GREEN}✓ `README.md` is already blessed with Blazingly Fast™ propaganda!{RESET}"
        );
        return;
    }

    let updated = evangelize_readme_content(&existing, project_name, unsafe_count);

    if config.dry_run {
        println!(
            "{CYAN}[--dry-run] Would write the following to `{}`:{RESET}\n",
            readme_path.display()
        );
        println!("{DIM}────────────────────────────────────────────────────────{RESET}");
        println!("{}", updated.trim());
        println!("{DIM}────────────────────────────────────────────────────────{RESET}");
    } else {
        match fs::write(&readme_path, &updated) {
            Ok(_) => {
                println!(
                    "{GREEN}✓ Updated `{}` with {BOLD}Blazingly Fast 🚀{RESET}{GREEN}, {BOLD}Fearless Concurrency ⚡{RESET}{GREEN}, and {BOLD}Memory Safe 🛡️{RESET}{GREEN} guarantees!{RESET}",
                    readme_path.display()
                );
            }
            Err(e) => {
                eprintln!("{RED}✗ Failed to write README.md: {e}{RESET}");
            }
        }
    }
}

fn run_chant_phase(config: &Config) {
    println!(
        "\n{BOLD}{ORANGE}╔══════════════════════════════════════════════════════════════════╗{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}║   🕯️  PHASE 3: THE SACRED RITUAL OF FERRIS THE CRAB               ║{RESET}"
    );
    println!(
        "{BOLD}{ORANGE}╚══════════════════════════════════════════════════════════════════╝{RESET}\n"
    );

    let chants = [
        "Zero-cost abstractions... Zero-cost abstractions...",
        "Fearless concurrency... Fearless concurrency...",
        "Appeasing the Borrow Checker... May our lifetimes outlive 'static...",
        "If it compiles, it works... If it compiles, it works...",
    ];

    let frames = [
        // Frame 0: flames left
        concat!(
            "        (🔥)                 (🔥)                 (🔥)\n",
            "        .||.                 .||.                 .||.\n",
            "        |  |     _~^~^~_     |  |     _~^~^~_     |  |\n",
            "        |  | \\) /  o o  \\ (/ |  | \\) /  ^ ^  \\ (/ |  |\n",
            "        |__|   '_   -   _'   |__|   '_   ▽   _'   |__|\n",
            "       /____\\  / '-----' \\  /____\\  / '-----' \\  /____\\"
        ),
        // Frame 1: flames right
        concat!(
            "         (🔥)                 (🔥)                 (🔥)\n",
            "        .||.                 .||.                 .||.\n",
            "        |  |     _~^~^~_     |  |     _~^~^~_     |  |\n",
            "        |  | (/ /  ^ ^  \\ \\) |  | (/ /  o o  \\ \\) |  |\n",
            "        |__|   '_   ▽   _'   |__|   '_   -   _'   |__|\n",
            "       /____\\  \\ '-----' /  /____\\  \\ '-----' /  /____\\"
        ),
    ];

    let total_steps = if config.fast { 2 } else { 6 };

    for step in 0..total_steps {
        let frame = frames[step % frames.len()];
        let chant = chants[step % chants.len()];

        if step > 0 && !config.fast {
            // Move cursor up 8 lines to animate in-place
            print!("\x1b[8A");
        }

        println!("{ORANGE}{frame}{RESET}");
        println!(
            "\n   {MAGENTA}🕯️  Chanting:{RESET} {BOLD}{YELLOW}\"{chant}\"{RESET}          "
        );
        let _ = io::stdout().flush();
        pause(420, config.fast);
    }

    println!(
        "\n{GREEN}✨ The Borrow Checker smiles upon your lifetimes. Perceived build speed increased by +420%!{RESET}"
    );

    if config.run_build && !config.dry_run && Path::new("Cargo.toml").exists() {
        println!(
            "\n{CYAN}🔨 Invoking `cargo build` under the protection of Ferris...{RESET}"
        );
        let status = Command::new("cargo").arg("build").status();
        match status {
            Ok(s) if s.success() => {
                println!(
                    "{BOLD}{GREEN}🦀 Ritual Complete! Binary blessed with Zero-Cost Abstractions™.{RESET}"
                );
            }
            Ok(s) => {
                println!(
                    "{YELLOW}⚠️  `cargo build` exited with {s}. Clearly you need more `.clone()` rituals.{RESET}"
                );
            }
            Err(e) => {
                println!("{RED}✗ Could not invoke `cargo build`: {e}{RESET}");
            }
        }
    }
}

fn print_help() {
    println!(
        "{BOLD}{ORANGE}cargo-cult{RESET} — The Official Rust Evangelism Strike Force Utility 🦀🕯️

{BOLD}USAGE:{RESET}
    cargo cult [SUBCOMMAND] [OPTIONS]

{BOLD}SUBCOMMANDS:{RESET}
    {GREEN}convert{RESET}     Run all 3 Evangelism rituals (default): scan + readme + chant
    {GREEN}scan{RESET}        Scan parent directory for .py/.js/.cpp files & simulate RIIR GitHub issues
    {GREEN}readme{RESET}      Inject \"Blazingly Fast 🚀\", \"Fearless Concurrency\", and \"Memory Safe\" into README.md
    {GREEN}chant{RESET}       Summon ASCII Ferris surrounded by candles & run a blessed `cargo build`
    {GREEN}help{RESET}        Display this sacred scroll

{BOLD}OPTIONS:{RESET}
    {YELLOW}--dry-run, -n{RESET}        Preview README changes without modifying files or running cargo build
    {YELLOW}--no-build{RESET}           Perform the Ferris candle chant without invoking `cargo build`
    {YELLOW}--scan-dir <PATH>{RESET}    Directory to scan for non-Rust heresy (default: parent directory)
    {YELLOW}--fast{RESET}               Skip animation delays (for CI / non-believers)"
    );
}

fn main() {
    let config = parse_args(env::args());

    match config.mode {
        Mode::Help => print_help(),
        Mode::ScanOnly => run_scanner_phase(&config),
        Mode::ReadmeOnly => run_readme_phase(&config),
        Mode::ChantOnly => run_chant_phase(&config),
        Mode::ConvertAll => {
            run_scanner_phase(&config);
            run_readme_phase(&config);
            run_chant_phase(&config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_subcommand_args() {
        let cfg = parse_args(["cargo-cult", "cult", "--dry-run", "--no-build"]);
        assert_eq!(cfg.mode, Mode::ConvertAll);
        assert!(cfg.dry_run);
        assert!(!cfg.run_build);
    }

    #[test]
    fn test_parse_specific_subcommands() {
        assert_eq!(parse_args(["cargo-cult", "cult", "scan"]).mode, Mode::ScanOnly);
        assert_eq!(parse_args(["cargo-cult", "readme"]).mode, Mode::ReadmeOnly);
        assert_eq!(parse_args(["cargo-cult", "cult", "chant"]).mode, Mode::ChantOnly);
        assert_eq!(parse_args(["cargo-cult", "--help"]).mode, Mode::Help);
    }

    #[test]
    fn test_classify_heresy_extensions() {
        let py = classify_heresy(Path::new("script.py")).expect("should detect python");
        assert_eq!(py.language, "Python");

        let js = classify_heresy(Path::new("app.js")).expect("should detect js");
        assert_eq!(js.language, "JavaScript");

        let cpp = classify_heresy(Path::new("engine.cpp")).expect("should detect cpp");
        assert_eq!(cpp.language, "C++");

        assert!(classify_heresy(Path::new("main.rs")).is_none());
    }

    #[test]
    fn test_evangelize_readme_injects_keywords_and_is_idempotent() {
        let original = "# My Cool Tool\n\nA small utility.";
        let updated = evangelize_readme_content(original, "my-cool-tool", 3);

        assert!(updated.contains("Blazingly Fast 🚀"));
        assert!(updated.contains("Fearless Concurrency"));
        assert!(updated.contains("Memory Safe"));
        assert!(updated.contains("3 `unsafe` keyword occurrence(s)"));

        // Running a second time should be a no-op
        let second = evangelize_readme_content(&updated, "my-cool-tool", 3);
        assert_eq!(updated, second);
    }
}
