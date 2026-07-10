use std::{fs, path::Path};

const RUST_FILE_LINE_LIMIT: usize = 500;

#[test]
fn cli_definition_files_stay_within_the_rust_file_line_limit() {
    let cli_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli");
    let mut files = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs")];

    collect_rust_files(&cli_root, &mut files);

    for path in files {
        let source = fs::read_to_string(&path).expect("read CLI definition file");
        let line_count = source.lines().count();
        assert!(
            line_count <= RUST_FILE_LINE_LIMIT,
            "{} has {line_count} lines; the limit is {RUST_FILE_LINE_LIMIT}",
            path.display()
        );
    }
}

fn collect_rust_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    if !directory.exists() {
        return;
    }

    for entry in fs::read_dir(directory).expect("read CLI module directory") {
        let path = entry.expect("read CLI module entry").path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
