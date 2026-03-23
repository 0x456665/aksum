//! Module generator.
//!
//! Handles the `aksum generate module <name>` command — creates a
//! complete module directory with controller, service, repository,
//! dto, and schema sub-modules, then auto-updates entry points.

use anyhow::{bail, Result};
use convert_case::{Case, Casing};
use serde::Serialize;
use std::path::Path;

use crate::engine;
use super::updater;

/// Context variables for module templates.
#[derive(Serialize, Clone)]
pub struct ModuleContext {
    pub module_name: String,
    #[serde(rename = "ModuleName")]
    pub module_name_pascal: String,
    pub no_repo: bool,
    pub no_service: bool,
    pub no_controller: bool,
    pub no_dto: bool,
    pub no_schema: bool,
}

/// Options for the `generate module` command.
pub struct ModuleOptions {
    pub name: String,
    pub no_repo: bool,
    pub no_service: bool,
    pub no_controller: bool,
    pub no_dto: bool,
    pub no_schema: bool,
}

/// Generate a new module inside an existing Aksum project.
pub fn generate(opts: &ModuleOptions) -> Result<()> {
    // Verify we're in an Aksum project
    let modules_dir = Path::new("src/modules");
    if !modules_dir.exists() {
        bail!("Not in an Aksum project root. Expected src/modules/ directory.");
    }

    let module_dir = modules_dir.join(&opts.name);
    if module_dir.exists() {
        bail!("Module '{}' already exists at {}", opts.name, module_dir.display());
    }

    let ctx = ModuleContext {
        module_name: opts.name.clone(),
        module_name_pascal: opts.name.to_case(Case::Pascal),
        no_repo: opts.no_repo,
        no_service: opts.no_service,
        no_controller: opts.no_controller,
        no_dto: opts.no_dto,
        no_schema: opts.no_schema,
    };

    println!("📦 Generating module: {}", opts.name);

    let r = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &module_dir.join(out), &ctx);

    // Module root
    r("module/mod.rs.tmpl", "mod.rs")?;

    // Sub-modules based on flags
    if !opts.no_controller {
        r("module/controller/mod.rs.tmpl", "controller/mod.rs")?;
    }
    if !opts.no_service {
        r("module/service/mod.rs.tmpl", "service/mod.rs")?;
    }
    if !opts.no_repo {
        r("module/repository/mod.rs.tmpl", "repository/mod.rs")?;
        r("module/repository/implementation.rs.tmpl", "repository/implementation.rs")?;
    }
    if !opts.no_dto {
        r("module/dto/mod.rs.tmpl", "dto/mod.rs")?;
    }
    if !opts.no_schema {
        r("module/schema/mod.rs.tmpl", "schema/mod.rs")?;
    }

    // Auto-update entry points
    println!("\n🔧 Updating entry points...");

    // 1. Add module declaration to src/modules/mod.rs
    updater::insert_at_marker(
        Path::new("src/modules/mod.rs"),
        "// aksum:module_declarations",
        &format!("pub mod {};", opts.name),
    )?;

    // 2. Add route merge to src/main.rs (only if controller exists)
    if !opts.no_controller {
        updater::insert_at_marker(
            Path::new("src/main.rs"),
            "// aksum:module_routes",
            &format!("        .merge(modules::{}::controller::routes())", opts.name),
        )?;

        // 3. Add OpenAPI path to src/main.rs
        updater::insert_at_marker(
            Path::new("src/main.rs"),
            "// aksum:openapi_paths",
            &format!("        modules::{}::controller::index,", opts.name),
        )?;
    }

    println!("\n✅ Module '{}' generated successfully!", opts.name);
    Ok(())
}
