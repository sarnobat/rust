use std::env;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};

use rayon::prelude::*; // <-- PARALLELISM SUPPORT

/* ANSI colors (matching git defaults) */
const C_CYAN: &str = "\x1b[36m";
const C_YELLOW: &str = "\x1b[33m";
const C_MAGENTA: &str = "\x1b[35m";
const C_GREEN: &str = "\x1b[32m";
const C_RED: &str = "\x1b[31m";
const C_RESET: &str = "\x1b[0m";

/* ------------------------------------------------------------ */
/* git helpers                                                  */
/* ------------------------------------------------------------ */

fn git_silent(dir: &str, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn git_capture(dir: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/* ------------------------------------------------------------ */
/* predicates                                                   */
/* ------------------------------------------------------------ */

fn is_git_repo(dir: &str) -> bool {
    git_silent(dir, &["rev-parse", "--is-inside-work-tree", "--quiet"])
}

fn has_unstaged_changes(dir: &str) -> bool {
    let st = Command::new("git")
        .args(&["diff", "--quiet"])
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    matches!(st, Ok(s) if s.code() == Some(1))
}

fn branch_ahead_of_upstream(dir: &str) -> bool {
    let out = git_capture(
        dir,
        &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
    );

    if let Some(s) = out {
        let mut it = s.split_whitespace();
        let _behind = it.next().and_then(|v| v.parse::<u32>().ok());
        let ahead = it.next().and_then(|v| v.parse::<u32>().ok());
        return matches!(ahead, Some(n) if n > 0);
    }
    false
}

/* ------------------------------------------------------------ */
/* git info                                                     */
/* ------------------------------------------------------------ */

fn last_commit(dir: &str) -> Option<(String, String, String, String)> {
    let out = git_capture(
        dir,
        &[
            "log",
            "-1",
            "--date=short",
            "--pretty=format:%h %cd %s|%an",
        ],
    )?;

    let mut parts = out.splitn(3, ' ');
    let hash = parts.next()?.to_string();
    let date = parts.next()?.to_string();
    let rest = parts.next()?;

    let mut ma = rest.splitn(2, '|');
    let msg = ma.next()?.to_string();
    let author = ma.next().unwrap_or("").to_string();

    Some((hash, date, msg, author))
}

fn refs_at_head(dir: &str) -> String {
    let mut refs = vec!["HEAD".to_string()];

    if let Some(b) = git_capture(
        dir,
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

    if let Some(t) = git_capture(dir, &["tag", "--points-at", "HEAD"]) {
        refs.extend(t.lines().map(|s| s.to_string()));
    }

    refs.join(", ")
}

fn unstaged_summary(dir: &str) -> Vec<(String, usize)> {
    let mut m = 0;
    let mut a = 0;
    let mut u = 0;

    if let Some(out) = git_capture(dir, &["status", "--porcelain"]) {
        for line in out.lines() {
            if line.starts_with("??") {
                u += 1;
            } else if line.chars().nth(1) == Some('M') {
                m += 1;
            } else if line.chars().nth(1) == Some('A') {
                a += 1;
            }
        }
    }

    let mut v = Vec::new();
    if m > 0 { v.push(("M".into(), m)); }
    if a > 0 { v.push(("A".into(), a)); }
    if u > 0 { v.push(("??".into(), u)); }
    v
}

/* ------------------------------------------------------------ */
/* main                                                         */
/* ------------------------------------------------------------ */

fn main() {
    let mut long_mode = false;
    let mut path_cols: usize = 50;

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "-l" | "--long" => long_mode = true,
            "-c" | "--columns" => {
                if let Some(v) = args.next() {
                    path_cols = v.parse().unwrap_or(50);
                }
            }
            _ => {}
        }
    }

    /* --------------------------------------------------------
     * READ ALL INPUT FIRST
     * (required for parallel processing)
     * -------------------------------------------------------- */
    let stdin = io::stdin();
    let paths: Vec<String> = stdin
        .lock()
        .lines()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    /* --------------------------------------------------------
     * PARALLEL PROCESSING OPTIMIZATION (Rayon)
     *
     * Each path is independent and executes multiple git calls.
     * This is the ONLY place parallelism is introduced.
     * -------------------------------------------------------- */
    let mut results: Vec<(usize, String)> = paths
        .par_iter()
        .enumerate()
        .filter_map(|(idx, dir)| {
            if !is_git_repo(dir) {
                return None;
            }

            let dirty = has_unstaged_changes(dir);
            let ahead = branch_ahead_of_upstream(dir);

            if !dirty && !ahead {
                return None;
            }

            if !long_mode {
                return Some((idx, dir.clone()));
            }

            let (hash, date, msg, author) = last_commit(dir)?;

            let refs = refs_at_head(dir);

            let mut line = format!(
                "{:<width$} {}{}{} {}{}{} {} {}{}{} {}{}{}",
                dir,
                C_CYAN, hash, C_RESET,
                C_YELLOW, date, C_RESET,
                msg,
                C_MAGENTA, author, C_RESET,
                C_GREEN, format!("({})", refs), C_RESET,
                width = path_cols
            );

            if dirty {
                let parts = unstaged_summary(dir);
                if !parts.is_empty() {
                    line.push_str("  ");
                    for (i, (k, v)) in parts.iter().enumerate() {
                        if i > 0 {
                            line.push_str(", ");
                        }
                        line.push_str(&format!(
                            "{}{}{} {} file{}",
                            C_RED, k, C_RESET,
                            v,
                            if *v == 1 { "" } else { "s" }
                        ));
                    }
                }
            }

            Some((idx, line))
        })
        .collect();

    /* --------------------------------------------------------
     * RESTORE ORIGINAL INPUT ORDER
     * -------------------------------------------------------- */
    results.sort_by_key(|(i, _)| *i);

    for (_, line) in results {
        println!("{}", line);
    }
}
