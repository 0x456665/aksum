//! Template engine powered by minijinja + rust-embed.
//!
//! Loads embedded template files, renders them with context variables,
//! and writes the output to disk.

use anyhow::{Context, Result};
use minijinja::Environment;
use rust_embed::Embed;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Embedded template files from the `templates/` directory.
#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

/// Render an embedded template with the given context.
///
/// # Arguments
/// * `template_path` - Path within the templates/ directory (e.g. "project/Cargo.toml.tmpl")
/// * `ctx` - Serializable context for template variables
pub fn render_template(template_path: &str, ctx: &impl Serialize) -> Result<String> {
    let template_file = Templates::get(template_path)
        .with_context(|| format!("Template not found: {}", template_path))?;

    let template_str = std::str::from_utf8(template_file.data.as_ref())
        .with_context(|| format!("Invalid UTF-8 in template: {}", template_path))?;

    let mut env = Environment::new();
    env.add_template("tpl", template_str)
        .with_context(|| format!("Failed to parse template: {}", template_path))?;

    let tmpl = env.get_template("tpl")?;
    let rendered = tmpl.render(ctx)?;

    Ok(rendered)
}

/// Write content to a file, creating parent directories as needed.
pub fn write_file(output_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(output_path, content)
        .with_context(|| format!("Failed to write file: {}", output_path.display()))?;
    Ok(())
}

/// Render a template and write the result to disk.
pub fn render_and_write(template_path: &str, output_path: &Path, ctx: &impl Serialize) -> Result<()> {
    let content = render_template(template_path, ctx)?;
    write_file(output_path, &content)?;
    println!("  ✔ Created {}", output_path.display());
    Ok(())
}
