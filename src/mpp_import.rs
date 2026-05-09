use chrono::NaiveDate;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::{ProjectSnapshot, TaskSnapshot};

const MPXJ_ZIP_URL: &str =
    "https://downloads.sourceforge.net/project/mpxj/mpxj/Version%2016.1.0/mpxj-16.1.0.zip";
const ECJ_JAR_URL: &str =
    "https://repo1.maven.org/maven2/org/eclipse/jdt/core/compiler/ecj/4.6.1/ecj-4.6.1.jar";

pub fn load_mpp(path: impl AsRef<Path>) -> Result<ProjectSnapshot, String> {
    let bridge = MppBridge::prepare()?;
    bridge.import(path.as_ref())
}

struct MppBridge {
    classes_dir: PathBuf,
    mpxj_jar: PathBuf,
    dependency_jars: Vec<PathBuf>,
}

impl MppBridge {
    fn prepare() -> Result<Self, String> {
        let cache_dir = std::env::temp_dir().join("rustyproject_mpp_bridge");
        let classes_dir = cache_dir.join("classes");
        let mpxj_dir = cache_dir.join("mpxj");
        let ecj_jar = cache_dir.join("ecj-4.6.1.jar");
        let mpxj_zip = cache_dir.join("mpxj-16.1.0.zip");
        let mpxj_root = mpxj_dir.join("mpxj");
        let mpxj_jar = mpxj_root.join("mpxj.jar");
        let lib_dir = mpxj_root.join("lib");
        let bridge_class = classes_dir.join("com/projectlibre/mppbridge/MppImporter.class");
        let bridge_source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("java_mpp_bridge/src/com/projectlibre/mppbridge/MppImporter.java");

        if bridge_needs_rebuild(&bridge_class, &bridge_source) {
            ensure_dir(&cache_dir)?;
            ensure_dir(&classes_dir)?;

            if !ecj_jar.exists() {
                download_file(ECJ_JAR_URL, &ecj_jar)?;
            }

            if !mpxj_jar.exists() {
                if !mpxj_zip.exists() {
                    download_file(MPXJ_ZIP_URL, &mpxj_zip)?;
                }
                extract_zip(&mpxj_zip, &mpxj_dir)?;
            }

            let dependency_jars = collect_jars(&lib_dir)?;

            if !bridge_source.exists() {
                return Err(format!(
                    "Missing helper source at {}",
                    bridge_source.display()
                ));
            }

            compile_bridge(&ecj_jar, &mpxj_jar, &bridge_source, &classes_dir)?;
            return Ok(Self {
                classes_dir,
                mpxj_jar,
                dependency_jars,
            });
        }

        Ok(Self {
            classes_dir,
            mpxj_jar,
            dependency_jars: collect_jars(&mpxj_dir.join("mpxj").join("lib"))?,
        })
    }

    fn import(&self, path: &Path) -> Result<ProjectSnapshot, String> {
        let output = Command::new("java")
            .arg("-cp")
            .arg(classpath(
                &self.classes_dir,
                &self.mpxj_jar,
                &self.dependency_jars,
            ))
            .arg("com.projectlibre.mppbridge.MppImporter")
            .arg(path)
            .output()
            .map_err(|err| format!("Failed to start MPP importer: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                "MPP importer failed".to_string()
            } else {
                stderr
            });
        }

        let payload = String::from_utf8(output.stdout)
            .map_err(|err| format!("Invalid MPP importer UTF-8 output: {err}"))?;
        let payload = payload
            .find('{')
            .map(|start| payload[start..].to_string())
            .ok_or_else(|| "MPP importer output did not contain JSON".to_string())?;
        let document: MppDocument =
            serde_json::from_str(&payload).map_err(|err| format!("Parse MPP JSON: {err}"))?;
        Ok(document.into_snapshot())
    }
}

fn bridge_needs_rebuild(class_file: &Path, source_file: &Path) -> bool {
    if !class_file.exists() {
        return true;
    }

    let class_modified = class_file.metadata().and_then(|meta| meta.modified()).ok();
    let source_modified = source_file.metadata().and_then(|meta| meta.modified()).ok();
    match (class_modified, source_modified) {
        (Some(class_modified), Some(source_modified)) => source_modified > class_modified,
        _ => true,
    }
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|err| format!("Create dir {}: {err}", path.display()))
}

