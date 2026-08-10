use provenance_core::coverage::{
    AnchorState, AnnotationResult, BindingResult, CoverageScan, ValidationWarning,
};
use std::collections::BTreeSet;

pub(super) fn reconcile(
    current: &mut CoverageScan,
    baseline: &CoverageScan,
    repo: &camino::Utf8Path,
    scan_path: &camino::Utf8Path,
    validate_rules: bool,
) {
    let paths = ScanPaths::new(repo, scan_path);
    current.report.annotations = reconcile_annotations(
        &baseline.report.annotations,
        &current.report.annotations,
        &paths,
    );
    current.report.bindings =
        reconcile_bindings(&baseline.report.bindings, &current.report.bindings, &paths);
    current.report.total_annotations = current
        .report
        .annotations
        .iter()
        .filter(|site| site.anchor_state != AnchorState::Gone)
        .count();
    if validate_rules {
        current.report.warnings.extend(gone_site_warnings(current));
    }
}

struct ScanPaths<'a> {
    repo: &'a camino::Utf8Path,
    canonical_repo: Option<camino::Utf8PathBuf>,
    scan_path: &'a camino::Utf8Path,
}

impl<'a> ScanPaths<'a> {
    fn new(repo: &'a camino::Utf8Path, scan_path: &'a camino::Utf8Path) -> Self {
        let canonical_repo = std::fs::canonicalize(repo)
            .ok()
            .and_then(|path| camino::Utf8PathBuf::from_path_buf(path).ok());
        Self {
            repo,
            canonical_repo,
            scan_path,
        }
    }

    fn relative<'p>(&self, path: &'p camino::Utf8Path) -> &'p camino::Utf8Path {
        if let Ok(relative) = path.strip_prefix(self.repo) {
            return relative;
        }
        if let Some(relative) = self
            .canonical_repo
            .as_deref()
            .and_then(|repo| path.strip_prefix(repo).ok())
        {
            return relative;
        }
        path.strip_prefix(".").unwrap_or(path)
    }

    fn contains(&self, path: &camino::Utf8Path) -> bool {
        self.normalized(path)
            .starts_with(self.normalized(self.scan_path))
    }

    fn same_file(&self, left: &camino::Utf8Path, right: &camino::Utf8Path) -> bool {
        self.normalized(left) == self.normalized(right)
    }

    fn normalized(&self, path: &camino::Utf8Path) -> camino::Utf8PathBuf {
        let canonical = std::fs::canonicalize(path)
            .ok()
            .and_then(|path| camino::Utf8PathBuf::from_path_buf(path).ok());
        lexical_normalize(self.relative(canonical.as_deref().unwrap_or(path)))
    }
}

fn lexical_normalize(path: &camino::Utf8Path) -> camino::Utf8PathBuf {
    let mut normalized = camino::Utf8PathBuf::new();
    for component in path.components() {
        match component {
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            _ => normalized.push(component.as_str()),
        }
    }
    normalized
}

