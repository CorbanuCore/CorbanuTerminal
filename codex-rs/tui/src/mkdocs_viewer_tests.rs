use super::*;
use pretty_assertions::assert_eq;

#[test]
fn loads_index_from_parent_mkdocs_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("docs/guide")).expect("docs dirs");
    fs::create_dir_all(root.join("src/deep")).expect("cwd dirs");
    fs::write(
        root.join("mkdocs.yml"),
        r#"
site_name: Terminal Docs
docs_dir: docs
nav:
  - Home: index.md
  - Install: guide/install.md
"#,
    )
    .expect("mkdocs config");
    fs::write(root.join("docs/index.md"), "# Home\n\nWelcome.").expect("index");
    fs::write(root.join("docs/guide/install.md"), "# Install\n\nRun it.").expect("install");

    let site = load_mkdocs_site(&root.join("src/deep"), /*args*/ None).expect("site");

    assert_eq!(site.title, "Terminal Docs");
    assert_eq!(site.project_root, root);
    assert_eq!(site.docs_dir, root.join("docs"));
    assert_eq!(
        site.pages[site.selected_index].abs_path,
        root.join("docs/index.md")
    );
    assert!(
        site.pages
            .iter()
            .position(|page| page.rel_path == Path::new("index.md"))
            .expect("index listed")
            < site
                .pages
                .iter()
                .position(|page| page.rel_path == Path::new("guide/install.md"))
                .expect("install listed")
    );
    assert_eq!(
        site.read_page_source(site.selected_index).expect("source"),
        "# Home\n\nWelcome."
    );
}

#[test]
fn resolves_page_hint_by_suffix() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("docs/reference")).expect("docs dirs");
    fs::write(root.join("mkdocs.yml"), "site_name: Docs\n").expect("mkdocs config");
    fs::write(root.join("docs/index.md"), "# Home").expect("index");
    fs::write(root.join("docs/reference/api.md"), "# API").expect("api");

    let site = load_mkdocs_site(root, Some("api.md")).expect("site");

    assert_eq!(
        site.pages[site.selected_index].abs_path,
        root.join("docs/reference/api.md")
    );
    assert_eq!(
        site.pages[site.selected_index].rel_path,
        Path::new("reference/api.md")
    );
    assert_eq!(
        site.read_page_source(site.selected_index).expect("source"),
        "# API"
    );
}

#[test]
fn can_open_explicit_repo_path_and_page_hint() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let repo = temp.path().join("repo");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::create_dir_all(repo.join("docs/reference")).expect("docs dirs");
    fs::write(repo.join("mkdocs.yml"), "site_name: Repo Docs\n").expect("mkdocs config");
    fs::write(repo.join("docs/index.md"), "# Home").expect("index");
    fs::write(repo.join("docs/reference/exec.md"), "# Exec").expect("exec");

    let args = format!("{} exec.md", repo.display());
    let site = load_mkdocs_site(&workspace, Some(&args)).expect("site");

    assert_eq!(site.title, "Repo Docs");
    assert_eq!(site.project_root, repo);
    assert_eq!(
        site.pages[site.selected_index].rel_path,
        Path::new("reference/exec.md")
    );
}

#[test]
fn can_open_explicit_docs_dir_without_mkdocs_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let docs = temp.path().join("loose-docs");
    fs::create_dir_all(&docs).expect("docs dir");
    fs::write(docs.join("index.md"), "# Loose").expect("index");
    fs::write(docs.join("exec.md"), "# Exec").expect("exec");

    let args = format!("--docs-dir {} exec.md", docs.display());
    let site = load_mkdocs_site(temp.path(), Some(&args)).expect("site");

    assert_eq!(site.docs_dir, docs);
    assert_eq!(
        site.pages[site.selected_index].rel_path,
        Path::new("exec.md")
    );
}

#[test]
fn rejects_docs_dir_outside_project_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&root).expect("root");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(root.join("mkdocs.yml"), "docs_dir: ../outside\n").expect("mkdocs config");

    let error = load_mkdocs_site(&root, /*args*/ None).expect_err("error");

    assert!(
        error
            .to_string()
            .contains("docs_dir must stay inside the project root")
    );
}

#[test]
fn searches_known_page_content_without_leaving_docs_tree() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("docs/reference")).expect("docs dirs");
    fs::write(root.join("mkdocs.yml"), "site_name: Docs\n").expect("mkdocs config");
    fs::write(root.join("docs/index.md"), "# Home\n\nOrdinary text.").expect("index");
    fs::write(
        root.join("docs/reference/api.md"),
        "# API\n\nUnique provider credential guidance.",
    )
    .expect("api");

    let site = load_mkdocs_site(root, /*args*/ None).expect("site");
    let api_index = site
        .pages
        .iter()
        .position(|page| page.rel_path == Path::new("reference/api.md"))
        .expect("api page");

    assert!(site.page_matches_query(api_index, "provider credential"));
    assert!(!site.page_matches_query(site.selected_index, "provider credential"));
}

