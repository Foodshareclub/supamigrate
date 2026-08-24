---
name: migration-workflow
description: Common migration workflows using supamigrate. Use for project-to-project migration, backup/restore, and storage sync operations. Step-by-step guides for each workflow.
disable-model-invocation: true
---

<objective>
Execute common Supabase migration workflows safely using supamigrate CLI commands.
</objective>

<essential_principles>
## Prerequisites

```bash
# Verify dependencies
supamigrate doctor

# Initialize config
supamigrate config init

# Add projects
supamigrate config add production --url "postgresql://..." --api-url "https://api.foodshare.club" --service-key "..."
supamigrate config add staging --url "postgresql://..." --api-url "https://staging.foodshare.club" --service-key "..."
```

## Workflow 1: Project-to-Project Migration

Migrate schema, data, and storage between Supabase projects:

```bash
# Full migration (schema + data + storage)
supamigrate migrate --source production --target staging

# Schema only
supamigrate migrate --source production --target staging --schema-only

# Specific tables
supamigrate migrate --source production --target staging --tables "food_listings,profiles"

# Dry run (preview what will happen)
supamigrate migrate --source production --target staging --dry-run
```

## Workflow 2: Backup

Create local backups of a Supabase project:

```bash
# Full backup (database + storage + vault + functions)
supamigrate backup --project production --output ./backups/

# Database only
supamigrate backup --project production --output ./backups/ --database-only

# Storage only
supamigrate backup --project production --output ./backups/ --storage-only

# Include vault secrets
supamigrate backup --project production --output ./backups/ --include-vault
```

## Workflow 3: Restore

Restore from a local backup:

```bash
# Full restore
supamigrate restore --project staging --input ./backups/production-20260210/

# Database only
supamigrate restore --project staging --input ./backups/production-20260210/ --database-only

# Preview (dry run)
supamigrate restore --project staging --input ./backups/production-20260210/ --dry-run
```

## Workflow 4: Storage Sync

Sync storage buckets between projects:

```bash
# List buckets
supamigrate storage list --project production

# Sync specific bucket
supamigrate storage sync --source production --target staging --bucket food-images

# Download bucket locally
supamigrate storage download --project production --bucket food-images --output ./storage/

# Upload to project
supamigrate storage upload --project staging --bucket food-images --input ./storage/
```

## Workflow 5: Vault Secrets

Manage Supabase Vault secrets:

```bash
# List secrets (names only)
supamigrate vault list --project production

# Export secrets
supamigrate vault export --project production --output ./secrets.json

# Import secrets
supamigrate vault import --project staging --input ./secrets.json

# Copy between projects
supamigrate vault copy --source production --target staging
```

## Safety Rules

1. **Always dry-run first** - Use `--dry-run` before any destructive operation
2. **Backup before restore** - Always create a backup before restoring to a project
3. **Check target project** - Verify you're targeting the correct project (staging, not production!)
4. **Vault secrets are sensitive** - Exported vault files contain decrypted secrets
5. **Storage sync is additive** - Does not delete files on target that don't exist on source
</essential_principles>

<success_criteria>
Migration workflow is correct when:
- [ ] Config verified with `supamigrate doctor`
- [ ] Source and target projects correctly identified
- [ ] Dry run executed first
- [ ] Backup created before destructive operations
- [ ] Progress bars shown for long operations
- [ ] Success/failure clearly reported
</success_criteria>
