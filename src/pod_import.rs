use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::ProjectSnapshot;
use crate::mpp_import::load_mpp;

const POD_SEPARATOR: &[u8] = b"@@@@@@@@@@ProjectLibreSeparator_MSXML@@@@@@@@@@";

pub fn load_xml(path: impl AsRef<Path>) -> Result<ProjectSnapshot, String> {
    load_mpp(path)
}

pub fn load_pod(path: impl AsRef<Path>) -> Result<ProjectSnapshot, String> {
    let pod_path = path.as_ref();
    let xml_payload = extract_embedded_xml(pod_path)?;
    let temp_path = unique_temp_xml_path(pod_path);

    fs::write(&temp_path, &xml_payload)
        .map_err(|err| format!("write temp XML {}: {err}", temp_path.display()))?;

    let result = load_xml(&temp_path);
    let _ = fs::remove_file(&temp_path);
    result
}

fn extract_embedded_xml(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|err| format!("read file {}: {err}", path.display()))?;
    let Some(start) = find_subsequence(&bytes, POD_SEPARATOR) else {
        return Err(format!("POD separator not found in {}", path.display()));
    };

    let xml_start = start + POD_SEPARATOR.len();
    if xml_start >= bytes.len() {
        return Err(format!(
            "POD file {} does not contain embedded XML",
            path.display()
        ));
    }

    Ok(bytes[xml_start..].to_vec())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn unique_temp_xml_path(source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = process::id();
    std::env::temp_dir().join(format!("{stem}_{pid}_{stamp}.xml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_embedded_xml_payload() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("sample data/Commercial construction project plan.pod");
        let xml = extract_embedded_xml(&path).expect("embedded XML should exist");
        let xml_text = String::from_utf8(xml).expect("embedded XML should be UTF-8");

        assert!(xml_text.starts_with("<?xml"));
        assert!(xml_text.contains("<Project xmlns=\"http://schemas.microsoft.com/project\">"));
    }

    #[test]
    fn imports_sample_xml_from_repo_path() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("sample data/Commercial construction project plan.xml");
        let snapshot = load_xml(&path).expect("sample XML should import");

        assert!(!snapshot.tasks.is_empty());
        assert!(snapshot.start_date <= snapshot.end_date);
    }
}
