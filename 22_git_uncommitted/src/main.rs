use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;
use std::time::SystemTime;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// ANSI colors (git-like)
const C_CYAN: &str = "\x1b[36m";
const C_YELLOW: &str = "\x1b[33m";
const C_MAGENTA: &str = "\x1b[35m";
const C_GREEN: &str = "\x1b[32m";
const C_RED: &str = "\x1b[31m";
const C_RESET: &str = "\x1b[0m";

// ------------------------------------------------------------
// CACHE
// ------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry {
    head_mtime: u64,
    index_mtime: u64,
    line: String, // full formatted stdout line
}

type Cache = HashMap<String, CacheEntry>;

fn cache_path() -> PathBuf {
    PathBuf::from("/tmp/git-uncommitted/cache.json")
}

fn load_cache() -> Cache {
    fs::read_to_string(cache_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_cache(cache: &Cache) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(path, json);
    }
}

// Cache key includes output-shaping options so -l / -c changes don't reuse stale lines.
fn cache_key(repo: &str, long_mode: bool, cols: usize) -> String {
    format!("{}|{}|{}", repo, if long_mode { 1 } else { 0 }, cols)
}

// ------------------------------------------------------------
// FS HELPERS (NO GIT)
// ------------------------------------------------------------

fn mtime(path: &str) -> Option<u64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn head_mtime(repo: &str) -> Option<u64> {
    mtime(&format!("{}/.git/HEAD", repo))
}

fn index_mtime(repo: &str) -> Option<u64> {
    mtime(&format!("{}/.git/index", repo))
}

fn is_git_repo(repo: &str) -> bool {
    fs::metadata(format!("{}/.git", repo)).is_ok()
}

// ------------------------------------------------------------
// GIT HELPERS
// ------------------------------------------------------------

fn git_capture(repo: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

fn has_unstaged_changes(repo: &str) -> bool {
    let st = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(repo)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    matches!(st, Ok(s) if s.code() == Some(1))
}

// Is HEAD ahead of any remote branch with the same branch name.
// (Matches refs/remotes/<remote>/<branch> for any <remote>.)
fn branch_ahead_of_any_remote_same_branch(repo: &str) -> bool {
    let branch = match git_capture(repo, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(b) if b != "HEAD" => b,
        _ => return false, // detached HEAD or error
    };

    let spec = format!("--remotes=*/{}", branch);
    let out = git_capture(repo, &["rev-list", "--count", "HEAD", "--not", &spec])
        .unwrap_or_default();

    out.trim().parse::<u32>().unwrap_or(0) > 0
}

// ------------------------------------------------------------
// BUILD OUTPUT LINE
// ------------------------------------------------------------

fn build_line(repo: &str, long_mode: bool, cols: usize) -> Option<String> {
    let dirty = has_unstaged_changes(repo);
    let ahead = branch_ahead_of_any_remote_same_branch(repo);

    if !dirty && !ahead {
        return None;
    }

    if !long_mode {
        return Some(repo.to_string());
    }

    let log = git_capture(
        repo,
        &[
            "log",
            "-1",
            "--date=short",
            "--pretty=format:%h %cd %s|%an",
        ],
    )?;

    let mut p = log.splitn(3, ' ');
    let hash = p.next()?;
    let date = p.next()?;
    let rest = p.next()?;
    let mut ma = rest.splitn(2, '|');
    let msg = ma.next().unwrap_or("");
    let author = ma.next().unwrap_or("");

    let mut refs = vec!["HEAD".to_string()];
    if let Some(b) = git_capture(
        repo,
        &[
            "branch",
            "--all",
            "--points-at",
            "HEAD",
            "--format=%(refname:short)",
        ],
    ) {
        refs.extend(b.lines().map(|s| s.to_string()));
    }

    let mut line = format!(
        "{:<width$} {}{}{} {}{}{} {} {}{}{} {}{}{}",
        repo,
        C_CYAN, hash, C_RESET,
        C_YELLOW, date, C_RESET,
        msg,
        C_MAGENTA, author, C_RESET,
        C_GREEN, format!("({})", refs.join(", ")), C_RESET,
        width = cols
    );

    if dirty {
        if let Some(status) = git_capture(repo, &["status", "--porcelain"]) {
            let (mut m, mut a, mut u) = (0, 0, 0);
            for l in status.lines() {
                if l.starts_with("??") {
                    u += 1;
                } else if l.chars().nth(1) == Some('M') {
                    m += 1;
                } else if l.chars().nth(1) == Some('A') {
                    a += 1;
                }
            }

            let mut parts = Vec::new();
            if m > 0 {
                parts.push(format!(
                    "{}M{} {} file{}",
                    C_RED,
                    C_RESET,
                    m,
                    if m == 1 { "" } else { "s" }
                ));
            }
            if a > 0 {
                parts.push(format!(
                    "{}A{} {} file{}",
                    C_RED,
                    C_RESET,
                    a,
                    if a == 1 { "" } else { "s" }
                ));
            }
            if u > 0 {
                parts.push(format!(
                    "{}??{} {} file{}",
                    C_RED,
                    C_RESET,
                    u,
                    if u == 1 { "" } else { "s" }
                ));
            }

            if !parts.is_empty() {
                line.push_str("  ");
                line.push_str(&parts.join(", "));
            }
        }
    }

    Some(line)
}

// ------------------------------------------------------------
// MAIN
// ------------------------------------------------------------

fn main() {
    let mut long_mode = false;
    let mut cols: usize = 50;

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "-l" | "--long" => long_mode = true,
            "-c" | "--columns" => {
                if let Some(v) = args.next() {
                    cols = v.parse().unwrap_or(50);
                }
            }
            _ => {}
        }
    }

    let stdin = io::stdin();
    let repos: Vec<String> = stdin.lock().lines().flatten().collect();

    let cache = load_cache();

    // Send cache key + entry so the printer thread can persist cache correctly.
    let (tx, rx) = channel::<(String, CacheEntry)>();

    let printer = thread::spawn(move || {
        let mut new_cache = load_cache();

        for (key, entry) in rx {
            println!("{}", entry.line);
            new_cache.insert(key, entry);
        }

        save_cache(&new_cache);
    });

    repos.par_iter().for_each(|repo| {
        if !is_git_repo(repo) {
            return;
        }

        let head_mt = match head_mtime(repo) {
            Some(v) => v,
            None => return,
        };
        let index_mt = match index_mtime(repo) {
            Some(v) => v,
            None => return,
        };

        let key = cache_key(repo, long_mode, cols);

        if let Some(entry) = cache.get(&key) {
            if entry.head_mtime == head_mt && entry.index_mtime == index_mt {
                let _ = tx.send((key, entry.clone()));
                return;
            }
        }

        let line = match build_line(repo, long_mode, cols) {
            Some(v) => v,
            None => return,
        };

        let entry = CacheEntry {
            head_mtime: head_mt,
            index_mtime: index_mt,
            line,
        };

        let _ = tx.send((key, entry));
    });

    drop(tx);
    let _ = printer.join();
}
