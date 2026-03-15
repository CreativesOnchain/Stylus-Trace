//! Viewer generation and browser orchestration.

use crate::parser::schema::Profile;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const HTML_TEMPLATE: &str = include_str!("viewer/index.html");
const CSS_TEMPLATE: &str = include_str!("viewer/viewer.css");
const JS_TEMPLATE: &str = include_str!("viewer/viewer.js");

/// Generate a self-contained HTML viewer for a profile
pub fn generate_viewer(profile: &Profile, output_path: &Path) -> Result<()> {
    let profile_json = serde_json::to_string(profile)?;

    // In a real implementation with more time, we'd use a proper template engine or
    // at least a better replacement strategy. For this "best effort" we'll do simple replacement.
    let mut html = HTML_TEMPLATE.to_string();

    // Inject data
    html = html.replace("/* PROFILE_DATA_JSON */", &profile_json);

    // Inline CSS and JS for "self-contained" requirement
    html = html.replace(
        "<link rel=\"stylesheet\" href=\"viewer.css\">",
        &format!("<style>{}</style>", CSS_TEMPLATE),
    );
    html = html.replace(
        "<script src=\"viewer.js\"></script>",
        &format!("<script>{}</script>", JS_TEMPLATE),
    );

    fs::write(output_path, html).context("Failed to write viewer HTML")?;

    Ok(())
}

/// Generate a self-contained HTML viewer for a diff
pub fn generate_diff_viewer(
    profile_a: &Profile,
    profile_b: &Profile,
    diff_report: &serde_json::Value,
    output_path: &Path,
) -> Result<()> {
    let profile_a_json = serde_json::to_string(profile_a)?;
    let profile_b_json = serde_json::to_string(profile_b)?;
    let diff_json = serde_json::to_string(diff_report)?;

    let mut html = HTML_TEMPLATE.to_string();

    // Inject data
    html = html.replace("/* PROFILE_DATA_JSON */", &profile_a_json);
    html = html.replace("/* PROFILE_B_DATA_JSON */", &profile_b_json);
    html = html.replace("/* DIFF_DATA_JSON */", &diff_json);

    // Inline CSS and JS
    html = html.replace(
        "<link rel=\"stylesheet\" href=\"viewer.css\">",
        &format!("<style>{}</style>", CSS_TEMPLATE),
    );
    html = html.replace(
        "<script src=\"viewer.js\"></script>",
        &format!("<script>{}</script>", JS_TEMPLATE),
    );

    fs::write(output_path, html).context("Failed to write diff viewer HTML")?;

    Ok(())
}

/// Open a path in the system default browser
pub fn open_browser(path: &Path) -> Result<()> {
    let url = format!("file://{}", path.canonicalize()?.display());

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .status()
            .context("Failed to open browser on macOS")?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .status()
            .context("Failed to open browser on Linux")?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(url)
            .status()
            .context("Failed to open browser on Windows")?;
    }

    Ok(())
}
