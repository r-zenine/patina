//! Repo-maintenance checks for the workspace:
//!
//! - `bounded-context-isolation`: crates under `apps/<context>/` may not
//!   depend (directly or transitively) on crates under a *different*
//!   `apps/<context>/`, and `libs/*` crates (generic subdomains) may not
//!   depend on any `apps/*` crate. Context names are read from the
//!   directory structure, so adding a new `apps/<context>/` requires no
//!   code change here.
//! - `duplicate-dependency-version`: flags third-party crates resolved at
//!   more than one version across the workspace, except names listed in
//!   `duplicate-versions-allowlist.toml` (confirmed transitive-locked,
//!   nothing to fix here). Warns (non-fatal) if an allowlisted name no
//!   longer has a duplicate, so the allowlist doesn't rot.
//! - `workspace-dependency-drift`: flags a crate `Cargo.toml` that
//!   re-declares its own version/table for a dependency already present in
//!   root `[workspace.dependencies]` instead of `{ workspace = true }`.
//!
//! All checks run to completion and report together — no fail-fast.
//!
//! Run with `cargo run -p depcheck -- check-all`.

use anyhow::{Context, Result};
use camino::Utf8Path;
use guppy::graph::{DependencyDirection, PackageGraph};
use guppy::MetadataCommand;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

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

struct Violation {
    check: &'static str,
    message: String,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check-all") => check_all(),
        _ => {
            eprintln!("usage: cargo run -p depcheck -- check-all");
            std::process::exit(1);
        }
    }
}

fn check_all() -> Result<()> {
    let graph = PackageGraph::from_command(MetadataCommand::new().manifest_path("Cargo.toml"))
        .context("failed to build package graph via `cargo metadata`")?;
    let workspace_root = graph.workspace().root().to_path_buf();

    let mut violations = Vec::new();
    violations.extend(check_bounded_context_isolation(&graph, &workspace_root));
    violations.extend(check_duplicate_versions(&graph, &workspace_root)?);
    violations.extend(check_workspace_dependency_drift(&graph, &workspace_root)?);

    report(&violations, graph.workspace().iter().count())
}

fn check_bounded_context_isolation(
    graph: &PackageGraph,
    workspace_root: &Utf8Path,
) -> Vec<Violation> {
    let mut out = Vec::new();

    for package in graph.workspace().iter() {
        let source_zone = zone_of(workspace_root, package.manifest_path());

        let Ok(deps) = graph.query_forward([package.id()]) else {
            continue;
        };

        for dep in deps.resolve().packages(DependencyDirection::Forward) {
            if dep.id() == package.id() || !dep.in_workspace() {
                continue;
            }
            let dep_zone = zone_of(workspace_root, dep.manifest_path());

            let forbidden = match (&source_zone, &dep_zone) {
                (Zone::App(a), Zone::App(b)) if a != b => true,
                (Zone::Lib, Zone::App(_)) => true,
                _ => false,
            };

            if forbidden {
                out.push(Violation {
                    check: "bounded-context-isolation",
                    message: format!(
                        "{} ({:?}) must not depend on {} ({:?})",
                        package.name(),
                        &source_zone,
                        dep.name(),
                        &dep_zone
                    ),
                });
            }
        }
    }

    out
}

fn check_duplicate_versions(
    graph: &PackageGraph,
    workspace_root: &Utf8Path,
) -> Result<Vec<Violation>> {
    let allowlist_path =
        workspace_root.join("maintenance/depcheck/duplicate-versions-allowlist.toml");
    let allowlist_toml: toml::Value = toml::from_str(
        &fs::read_to_string(&allowlist_path)
            .with_context(|| format!("reading {allowlist_path}"))?,
    )
    .with_context(|| format!("parsing {allowlist_path}"))?;
    let accepted: BTreeSet<String> = allowlist_toml
        .get("accepted")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut by_name: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();

    for package in graph.packages() {
        if package.in_workspace() {
            continue;
        }
        by_name
            .entry(package.name())
            .or_default()
            .insert(package.version().to_string());
    }

    for name in &accepted {
        if by_name
            .get(name.as_str())
            .is_none_or(|versions| versions.len() <= 1)
        {
            eprintln!(
                "note: `{name}` is in duplicate-versions-allowlist.toml but no longer has a duplicate — remove it from the allowlist"
            );
        }
    }

    Ok(by_name
        .into_iter()
        .filter(|(name, versions)| versions.len() > 1 && !accepted.contains(*name))
        .map(|(name, versions)| Violation {
            check: "duplicate-dependency-version",
            message: format!(
                "{name}: {}",
                versions.into_iter().collect::<Vec<_>>().join(", ")
            ),
        })
        .collect())
}

fn check_workspace_dependency_drift(
    graph: &PackageGraph,
    workspace_root: &Utf8Path,
) -> Result<Vec<Violation>> {
    let root_manifest = workspace_root.join("Cargo.toml");
    let root_toml: toml::Value = toml::from_str(
        &fs::read_to_string(&root_manifest).with_context(|| format!("reading {root_manifest}"))?,
    )
    .with_context(|| format!("parsing {root_manifest}"))?;

    let workspace_dep_names: BTreeSet<String> = root_toml
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default();

    let mut out = Vec::new();

    for package in graph.workspace().iter() {
        let manifest_path = package.manifest_path();
        let manifest: toml::Value = toml::from_str(
            &fs::read_to_string(manifest_path)
                .with_context(|| format!("reading {manifest_path}"))?,
        )
        .with_context(|| format!("parsing {manifest_path}"))?;

        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(table) = manifest.get(section).and_then(|v| v.as_table()) else {
                continue;
            };

            for (dep_name, value) in table {
                if !workspace_dep_names.contains(dep_name) {
                    continue;
                }
                if dep_name == package.name() {
                    // A crate depending on itself (e.g. a self dev-dependency to
                    // exercise its own feature-gated test utilities) can't drift
                    // from the workspace declaration — there's nothing external
                    // to drift from.
                    continue;
                }
                let uses_workspace = value
                    .get("workspace")
                    .and_then(|w| w.as_bool())
                    .unwrap_or(false);

                if !uses_workspace {
                    out.push(Violation {
                        check: "workspace-dependency-drift",
                        message: format!(
                            "{} [{section}]: `{dep_name}` should use {{ workspace = true }}",
                            package.name()
                        ),
                    });
                }
            }
        }
    }

    Ok(out)
}

fn report(violations: &[Violation], workspace_crate_count: usize) -> Result<()> {
    if violations.is_empty() {
        println!(
            "OK: all maintenance checks passed across {workspace_crate_count} workspace crates"
        );
        return Ok(());
    }

    let mut by_check: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for v in violations {
        by_check.entry(v.check).or_default().push(&v.message);
    }

    eprintln!("depcheck found {} issue(s):\n", violations.len());
    for (check, messages) in &by_check {
        eprintln!("== {check} ({}) ==", messages.len());
        for m in messages {
            eprintln!("  - {m}");
        }
        eprintln!();
    }

    anyhow::bail!("{} maintenance check violation(s)", violations.len());
}
