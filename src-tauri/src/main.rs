// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use askama::Template;
use serde_yaml::{Mapping, Value};
use tauri::{Manager, Window};
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

#[tauri::command]
async fn template(chart: String, values: String) -> String {
    let name = "helmad";
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
#[template(path = "local_chart.html")]
struct LocalChartTemplate {
    path: String,
    resources: Vec<K8sResourceTemplate>,
}

#[tauri::command]
async fn chart_selection(repo: String) -> String {
    let charts = ChartsTemplate {
        charts: helm_cli::search_repo(&repo, None).unwrap(),
    };
    return charts.render().unwrap();
}

#[tauri::command]
async fn remote_chart() -> String {
    let repos = helm_cli::repo_list().unwrap();
    let selected_repo = repos[0].clone();
    let charts = helm_cli::search_repo(&selected_repo, None).unwrap();
    let charts = ReposTemplate {
        repos,
        selected_repo: &selected_repo,
        charts: ChartsTemplate { charts },
    };
    return charts.render().unwrap();
}

#[tauri::command]
async fn local_chart(path: String) -> String {
    let k8s_resources = helm_cli::template("helmad", &path, Mapping::new());
    let resources = format_templates(k8s_resources.unwrap());
    let charts = LocalChartTemplate { path, resources };
    return charts.render().unwrap();
}

#[tauri::command]
async fn close_splashscreen(window: Window) {
    // Close splashscreen
    window
        .get_window("splashscreen")
        .expect("no window labeled 'splashscreen' found")
        .close()
        .unwrap();
    // Show main window
    window
        .get_window("main")
        .expect("no window labeled 'main' found")
        .show()
        .unwrap();
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
        ])
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
