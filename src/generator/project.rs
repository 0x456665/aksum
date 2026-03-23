//! Project scaffolding generator.
//!
//! Handles the `aksum new <name>` command — creates the full
//! project directory structure with all configuration and a
//! default health module.

use anyhow::{bail, Result};
use convert_case::{Case, Casing};
use serde::Serialize;
use std::path::Path;

use crate::engine;

/// Context variables passed to project templates.
#[derive(Serialize, Clone)]
pub struct ProjectContext {
    pub project_name: String,
    #[serde(rename = "ProjectName")]
    pub project_name_pascal: String,
    pub with_db: bool,
    pub with_redis: bool,
    pub infra_as_shared: bool,
}

/// Options for the `new` command.
pub struct ProjectOptions {
    pub name: String,
    pub with_db: bool,
    pub with_redis: bool,
    pub infra_as_shared: bool,
}

/// Generate a new Axum project.
pub fn generate(opts: &ProjectOptions) -> Result<()> {
    let project_dir = Path::new(&opts.name);

    if project_dir.exists() {
        bail!("Directory '{}' already exists", opts.name);
    }

    let ctx = ProjectContext {
        project_name: opts.name.clone(),
        project_name_pascal: opts.name.to_case(Case::Pascal),
        with_db: opts.with_db,
        with_redis: opts.with_redis,
        infra_as_shared: opts.infra_as_shared,
    };

    println!("📦 Creating new Axum project: {}", opts.name);

    // Core project files
    let r = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &project_dir.join(out), &ctx);

    r("project/Cargo.toml.tmpl", "Cargo.toml")?;
    r("project/dot_env.tmpl", ".env")?;
    r("project/gitignore.tmpl", ".gitignore")?;

    // Source files
    let s = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &project_dir.join("src").join(out), &ctx);

    s("project/src/main.rs.tmpl", "main.rs")?;
    s("project/src/app_state.rs.tmpl", "app_state.rs")?;

    // Config
    s("project/src/config/mod.rs.tmpl", "config/mod.rs")?;
    s("project/src/config/settings.rs.tmpl", "config/settings.rs")?;

    // Shared
    s("project/src/shared/mod.rs.tmpl", "shared/mod.rs")?;
    s("project/src/shared/errors.rs.tmpl", "shared/errors.rs")?;

    // Modules
    s("project/src/modules/mod.rs.tmpl", "modules/mod.rs")?;

    // Default health module
    let h = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &project_dir.join("src/modules/health").join(out), &ctx);

    h("project/src/modules/health/mod.rs.tmpl", "mod.rs")?;
    h("project/src/modules/health/controller/mod.rs.tmpl", "controller/mod.rs")?;
    h("project/src/modules/health/service/mod.rs.tmpl", "service/mod.rs")?;
    h("project/src/modules/health/repository/mod.rs.tmpl", "repository/mod.rs")?;
    h("project/src/modules/health/repository/implementation.rs.tmpl", "repository/implementation.rs")?;
    h("project/src/modules/health/dto/mod.rs.tmpl", "dto/mod.rs")?;
    h("project/src/modules/health/schema/mod.rs.tmpl", "schema/mod.rs")?;

    // Infrastructure (optional)
    if opts.with_db || opts.with_redis {
        generate_infra(&project_dir.join("src"), &ctx)?;
    }

    println!("\n✅ Project '{}' created successfully!", opts.name);
    println!("\nNext steps:");
    println!("  cd {}", opts.name);
    println!("  cargo run");

    Ok(())
}

/// Generate infrastructure modules (DB, Redis).
fn generate_infra(src: &Path, ctx: &ProjectContext) -> Result<()> {
    if ctx.infra_as_shared {
        // Place infra inside shared/
        if ctx.with_db {
            let db = src.join("shared/db");
            render_db_infra(&db, ctx)?;
        }
        if ctx.with_redis {
            let redis = src.join("shared/cache");
            render_redis_infra(&redis, ctx)?;
        }
    } else {
        // Separate infra/ directory
        engine::render_and_write("infra/mod.rs.tmpl", &src.join("infra/mod.rs"), ctx)?;
        if ctx.with_db {
            render_db_infra(&src.join("infra/db"), ctx)?;
        }
        if ctx.with_redis {
            render_redis_infra(&src.join("infra/cache"), ctx)?;
        }
    }
    Ok(())
}

fn render_db_infra(db_dir: &Path, ctx: &ProjectContext) -> Result<()> {
    let r = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &db_dir.join(out), ctx);
    r("infra/db/mod.rs.tmpl", "mod.rs")?;
    r("infra/db/connection.rs.tmpl", "connection.rs")?;
    Ok(())
}

fn render_redis_infra(redis_dir: &Path, ctx: &ProjectContext) -> Result<()> {
    let r = |tmpl: &str, out: &str| engine::render_and_write(tmpl, &redis_dir.join(out), ctx);
    r("infra/redis/mod.rs.tmpl", "mod.rs")?;
    r("infra/redis/connection.rs.tmpl", "connection.rs")?;
    Ok(())
}
