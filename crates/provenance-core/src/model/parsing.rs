pub(super) fn normalize_enum_value(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_lowercase()
}
