// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::module_name_repetitions)]

//! `biome.yaml` manifest validation for NUCLEUS compositions.
//!
//! Validates that a gate's composition manifest declares primals that exist
//! in the `nucleus-primals` registry, compositions follow dependency ordering,
//! and primal configurations are consistent.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;

use crate::util;

// ── Manifest schema (subset matching toadstool-core canonical shape) ──

#[derive(Debug, Deserialize)]
pub struct BiomeManifest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub metadata: BiomeMetadata,
    #[serde(default)]
    pub primals: HashMap<String, ManifestPrimalConfig>,
    #[serde(default)]
    pub compositions: Vec<CompositionGraph>,
    pub federation: Option<ManifestFederation>,
}

fn default_api_version() -> String {
    "v1".into()
}

fn default_kind() -> String {
    "Biome".into()
}

#[derive(Debug, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    #[expect(dead_code, reason = "schema completeness")]
    #[serde(default)]
    pub description: Option<String>,
    #[expect(dead_code, reason = "schema completeness")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[expect(dead_code, reason = "schema completeness")]
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestPrimalConfig {
    #[expect(
        dead_code,
        reason = "schema completeness — checked in future version validation"
    )]
    pub version: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[expect(
        dead_code,
        reason = "schema completeness — validated against capability_registry"
    )]
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[expect(
        dead_code,
        reason = "schema completeness — validated against gossip injection"
    )]
    #[serde(default)]
    pub gossip_events: Vec<String>,
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct CompositionGraph {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, Vec<String>>,
    #[expect(
        dead_code,
        reason = "schema completeness — used in future lifecycle executor"
    )]
    #[serde(default = "default_true")]
    pub auto_start: bool,
    #[expect(
        dead_code,
        reason = "schema completeness — used in future startup ordering"
    )]
    #[serde(default)]
    pub priority: u32,
    pub readiness: Option<CompositionReadiness>,
}

#[derive(Debug, Deserialize)]
pub struct CompositionReadiness {
    #[serde(default)]
    pub require_healthy: Vec<String>,
    #[expect(
        dead_code,
        reason = "schema completeness — used in future readiness probe"
    )]
    #[serde(default = "default_readiness_timeout")]
    pub timeout_secs: u64,
}

const fn default_readiness_timeout() -> u64 {
    120
}

#[derive(Debug, Deserialize)]
pub struct ManifestFederation {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub peers: Vec<String>,
}

// ── Validation report ──

struct ManifestReport {
    pass: u32,
    fail: u32,
    warn: u32,
    lines: Vec<String>,
}

impl ManifestReport {
    const fn new() -> Self {
        Self {
            pass: 0,
            fail: 0,
            warn: 0,
            lines: Vec::new(),
        }
    }

    fn pass(&mut self, tag: &str, msg: &str) {
        eprintln!("  PASS  [{tag}] {msg}");
        self.pass += 1;
        self.lines.push(format!("| {tag} | PASS | {msg} |"));
    }

    fn fail(&mut self, tag: &str, msg: &str) {
        eprintln!("  FAIL  [{tag}] {msg}");
        self.fail += 1;
        self.lines.push(format!("| {tag} | FAIL | {msg} |"));
    }

    fn warn(&mut self, tag: &str, msg: &str) {
        eprintln!("  WARN  [{tag}] {msg}");
        self.warn += 1;
        self.lines.push(format!("| {tag} | WARN | {msg} |"));
    }
}

fn log(msg: &str) {
    util::tlog(msg);
}

/// Validate a `biome.yaml` manifest file against the `nucleus-primals` registry.
///
/// Returns `Ok(true)` if no failures, `Ok(false)` if any validation failed.
pub async fn validate(path: &Path) -> Result<bool, std::io::Error> {
    log("═══════════════════════════════════════════════════════════");
    log("  biome.yaml Manifest Validation");
    log(&format!("  File: {}", path.display()));
    log("═══════════════════════════════════════════════════════════");
    log("");

    let content = tokio::fs::read_to_string(path).await?;
    let manifest: BiomeManifest = match serde_yaml::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("  FAIL  [PARSE] Cannot parse biome.yaml: {e}");
            return Ok(false);
        }
    };

    let mut report = ManifestReport::new();

    validate_schema(&manifest, &mut report);
    validate_primals(&manifest, &mut report);
    validate_compositions(&manifest, &mut report);
    validate_dependencies(&manifest, &mut report);
    validate_federation(&manifest, &mut report);

    log("");
    log("═══════════════════════════════════════════════════════════");
    log("  biome.yaml Validation — Results");
    log(&format!(
        "  PASS: {}  FAIL: {}  WARN: {}",
        report.pass, report.fail, report.warn
    ));
    log("═══════════════════════════════════════════════════════════");

    Ok(report.fail == 0)
}

