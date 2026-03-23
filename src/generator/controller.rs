//! Controller generator.
//!
//! Handles `aksum generate controller <module>/<name>` — adds a new
//! controller handler file to an existing module's controller/ folder.

use anyhow::{bail, Result};
use convert_case::{Case, Casing};
use serde::Serialize;
use std::path::Path;

use crate::engine;
use super::updater;

/// Context for controller templates.
#[derive(Serialize)]
pub struct ControllerContext {
    pub module_name: String,
    #[serde(rename = "ModuleName")]
    pub module_name_pascal: String,
    pub controller_name: String,
    #[serde(rename = "ControllerName")]
    pub controller_name_pascal: String,
}

/// Generate a new controller inside an existing module.
///
/// `path` should be in format "module_name/controller_name".
pub fn generate(path: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 2 {
        bail!("Path must be in format: module_name/controller_name");
    }

    let (module_name, controller_name) = (parts[0], parts[1]);

    let controller_dir = Path::new("src/modules")
        .join(module_name)
        .join("controller");

    if !controller_dir.exists() {
        bail!(
            "Controller directory not found: {}. Is '{}' a valid module with controllers?",
            controller_dir.display(),
            module_name
        );
    }

    let output_file = controller_dir.join(format!("{}.rs", controller_name));
    if output_file.exists() {
        bail!("Controller '{}' already exists in module '{}'", controller_name, module_name);
    }

    let ctx = ControllerContext {
        module_name: module_name.to_string(),
        module_name_pascal: module_name.to_case(Case::Pascal),
        controller_name: controller_name.to_string(),
        controller_name_pascal: controller_name.to_case(Case::Pascal),
    };

    println!("📦 Generating controller: {}/{}", module_name, controller_name);

    engine::render_and_write("controller/handler.rs.tmpl", &output_file, &ctx)?;

    // Add re-export to controller/mod.rs
    updater::insert_at_marker(
        &controller_dir.join("mod.rs"),
        "// aksum:controller_handlers",
        &format!("pub mod {};", controller_name),
    )?;

    println!("\n✅ Controller '{}/{}' generated!", module_name, controller_name);
    println!("   Don't forget to wire the new handler into the routes() function.");

    Ok(())
}
