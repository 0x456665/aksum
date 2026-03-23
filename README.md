# Aksum

An opinionated Axum project scaffolder — think NestJS CLI, but for Rust.

Aksum generates modular Axum projects with trait-based dependency injection, utoipa OpenAPI documentation, and a clean separation of concerns. It automates the boilerplate so you can focus on business logic.

## Features

- **Project Scaffolding** — `aksum new` creates a production-ready Axum project structure
- **Module Generator** — `aksum generate module` creates self-contained modules with controller, service, repository, DTOs, and schemas
- **Controller/Service Generators** — Add handlers and services to existing modules
- **Auto-Wiring** — New modules are automatically registered in the router, OpenAPI docs, and module registry
- **Trait-Based DI** — Each module defines repository traits implemented on `AppState`, enabling loose coupling and easy testing
- **OpenAPI/Swagger** — Controllers are documented with [utoipa](https://github.com/juhaku/utoipa), Swagger UI included out of the box
- **Optional Infrastructure** — Add database (SQLx/PostgreSQL) and Redis support with flags
- **Flexible Infra Placement** — Infrastructure can live in `infra/` or `shared/`, your choice

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Create a new project
aksum new my_api

# Enter the project
cd my_api

# Run the server
cargo run
# => 🚀 Server listening on 0.0.0.0:3000
# => Swagger UI at http://localhost:3000/swagger-ui
```

## Commands

### `aksum new <name>`

Scaffold a complete Axum project with a default `health` module.

```bash
aksum new my_api
aksum new my_api --with-db              # Include SQLx + PostgreSQL
aksum new my_api --with-redis           # Include Redis
aksum new my_api --with-db --with-redis # Both
aksum new my_api --with-db --infra-as-shared  # Place DB setup in shared/ instead of infra/
```

| Flag                | Description                                        |
| ------------------- | -------------------------------------------------- |
| `--with-db`         | Add SQLx + PostgreSQL support                      |
| `--with-redis`      | Add Redis support                                  |
| `--infra-as-shared` | Place infrastructure inside `shared/` instead of `infra/` |

### `aksum generate module <name>`

Generate a new module with all sub-components, automatically wired into the app.

```bash
aksum generate module users
aksum generate module orders --no-dto --no-schema  # Skip optional components
```

| Flag              | Description                |
| ----------------- | -------------------------- |
| `--no-repo`       | Skip repository generation |
| `--no-service`    | Skip service generation    |
| `--no-controller` | Skip controller generation |
| `--no-dto`        | Skip DTO generation        |
| `--no-schema`     | Skip schema generation     |

**What happens automatically:**
1. Module directory created under `src/modules/<name>/`
2. `pub mod <name>;` added to `src/modules/mod.rs`
3. Route `.merge()` added to `src/main.rs`
4. OpenAPI path registered in the `ApiDoc` struct

### `aksum generate controller <module>/<name>`

Add a new controller handler file to an existing module.

```bash
aksum generate controller users/get_profile
```

Creates `src/modules/users/controller/get_profile.rs` and adds `pub mod get_profile;` to the controller's `mod.rs`. You then wire the handler into the module's `routes()` function.

### `aksum generate service <module>/<name>`

Add a new service file to an existing module.

```bash
aksum generate service users/notification
```

Creates `src/modules/users/service/notification.rs` and adds `pub mod notification;` to the service's `mod.rs`.

## Generated Project Structure

```
my_api/
├── Cargo.toml
├── .env
├── .gitignore
└── src/
    ├── main.rs                          # Server entry point + OpenAPI aggregator
    ├── app_state.rs                     # Central AppState struct
    ├── config/
    │   ├── mod.rs
    │   └── settings.rs                  # Env-based configuration
    ├── shared/
    │   ├── mod.rs
    │   └── errors.rs                    # AppError + IntoResponse
    ├── modules/
    │   ├── mod.rs                       # Module registry
    │   └── health/                      # Default example module
    │       ├── mod.rs
    │       ├── controller/
    │       │   └── mod.rs               # Axum handlers + utoipa docs
    │       ├── service/
    │       │   └── mod.rs               # Business logic
    │       ├── repository/
    │       │   ├── mod.rs               # Trait definition
    │       │   └── implementation.rs    # Trait impl on AppState
    │       ├── dto/
    │       │   └── mod.rs               # Input types (Deserialize)
    │       └── schema/
    │           └── mod.rs               # Response types (Serialize + ToSchema)
    └── infra/                           # Only with --with-db / --with-redis
        ├── mod.rs
        ├── db/
        │   ├── mod.rs
        │   └── connection.rs            # Pool creation helper
        └── cache/
            ├── mod.rs
            └── connection.rs            # Redis client helper
```

## Architecture: The Module Pattern

Each module follows a consistent structure designed for **modularity** and **testability**.

### Repository (Trait-Based DI)

Every module defines a repository **trait**. The trait is implemented on `AppState`, meaning modules only depend on the trait contract, not on `AppState` directly.

```rust
// modules/users/repository/mod.rs
pub trait UserRepository: Clone + Send + Sync + 'static {
    fn find_by_id(&self, id: i64)
        -> impl Future<Output = Result<Option<User>, AppError>> + Send;
}