fn validate_schema(manifest: &BiomeManifest, report: &mut ManifestReport) {
    log("── Schema ──");

    if manifest.api_version == "v1" {
        report.pass("SCHEMA-01", "api_version is v1");
    } else {
        report.warn(
            "SCHEMA-01",
            &format!("api_version '{}' (expected v1)", manifest.api_version),
        );
    }

    if manifest.kind == "Biome" {
        report.pass("SCHEMA-02", "kind is Biome");
    } else {
        report.warn(
            "SCHEMA-02",
            &format!("kind '{}' (expected Biome)", manifest.kind),
        );
    }

    if manifest.metadata.name.is_empty() {
        report.fail("SCHEMA-03", "metadata.name is empty");
    } else {
        report.pass(
            "SCHEMA-03",
            &format!("metadata.name = '{}'", manifest.metadata.name),
        );
    }

    if manifest.metadata.version.is_empty() {
        report.fail("SCHEMA-04", "metadata.version is empty");
    } else {
        report.pass(
            "SCHEMA-04",
            &format!("metadata.version = '{}'", manifest.metadata.version),
        );
    }
}

fn validate_primals(manifest: &BiomeManifest, report: &mut ManifestReport) {
    log("");
    log("── Primals ──");

    if manifest.primals.is_empty() {
        report.warn("PRIMAL-00", "No primals declared in manifest");
        return;
    }

    let registry_slugs: HashSet<&str> = nucleus_primals::all_slugs().into_iter().collect();

    for (slug, config) in &manifest.primals {
        if registry_slugs.contains(slug.as_str()) {
            report.pass(
                &format!("PRIMAL-{}", slug.to_uppercase()),
                &format!("{slug} exists in nucleus-primals registry"),
            );
        } else {
            report.fail(
                &format!("PRIMAL-{}", slug.to_uppercase()),
                &format!("{slug} NOT in nucleus-primals registry"),
            );
        }

        if !config.enabled {
            report.warn(
                &format!("PRIMAL-{}-EN", slug.to_uppercase()),
                &format!("{slug} declared but disabled"),
            );
        }

        for dep in &config.dependencies {
            if !manifest.primals.contains_key(dep) && !registry_slugs.contains(dep.as_str()) {
                report.warn(
                    &format!("PRIMAL-{}-DEP", slug.to_uppercase()),
                    &format!("{slug} depends on '{dep}' which is not declared"),
                );
            }
        }
    }

    report.pass(
        "PRIMAL-COUNT",
        &format!(
            "{}/{} declared primals in registry",
            manifest
                .primals
                .keys()
                .filter(|s| registry_slugs.contains(s.as_str()))
                .count(),
            manifest.primals.len()
        ),
    );
}

fn validate_compositions(manifest: &BiomeManifest, report: &mut ManifestReport) {
    log("");
    log("── Compositions ──");

    if manifest.compositions.is_empty() {
        report.warn("COMP-00", "No compositions declared");
        return;
    }

    let declared_primals: HashSet<&str> = manifest.primals.keys().map(String::as_str).collect();
    let valid_kinds = ["Tower", "Nest", "Node", "Custom"];

    for (i, comp) in manifest.compositions.iter().enumerate() {
        let tag = format!("COMP-{:02}", i + 1);

        if comp.name.is_empty() {
            report.fail(&tag, &format!("composition[{i}] has empty name"));
            continue;
        }

        if valid_kinds.contains(&comp.kind.as_str()) {
            report.pass(&tag, &format!("'{}' kind={}", comp.name, comp.kind));
        } else {
            report.warn(
                &tag,
                &format!(
                    "'{}' unknown kind '{}' (expected Tower/Nest/Node/Custom)",
                    comp.name, comp.kind
                ),
            );
        }

        validate_composition_kind(comp, report);

        for member in &comp.members {
            if !declared_primals.contains(member.as_str())
                && nucleus_primals::lookup(member).is_none()
            {
                report.warn(
                    &format!("{tag}-MBR"),
                    &format!(
                        "'{}' member '{member}' not in manifest primals or registry",
                        comp.name
                    ),
                );
            }
        }

        if let Some(readiness) = &comp.readiness {
            let members_set: HashSet<&str> = comp.members.iter().map(String::as_str).collect();
            for required in &readiness.require_healthy {
                if !members_set.contains(required.as_str()) {
                    report.fail(
                        &format!("{tag}-RDY"),
                        &format!(
                            "'{}' readiness requires '{required}' which is not a member",
                            comp.name
                        ),
                    );
                }
            }
        }
    }
}

