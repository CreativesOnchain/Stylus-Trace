//! Viewer generation and browser orchestration.

use crate::parser::schema::Profile;
use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::fs;
use std::path::Path;

const HTML_TEMPLATE: &str = include_str!("viewer/index.html");
const CSS_TEMPLATE: &str = include_str!("viewer/viewer.css");
const JS_TEMPLATE: &str = include_str!("viewer/viewer.js");

/// Encode a JSON string as Base64 for safe inline HTML injection.
/// This prevents any `</script>` sequences in the JSON from breaking the page.
fn encode_json(json: &str) -> String {
    BASE64.encode(json.as_bytes())
}

/// Build the final, self-contained HTML from the template.
///
/// Inlines CSS and JS, and injects the base64 JSON blobs + optional SVG.
fn build_html(
    profile_a_b64: &str,
    profile_b_b64: Option<&str>,
    diff_b64: Option<&str>,
    flamegraph_svg: Option<&str>,
) -> String {
    let mut html = HTML_TEMPLATE.to_string();

    // Inject data placeholders (base64-encoded JSON)
    html = html.replace("/* PROFILE_DATA_B64 */", profile_a_b64);
    html = html.replace(
        "/* PROFILE_B_DATA_B64 */",
        profile_b_b64.unwrap_or_default(),
    );
    html = html.replace("/* DIFF_DATA_B64 */", diff_b64.unwrap_or_default());
    html = html.replace("/* FLAMEGRAPH_SVG */", flamegraph_svg.unwrap_or_default());

    // Inline CSS
    html = html.replace(
        "<link rel=\"stylesheet\" href=\"viewer.css\">",
        &format!("<style>{}</style>", CSS_TEMPLATE),
    );

    // Inline JS
    html = html.replace(
        "<script src=\"viewer.js\"></script>",
        &format!("<script>{}</script>", JS_TEMPLATE),
    );

    html
}

/// Generate a self-contained HTML viewer for a single profile.
pub fn generate_viewer(
    profile: &Profile,
    flamegraph_svg: Option<&str>,
    output_path: &Path,
) -> Result<()> {
    let profile_json = serde_json::to_string(profile)?;
    let profile_b64 = encode_json(&profile_json);

    let html = build_html(&profile_b64, None, None, flamegraph_svg);

    fs::write(output_path, html).context("Failed to write viewer HTML")?;
    Ok(())
}

/// Generate a self-contained HTML viewer for a diff (baseline vs target).
pub fn generate_diff_viewer(
    profile_a: &Profile,
    profile_b: &Profile,
    diff_report: &serde_json::Value,
    flamegraph_svg: Option<&str>,
    output_path: &Path,
) -> Result<()> {
    let profile_a_json = serde_json::to_string(profile_a)?;
    let profile_b_json = serde_json::to_string(profile_b)?;
    let diff_json = serde_json::to_string(diff_report)?;

    let a_b64 = encode_json(&profile_a_json);
    let b_b64 = encode_json(&profile_b_json);
    let diff_b64 = encode_json(&diff_json);

    let html = build_html(&a_b64, Some(&b_b64), Some(&diff_b64), flamegraph_svg);

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