// modules/users/repository/implementation.rs
impl UserRepository for AppState {
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        // Use self.db_pool to query the database
        todo!()
    }
}
```

### Service (Business Logic)

Services accept `&impl Trait` references, making them easy to test with mocks:

```rust
// modules/users/service/mod.rs
pub async fn get_user(repo: &impl UserRepository, id: i64) -> Result<User, AppError> {
    repo.find_by_id(id).await?.ok_or(AppError::NotFound("User not found".into()))
}
```

### Controller (HTTP Handlers)

Controllers are thin — they extract request data, call services, and return responses. All handlers are documented with utoipa:

```rust
// modules/users/controller/mod.rs
#[utoipa::path(get, path = "/users/{id}", tag = "Users")]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    let user = service::get_user(&state, id).await?;
    Ok(Json(user.into()))
}
```

### Data Flow

```
Request → Controller → Service → Repository (trait) → AppState (impl) → Database/Cache
```

## Marker Comments

Aksum uses marker comments (e.g. `// aksum:module_routes`) to know where to insert new code when generating modules. **Don't remove these comments** — they enable the auto-wiring feature.

| Marker                        | File              | Purpose                        |
| ----------------------------- | ----------------- | ------------------------------ |
| `// aksum:module_declarations`| `modules/mod.rs`  | New `pub mod` declarations     |
| `// aksum:module_routes`      | `main.rs`         | New `.merge()` route calls     |
| `// aksum:openapi_paths`      | `main.rs`         | New OpenAPI handler paths      |
| `// aksum:openapi_schemas`    | `main.rs`         | New OpenAPI schema types       |
| `// aksum:state_fields`       | `app_state.rs`    | New AppState fields            |
| `// aksum:state_imports`      | `app_state.rs`    | New import statements          |
| `// aksum:controller_handlers`| `controller/mod.rs`| New handler re-exports        |
| `// aksum:service_functions`  | `service/mod.rs`  | New service re-exports         |

## Infrastructure

Infrastructure (database, Redis) is **optional** and **unopinionated**. When you pass `--with-db` or `--with-redis`, Aksum generates simple connection setup helpers. How you wire them into your app is up to you:

- **Use `infra/`** (default) — a separate top-level directory for infrastructure concerns
- **Use `shared/`** (`--infra-as-shared`) — place infrastructure inside the shared module
- **Do it yourself** — skip the flags entirely and set up connections manually in `main.rs`

## Tech Stack (Generated Projects)

| Crate                 | Purpose                    |
| --------------------- | -------------------------- |
| `axum`                | Web framework              |
| `tokio`               | Async runtime              |
| `serde` / `serde_json`| Serialization              |
| `utoipa`              | OpenAPI spec generation    |
| `utoipa-swagger-ui`   | Swagger UI                 |
| `tracing`             | Structured logging         |
| `thiserror`           | Error type derivation      |
| `dotenvy`             | `.env` file loading        |
| `sqlx` *(optional)*   | PostgreSQL driver          |
| `redis` *(optional)*  | Redis client               |

## License

MIT
