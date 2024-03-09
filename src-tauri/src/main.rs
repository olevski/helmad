// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use askama::Template;
use serde_yaml::{Mapping, Value};
use core::time;
use std::thread::sleep;
use std::{fs::File, io::Read};
use std::path::Path;
use tauri::{Manager, Window};
use tempfile::TempDir;
use walkdir::WalkDir;
mod helm_cli;

#[derive(Template)]
#[template(path = "k8s_resource.html")]
struct K8sResourceTemplate {
    kind: String,
    name: String,
    contents: Mapping,
}

#[derive(Template)]
#[template(path = "template_output.html")]
struct TemplateOutput {
    resources: Vec<K8sResourceTemplate>,
}

fn format_templates(input: Vec<Mapping>) -> Vec<K8sResourceTemplate> {
    let mut templates: Vec<K8sResourceTemplate> = input
        .iter()
        .map(|x| K8sResourceTemplate {
            kind: x
                .get("kind")
                .unwrap_or(&Value::String("Unknown".to_string()))
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            name: x
                .get("metadata")
                .unwrap_or(&Value::Mapping(Mapping::new()))
                .get("name")
                .unwrap_or(&Value::String("Unknown".to_string()))
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            contents: x.clone(),
        })
        .collect();
    templates.sort_by(|a, b| a.kind.partial_cmp(&b.kind).unwrap());
    return templates;
}

fn format_helm_templates(location: &Path) -> Vec<HelmTemplate> {
    let template_loc = location.join("templates");
    let all_files = WalkDir::new(template_loc)
        .follow_links(true)
        .into_iter()
        .map(|entry| entry.unwrap());
    let helm_templates: Vec<HelmTemplate> = all_files
        .filter(|entry| {
            let ftype = entry.file_type();
            let fname = entry.file_name().to_string_lossy();
            return ftype.is_file() && (fname.ends_with("yaml") || fname.ends_with("yml"));
        })
        .map(|entry| {
            let mut f = File::open(entry.path()).unwrap();
            let mut contents: String = String::new(); 
            f.read_to_string(&mut contents).unwrap();
            return HelmTemplate{ file_name: entry.file_name().to_str().unwrap().to_owned(), contents};
        }).collect();
    return helm_templates;
}

#[tauri::command]
async fn template(chart: String, name: String, values: String) -> String {
    let values_mapping: Mapping = serde_yaml::from_str(&values).unwrap();
    let raw_templates = helm_cli::template(&name, &chart, values_mapping).unwrap();
    let templates = format_templates(raw_templates);
    return TemplateOutput {
        resources: templates,
    }
    .render()
    .unwrap();
}

#[tauri::command]
fn repo_selection() -> String {
    let repos = helm_cli::repo_list().unwrap();
    #[derive(Template)]
    #[template(path = "repos.html")]
    struct ReposTemplate {
        repos: Vec<String>,
    }
    let repos = ReposTemplate { repos };
    return repos.render().unwrap();
}

#[derive(Template)]
#[template(path = "charts.html")]
struct ChartsTemplate {
    charts: Vec<helm_cli::Chart>,
}

#[derive(Template)]
#[template(path = "remote_chart.html")]
struct ReposTemplate<'a> {
    repos: Vec<String>,
    selected_repo: &'a str,
    charts: ChartsTemplate,
}

#[derive(Template)]
#[template(path = "chart.html")]
struct ChartTemplate {
    chart: String,
    name: String,
    local: bool,
    values: String,
    resources: Vec<K8sResourceTemplate>,
    templates: Vec<HelmTemplate>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

#[derive(Template)]
#[template(path = "helm_template.html")]
struct HelmTemplate {
    file_name: String,
    contents: String,
}

#[tauri::command]
async fn chart_selection(repo: String) -> String {
    let charts = ChartsTemplate {
        charts: helm_cli::search_repo(&repo, None).unwrap(),
    };
    return charts.render().unwrap();
}

#[tauri::command]
async fn remote_chart(chart: String, name: String, values: String) -> String {
    let is_local = false;
    let tmp_dir = TempDir::new().unwrap();
    let chart_name = chart.split("/").last().unwrap();
    helm_cli::pull(&chart, tmp_dir.path()).unwrap();
    // println!("{:?}", tmp_dir.path());
    // sleep(time::Duration::new(300, 0));
    let templates = format_helm_templates(tmp_dir.path().join(chart_name).as_path());
    let values_mapping: Mapping = serde_yaml::from_str(&values).unwrap();
    let k8s_resources = helm_cli::template(&name, &chart, values_mapping);
    let resources = format_templates(k8s_resources.unwrap());
    let charts = ChartTemplate {
        chart,
        resources,
        templates,
        name,
        values,
        local: is_local,
    };
    return charts.render().unwrap();
}

#[tauri::command]
async fn local_chart(chart: String, name: String, values: String, local: String) -> String {
    let is_local = local == "true";
    let values_mapping: Mapping = serde_yaml::from_str(&values).unwrap();
    let k8s_resources = helm_cli::template(&name, &chart, values_mapping);
    let resources = format_templates(k8s_resources.unwrap());
    let path = Path::new(&chart);
    let templates = format_helm_templates(path);
    let charts = ChartTemplate {
        chart,
        resources,
        templates,
        name,
        values,
        local: is_local,
    };
    return charts.render().unwrap();
}

#[tauri::command]
async fn close_splashscreen(window: Window) {
    // Close splashscreen
    let splash = window.get_window("splashscreen");
    if splash.is_some() {
        splash.unwrap().close().unwrap();
    }
    // Show main window
    window
        .get_window("main")
        .expect("no window labeled 'main' found")
        .show()
        .unwrap();
}

#[tauri::command]
async fn home() -> String {
    let home = HomeTemplate {};
    return home.render().unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            template,
            repo_selection,
            chart_selection,
            remote_chart,
            local_chart,
            close_splashscreen,
            home,
        ])
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
