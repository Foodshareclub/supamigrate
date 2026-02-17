# GitHub Secrets Update for Self-Hosted Supabase Backup

## Repository
`https://github.com/Foodshareclub/supamigrate/settings/secrets/actions`

## Secrets to Update

### Database Connection
```
SUPABASE_POOLER_HOST = db.foodshare.club
SUPABASE_DB_USER = postgres
SUPABASE_DB_PASS = 52ea2d70c736c1b05735cdc8b57fe262
```

### Optional (can be removed, no longer used)
```
SUPABASE_PROJECT_ID = (not needed for self-hosted)
```

## What Changed

The workflow has been updated to connect to your self-hosted Supabase instance instead of the cloud instance.

**Before:** Connected to cloud instance `iazmjdjwnkilycbjwpzp`
**After:** Connects to self-hosted instance at `db.foodshare.club`

## Backup Schedule

- **Frequency:** Daily at 2 AM UTC
- **Storage:** Cloudflare R2 bucket
- **Retention:** 30 days (automatic cleanup)
- **What's backed up:**
  - Full database dump
  - RLS policies
  - Triggers & functions
  - Edge functions
  - Auth configuration
  - Secrets (names only)
  - Cron jobs

## Next Steps

1. Go to GitHub repository secrets
2. Update the three secrets listed above
3. Optionally trigger a manual workflow run to test:
   - Go to Actions → Pipeline → Run workflow
4. Check the backup succeeded in R2

## Notes

- The workflow now uses standard PostgreSQL connection format (`-U postgres`) instead of Supabase Cloud format (`-U postgres.project_id`)
- Make sure `db.foodshare.club` resolves to your VPS or use the direct IP/hostname
- Port 5432 must be accessible from GitHub Actions runners
