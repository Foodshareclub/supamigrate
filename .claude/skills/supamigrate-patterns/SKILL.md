---
name: supamigrate-patterns
description: Architecture and patterns for the supamigrate Rust CLI. Use when understanding the codebase, command pattern, error handling, config loading, or pg_dump/psql wrappers.
---

<objective>
Understand and extend the supamigrate CLI following its established patterns for command dispatch, error handling, config management, and database operations.
</objective>

<essential_principles>
## Architecture

```
src/
├── main.rs         # Entry point, tracing setup, command dispatch
├── cli.rs          # Clap-based CLI definitions (Commands, Args structs)
├── config.rs       # TOML config loading
├── error.rs        # Custom error types (thiserror)
├── commands/       # Command implementations
│   ├── migrate.rs  # Project-to-project migration
│   ├── backup.rs   # Backup to local disk
│   ├── restore.rs  # Restore from backup
│   ├── storage.rs  # Storage operations (list, sync, download, upload)
│   ├── vault.rs    # Vault secrets management
│   ├── secrets.rs  # Edge function secrets
│   ├── doctor.rs   # System dependency checks
│   └── config.rs   # Config management (init, add, list, show)
├── db/             # Database operations
│   ├── dump.rs     # pg_dump wrapper with auto-version detection
│   ├── restore.rs  # psql restore
│   ├── vault.rs    # Supabase Vault secrets via SQL
│   └── transform.rs# SQL transformations
├── storage/        # Supabase Storage API client
│   ├── client.rs   # HTTP client for storage operations
│   └── transfer.rs # Parallel file transfers with progress
└── functions/      # Edge Functions API client
    ├── client.rs   # Backup/restore edge functions
    └── secrets.rs  # Secrets API client
```

## Key Design Patterns

### Command Pattern
```rust
// cli.rs - Clap derive
#[derive(Parser)]
#[command(name = "supamigrate", about = "Supabase migration CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Migrate between Supabase projects
    Migrate(MigrateArgs),
    /// Backup a project
    Backup(BackupArgs),
    /// Restore from backup
    Restore(RestoreArgs),
    // ...
}

// main.rs - dispatch
match cli.command {
    Commands::Migrate(args) => commands::migrate::run(args).await,
    Commands::Backup(args) => commands::backup::run(args).await,
    // ...
}
```

### Error Handling
```rust
// thiserror for library errors
#[derive(Error, Debug)]
pub enum SupamigrateError {
    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    #[error("pg_dump failed: {0}")]
    PgDump(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

// anyhow for application-level errors in main
fn main() -> anyhow::Result<()> { ... }
```

### Config Loading
```rust
// Config loaded from (in order):
// 1. ./supamigrate.toml
// 2. ~/.config/supamigrate/config.toml
// 3. ~/.supamigrate.toml
//
// Projects referenced by alias: "production", "staging"
```

### Database Operations
```rust
// Shell out to pg_dump/psql with automatic version detection
pub async fn dump(config: &ProjectConfig) -> Result<PathBuf> {
    let pg_dump = find_pg_dump()?;  // Auto-detect version
    let output = Command::new(pg_dump)
        .args(["--format=custom", "--dbname", &config.database_url])
        .output()
        .await?;
    // ...
}
```

## Key Libraries

| Library | Purpose |
|---------|---------|
| tokio | Async runtime |
| clap | CLI parsing with derive macros |
| reqwest + rustls | HTTP client (no OpenSSL) |
| thiserror | Library error types |
| anyhow | Application error handling |
| indicatif | Progress bars for transfers |
| serde + toml | Config serialization |

## Build & Test

```bash
cargo build                    # Debug
cargo build --release          # Release
cargo test --all-features      # All tests
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo fmt --all -- --check     # Format check
```

## CI/CD

Tag push triggers pipeline building 6 platform binaries:
- linux-x86_64, linux-x86_64-musl, linux-aarch64
- darwin-x86_64, darwin-aarch64
- windows-x86_64

Releases: update Cargo.toml version, tag `v*`, push.
</essential_principles>

<success_criteria>
Code follows patterns when:
- [ ] New commands added to Commands enum and dispatched in main
- [ ] Errors use thiserror with descriptive messages
- [ ] Config loaded through standard config chain
- [ ] Database ops use pg_dump/psql wrappers (not direct SQL)
- [ ] HTTP uses reqwest with rustls
- [ ] Progress shown with indicatif for long operations
</success_criteria>
