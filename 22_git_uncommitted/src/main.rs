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

/* ANSI colors */
const C_CYAN: &str = "\x1b[36m";
const C_YELLOW: &str = "\x1b[33m";
const C_MAGENTA: &str = "\x1b[35m";
const C_GREEN: &str = "\x1b[32m";
const C_RED: &str = "\x1b[31m";
const C_RESET: &str = "\x1b[0m";

/* ============================== CACHE ============================== */

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry {
    head_mtime: u64,
    index_mtime: u64,
    line: String,
    #[serde(default)]
    saved_at: u64,
}

const CACHE_TTL_SECS: u64 = 300; // 5 minutes

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

fn cache_key(repo: &str, long_mode: bool, cols: usize) -> String {
    format!("{}|{}|{}", repo, if long_mode { 1 } else { 0 }, cols)
}

/* ============================ FS HELPERS ============================ */

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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_git_repo(repo: &str) -> bool {
    fs::metadata(format!("{}/.git", repo)).is_ok()
}

/* ============================ GIT HELPERS ============================ */

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

/* HEAD ahead-of-remote using merge-base correctness */
fn branch_ahead_of_any_remote_same_branch(repo: &str) -> bool {
    let branch = match git_capture(repo, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(b) if b != "HEAD" => b,
        _ => return false,
    };

    let remotes = git_capture(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)",
            &format!("refs/remotes/*/{}", branch),
        ],
    )
    .unwrap_or_default();

    for r in remotes.lines() {
        let base = git_capture(repo, &["merge-base", "HEAD", r]);
        let head = git_capture(repo, &["rev-parse", "HEAD"]);
        let remote = git_capture(repo, &["rev-parse", r]);

        if let (Some(b), Some(h), Some(ro)) = (base, head, remote) {
            if b == ro && h != ro {
                return true;
            }
        }
    }

    false
}

/* ========================== BUILD OUTPUT ============================ */

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
        &["log", "-1", "--date=short", "--pretty=format:%h %cd %s|%an"],
    )?;

    let mut p = log.splitn(3, ' ');
    let hash = p.next()?;
    let date = p.next()?;
    let rest = p.next()?;
    let mut ma = rest.splitn(2, '|');
    let msg = ma.next().unwrap_or("");
    let author = ma.next().unwrap_or("");

    /* -------- COLLECT REFS CORRECTLY -------- */

    let mut refs = Vec::new();

    // Direct refs (HEAD, local branches, tags)
    if let Some(r) = git_capture(
        repo,
        &[
            "for-each-ref",
            "--points-at",
            "HEAD",
            "--format=%(refname:short)",
        ],
    ) {
        refs.extend(r.lines().map(|s| s.to_string()));
    }

    // Remote-tracking branches equal to HEAD
    if let Some(branch) = git_capture(repo, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        if branch != "HEAD" {
            let head_oid = git_capture(repo, &["rev-parse", "HEAD"]);
            if let Some(remotes) = git_capture(
                repo,
                &[
                    "for-each-ref",
                    "--format=%(refname)",
                    &format!("refs/remotes/*/{}", branch),
                ],
            ) {
                for r in remotes.lines() {
                    let oid = git_capture(repo, &["rev-parse", r]);
                    if oid == head_oid {
                        if let Some(short) = r.strip_prefix("refs/remotes/") {
                            refs.push(short.to_string());
                        }
                    }
                }
            }
        }
    }

    refs.sort();
    refs.dedup();

    let mut line = format!(
        "{:<width$} {}{}{} {}{}{} {} {}{}{} {}{}{}",
        repo,
        C_CYAN,
        hash,
        C_RESET,
        C_YELLOW,
        date,
        C_RESET,
        msg,
        C_MAGENTA,
        author,
        C_RESET,
        C_GREEN,
        format!("({})", refs.join(", ")),
        C_RESET,
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

/* =============================== MAIN =============================== */

fn main() {
    // Parse CLI flags
    let mut long_mode = false;
    let mut cols: usize = 50;
    let mut use_cache = true;
    {
        let mut args = env::args().skip(1).peekable();
        while let Some(a) = args.next() {
            match a.as_str() {
                "-l" | "--long" => long_mode = true,
                "-c" | "--columns" => {
                    if let Some(v) = args.next() {
                        cols = v.parse().unwrap_or(50);
                    }
                }
                "--no-cache" => use_cache = false,
                _ => {}
            }
        }
    }

    let (tx, rx) = channel::<(String, CacheEntry)>();

    // Spawn printer thread to emit lines and store cache when enabled
    let printer = thread::spawn(move || {
        if !use_cache {
            for (_, entry) in rx {
                println!("{}", entry.line);
            }
            return;
        }

        let mut new_cache = load_cache();
        for (key, entry) in rx {
            println!("{}", entry.line);
            new_cache.insert(key, entry);
        }
        save_cache(&new_cache);
    });

    // Read all repo paths from stdin
    let stdin = io::stdin();
    let repos: Vec<String> = stdin.lock().lines().flatten().collect();

    // Process repos in parallel
    repos.par_iter().for_each(|repo| {
        if !is_git_repo(repo) {
            return;
        }

        let key = cache_key(repo, long_mode, cols);

        {
            let now = now_secs();
            let head_mt = match head_mtime(repo) {
                Some(v) => v,
                None => return,
            };
            let index_mt = match index_mtime(repo) {
                Some(v) => v,
                None => return,
            };
            if use_cache {
                let cache = if use_cache {
                    load_cache()
                } else {
                    HashMap::new()
                };
                if let Some(entry) = cache.get(&key) {
                    let fresh = now.saturating_sub(entry.saved_at) <= CACHE_TTL_SECS;
                    if fresh && entry.head_mtime == head_mt && entry.index_mtime == index_mt {
                        let _ = tx.send((key, entry.clone()));
                        return;
                    }
                }
            }

            let entry;
            {
                let line = match build_line(repo, long_mode, cols) {
                    Some(v) => v,
                    None => return,
                };

                entry = CacheEntry {
                    head_mtime: head_mt,
                    index_mtime: index_mt,
                    line,
                    saved_at: now,
                };

                let _ = tx.send((key, entry));
            }
        }
    });

    drop(tx);
    let _ = printer.join();
}
