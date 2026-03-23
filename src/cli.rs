//! CLI argument definitions for Aksum.
//!
//! Uses clap derive API to define all commands and flags.

use clap::{Parser, Subcommand};

/// Aksum — An opinionated Axum project scaffolder.
#[derive(Parser)]
#[command(name = "aksum")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Axum project
    New {
        /// Project name (snake_case recommended)
        name: String,

        /// Include database support (SQLx + PostgreSQL)
        #[arg(long)]
        with_db: bool,

        /// Include Redis support
        #[arg(long)]
        with_redis: bool,

        /// Place infrastructure inside shared/ instead of a separate infra/ directory
        #[arg(long)]
        infra_as_shared: bool,
    },

    /// Generate project components
    Generate {
        #[command(subcommand)]
        component: GenerateCommands,
    },
}

#[derive(Subcommand)]
pub enum GenerateCommands {
    /// Generate a new module with repository, service, controller, dto, and schema
    Module {
        /// Module name (snake_case)
        name: String,

        /// Skip repository generation
        #[arg(long)]
        no_repo: bool,

        /// Skip service generation
        #[arg(long)]
        no_service: bool,

        /// Skip controller generation
        #[arg(long)]
        no_controller: bool,

        /// Skip DTO generation
        #[arg(long)]
        no_dto: bool,

        /// Skip schema generation
        #[arg(long)]
        no_schema: bool,
    },

    /// Add a controller to an existing module (path: module_name/controller_name)
    Controller {
        /// Path in format: module_name/controller_name
        path: String,
    },

    /// Add a service to an existing module (path: module_name/service_name)
    Service {
        /// Path in format: module_name/service_name
        path: String,
    },
}
