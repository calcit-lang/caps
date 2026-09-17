use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "calcit-caps-contract-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create caps contract test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!(
                "failed to remove caps contract test directory {}: {error}",
                self.0.display()
            );
        }
    }
}

fn run_caps(args: &[&str], modules_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caps"))
        .args(args)
        .env("CALCIT_MODULES_DIR", modules_dir)
        .output()
        .expect("run caps")
}

fn snapshot_files(paths: &[PathBuf]) -> Vec<(PathBuf, Vec<u8>)> {
    paths
        .iter()
        .map(|path| {
            (
                path.clone(),
                fs::read(path).unwrap_or_else(|error| {
                    panic!("failed to snapshot {}: {error}", path.display())
                }),
            )
        })
        .collect()
}

fn assert_files_unchanged(snapshots: &[(PathBuf, Vec<u8>)]) {
    for (path, expected) in snapshots {
        assert_eq!(
            fs::read(path)
                .unwrap_or_else(|error| panic!("failed to re-read {}: {error}", path.display())),
            *expected,
            "workspace planning modified {}",
            path.display()
        );
    }
}

#[test]
fn top_level_help_keeps_the_public_command_surface() {
    let test_dir = TestDir::new("help");
    let output = run_caps(&["--help"], &test_dir.path().join("modules"));
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help output");
    for command in [
        "outdated", "upgrade", "download", "add", "remove", "tree", "why", "version", "status",
        "verify", "reset", "clean",
    ] {
        assert!(
            stdout
                .lines()
                .any(|line| line.trim_start().starts_with(command)),
            "missing command `{command}` in:\n{stdout}"
        );
    }
}