/// Cross-check composition kind against `nucleus-primals` composition constants.
fn validate_composition_kind(comp: &CompositionGraph, report: &mut ManifestReport) {
    let expected = match comp.kind.as_str() {
        "Tower" => Some(nucleus_primals::COMP_TOWER),
        "Nest" => Some(nucleus_primals::COMP_NEST),
        "Node" => Some(nucleus_primals::COMP_NODE),
        _ => None,
    };

    let Some(expected_members) = expected else {
        return;
    };

    let manifest_members: HashSet<&str> = comp.members.iter().map(String::as_str).collect();
    let registry_members: HashSet<&str> = expected_members.iter().copied().collect();

    let missing: Vec<&&str> = registry_members.difference(&manifest_members).collect();
    let extra: Vec<&&str> = manifest_members.difference(&registry_members).collect();

    if missing.is_empty() && extra.is_empty() {
        report.pass(
            &format!("COMP-{}-KIND", comp.kind.to_uppercase()),
            &format!(
                "'{}' members match registry {} definition",
                comp.name, comp.kind
            ),
        );
    } else {
        if !missing.is_empty() {
            report.warn(
                &format!("COMP-{}-MISS", comp.kind.to_uppercase()),
                &format!(
                    "'{}' missing registry {} members: {:?}",
                    comp.name, comp.kind, missing
                ),
            );
        }
        if !extra.is_empty() {
            report.warn(
                &format!("COMP-{}-XTRA", comp.kind.to_uppercase()),
                &format!(
                    "'{}' has extra members not in registry {}: {:?}",
                    comp.name, comp.kind, extra
                ),
            );
        }
    }
}

fn validate_dependencies(manifest: &BiomeManifest, report: &mut ManifestReport) {
    log("");
    log("── Dependencies ──");

    for comp in &manifest.compositions {
        for (primal, deps) in &comp.dependencies {
            if !comp.members.contains(primal) {
                report.fail(
                    "DEP-ORPHAN",
                    &format!(
                        "'{}' dependency key '{primal}' not in composition members",
                        comp.name
                    ),
                );
            }

            for dep in deps {
                if !comp.members.contains(dep) {
                    report.fail(
                        "DEP-MISSING",
                        &format!(
                            "'{}': '{primal}' depends on '{dep}' which is not a member",
                            comp.name
                        ),
                    );
                }
            }
        }

        if has_cycle(&comp.members, &comp.dependencies) {
            report.fail(
                "DEP-CYCLE",
                &format!("'{}' has a dependency cycle", comp.name),
            );
        } else if !comp.dependencies.is_empty() {
            report.pass(
                &format!("DEP-{}", comp.name.to_uppercase().replace('-', "_")),
                &format!("'{}' dependency graph is acyclic", comp.name),
            );
        }
    }
}

/// Simple cycle detection via DFS on the dependency graph.
fn has_cycle(members: &[String], deps: &HashMap<String, Vec<String>>) -> bool {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut in_stack: HashSet<&str> = HashSet::new();

    for member in members {
        if !visited.contains(member.as_str())
            && dfs_cycle(member, deps, &mut visited, &mut in_stack)
        {
            return true;
        }
    }
    false
}

fn dfs_cycle<'a>(
    node: &'a str,
    deps: &'a HashMap<String, Vec<String>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
) -> bool {
    visited.insert(node);
    in_stack.insert(node);

    if let Some(neighbors) = deps.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor.as_str()) {
                if dfs_cycle(neighbor, deps, visited, in_stack) {
                    return true;
                }
            } else if in_stack.contains(neighbor.as_str()) {
                return true;
            }
        }
    }

    in_stack.remove(node);
    false
}

