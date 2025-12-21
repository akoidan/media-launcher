use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{Map, Value};

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "ROOT_DIR")]
    root_dir: PathBuf,
}

fn insert_path(tree: &mut Map<String, Value>, parts: &[String]) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        tree.insert(parts[0].clone(), Value::String(String::new()));
        return;
    }

    let head = parts[0].clone();
    let tail = &parts[1..];

    let entry = tree
        .entry(head)
        .or_insert_with(|| Value::Object(Map::new()));

    match entry {
        Value::Object(map) => insert_path(map, tail),
        Value::String(_) => {
            // Path conflict (file vs dir). Prefer dir shape.
            *entry = Value::Object(Map::new());
            if let Value::Object(map) = entry {
                insert_path(map, tail);
            }
        }
        _ => {
            *entry = Value::Object(Map::new());
            if let Value::Object(map) = entry {
                insert_path(map, tail);
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

fn main() -> Result<()> {
    let args = Args::parse();
    let root = args
        .root_dir
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize {}", args.root_dir.display()))?;

    let mut tree = Map::<String, Value>::new();

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

            if matches!(ext.as_str(), "cmd" | "bash" | "sh") {
                continue;
            }

            if !matches!(ext.as_str(), "mkv" | "mka" | "ass" | "ttf") {
                continue;
            }

            let parts = rel_parts(&root, &path)?;
            insert_path(&mut tree, &parts);
        }
    }

    let out = Value::Object(tree);
    println!("{}", serde_json::to_string_pretty(&out)?);

    Ok(())
}
