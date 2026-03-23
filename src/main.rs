//! Aksum — An opinionated Axum project scaffolder.
//!
//! Similar to NestJS CLI, Aksum scaffolds modular Axum projects
//! with trait-based dependency injection via AppState, utoipa OpenAPI
//! documentation, and a clean separation of concerns.

mod cli;
mod engine;
mod generator;

use clap::Parser;
use cli::{Cli, Commands, GenerateCommands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            with_db,
            with_redis,
            infra_as_shared,
        } => {
            generator::project::generate(&generator::project::ProjectOptions {
                name,
                with_db,
                with_redis,
                infra_as_shared,
            })?;
        }

        Commands::Generate { component } => match component {
            GenerateCommands::Module {
                name,
                no_repo,
                no_service,
                no_controller,
                no_dto,
                no_schema,
            } => {
                generator::module::generate(&generator::module::ModuleOptions {
                    name,
                    no_repo,
                    no_service,
                    no_controller,
                    no_dto,
                    no_schema,
                })?;
            }

            GenerateCommands::Controller { path } => {
                generator::controller::generate(&path)?;
            }

            GenerateCommands::Service { path } => {
                generator::service::generate(&path)?;
            }
        },
    }

    Ok(())
}
