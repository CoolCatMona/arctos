# Operational and development scripts

Read a script's module docstring and `--help` before running it. Database
maintenance commands may change data; take a backup and verify the target path.
New maintenance scripts should default to a dry run and require an explicit
write option. Dedicated demo commands are an exception: their purpose is to
create isolated sample data.

## Local demo data

```bash
just demo                    # Flask on 127.0.0.1:5006
# In another terminal:
just frontend                # Dioxus on 127.0.0.1:8080
```

`dev_demo.py` creates `instance/demo.db` if empty and preserves existing data
on restart. Accounts `demo-organizer`, `demo-player`, and `demo-team` use
`arctos-demo-only`; startup also prints the credentials. The launcher does not
load `.env` or use `instance/tournament.db`. Keep it local.

To start from an existing versioned SQLite snapshot:

```bash
uv run python -m scripts.dev_demo --snapshot /absolute/path/to/snapshot.db
```

This opens the source read-only, copies it through SQLite's backup API,
migrates the copy, and adds fictional fixtures at
`instance/snapshot-demo/demo.db`. It rejects unknown/unversioned schemas and
reserved demo-ID collisions. It does not overwrite an existing prepared copy.
Use `--seed-only` to prepare data without starting Flask. Uploaded media is not
copied; provide the corresponding `static/uploads` files separately.

