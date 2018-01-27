use std::collections::BTreeMap;

use hyper::Response;
use hyper::header::ContentType;
use maud::{Markup, html};

use ::engine::AnalyzeDependenciesOutcome;
use ::models::crates::{CrateName, AnalyzedDependency};
use ::models::repo::RepoPath;

const SELF_BASE_URL: &'static str = "https://shiny-robots.herokuapp.com";

fn dependency_table(title: &str, deps: BTreeMap<CrateName, AnalyzedDependency>) -> Markup {
    let count_total = deps.len();
    let count_outdated = deps.iter().filter(|&(_, dep)| dep.is_outdated()).count();

    html! {
        h3 {
            (title)
            span class="summary" {
                @if count_outdated > 0 {
                    (format!(" ({} total, {} up-to-date, {} outdated)", count_total, count_total - count_outdated, count_outdated))
                } @else {
                    (format!(" ({} total, all up-to-date)", count_total))
                }
            }
        }

        table {
            tr {
                th "Crate"
                th "Required"
                th "Latest"
                th "Status"
            }
            @for (name, dep) in deps {
                tr {
                    td {
                        a href=(format!("https://crates.io/crates/{}", name.as_ref())) (name.as_ref())
                    }
                    td code (dep.required.to_string())
                    td {
                        @if let Some(ref latest) = dep.latest {
                            code (latest.to_string())
                        } @else {
                            "N/A"
                        }
                    }
                    td {
                        @if dep.is_outdated() {
                            span class="status outdated" "out of date"
                        } @else {
                            span class="status up-to-date" "up to date"
                        }
                    }
                }
            }
        }
    }
}

pub fn status_html(analysis_outcome: AnalyzeDependenciesOutcome, repo_path: RepoPath) -> Response {
    let self_path = format!("repo/{}/{}/{}", repo_path.site.as_ref(), repo_path.qual.as_ref(), repo_path.name.as_ref());
    let status_base_url = format!("{}/{}", SELF_BASE_URL, self_path);
    let title = format!("{} / {} - Dependency Status", repo_path.qual.as_ref(), repo_path.name.as_ref());

    let rendered = html! {
        html {
            head {
                title (title)
                link rel="stylesheet" type="text/css" href="/static/style.css";
            }
            body {
                header {
                    h1 {
                        "Dependency status for "
                        a href=(format!("{}/{}/{}", repo_path.site.to_base_uri(), repo_path.qual.as_ref(), repo_path.name.as_ref())) {
                            code (format!("{}/{}", repo_path.qual.as_ref(), repo_path.name.as_ref()))
                        }
                    }

                    img src=(format!("/{}/status.svg", self_path));

                    pre {
                        (format!("[![dependency status]({}/status.svg)]({})", status_base_url, status_base_url))
                    }

                    h2 {
                        "Crate "
                        code (analysis_outcome.name.as_ref())
                    }

                    @if !analysis_outcome.deps.main.is_empty() {
                        (dependency_table("Dependencies", analysis_outcome.deps.main))
                    }

                    @if !analysis_outcome.deps.dev.is_empty() {
                        (dependency_table("Dev dependencies", analysis_outcome.deps.dev))
                    }

                    @if !analysis_outcome.deps.build.is_empty() {
                        (dependency_table("Build dependencies", analysis_outcome.deps.build))
                    }
                }
            }
        }
    };

    Response::new()
        .with_header(ContentType::html())
        .with_body(rendered.0)
}