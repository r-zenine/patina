//! Enforces bounded-context isolation across the workspace: crates under
//! `apps/<context>/` may not depend (directly or transitively) on crates
//! under a *different* `apps/<context>/`, and `libs/*` crates (generic
//! subdomains) may not depend on any `apps/*` crate. Context names are
//! read from the directory structure, so adding a new `apps/<context>/`
//! requires no change here.
//!
//! Run with `cargo run -p depcheck -- check-deps`.

use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use guppy::graph::{DependencyDirection, PackageGraph};
use guppy::MetadataCommand;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Zone {
    App(String),
    Lib,
    Other,
}

/// Bounded-context name is whatever directory sits directly under `apps/` —
/// new contexts (e.g. `apps/patina-detect/`) are picked up with no code change.
fn zone_of(workspace_root: &Utf8Path, manifest_path: &Utf8Path) -> Zone {
    let rel = manifest_path
        .strip_prefix(workspace_root)
        .unwrap_or(manifest_path);
    let mut components = rel.components();
    match components.next().map(|c| c.as_str()) {
        Some("apps") => match components.next().map(|c| c.as_str()) {
            Some(context) => Zone::App(context.to_string()),
            None => Zone::Other,
        },
        Some("libs") => Zone::Lib,
        _ => Zone::Other,
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check-deps") => check_deps(),
        _ => {
            eprintln!("usage: cargo run -p depcheck -- check-deps");
            std::process::exit(1);
        }
    }
}

fn check_deps() -> Result<()> {
    let graph = PackageGraph::from_command(MetadataCommand::new().manifest_path("Cargo.toml"))
        .context("failed to build package graph via `cargo metadata`")?;
    let workspace_root = graph.workspace().root().to_path_buf();

    let mut violations = Vec::new();

    for package in graph.workspace().iter() {
        let source_zone = zone_of(&workspace_root, package.manifest_path());

        let deps = graph
            .query_forward([package.id()])
            .context("package id not found in graph")?
            .resolve();

        for dep in deps.packages(DependencyDirection::Forward) {
            if dep.id() == package.id() || !dep.in_workspace() {
                continue;
            }
            let dep_zone = zone_of(&workspace_root, dep.manifest_path());

            let forbidden = match (&source_zone, &dep_zone) {
                (Zone::App(a), Zone::App(b)) if a != b => true,
                (Zone::Lib, Zone::App(_)) => true,
                _ => false,
            };

            if forbidden {
                violations.push(format!(
                    "{} ({:?}) must not depend on {} ({:?})",
                    package.name(),
                    &source_zone,
                    dep.name(),
                    &dep_zone
                ));
            }
        }
    }

    let violations: BTreeSet<_> = violations.into_iter().collect();

    if !violations.is_empty() {
        eprintln!("Bounded-context dependency violations found:\n");
        for v in &violations {
            eprintln!("  - {v}");
        }
        bail!("{} dependency policy violation(s)", violations.len());
    }

    println!(
        "OK: no bounded-context dependency violations across {} workspace crates",
        graph.workspace().iter().count()
    );
    Ok(())
}