Existing demo copies need explicit migrations when the schema changes; see
[migrations/README.md](../migrations/README.md#profile-and-event-banners).

For an additional populated practice event:

```bash
uv run python -m scripts.dev_practice --database instance/snapshot-demo/demo.db
```

This adds `/demo-practice`, owned by `demo-organizer`, with teams, rosters,
fields, matches in several states, and an active dueling competition. It prints
links to the relevant tools. Repeated runs preserve edits and deletions; a new
`--slug demo-practice-2` creates another independent example. Use
`--database instance/demo.db` for the ordinary demo. Both commands restrict
writes to a regular demo database; do not give them the original snapshot.

## Browser checks

```bash
uv run playwright install chromium
uv run python scripts/check_mobile_ui.py --screenshots /tmp/arctos-ui
```

Keep the demo and frontend running. The checks require a demo marker and guard
against real data writes. See [TESTING.md](../TESTING.md#mobile-browser-verification)
for focused suites and device/integration limits.

## Public HTML assets and browser checks

`prepare_public_assets.py` generates shared presentation from the frontend CSS
and vendored Bootstrap, validates the compiled module and Wasm references,
creates deterministic gzip companions, and atomically writes
`arctos-assets.json` in the selected build directory.

```bash
cd frontend
dx bundle --web --release
cd ..
uv run python scripts/prepare_public_assets.py --dist frontend/dist/public
ARCTOS_PUBLIC_HTML_ENABLED=1 \
ARCTOS_PUBLIC_ENTRY_MODE=content \
ARCTOS_FRONTEND_DIST="$PWD/frontend/dist/public" just demo
```

Use the public output directory reported by Dioxus for both `--dist` and
`ARCTOS_FRONTEND_DIST`. Open Flask at **http://127.0.0.1:5006**. `content`
provides Events, Leagues, and their published Info pages as HTML; `loading` selects lightweight startup
feedback followed by Dioxus. Both modes use the configured database.

Regenerate presentation when shared component styles change:

```bash
uv run python scripts/prepare_public_assets.py --theme-only
uv run python scripts/prepare_public_assets.py --check-theme
```

These options cover generated `bootstrap.css`, `design.css`, and `theme.css`.
The scoped `startup.css` and native-control `public.css` adapters are maintained
alongside the corresponding frontend components.

`check_public_html.py` verifies anonymous content-mode reading, usable sign-in
controls, navigation, caching, overflow, and failed/delayed Wasm recovery:

```bash
uv run playwright install chromium
uv run python scripts/check_public_html.py --league-path /leagues/demo-coast-league \
  --runs 5 --output /tmp/arctos-public-html
```

Default targets are `/`, `/demo-open`, and login with a return to the event's
registration. The default profile uses a 390px viewport, mobile 4G network, and
4× CPU slowdown. `--public-only` checks public reading before a Wasm build;
`--width 320 --profile unthrottled --runs 1` provides a narrow functional pass.
Repeated `--*-ready-selector` options define all required facts or controls for
another route. Remote staging requires `--base-url` and `--allow-remote`.
Screenshots are opt-in because they include displayed content.
`--league-path` adds the Leagues directory, league Info, and their native
navigation and JavaScript-disabled checks.

`check_public_reading_navigation.py` exercises public document replacement and
the first application handoff under deliberate delays and failures:

```bash
uv run python scripts/check_public_reading_navigation.py \
  --base-url http://127.0.0.1:5006 \
  --event-path /demo-open --league-path /leagues/demo-coast-league \
  --output /tmp/arctos-public-reading-check.json
```

It verifies reading during held/failed Wasm, superseded responses, metadata,
appearance rebinding, pre/post-activation history, stylesheet-gated handoff,
failure recovery, and native links. It accepts loopback origins, blocks writes,
and creates a fresh assertion report. Run it separately from timing comparisons.

`check_directory_account_state.py` checks retained directory behavior against a
local release in either entry mode:

```bash
uv run python scripts/check_directory_account_state.py \
  --base-url http://127.0.0.1:5006 \
  --output /tmp/arctos-directory-account-check.json
```

It verifies removal of private Events cards during logout, rejection of delayed
prior-account responses, Leagues Retry pending/recovery states, and encoded
Leagues pagination across workflow navigation and Back/Forward. Synthetic
responses and intercepted logout keep server records and sessions unchanged;
unexpected API calls fail the check. The runner accepts loopback origins.
Choose a fresh output path, or omit `--output` for a timestamped report.

`compare_startup_profiles.py` compares prepared variants across four network/CPU
profiles. Prepare matching data, backend APIs, and asset policies, then select
the original frontend and content mode explicitly:

```bash
uv run python scripts/compare_startup_profiles.py \
  --variant original=http://127.0.0.1:5017 \
  --variant public-html=http://127.0.0.1:5016 \
  --runs 3 \
  --output /tmp/arctos-startup-comparison
uv run python scripts/compare_startup_profiles.py \
  --variant original=http://127.0.0.1:5017 \
  --variant public-html=http://127.0.0.1:5016 \
  --direct-leagues-only \
  --profile mobile-4g --runs 3 --output /tmp/arctos-leagues-comparison
```

Default origins are `original=5017`, `public-html=5016`, and
`loading-entry=5019` on loopback. To include the optional loading-mode server,
add `--variant loading-entry=http://127.0.0.1:5019`. Repeated `--variant NAME=ORIGIN` and
`--profile` options select a focused comparison. The runner rotates version
order, preserves normal browser caching, and records separate startup-feedback,
content, and retained-navigation timings. `--include-direct-leagues` adds
direct-entry pairs to the Events run.

The reading and comparison tools inject observations into their own browser
sessions and write local reports. Reading and timing runs use anonymous GET workflows; the known
Markdown-rendering POST is a read operation. Run comparisons while builds and
other browser tests are idle. See
[verification](../docs/public-html-verification.md) for setup, acceptance
criteria, report interpretation, and measured results.

## Database maintenance

```bash
just db-backup
./scripts/backup_db.sh before-migration
just db-check-duplicates
uv run python scripts/check_duplicates.py --db /path/to/copy.db
uv run python scripts/cleanup_data_quality.py          # report only
uv run python scripts/cleanup_data_quality.py --apply  # apply changes
```

Check each script's database argument or `SQLALCHEMY_DATABASE_URI` handling;
not every utility resolves its target in the same way.

`backfill_normalised_tables.py` is only for databases between migrations
`0002_phase1_additive` and `0003_phase4_cleanup`, while the legacy source columns
still exist. It is not a normal setup step for current databases. Inspect with
`--dry-run` first; running without that flag writes and validates the backfill.

## Adding a script

State its purpose, inputs, write behavior, and recovery expectations in the
module docstring. Make the database target explicit and test destructive or
irreversible behavior with temporary data. Add commonly used commands to the
root `justfile`; document migration helpers in the migration guide.
