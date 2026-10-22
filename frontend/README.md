# `frontend/` - the Dioxus SPA

A single-page application written in Rust using
[Dioxus](https://dioxuslabs.com/) and compiled to WebAssembly. nginx
serves the built bundle at `/`; the SPA calls the Flask backend at
`/_api/...`.

## Layout

The crate is a standard Dioxus + WASM project (`Cargo.toml`,
`Dioxus.toml`, `rust-toolchain.toml`, `index.html` shell). All
application code lives under `src/`:

- `main.rs` - router definition and bootstrap.
- `api.rs` - reqwest-based wrapper for every `/_api/` endpoint.
- `types.rs` - serde structs mirroring API responses.
- `pages/` - one file per route (mirrors the SPA's URL structure).
- `components/` - shared widget library.
- `time_format.rs`, `stones_filter.rs` - smaller cross-cutting modules;
  see each file's top comment.

## Running

From the repository root:

```bash
uv sync --group test
just setup-frontend
just demo                    # isolated sample backend on 127.0.0.1:5006
# In a second terminal:
just frontend                # frontend on 127.0.0.1:8080
```

Use `just dev` instead of `just demo` for your normal local database.
Dioxus proxies API and static requests to Flask, so requests and cookies use
the same origin. For phone testing on your LAN:
`just frontend --addr 0.0.0.0`, then open the computer's LAN address at port
8080. Recording requires a browser secure context (localhost or HTTPS).

For production, `cd frontend && dx bundle --release`; follow the build output
path reported by Dioxus and the repository deployment guide.

## Design system

`src/theme.css` styles the shared shell and existing Bootstrap components on
all routes. `src/pages/public_pages.css` contains event/discovery layouts;
`src/pages/operator.css` contains match and stones refinements. Colors, focus,
touch spacing, safe areas, forms, cards, tables, and dialogs share the same
mobile-first system. Keep canvas node/port geometry in the dedicated bracket
and schedule stylesheets. Scoreboard/OBS and exact-size offline frame routes
remain outside the app theme. Styles and icons are vendored under
`assets/vendor/` with licenses; no CSS/font CDN or Node build is needed.

## Routing

Routes are declared in [`src/main.rs`](src/main.rs) as a single
`Route` enum with `#[route("...")]` attributes. A representative slice:

```rust
#[route("/")]                          Index {}
#[route("/login")]                     Login {}
#[route("/leagues/:league_url")]       LeagueHome { league_url: String }
#[route("/:url")]                      TournamentHome { url: String }
#[route("/:url/schedule")]             Schedule { url: String }
#[route("/:url/manage")]               Manage { url: String }
#[route("/:url/run-match")]            RunMatch { url: String }
#[route("/:url/finalize-match")]       FinalizeMatch { url: String }
```

Note the order - the catch-all `/:url` for tournaments comes after all
the more specific paths. Adding a new top-level page means adding a
new route entry and a matching file in `src/pages/`.

## Talking to the backend

[`src/api.rs`](src/api.rs) wraps `reqwest` with credentials and base
URL handling. There's one public `pub async fn` per backend endpoint
(e.g. `login`, `tournaments`, `start_match`, `finalize_match`,
`update_tournament_settings`, ...). Pages call them directly:

```rust
use crate::api;

let user = api::login(&username, &password).await?;
let detail = api::tournament_detail(&tournament_url).await?;
```

Every call goes through the `with_credentials` wrapper, which on
`wasm32` calls `fetch_credentials_include()` so the session cookie is
sent.

Two important details:

- **Base URL.** Uses `window.location.origin` in browser builds. The Dioxus
  development proxy forwards `/_api/`, `/api/`, and `/static/` to Flask on
  `127.0.0.1:5006`. An explicit `ARCTOS_API_BASE` build-time environment
  variable overrides this for separate-host deployments.
- **Credentials.** Without `fetch_credentials_include()`, all `/_api/`
  calls would 401.

## Types

`src/types.rs` holds serde structs that mirror Flask's JSON responses.
When you add or change a backend response shape, update the matching
struct here. Shapes only used in one page can live in that page's file.

## Footage

Tournament organizers attach match footage via the Manage Footage page
(`src/pages/manage_footage.rs`) and the backend footage API
(`app/routes/tournaments/footage.py`): either a YouTube link or a chunked
file upload that the server pushes to YouTube. Jump-to-point uses
`camera_timepoints` on the match page iframe player.

## Conventions

- One file per page in `src/pages/` (matches the route enum).
- Reusable widgets go in `src/components/`.
- API calls go through `src/api.rs`, not direct `reqwest` calls in pages.
- Format with `cargo fmt` before committing.
- The Rust toolchain is pinned in `rust-toolchain.toml` (channel `stable`).

## Testing

Backend: `just test` / `just coverage-check`.

Frontend: Pure Rust unit tests (time sync, filters, display helpers,
…) run on the host target with the `just frontend-test` recipe.

Run the browser matrix against the demo with
`uv run python scripts/check_mobile_ui.py` from the repository root while
`just demo` and `just frontend` are running. It covers phone, tablet, and
desktop layouts plus common input and save failures. See
[`TESTING.md`](../TESTING.md#mobile-browser-verification) for browser setup
and the remaining real-device checks.

## When the SPA cannot reach the backend

1. Start Flask on `127.0.0.1:5006` (`just demo` or `just dev`).
2. Open the frontend on port 8080. Its `/_api/tournaments` request should use
   that same origin. Restart `dx serve` after changing proxy configuration.
3. With the default proxy, keep `ARCTOS_CORS_DEV` unset/0; secure cross-origin
   cookies are unnecessary and will not work on an ordinary HTTP LAN address.
4. If intentionally setting `ARCTOS_API_BASE`, rebuild after changing it and
   configure backend CORS and secure cookies for that separate origin.
