use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{Map, Value};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "INPUT_DIR")]
    input_dir: PathBuf,
}

fn insert_path_with_value(tree: &mut Map<String, Value>, parts: &[String], value: Value) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        tree.insert(parts[0].clone(), value);
        return;
    }

    let head = parts[0].clone();
    let tail = &parts[1..];

    let entry = tree.entry(head).or_insert_with(|| Value::Object(Map::new()));

    match entry {
        Value::Object(map) => insert_path_with_value(map, tail, value),
        Value::String(_) => {
            // Path conflict (file vs dir). Prefer dir shape.
            *entry = Value::Object(Map::new());
            if let Value::Object(map) = entry {
                insert_path_with_value(map, tail, value);
            }
        }
        _ => {
            *entry = Value::Object(Map::new());
            if let Value::Object(map) = entry {
                insert_path_with_value(map, tail, value);
            }
        }
    }
}

fn rel_parts(root: &Path, full: &Path) -> Result<Vec<String>> {
    let rel = full
        .strip_prefix(root)
        .with_context(|| format!("Failed to strip prefix {} from {}", root.display(), full.display()))?;

    let mut parts = Vec::new();
    for p in rel.components() {
        let s = p.as_os_str().to_str().ok_or_else(|| {
            anyhow::anyhow!("Non-UTF8 path component in {}", full.display())
        })?;
        if !s.is_empty() {
            parts.push(s.to_string());
        }
    }
    Ok(parts)
}

fn rel_key_for_expected(root: &Path, full: &Path, prefix: &str) -> Result<String> {
    let parts = rel_parts(root, full)?;
    Ok(format!("{prefix}/{}", parts.join("/")))
}

#[derive(Debug, Serialize, Deserialize)]
struct FixtureOut {
    root: String,
    player: String,
    path_dirs: Vec<String>,
    binaries: Vec<String>,
    files: Value,
    expected_windows_writes: BTreeMap<String, String>,
    expected_unix_writes: BTreeMap<String, String>,
}

fn normalize_root_string(path: &Path) -> String {
    let s = path.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        // Path::canonicalize on Windows can produce verbatim paths like \\?\D:\... or \\?\UNC\server\share\...
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", rest);
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return rest.to_string();
        }
    }

    s
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = args
        .input_dir
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", args.input_dir.display()))?;

    let mut files_tree = Map::<String, Value>::new();
    let mut expected_windows_writes = BTreeMap::<String, String>::new();
    let mut expected_unix_writes = BTreeMap::<String, String>::new();

    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).with_context(|| format!("read_dir {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let ty = entry.file_type()?;

            if ty.is_dir() {
                stack.push(path);
                continue;
            }

            if !ty.is_file() {
                continue;
            }

            let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
            let ext = ext.to_ascii_lowercase();

            if ext == "cmd" {
                let key = rel_key_for_expected(&root, &path, "root")?;
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                expected_windows_writes.insert(key, contents);
                continue;
            }

            if ext == "bash" {
                let key = rel_key_for_expected(&root, &path, "root")?;
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                expected_unix_writes.insert(key, contents);
                continue;
            }

            let parts = rel_parts(&root, &path)?;
            insert_path_with_value(&mut files_tree, &parts, Value::String(String::new()));
        }
    }

    let fixture = FixtureOut {
        root: normalize_root_string(&root),
        player: "mpv".to_string(),
        path_dirs: vec!["bin".to_string()],
        binaries: vec!["mpv".to_string()],
        files: Value::Object(files_tree),
        expected_windows_writes,
        expected_unix_writes,
    };

    let out_path = {
        let base = root
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Could not determine directory name from {}", root.display()))?;
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("tests");
        p.push("fixtures");
        p.push(format!("{base}.json"));
        p
    };

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&fixture)?;
    fs::write(&out_path, json).with_context(|| format!("Failed to write {}", out_path.display()))?;
    println!("{}", out_path.display());

    Ok(())
}
