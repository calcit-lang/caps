use crate::PackageDeps;
use cirru_edn::{Edn, EdnListView, EdnMapView};
use semver::Version;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectState {
    Active,
    Archived,
    Excluded,
}

impl ProjectState {
    fn parse(value: Edn, repository: &str) -> Result<Self, String> {
        let raw = match value {
            Edn::Nil => "active".to_owned(),
            Edn::Tag(tag) => tag.ref_str().to_owned(),
            Edn::Str(text) => text.to_string(),
            other => {
                return Err(format!(
                    "workspace project {repository} :state must be a tag or string, got {other}"
                ));
            }
        };
        match raw.as_str() {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            "excluded" => Ok(Self::Excluded),
            _ => Err(format!(
                "workspace project {repository} has unsupported :state {raw}; expected :active, :archived, or :excluded"
            )),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Excluded => "excluded",
        }
    }
}

#[derive(Debug, Clone)]
struct ProjectSpec {
    path: String,
    deps_file: String,
    current_ref: Option<String>,
    latest_release: Option<String>,
    state: ProjectState,
    protected: bool,
    source_migration: bool,
    release_required: bool,
}

#[derive(Debug, Clone)]
struct WorkspaceInventory {
    target_calcit: String,
    projects: BTreeMap<String, ProjectSpec>,
}

#[derive(Debug, Clone)]
struct LoadedProject {
    spec: ProjectSpec,
    package_version: Option<String>,
    calcit_version: Option<String>,
    dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Blocker {
    repository: String,
    reason: String,
}

pub fn run_workspace_plan(inventory_path: &str, format: &str) -> Result<(), String> {
    let inventory_path = Path::new(inventory_path);
    let inventory = read_inventory(inventory_path)?;
    let projects = load_projects(&inventory, inventory_path)?;
    let plan = build_plan(&inventory, &projects);
    match format {
        "cirru" => println!("{}", cirru_edn::format(&json_to_edn(&plan)?, false)?),
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&plan).map_err(|e| e.to_string())?
        ),
        other => {
            return Err(format!(
                "unsupported workspace output format {other}; expected cirru or json"
            ));
        }
    }
    Ok(())
}

fn read_inventory(path: &Path) -> Result<WorkspaceInventory, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read workspace inventory {}: {e}", path.display()))?;
    let parsed = cirru_edn::parse(&content).map_err(|e| {
        format!(
            "failed to parse workspace inventory {}: {e}",
            path.display()
        )
    })?;
    let root = parsed
        .view_map()
        .map_err(|e| format!("workspace inventory {} must be a map: {e}", path.display()))?;
    let schema_version = required_string(&root, "schema-version", "workspace inventory")?;
    if schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported workspace inventory :schema-version {schema_version}; expected {SCHEMA_VERSION}"
        ));
    }
    let target_calcit = required_string(&root, "target-calcit", "workspace inventory")?;
    Version::parse(&target_calcit)
        .map_err(|e| format!("workspace :target-calcit must be exact SemVer: {e}"))?;
    let entries = root
        .get_or_nil("projects")
        .view_list()
        .map_err(|e| format!("workspace inventory :projects must be a list: {e}"))?;
    if entries.0.is_empty() {
        return Err("workspace inventory :projects must not be empty".to_owned());
    }
    let mut projects = BTreeMap::new();
    for entry in entries.0 {
        let map = entry
            .view_map()
            .map_err(|e| format!("workspace project must be a map: {e}"))?;
        let repository = required_string(&map, "repository", "workspace project")?;
        validate_repository(&repository)?;
        let latest_release = optional_string(&map, "latest-release", &repository)?;
        if let Some(release) = &latest_release {
            Version::parse(release).map_err(|e| {
                format!("workspace project {repository} :latest-release must be exact SemVer: {e}")
            })?;
        }
        let spec = ProjectSpec {
            path: required_string(&map, "path", &format!("workspace project {repository}"))?,
            deps_file: optional_string(&map, "deps-file", &repository)?
                .unwrap_or_else(|| "deps.cirru".to_owned()),
            current_ref: optional_string(&map, "current-ref", &repository)?,
            latest_release,
            state: ProjectState::parse(map.get_or_nil("state"), &repository)?,
            protected: optional_bool(&map, "protected", &repository)?.unwrap_or(false),
            source_migration: optional_bool(&map, "source-migration", &repository)?
                .unwrap_or(false),
            release_required: optional_bool(&map, "release-required", &repository)?
                .unwrap_or(false),
        };
        if projects.insert(repository.clone(), spec).is_some() {
            return Err(format!("duplicate workspace project {repository}"));
        }
    }
    Ok(WorkspaceInventory {
        target_calcit,
        projects,
    })
}