#[test]
fn version_flag_reports_caps_instead_of_calcit() {
    let test_dir = TestDir::new("caps-version");
    let output = run_caps(&["--version"], &test_dir.path().join("modules"));
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 version output");
    assert_eq!(stdout, format!("caps {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_get_reads_the_explicit_deps_file_without_mutation() {
    let test_dir = TestDir::new("version-get");
    let deps_file = test_dir.path().join("deps.cirru");
    let source = "{} (:version |1.2.3) (:dependencies $ {})\n";
    fs::write(&deps_file, source).expect("write deps.cirru");

    let output = run_caps(
        &[
            deps_file.to_str().expect("UTF-8 temporary path"),
            "version",
            "get",
        ],
        &test_dir.path().join("modules"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 version output"),
        "1.2.3\n"
    );
    assert_eq!(
        fs::read_to_string(deps_file).expect("read unchanged deps.cirru"),
        source
    );
}

#[test]
fn missing_explicit_deps_file_is_a_failure() {
    let test_dir = TestDir::new("missing-input");
    let deps_file = test_dir.path().join("missing.cirru");
    let output = run_caps(
        &[
            deps_file.to_str().expect("UTF-8 temporary path"),
            "version",
            "get",
        ],
        &test_dir.path().join("modules"),
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 missing-file error")
            .contains(&format!("Error: no {} found!", deps_file.display()))
    );
}

#[test]
fn workspace_tree_plans_stable_layers_without_mutation() {
    let test_dir = TestDir::new("workspace-plan");
    let workspace = test_dir.path().join("workspace.cirru");
    let projects = [
        (
            "base",
            "{} (:version |1.1.0) (:calcit-version |0.15.8) (:dependencies $ {})\n",
        ),
        (
            "middle",
            "{} (:version |2.0.0) (:calcit-version |0.15.8) (:dependencies $ {} (|org/base |1.1.0))\n",
        ),
        (
            "released-consumer",
            "{} (:version |3.0.0) (:calcit-version |0.15.8) (:dependencies $ {} (|org/base |v1.0.0))\n",
        ),
        (
            "app",
            "{} (:version |4.0.0) (:calcit-version |0.15.7) (:dependencies $ {} (|org/middle |main))\n",
        ),
        (
            "source",
            "{} (:version |5.0.0) (:calcit-version |0.15.7) (:dependencies $ {})\n",
        ),
        (
            "tool",
            "{} (:version |6.0.0) (:calcit-version |0.15.7) (:dependencies $ {})\n",
        ),
        (
            "archived",
            "{} (:version |7.0.0) (:calcit-version |0.15.8) (:dependencies $ {} (|org/base |main))\n",
        ),
        (
            "excluded",
            "{} (:version |8.0.0) (:calcit-version |0.15.8) (:dependencies $ {})\n",
        ),
        (
            "blocked-consumer",
            "{} (:version |9.0.0) (:calcit-version |0.15.8) (:dependencies $ {} (|org/archived |7.0.0) (|org/excluded |8.0.0))\n",
        ),
    ];
    let mut input_paths = Vec::new();
    for (name, deps) in projects {
        let path = test_dir.path().join(name);
        fs::create_dir_all(&path).expect("create fixture project");
        let deps_path = path.join("deps.cirru");
        fs::write(&deps_path, deps).expect("write fixture deps.cirru");
        input_paths.push(deps_path);
    }
    fs::write(
        &workspace,
        r#"{}
  :schema-version |1
  :target-calcit |0.15.8
  :projects $ []
    {} (:repository |org/base) (:path |base) (:latest-release |v1.1.0)
    {} (:repository |org/middle) (:path |middle) (:latest-release |2.0.0)
    {} (:repository |org/released-consumer) (:path |released-consumer) (:latest-release |3.0.0)
    {} (:repository |org/app) (:path |app) (:latest-release |4.0.0) (:protected true)
    {} (:repository |org/source) (:path |source) (:latest-release |5.0.0) (:source-migration true)
    {} (:repository |org/tool) (:path |tool) (:latest-release |6.0.0)
    {} (:repository |org/archived) (:path |archived) (:latest-release |7.0.0) (:state :archived)
    {} (:repository |org/excluded) (:path |excluded) (:latest-release |8.0.0) (:state :excluded)
    {} (:repository |org/blocked-consumer) (:path |blocked-consumer) (:latest-release |9.0.0)
"#,
    )
    .expect("write workspace inventory");
    input_paths.push(workspace.clone());
    let modules_dir = test_dir.path().join("must-not-be-created");
    let json_inputs = snapshot_files(&input_paths);
    let output = run_caps(
        &[
            "tree",
            "--workspace",
            workspace.to_str().expect("UTF-8 temporary path"),
            "--format",
            "json",
        ],
        &modules_dir,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON plan");
    assert_eq!(
        plan["layers"],
        serde_json::json!([
            ["org/base", "org/blocked-consumer", "org/source", "org/tool"],
            ["org/middle", "org/released-consumer"],
            ["org/app"]
        ])
    );
    assert_eq!(plan["publish-first"], serde_json::json!(["org/middle"]));
    let projects = plan["projects"].as_array().expect("project list");
    let released_consumer = projects
        .iter()
        .find(|project| project["repository"] == "org/released-consumer")
        .expect("released dependency consumer");
    assert_eq!(released_consumer["classification"], "released-dependency");
    let source = projects
        .iter()
        .find(|project| project["repository"] == "org/source")
        .expect("source migration project");
    assert_eq!(source["classification"], "source-migration");
    let tool = projects
        .iter()
        .find(|project| project["repository"] == "org/tool")
        .expect("toolchain-only project");
    assert_eq!(tool["classification"], "toolchain-only");
    let app = projects
        .iter()
        .find(|project| project["repository"] == "org/app")
        .expect("app project");
    assert_eq!(app["protected"], true);
    assert_eq!(
        app["blocked-by"],
        serde_json::json!([{"repository": "org/middle", "reason": "unpublished-ref"}])
    );
    let blocked_consumer = projects
        .iter()
        .find(|project| project["repository"] == "org/blocked-consumer")
        .expect("blocked consumer");
    assert_eq!(
        blocked_consumer["blocked-by"],
        serde_json::json!([
            {"repository": "org/archived", "reason": "archived"},
            {"repository": "org/excluded", "reason": "excluded"}
        ])
    );
    assert!(
        !modules_dir.exists(),
        "workspace planning must stay read-only"
    );
    assert_files_unchanged(&json_inputs);

    let cirru_inputs = snapshot_files(&input_paths);
    let cirru_output = run_caps(
        &[
            "tree",
            "--workspace",
            workspace.to_str().expect("UTF-8 temporary path"),
        ],
        &modules_dir,
    );
    assert!(cirru_output.status.success());
    let cirru = String::from_utf8(cirru_output.stdout).expect("UTF-8 Cirru EDN plan");
    let parsed = cirru_edn::parse(&cirru).expect("parse default Cirru EDN output");
    let plan = parsed.view_map().expect("plan map");
    assert_eq!(plan.get_or_nil("schema-version"), cirru_edn::Edn::str("1"));
    assert_eq!(
        plan.get_or_nil("layers"),
        cirru_edn::parse(
            "[] ([] |org/base |org/blocked-consumer |org/source |org/tool) ([] |org/middle |org/released-consumer) ([] |org/app)"
        )
        .expect("expected Cirru layers")
    );
    assert_eq!(
        plan.get_or_nil("publish-first"),
        cirru_edn::parse("[] |org/middle").expect("expected Cirru publish-first")
    );
    let app = plan
        .get_or_nil("projects")
        .view_list()
        .expect("Cirru project list")
        .0
        .into_iter()
        .find(|project| {
            project.view_map().is_ok_and(|project| {
                project.get_or_nil("repository") == cirru_edn::Edn::str("org/app")
            })
        })
        .expect("Cirru app project")
        .view_map()
        .expect("Cirru app map");
    assert_eq!(
        app.get_or_nil("blocked-by"),
        cirru_edn::parse("[] $ {} (:reason |unpublished-ref) (:repository |org/middle)")
            .expect("expected Cirru blocker")
    );
    assert!(!modules_dir.exists(), "Cirru planning must stay read-only");
    assert_files_unchanged(&cirru_inputs);
}

#[test]
fn workspace_tree_rejects_invalid_inventory_without_touching_cache() {
    let test_dir = TestDir::new("workspace-invalid");
    let workspace = test_dir.path().join("workspace.cirru");
    fs::write(
        &workspace,
        "{} (:schema-version |2) (:target-calcit |0.15.8) (:projects $ [])\n",
    )
    .expect("write invalid workspace inventory");
    let modules_dir = test_dir.path().join("must-not-be-created");
    let output = run_caps(
        &[
            "tree",
            "--workspace",
            workspace.to_str().expect("UTF-8 temporary path"),
        ],
        &modules_dir,
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 inventory error")
            .contains("unsupported workspace inventory :schema-version 2")
    );
    assert!(!modules_dir.exists());
}

#[test]
fn tree_format_requires_workspace_mode() {
    let test_dir = TestDir::new("tree-format-scope");
    let modules_dir = test_dir.path().join("must-not-be-created");
    let output = run_caps(&["tree", "--format", "json"], &modules_dir);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("UTF-8 option error")
            .contains("tree --format requires --workspace")
    );
    assert!(!modules_dir.exists());
}

#[test]
fn workspace_tree_rejects_malformed_project_versions_and_dependencies() {
    let test_dir = TestDir::new("workspace-project-validation");
    let project_dir = test_dir.path().join("project");
    fs::create_dir_all(&project_dir).expect("create fixture project");
    let deps_file = project_dir.join("deps.cirru");
    let workspace = test_dir.path().join("workspace.cirru");
    fs::write(
        &workspace,
        "{} (:schema-version |1) (:target-calcit |0.15.8) (:projects $ [] ({} (:repository |org/project) (:path |project) (:latest-release |1.0.0)))\n",
    )
    .expect("write workspace inventory");
    let modules_dir = test_dir.path().join("must-not-be-created");
    for (deps, expected) in [
        (
            "{} (:version |broken) (:calcit-version |0.15.8) (:dependencies $ {})\n",
            ":version for org/project",
        ),
        (
            "{} (:version |1.0.0) (:calcit-version |broken) (:dependencies $ {})\n",
            ":calcit-version for org/project",
        ),
        (
            "{} (:version |1.0.0) (:calcit-version |0.15.8) (:dependencies $ {} (|org/../outside |1.0.0))\n",
            "invalid dependency org/../outside",
        ),
    ] {
        fs::write(&deps_file, deps).expect("write malformed deps.cirru");
        let output = run_caps(
            &[
                "tree",
                "--workspace",
                workspace.to_str().expect("UTF-8 temporary path"),
            ],
            &modules_dir,
        );
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 validation error");
        assert!(
            stderr.contains(expected) && stderr.contains(deps_file.to_str().unwrap()),
            "expected `{expected}` and deps path in:\n{stderr}"
        );
    }
    assert!(!modules_dir.exists());
}