fn download_file(url: &str, destination: &Path) -> Result<(), String> {
    let status = Command::new("curl.exe")
        .arg("-L")
        .arg("-o")
        .arg(destination)
        .arg(url)
        .status()
        .map_err(|err| format!("Download failed to launch: {err}"))?;
    if !status.success() {
        return Err(format!("Download failed: {url}"));
    }
    Ok(())
}

fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(), String> {
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!(
            "Expand-Archive -Force '{}' '{}'",
            zip_path.display(),
            destination.display()
        ))
        .status()
        .map_err(|err| format!("Extract failed to launch: {err}"))?;
    if !status.success() {
        return Err(format!("Extract failed: {}", zip_path.display()));
    }
    Ok(())
}

fn compile_bridge(
    ecj_jar: &Path,
    mpxj_jar: &Path,
    source: &Path,
    classes_dir: &Path,
) -> Result<(), String> {
    let status = Command::new("java")
        .arg("-jar")
        .arg(ecj_jar)
        .arg("-1.8")
        .arg("-classpath")
        .arg(mpxj_jar)
        .arg("-d")
        .arg(classes_dir)
        .arg(source)
        .status()
        .map_err(|err| format!("Compile failed to launch: {err}"))?;
    if !status.success() {
        return Err("MPP bridge compilation failed".to_string());
    }
    Ok(())
}

fn collect_jars(lib_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut jars = Vec::new();
    if !lib_dir.exists() {
        return Ok(jars);
    }

    let entries = std::fs::read_dir(lib_dir)
        .map_err(|err| format!("Read dir {}: {err}", lib_dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("Read dir entry {}: {err}", lib_dir.display()))?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("jar"))
            .unwrap_or(false)
        {
            jars.push(path);
        }
    }
    jars.sort();
    Ok(jars)
}

fn classpath(classes_dir: &Path, mpxj_jar: &Path, jars: &[PathBuf]) -> String {
    let mut parts = vec![
        classes_dir.display().to_string(),
        mpxj_jar.display().to_string(),
    ];
    parts.extend(jars.iter().map(|path| path.display().to_string()));
    parts.join(";")
}

#[derive(Deserialize)]
struct MppDocument {
    start_date: String,
    end_date: String,
    tasks: Vec<MppTask>,
}

#[derive(Deserialize)]
struct MppTask {
    id: usize,
    name: String,
    start: String,
    finish: String,
    progress: f32,
    indent: usize,
    summary: bool,
    milestone: bool,
    predecessors: Vec<usize>,
    #[serde(default)]
    resource_names: Vec<String>,
    #[serde(default)]
    start_text: Option<String>,
    #[serde(default)]
    finish_text: Option<String>,
    #[serde(default)]
    duration_text: Option<String>,
}

impl MppDocument {
    fn into_snapshot(self) -> ProjectSnapshot {
        let tasks = self
            .tasks
            .into_iter()
            .map(|task| TaskSnapshot {
                number: task.id,
                name: task.name,
                start: parse_date(&task.start),
                finish: parse_date(&task.finish),
                progress: task.progress,
                indent: task.indent,
                summary: task.summary,
                milestone: task.milestone,
                predecessors: task.predecessors,
                resource_names: task.resource_names,
                start_text: task.start_text,
                finish_text: task.finish_text,
                duration_text: task.duration_text,
            })
            .collect();

        ProjectSnapshot {
            start_date: parse_date(&self.start_date),
            end_date: parse_date(&self.end_date),
            tasks,
        }
    }
}

fn parse_date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("helper emits valid dates")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_sample_mpp_from_env_path() {
        let Ok(path) = std::env::var("RUSTYPROJECT_SAMPLE_MPP") else {
            eprintln!("Skipping sample MPP import test: RUSTYPROJECT_SAMPLE_MPP is not set");
            return;
        };

        let snapshot = load_mpp(path).expect("sample MPP should import");
        assert!(
            !snapshot.tasks.is_empty(),
            "sample MPP should contain tasks"
        );
        assert!(snapshot.start_date <= snapshot.end_date);
        assert!(
            snapshot
                .tasks
                .iter()
                .any(|task| !task.name.trim().is_empty()),
            "sample MPP should contain named tasks"
        );
    }
}