fn load_projects(
    inventory: &WorkspaceInventory,
    inventory_path: &Path,
) -> Result<BTreeMap<String, LoadedProject>, String> {
    let base = inventory_path.parent().unwrap_or_else(|| Path::new("."));
    let mut loaded = BTreeMap::new();
    for (repository, spec) in &inventory.projects {
        let project_path = resolve_inventory_path(base, &spec.path);
        let deps_path = project_path.join(&spec.deps_file);
        let content = fs::read_to_string(&deps_path).map_err(|e| {
            format!(
                "failed to read deps.cirru for {repository} at {}: {e}",
                deps_path.display()
            )
        })?;
        let parsed = cirru_edn::parse(&content).map_err(|e| {
            format!(
                "failed to parse {} for {repository}: {e}",
                deps_path.display()
            )
        })?;
        let deps: PackageDeps = parsed
            .try_into()
            .map_err(|e| format!("invalid {} for {repository}: {e}", deps_path.display()))?;
        let dependencies = deps
            .root_dependencies()?
            .into_iter()
            .map(|(name, reference)| (name.to_string(), reference.to_string()))
            .collect();
        loaded.insert(
            repository.clone(),
            LoadedProject {
                spec: spec.clone(),
                package_version: deps.version,
                calcit_version: deps.calcit_version,
                dependencies,
            },
        );
    }
    Ok(loaded)
}

