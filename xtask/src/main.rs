use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BASELINE: &str = "xtask/comment-baseline.json";
const RUSTDOC_CRATES: &[&str] = &["crates/guest/", "crates/native-sdk/"];

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["comments"] => comments(&root, false),
        ["comments", "--lower-baseline"] => comments(&root, true),
        _ => Err("usage: cargo xtask comments [--lower-baseline]".to_owned()),
    };
    match outcome {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(problems) => {
            eprintln!("{problems}");
            ExitCode::FAILURE
        }
    }
}

fn comment_lines(path: &str, text: &str) -> usize {
    let rustdoc_allowed = RUSTDOC_CRATES.iter().any(|c| path.starts_with(c));
    text.lines()
        .map(str::trim_start)
        .filter(|l| l.starts_with("//"))
        .filter(|l| !(rustdoc_allowed && (l.starts_with("///") || l.starts_with("//!"))))
        .count()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && !path.ends_with("target") {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn counts(root: &Path) -> BTreeMap<String, usize> {
    let mut files = Vec::new();
    for dir in ["crates", "xtask", "examples", "templates"] {
        rust_files(&root.join(dir), &mut files);
    }
    files
        .into_iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
            let text = std::fs::read_to_string(&p).ok()?;
            Some((rel.clone(), comment_lines(&rel, &text)))
        })
        .collect()
}

fn comments(root: &Path, lower: bool) -> Result<String, String> {
    let now = counts(root);
    let recorded = std::fs::read(root.join(BASELINE)).ok().and_then(|b| serde_json::from_slice::<BTreeMap<String, usize>>(&b).ok());
    let baseline = match recorded {
        Some(b) => b,
        None if lower => now.clone(),
        None => return Err(format!("comments: {BASELINE} is missing; record it with `cargo xtask comments --lower-baseline`")),
    };
    let problems: Vec<String> = now
        .iter()
        .filter_map(|(file, &n)| {
            let allowed = baseline.get(file).copied().unwrap_or(0);
            (n > allowed).then(|| format!("{file}: {n} comment lines, at most {allowed} allowed (new files: 0)"))
        })
        .collect();
    if !problems.is_empty() {
        return Err(format!("comments: let names, types and tests carry the meaning\n{}", problems.join("\n")));
    }
    let total: usize = now.values().sum();
    if lower {
        let kept: Map<String, Value> = now.iter().filter(|(_, n)| **n > 0).map(|(f, n)| (f.clone(), Value::from(*n))).collect();
        let text = serde_json::to_string_pretty(&kept).map_err(|e| e.to_string())? + "\n";
        std::fs::write(root.join(BASELINE), text).map_err(|e| e.to_string())?;
    }
    Ok(format!("comments: {total} lines in {} files, none above the baseline", now.values().filter(|n| **n > 0).count()))
}

#[cfg(test)]
mod tests {
    use super::comment_lines;

    #[test]
    fn rustdoc_counts_outside_the_author_sdk_and_plain_comments_count_everywhere() {
        let text = "/// doc\n//! module\n// note\nlet url = \"https://x\";\n    // indented\n";
        assert_eq!(comment_lines("crates/ext-host/src/a.rs", text), 4);
        assert_eq!(comment_lines("crates/guest/src/a.rs", text), 2);
    }
}