fn reconcile_annotations(
    baseline: &[AnnotationResult],
    current: &[AnnotationResult],
    paths: &ScanPaths<'_>,
) -> Vec<AnnotationResult> {
    let mut used = BTreeSet::new();
    let mut by_previous = vec![None; baseline.len()];
    for (previous_index, previous) in baseline.iter().enumerate() {
        if previous.anchor.is_none() || !paths.contains(&previous.file_path) {
            continue;
        }
        let previous_count = baseline
            .iter()
            .filter(|site| {
                site.anchor.is_some()
                    && paths.contains(&site.file_path)
                    && annotations_match(previous, site, paths)
            })
            .count();
        let current_count = current
            .iter()
            .filter(|site| annotations_match(previous, site, paths))
            .count();
        if current_count < previous_count {
            continue;
        }
        if let Some((current_index, found)) = current.iter().enumerate().find(|(index, site)| {
            !used.contains(index)
                && site.line == previous.line
                && annotations_match(previous, site, paths)
        }) {
            used.insert(current_index);
            by_previous[previous_index] = Some(unchanged_annotation(found));
        }
    }
    for (previous_index, previous) in baseline.iter().enumerate() {
        if by_previous[previous_index].is_some()
            || previous.anchor.is_none()
            || !paths.contains(&previous.file_path)
        {
            continue;
        }
        let candidates = current
            .iter()
            .enumerate()
            .filter(|(index, site)| {
                !used.contains(index) && annotations_match(previous, site, paths)
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let mut gone = previous.clone();
            gone.anchor_state = AnchorState::Gone;
            gone.original_line = None;
            by_previous[previous_index] = Some(gone);
            continue;
        }
        let unresolved_duplicates = baseline
            .iter()
            .enumerate()
            .filter(|(index, site)| {
                by_previous[*index].is_none()
                    && site.anchor.is_some()
                    && paths.contains(&site.file_path)
                    && annotations_match(site, previous, paths)
            })
            .count();
        if candidates.len() == 1 && unresolved_duplicates == 1 {
            let (current_index, found) = candidates[0];
            used.insert(current_index);
            let mut moved = found.clone();
            moved.anchor_state = AnchorState::Moved;
            moved.original_line = Some(previous.line);
            by_previous[previous_index] = Some(moved);
        }
    }
    let mut resolved = by_previous.into_iter().flatten().collect::<Vec<_>>();
    resolved.extend(
        current
            .iter()
            .enumerate()
            .filter(|(index, _)| !used.contains(index))
            .map(|(_, site)| unchanged_annotation(site)),
    );
    resolved
}

fn annotations_match(
    left: &AnnotationResult,
    right: &AnnotationResult,
    paths: &ScanPaths<'_>,
) -> bool {
    paths.same_file(&left.file_path, &right.file_path)
        && left.rule_id == right.rule_id
        && left.anchor == right.anchor
}

fn unchanged_annotation(site: &AnnotationResult) -> AnnotationResult {
    let mut unchanged = site.clone();
    unchanged.anchor_state = AnchorState::Unchanged;
    unchanged.original_line = None;
    unchanged
}

fn reconcile_bindings(
    baseline: &[BindingResult],
    current: &[BindingResult],
    paths: &ScanPaths<'_>,
) -> Vec<BindingResult> {
    let mut used = BTreeSet::new();
    let mut by_previous = vec![None; baseline.len()];
    for (previous_index, previous) in baseline.iter().enumerate() {
        if previous.anchor.is_none() || !paths.contains(&previous.file_path) {
            continue;
        }
        let previous_count = baseline
            .iter()
            .filter(|site| {
                site.anchor.is_some()
                    && paths.contains(&site.file_path)
                    && bindings_match(previous, site, paths)
            })
            .count();
        let current_count = current
            .iter()
            .filter(|site| bindings_match(previous, site, paths))
            .count();
        if current_count < previous_count {
            continue;
        }
        if let Some((current_index, found)) = current.iter().enumerate().find(|(index, site)| {
            !used.contains(index)
                && site.line == previous.line
                && bindings_match(previous, site, paths)
        }) {
            used.insert(current_index);
            by_previous[previous_index] = Some(unchanged_binding(found));
        }
    }
    for (previous_index, previous) in baseline.iter().enumerate() {
        if by_previous[previous_index].is_some()
            || previous.anchor.is_none()
            || !paths.contains(&previous.file_path)
        {
            continue;
        }
        let candidates = current
            .iter()
            .enumerate()
            .filter(|(index, site)| !used.contains(index) && bindings_match(previous, site, paths))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let mut gone = previous.clone();
            gone.anchor_state = AnchorState::Gone;
            gone.original_line = None;
            by_previous[previous_index] = Some(gone);
            continue;
        }
        let unresolved_duplicates = baseline
            .iter()
            .enumerate()
            .filter(|(index, site)| {
                by_previous[*index].is_none()
                    && site.anchor.is_some()
                    && paths.contains(&site.file_path)
                    && bindings_match(site, previous, paths)
            })
            .count();
        if candidates.len() == 1 && unresolved_duplicates == 1 {
            let (current_index, found) = candidates[0];
            used.insert(current_index);
            let mut moved = found.clone();
            moved.anchor_state = AnchorState::Moved;
            moved.original_line = Some(previous.line);
            by_previous[previous_index] = Some(moved);
        }
    }
    let mut resolved = by_previous.into_iter().flatten().collect::<Vec<_>>();
    resolved.extend(
        current
            .iter()
            .enumerate()
            .filter(|(index, _)| !used.contains(index))
            .map(|(_, site)| unchanged_binding(site)),
    );
    resolved
}

fn bindings_match(left: &BindingResult, right: &BindingResult, paths: &ScanPaths<'_>) -> bool {
    paths.same_file(&left.file_path, &right.file_path)
        && left.rule_id == right.rule_id
        && left.anchor == right.anchor
}

fn unchanged_binding(site: &BindingResult) -> BindingResult {
    let mut unchanged = site.clone();
    unchanged.anchor_state = AnchorState::Unchanged;
    unchanged.original_line = None;
    unchanged
}

fn gone_site_warnings(report: &CoverageScan) -> Vec<ValidationWarning> {
    report
        .annotations
        .iter()
        .filter(|site| site.anchor_state == AnchorState::Gone)
        .map(|site| warning(&site.rule_id, &site.file_path, site.line, "annotation"))
        .chain(
            report
                .bindings
                .iter()
                .filter(|site| site.anchor_state == AnchorState::Gone)
                .map(|site| warning(&site.rule_id, &site.file_path, site.line, "binding")),
        )
        .collect()
}

fn warning(
    rule_id: &str,
    file_path: &camino::Utf8Path,
    line: usize,
    kind: &str,
) -> ValidationWarning {
    ValidationWarning {
        rule_id: rule_id.to_string(),
        file_path: Some(file_path.to_path_buf()),
        line: Some(line),
        message: format!("anchored {kind} site is gone"),
    }
}