#[test]
fn resolves_relative_site_absolute_directory_and_anchor_links() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("docs/guide/setup")).expect("docs dirs");
    fs::create_dir_all(root.join("docs/reference")).expect("reference dirs");
    fs::write(root.join("mkdocs.yml"), "site_name: Docs\n").expect("mkdocs config");
    fs::write(root.join("docs/index.md"), "# Home").expect("index");
    fs::write(root.join("docs/guide/setup/index.md"), "# Setup").expect("setup");
    fs::write(root.join("docs/reference/api.md"), "# API").expect("api");

    let site = load_mkdocs_site(root, Some("guide/setup/index.md")).expect("site");
    let from = site.selected_index;
    let api = site
        .pages
        .iter()
        .position(|page| page.rel_path == Path::new("reference/api.md"))
        .expect("api page");

    assert_eq!(
        site.resolve_internal_link(from, "../../reference/api.md#methods")
            .expect("relative link"),
        ResolvedDocLink {
            page_index: api,
            anchor: Some("methods".to_string()),
        }
    );
    assert_eq!(
        site.resolve_internal_link(from, "/guide/setup/")
            .expect("directory link")
            .page_index,
        from
    );
    assert_eq!(
        site.resolve_internal_link(from, "#install")
            .expect("same page anchor"),
        ResolvedDocLink {
            page_index: from,
            anchor: Some("install".to_string()),
        }
    );
}

#[test]
fn rejects_external_broken_and_escaping_links_with_actionable_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("docs")).expect("docs dir");
    fs::write(root.join("mkdocs.yml"), "site_name: Docs\n").expect("mkdocs config");
    fs::write(root.join("docs/index.md"), "# Home").expect("index");
    let site = load_mkdocs_site(root, /*args*/ None).expect("site");

    assert!(
        site.resolve_internal_link(site.selected_index, "https://example.com")
            .expect_err("external error")
            .to_string()
            .contains("terminal/browser policy")
    );
    assert!(
        site.resolve_internal_link(site.selected_index, "../secret.md")
            .expect_err("escape error")
            .to_string()
            .contains("escapes the configured docs directory")
    );
    assert!(
        site.resolve_internal_link(site.selected_index, "missing.md")
            .expect_err("missing error")
            .to_string()
            .contains("No MkDocs page matched link")
    );
}

#[test]
fn bare_docs_falls_back_to_managed_package_offline_docs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let package = temp.path().join("package");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::create_dir_all(package.join("docs")).expect("package docs");
    fs::write(
        package.join("mkdocs.yml"),
        "site_name: Packaged PF Terminal\ndocs_dir: docs\n",
    )
    .expect("mkdocs config");
    fs::write(package.join("docs/index.md"), "# Offline Home").expect("index");

    let (config, page_hint) =
        resolve_mkdocs_request_with_package_root(&workspace, /*args*/ None, Some(&package))
            .expect("packaged config");

    assert_eq!(config.path, package.join("mkdocs.yml"));
    assert_eq!(config.docs_dir, package.join("docs"));
    assert_eq!(page_hint, None);
}

#[test]
fn targeted_docs_fall_back_to_managed_package_offline_docs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let package = temp.path().join("package");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::create_dir_all(package.join("docs/features")).expect("package docs");
    fs::write(
        package.join("mkdocs.yml"),
        "site_name: Packaged PF Terminal\ndocs_dir: docs\n",
    )
    .expect("mkdocs config");
    fs::write(
        package.join("docs/features/spawn.md"),
        "# Spawn Orchestration",
    )
    .expect("target page");

    let (config, page_hint) = resolve_mkdocs_request_with_package_root(
        &workspace,
        Some("features/spawn"),
        Some(&package),
    )
    .expect("packaged target config");

    assert_eq!(config.path, package.join("mkdocs.yml"));
    assert_eq!(config.docs_dir, package.join("docs"));
    assert_eq!(page_hint.as_deref(), Some("features/spawn"));
}

#[test]
fn docs_path_options_report_a_missing_path() {
    let temp = tempfile::tempdir().expect("tempdir");

    for option in ["--config", "--docs-dir"] {
        let error = resolve_mkdocs_request_with_package_root(
            temp.path(),
            Some(option),
            /*managed_package_root*/ None,
        )
        .expect_err("missing option path");
        assert_eq!(
            error.to_string(),
            format!("Expected a path after /docs {option}.")
        );
    }
}

#[test]
fn missing_local_and_packaged_docs_returns_recovery_instruction() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace = temp.path().join("workspace");
    let package = temp.path().join("broken-package");
    fs::create_dir_all(&workspace).expect("workspace");
    fs::create_dir_all(&package).expect("package");

    let error =
        resolve_mkdocs_request_with_package_root(&workspace, /*args*/ None, Some(&package))
            .expect_err("missing docs error");

    assert!(
        error
            .to_string()
            .contains("packaged documentation is missing")
    );
    assert!(error.to_string().contains("Reinstall Corbanu Terminal"));
    assert!(error.to_string().contains("/docs --config <path>"));
}
