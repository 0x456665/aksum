//! Service generator.
//!
//! Handles `aksum generate service <module>/<name>` — adds a new
//! service file to an existing module's service/ folder.

use anyhow::{bail, Result};
use convert_case::{Case, Casing};
use serde::Serialize;
use std::path::Path;

use crate::engine;
use super::updater;

/// Context for service templates.
#[derive(Serialize)]
pub struct ServiceContext {
    pub module_name: String,
    #[serde(rename = "ModuleName")]
    pub module_name_pascal: String,
    pub service_name: String,
    #[serde(rename = "ServiceName")]
    pub service_name_pascal: String,
}

/// Generate a new service inside an existing module.
///
/// `path` should be in format "module_name/service_name".
pub fn generate(path: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 2 {
        bail!("Path must be in format: module_name/service_name");
    }

    let (module_name, service_name) = (parts[0], parts[1]);

    let service_dir = Path::new("src/modules")
        .join(module_name)
        .join("service");

    if !service_dir.exists() {
        bail!(
            "Service directory not found: {}. Is '{}' a valid module with services?",
            service_dir.display(),
            module_name
        );
    }

    let output_file = service_dir.join(format!("{}.rs", service_name));
    if output_file.exists() {
        bail!("Service '{}' already exists in module '{}'", service_name, module_name);
    }

    let ctx = ServiceContext {
        module_name: module_name.to_string(),
        module_name_pascal: module_name.to_case(Case::Pascal),
        service_name: service_name.to_string(),
        service_name_pascal: service_name.to_case(Case::Pascal),
    };

    println!("📦 Generating service: {}/{}", module_name, service_name);

    engine::render_and_write("service/service.rs.tmpl", &output_file, &ctx)?;

    // Add re-export to service/mod.rs
    updater::insert_at_marker(
        &service_dir.join("mod.rs"),
        "// aksum:service_functions",
        &format!("pub mod {};", service_name),
    )?;

    println!("\n✅ Service '{}/{}' generated!", module_name, service_name);

    Ok(())
}
