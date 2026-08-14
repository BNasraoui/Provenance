use camino::Utf8Path;
use provenance_scanner::scan_path;

fn write_source(root: &Utf8Path, relative: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "fn scanned() {}\n").unwrap();
}

fn scanned_relative_paths(root: &Utf8Path) -> Vec<String> {
    scan_path(root)
        .unwrap()
        .into_iter()
        .map(|scan| {
            scan.file_path
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|component| component.as_str())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

#[test]
fn repository_scan_excludes_dependency_build_and_metadata_trees() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    write_source(root, "src/application.rs");
    write_source(root, "node_modules/package/dependency.rs");
    write_source(root, "target/debug/generated.rs");
    write_source(root, ".git/hooks/metadata.rs");

    assert_eq!(scanned_relative_paths(root), ["src/application.rs"]);
}

#[test]
fn repository_scan_excludes_nested_dependency_boundaries() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    write_source(root, "packages/web/src/application.ts");
    write_source(root, "packages/web/node_modules/package/dependency.ts");
    write_source(root, "packages/api/target/generated.rs");

    assert_eq!(
        scanned_relative_paths(root),
        ["packages/web/src/application.ts"]
    );
}

#[test]
fn similarly_named_directories_remain_scannable() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    write_source(root, "node_modules_cache/cached.js");
    write_source(root, "targeting/domain.rs");
    write_source(root, ".github/workflows/check.ts");

    assert_eq!(
        scanned_relative_paths(root),
        [
            ".github/workflows/check.ts",
            "node_modules_cache/cached.js",
            "targeting/domain.rs",
        ]
    );
}

#[test]
fn explicitly_selected_excluded_directory_remains_scannable() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let dependency_root = root.join("node_modules");
    let build_root = root.join("target");
    write_source(root, "node_modules/package/dependency.js");
    write_source(root, "target/debug/generated.rs");

    assert_eq!(
        scanned_relative_paths(&dependency_root),
        ["package/dependency.js"]
    );
    assert_eq!(scanned_relative_paths(&build_root), ["debug/generated.rs"]);
}