fn build_plan(inventory: &WorkspaceInventory, projects: &BTreeMap<String, LoadedProject>) -> Value {
    let active = projects
        .iter()
        .filter(|(_, project)| project.spec.state == ProjectState::Active)
        .map(|(repository, _)| repository.clone())
        .collect::<BTreeSet<_>>();
    let edges = active
        .iter()
        .map(|repository| {
            let dependencies = projects[repository]
                .dependencies
                .keys()
                .filter(|dependency| active.contains(*dependency))
                .cloned()
                .collect::<BTreeSet<_>>();
            (repository.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    let cycles = find_cycles(&edges);
    let cycle_nodes = cycles.iter().flatten().cloned().collect::<BTreeSet<_>>();
    let cycle_affected = cycle_affected_nodes(&edges, &cycle_nodes);
    let layers = topological_layers(&edges, &cycle_affected);

    let mut required_by_branch = BTreeSet::new();
    for project in projects.values() {
        for (dependency, reference) in &project.dependencies {
            if active.contains(dependency) && Version::parse(reference).is_err() {
                required_by_branch.insert(dependency.clone());
            }
        }
    }

    let mut project_values = Vec::new();
    let mut release_required = BTreeSet::new();
    for (repository, project) in projects {
        let release_state = release_state(project, required_by_branch.contains(repository));
        if release_state == "release-required" && project.spec.state == ProjectState::Active {
            release_required.insert(repository.clone());
        }
        let blockers = blockers_for(project, projects, &cycle_affected, &required_by_branch);
        let classification =
            classification_for(project, projects, &inventory.target_calcit, release_state);
        let dependencies = project
            .dependencies
            .iter()
            .map(|(name, reference)| (name.clone(), Value::String(reference.clone())))
            .collect::<Map<_, _>>();
        let blocked_by = blockers
            .into_iter()
            .map(|blocker| {
                json!({
                    "repository": blocker.repository,
                    "reason": blocker.reason,
                })
            })
            .collect::<Vec<_>>();
        project_values.push(json!({
            "repository": repository,
            "path": project.spec.path,
            "deps-file": project.spec.deps_file,
            "state": project.spec.state.as_str(),
            "protected": project.spec.protected,
            "current-ref": project.spec.current_ref.as_deref().or(project.package_version.as_deref()),
            "package-version": project.package_version,
            "latest-release": project.spec.latest_release,
            "calcit-version": project.calcit_version,
            "release-state": release_state,
            "classification": classification,
            "dependencies": dependencies,
            "blocked-by": blocked_by,
        }));
    }
    let publish_first = layers
        .iter()
        .flatten()
        .filter(|repository| release_required.contains(*repository))
        .cloned()
        .collect::<Vec<_>>();

    let external_dependencies = projects
        .values()
        .flat_map(|project| project.dependencies.keys())
        .filter(|dependency| !projects.contains_key(*dependency))
        .cloned()
        .collect::<BTreeSet<_>>();

    json!({
        "schema-version": SCHEMA_VERSION,
        "target-calcit": inventory.target_calcit,
        "projects": project_values,
        "layers": layers,
        "publish-first": publish_first,
        "cycles": cycles,
        "external-dependencies": external_dependencies,
    })
}

fn release_state(project: &LoadedProject, required_by_branch: bool) -> &'static str {
    if project.spec.release_required || required_by_branch {
        return "release-required";
    }
    let Some(latest) = &project.spec.latest_release else {
        return "missing-release";
    };
    match (&project.package_version, Version::parse(latest)) {
        (Some(current), Ok(latest)) => match Version::parse(current) {
            Ok(current) if current > latest => "release-required",
            _ => "released",
        },
        _ => "released",
    }
}

fn classification_for(
    project: &LoadedProject,
    projects: &BTreeMap<String, LoadedProject>,
    target_calcit: &str,
    release_state: &str,
) -> &'static str {
    if project.spec.state != ProjectState::Active {
        return "not-planned";
    }
    if project.spec.source_migration {
        return "source-migration";
    }
    if release_state == "release-required" {
        return "release-required";
    }
    if project.dependencies.iter().any(|(dependency, reference)| {
        projects
            .get(dependency)
            .and_then(|dependency| dependency.spec.latest_release.as_ref())
            .is_some_and(|latest| is_older_release(reference, latest))
    }) {
        return "released-dependency";
    }
    if project.calcit_version.as_deref() != Some(target_calcit) {
        return "toolchain-only";
    }
    "current"
}

fn blockers_for(
    project: &LoadedProject,
    projects: &BTreeMap<String, LoadedProject>,
    cycle_affected: &BTreeSet<String>,
    required_by_branch: &BTreeSet<String>,
) -> BTreeSet<Blocker> {
    let mut blockers = BTreeSet::new();
    for (dependency, reference) in &project.dependencies {
        let Some(target) = projects.get(dependency) else {
            continue;
        };
        let reason = if target.spec.state == ProjectState::Archived {
            Some("archived")
        } else if target.spec.state == ProjectState::Excluded {
            Some("excluded")
        } else if cycle_affected.contains(dependency) {
            Some("dependency-cycle")
        } else if Version::parse(reference).is_err() {
            Some("unpublished-ref")
        } else if target.spec.latest_release.is_none() {
            Some("missing-release")
        } else if release_state(target, required_by_branch.contains(dependency))
            == "release-required"
        {
            Some("release-required")
        } else {
            None
        };
        if let Some(reason) = reason {
            blockers.insert(Blocker {
                repository: dependency.clone(),
                reason: reason.to_owned(),
            });
        }
    }
    blockers
}

fn is_older_release(reference: &str, latest: &str) -> bool {
    match (Version::parse(reference), Version::parse(latest)) {
        (Ok(reference), Ok(latest)) => reference < latest,
        _ => false,
    }
}

fn find_cycles(edges: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    fn visit(
        node: &str,
        edges: &BTreeMap<String, BTreeSet<String>>,
        visited: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) {
        if !visited.insert(node.to_owned()) {
            return;
        }
        if let Some(dependencies) = edges.get(node) {
            for dependency in dependencies {
                visit(dependency, edges, visited, order);
            }
        }
        order.push(node.to_owned());
    }

    let mut order = Vec::new();
    let mut visited = BTreeSet::new();
    for node in edges.keys() {
        visit(node, edges, &mut visited, &mut order);
    }
    let mut reverse = edges
        .keys()
        .map(|node| (node.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (node, dependencies) in edges {
        for dependency in dependencies {
            reverse
                .entry(dependency.clone())
                .or_default()
                .insert(node.clone());
        }
    }
    let mut components = Vec::new();
    visited.clear();
    while let Some(node) = order.pop() {
        if visited.contains(&node) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            component.push(current.clone());
            if let Some(next) = reverse.get(&current) {
                stack.extend(next.iter().rev().cloned());
            }
        }
        component.sort();
        let self_cycle = component.len() == 1 && edges[&component[0]].contains(&component[0]);
        if component.len() > 1 || self_cycle {
            components.push(component);
        }
    }
    components.sort();
    components
}

fn cycle_affected_nodes(
    edges: &BTreeMap<String, BTreeSet<String>>,
    cycle_nodes: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut unsafe_nodes = cycle_nodes.clone();
    loop {
        let before = unsafe_nodes.len();
        for (node, dependencies) in edges {
            if dependencies
                .iter()
                .any(|dependency| unsafe_nodes.contains(dependency))
            {
                unsafe_nodes.insert(node.clone());
            }
        }
        if unsafe_nodes.len() == before {
            break;
        }
    }
    unsafe_nodes
}

fn topological_layers(
    edges: &BTreeMap<String, BTreeSet<String>>,
    unsafe_nodes: &BTreeSet<String>,
) -> Vec<Vec<String>> {
    let mut remaining = edges
        .keys()
        .filter(|node| !unsafe_nodes.contains(*node))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut completed = BTreeSet::new();
    let mut layers = Vec::new();
    while !remaining.is_empty() {
        let layer = remaining
            .iter()
            .filter(|node| {
                edges[*node]
                    .iter()
                    .filter(|dependency| !unsafe_nodes.contains(*dependency))
                    .all(|dependency| completed.contains(dependency))
            })
            .cloned()
            .collect::<Vec<_>>();
        if layer.is_empty() {
            break;
        }
        for node in &layer {
            remaining.remove(node);
            completed.insert(node.clone());
        }
        layers.push(layer);
    }
    layers
}

fn required_string(map: &EdnMapView, key: &str, context: &str) -> Result<String, String> {
    optional_string(map, key, context)?.ok_or_else(|| format!("{context} requires :{key}"))
}

fn optional_string(map: &EdnMapView, key: &str, context: &str) -> Result<Option<String>, String> {
    match map.get_or_nil(key) {
        Edn::Nil => Ok(None),
        Edn::Str(value) => Ok(Some(value.to_string())),
        other => Err(format!("{context} :{key} must be a string, got {other}")),
    }
}

fn optional_bool(map: &EdnMapView, key: &str, context: &str) -> Result<Option<bool>, String> {
    match map.get_or_nil(key) {
        Edn::Nil => Ok(None),
        Edn::Bool(value) => Ok(Some(value)),
        other => Err(format!(
            "workspace project {context} :{key} must be a bool, got {other}"
        )),
    }
}

fn validate_repository(repository: &str) -> Result<(), String> {
    let Some((owner, name)) = repository.split_once('/') else {
        return Err(format!(
            "workspace :repository must use owner/repo form: {repository}"
        ));
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return Err(format!(
            "workspace :repository must use owner/repo form: {repository}"
        ));
    }
    Ok(())
}

fn resolve_inventory_path(base: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    }
}

fn json_to_edn(value: &Value) -> Result<Edn, String> {
    Ok(match value {
        Value::Null => Edn::Nil,
        Value::Bool(value) => Edn::Bool(*value),
        Value::Number(value) => {
            Edn::Number(value.as_f64().ok_or_else(|| {
                format!("JSON number cannot be represented in Cirru EDN: {value}")
            })?)
        }
        Value::String(value) => Edn::str(value.as_str()),
        Value::Array(values) => Edn::List(EdnListView(
            values
                .iter()
                .map(json_to_edn)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Object(values) => {
            let mut map = EdnMapView::default();
            for (key, value) in values {
                map.insert(Edn::tag(key.as_str()), json_to_edn(value)?);
            }
            Edn::Map(map)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_are_reported_without_linearizing_dependents() {
        let edges = BTreeMap::from([
            ("a".to_owned(), BTreeSet::from(["b".to_owned()])),
            ("b".to_owned(), BTreeSet::from(["a".to_owned()])),
            ("c".to_owned(), BTreeSet::from(["a".to_owned()])),
            ("d".to_owned(), BTreeSet::new()),
        ]);
        let cycles = find_cycles(&edges);
        assert_eq!(cycles, vec![vec!["a".to_owned(), "b".to_owned()]]);
        let cycle_nodes = cycles.into_iter().flatten().collect();
        let cycle_affected = cycle_affected_nodes(&edges, &cycle_nodes);
        assert_eq!(
            cycle_affected,
            BTreeSet::from(["a".to_owned(), "b".to_owned(), "c".to_owned()])
        );
        assert_eq!(
            topological_layers(&edges, &cycle_affected),
            vec![vec!["d".to_owned()]]
        );
    }
}