fn validate_federation(manifest: &BiomeManifest, report: &mut ManifestReport) {
    log("");
    log("── Federation ──");

    let Some(fed) = &manifest.federation else {
        report.warn("FED-00", "No federation section declared");
        return;
    };

    if fed.enabled {
        report.pass("FED-01", "Federation enabled");
        if fed.peers.is_empty() {
            report.warn("FED-02", "Federation enabled but no peers declared");
        } else {
            report.pass(
                "FED-02",
                &format!("{} federation peers declared", fed.peers.len()),
            );
        }
    } else {
        report.warn("FED-01", "Federation disabled");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_manifest() -> BiomeManifest {
        BiomeManifest {
            api_version: "v1".into(),
            kind: "Biome".into(),
            metadata: BiomeMetadata {
                name: "test".into(),
                version: "1.0.0".into(),
                description: None,
                tags: vec![],
                labels: HashMap::new(),
            },
            primals: HashMap::new(),
            compositions: vec![],
            federation: None,
        }
    }

    #[test]
    fn parse_minimal_yaml() {
        let yaml = r"
metadata:
  name: test
  version: '1.0.0'
";
        let manifest: BiomeManifest = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(manifest.metadata.name, "test");
        assert_eq!(manifest.api_version, "v1");
        assert_eq!(manifest.kind, "Biome");
    }

    #[test]
    fn parse_with_compositions() {
        let yaml = r"
metadata:
  name: irongate
  version: '157g'
compositions:
  - name: tower-atomic
    kind: Tower
    members: [beardog, songbird, skunkbat]
    dependencies:
      songbird: [beardog]
    priority: 0
";
        let manifest: BiomeManifest = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(manifest.compositions.len(), 1);
        assert_eq!(manifest.compositions[0].members.len(), 3);
        assert_eq!(manifest.compositions[0].kind, "Tower");
    }

    #[test]
    fn cycle_detection_no_cycle() {
        let members = vec!["a".into(), "b".into(), "c".into()];
        let mut deps = HashMap::new();
        deps.insert("b".to_string(), vec!["a".to_string()]);
        deps.insert("c".to_string(), vec!["b".to_string()]);
        assert!(!has_cycle(&members, &deps));
    }

    #[test]
    fn cycle_detection_with_cycle() {
        let members = vec!["a".into(), "b".into(), "c".into()];
        let mut deps = HashMap::new();
        deps.insert("b".to_string(), vec!["a".to_string()]);
        deps.insert("a".to_string(), vec!["c".to_string()]);
        deps.insert("c".to_string(), vec!["b".to_string()]);
        assert!(has_cycle(&members, &deps));
    }

    #[test]
    fn schema_validation_passes_for_minimal() {
        let manifest = minimal_manifest();
        let mut report = ManifestReport::new();
        validate_schema(&manifest, &mut report);
        assert_eq!(report.fail, 0);
        assert!(report.pass >= 4);
    }

    #[test]
    fn primal_validation_catches_unknown_slug() {
        let mut manifest = minimal_manifest();
        manifest.primals.insert(
            "nonexistent_primal".into(),
            ManifestPrimalConfig {
                version: None,
                enabled: true,
                capabilities: vec![],
                dependencies: vec![],
                gossip_events: vec![],
            },
        );
        let mut report = ManifestReport::new();
        validate_primals(&manifest, &mut report);
        assert!(report.fail > 0);
    }

    #[test]
    fn primal_validation_passes_known_slug() {
        let mut manifest = minimal_manifest();
        manifest.primals.insert(
            "beardog".into(),
            ManifestPrimalConfig {
                version: Some("0.9.0".into()),
                enabled: true,
                capabilities: vec![],
                dependencies: vec![],
                gossip_events: vec![],
            },
        );
        let mut report = ManifestReport::new();
        validate_primals(&manifest, &mut report);
        assert_eq!(report.fail, 0);
    }

    #[test]
    fn composition_kind_tower_matches_registry() {
        let comp = CompositionGraph {
            name: "tower".into(),
            kind: "Tower".into(),
            members: vec!["beardog".into(), "songbird".into(), "skunkbat".into()],
            dependencies: HashMap::new(),
            auto_start: true,
            priority: 0,
            readiness: None,
        };
        let mut report = ManifestReport::new();
        validate_composition_kind(&comp, &mut report);
        assert_eq!(report.fail, 0);
        assert!(report.pass > 0);
    }
}
