use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use serde_yaml::{Mapping, Sequence, Value};
use std::error;
use std::fmt;
use tempfile::NamedTempFile;

const HELM: &str = "helm";

#[derive(Debug, Clone)]
struct HelmError;

impl fmt::Display for HelmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something went wrong when executing helm command")
    }
}

impl error::Error for HelmError {}

pub fn pull(chart: &str, location: &Path) -> Result<(), Box<dyn error::Error>> {
    let res = Command::new(HELM)
        .args(["pull", chart, "--untar", "--destination", location.to_str().ok_or(HelmError{})?])
        .output()?;
    if !res.status.success() {
        return Err(Box::new(HelmError{}));
    }
    return Ok(());
}

pub fn template(
    name: &str,
    chart: &str,
    values: Mapping,
) -> Result<Vec<Mapping>, Box<dyn error::Error>> {
    let values_file = NamedTempFile::new()?;
    serde_yaml::to_writer(&values_file, &values)?;
    let res = Command::new(HELM)
        .args([
            "template",
            name,
            chart,
            "-f",
            values_file.path().to_str().unwrap(),
        ])
        .output()?;
    let stdout = String::from_utf8(res.stdout)?;
    let mut output: Vec<Mapping> = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(&stdout) {
        let map = Mapping::deserialize(doc)?;
        output.push(map);
    }
    return Ok(output);
}

pub fn repo_list() -> Result<Vec<String>, Box<dyn error::Error>> {
    let res = Command::new(HELM)
        .args(["repo", "list", "-o", "yaml"])
        .output()?;
    let de = serde_yaml::Deserializer::from_slice(&res.stdout[..]);
    let repos = Sequence::deserialize(de)?;
    let mut output = repos
        .iter()
        .map(|i| {
            i.get("name")
                .unwrap_or(&Value::String("unknown".to_owned()))
                .as_str()
                .unwrap_or("unknown")
                .to_owned()
        })
        .collect::<Vec<String>>();
    output.sort();
    return Ok(output);
}

#[derive(Debug, Deserialize)]
pub struct Chart {
    pub name: String,
    pub version: String,
    pub app_version: String,
    pub description: String,
}

pub fn search_repo(repo: &str, search: Option<&str>) -> Result<Vec<Chart>, Box<dyn error::Error>> {
    let mut cmd = &mut Command::new(HELM);
    cmd = cmd.args(["search", "repo", repo, "-o", "yaml"]);
    if search.is_some() {
        cmd = cmd.arg(search.unwrap());
    }
    let res = cmd.output()?;
    let mut output: Vec<Chart> = serde_yaml::from_slice(&res.stdout[..])?;
    output.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
    return Ok(output);
}
