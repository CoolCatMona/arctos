# Mobile redesign previews

Captured from the Rust/Dioxus application against a migrated, isolated copy
of a database snapshot. Public views show existing events, leagues, and profiles;
organizer controls and registration examples use added fictional demo accounts.
California Jugger/Light is used unless noted otherwise.

Only the top-level Events, Leagues, Players, Teams, and Stones introductions
can be collapsed with a chevron and restored in Appearance settings. Profile, tournament, and league detail headers stay visible.

- `home-mobile.png`: 360px Events page with date filters and contextual Organize control.
- `home-dismissed-mobile.png`: 360px viewport after dismissing the Events introduction.
- `home-desktop.png`: 1280px full page.
- `organize-mobile.png`: 360px Events page with its tournament creation action open.
- `leagues-mobile.png`: 360px league directory with Active/Past filters and season dates.
- `event-desktop.png`: refreshed 1280px event banner, navigation, and information.
- `event-mobile.png`: refreshed 390px full page with event banner and navigation tabs.
- `event-schedule-mobile.png`: refreshed 390px field schedule within the shared event layout.
- `league-mobile.png`: refreshed 390px league banner, navigation, and information.
- `league-event-mobile.png`: refreshed 390px league event with compact registered-team rows.
- `player-profile-mobile.png`: 360px Reid Dye profile with cover, avatar, and information/history tabs.
- `team-profile-mobile.png`: 360px Ursae Majoris profile with cover, avatar, tabs, and a compact past-event count.
- `players-mobile.png`: 360px player directory with past-event counts.
- `home-registration-mobile.png`: pending approval and unpaid badges for a fictional player registration.
- `organizer-tools-mobile.png`: compact organizer menu in the event banner.
- `organizer-settings-mobile.png`: refreshed flat settings with expanded referee options, a Jump to save action, and an explicit independent banner-save boundary.
- `penalty-types-mobile.png`: Add beside the penalty heading, with labeled fields stacked for phones.
- `profile-crop-mobile.png`: refreshed team image positioning with one Zoom control and visible guidance that the profile Save stores the draft.
- `profile-edit-mobile.png`: refreshed editor with unchanged-state feedback beside Save and Cancel.
- `profile-preview-mobile.png`: refreshed unsaved Markdown preview with save-state feedback on a phone.
- `profile-edit-desktop.png`: refreshed live Markdown preview beside the editing fields and shared save toolbar.
- `profile-history-mobile.png`: a plain history table with contained horizontal scrolling.
- `stones-mobile.png`: 360px playback and sound selection controls.
- `match-mobile.png`: refreshed 390px viewport with Correct count, one phase label, primary Start point and secondary Review result.
- `appearance-mobile.png`: 360px viewport with the Appearance panel open in California Jugger/Light.
- `home-dark-mobile.png`: 360px viewport showing discovery in California Jugger/Dark.
- `home-field-mobile.png`: 360px viewport showing discovery in Field/Light.
- `home-neutral-mobile.png`: 360px viewport showing discovery in Neutral/Light.

## Button interaction states — 2026-10-24

These captures accompany the registration, profile, settings, and live match action
changes. Mobile views use a requested 390×844 CSS-pixel viewport; desktop views use
1280×900. The browser captures the content area, so image dimensions may exclude
scrollbars and browser-owned space. Full-page captures extend below the viewport.
The corresponding checks also exercised 320px and short-height layouts.

Normal and error views were captured from the development bundle; the final event,
settings, schedule, league-event, live-count, and finalization-requirement captures
were refreshed from the optimized build. All show the implemented controls; images
are not composited mockups.

| File | State and purpose |
| --- | --- |
| `registration-edit-mobile.png` | Save changes and Cancel stay together; withdrawal has a separate section. |
| `registration-withdraw-mobile.png` | Nested withdrawal confirmation, with Keep registration initially focused. |
| `registration-saving-mobile.png` | In-flight save disables duplicate actions and dismissal. |
| `registration-save-error-mobile.png` | Rejected save retains the draft and shows feedback in the footer. |
| `registration-waiver-mobile.png` | Read-only response fixture demonstrates the readable waiver reminder beside Edit registration. |
| `settings-actions-mobile.png` | Single in-flow Save settings destination. |
| `settings-save-mobile.png` | Failed event settings save focuses the action group and retains the draft. |
| `league-settings-save-mobile.png` | Equivalent feedback and focus on league settings. |
| `profile-discard-mobile.png` | Changed profile asks whether to discard; Keep editing is the safe initial focus. |
| `profile-save-error-mobile.png` | Player profile retains edits and exposes the save error beside the action. |
| `team-profile-edit-error-mobile.png` | The same behavior with team-specific fields and a read-only team-session fixture. |
| `match-start-blocked-mobile.png` | Named-team roster-limit explanations beside disabled Start match. |
| `match-start-ready-mobile.png` | Start match becomes available after both rosters meet the limit. |
| `match-start-pending-mobile.png` | Starting state locks the roster and duplicate submission. |
| `stone-correction-mobile.png` | Explicit count draft with Save correction and Cancel; focus loss does not save. |
| `stone-correction-error-mobile.png` | Retained count draft with Retry and Check current count; conflicting actions remain blocked. |
| `match-winner-mobile.png` | Large, named winner choices with a visible selected state. |
| `match-finalize-requirements-mobile.png` | Visible missing winner/signature requirements explain the disabled final action. |
| `match-finalize-ready-mobile.png` | Deliberate Confirm and finalize match action beside Back to scoring. |
| `match-finalize-pending-mobile.png` | Pending finalization locks winner, signature, and submit controls. |

Failure and pending captures use a loopback test proxy that never forwards mutations.
The error message intentionally identifies an intercepted connection failure. No
registration, waiver, profile, score, or final result was saved while capturing these
states. Signature marks are local test strokes, not submitted signatures. The practice
match is a dated fixture; its elapsed display is not a claim of an event running now.
The waiver reminder and team session are isolated read-only response fixtures. Profile
image selection remains a local draft, using an existing public team image, and was
discarded afterward.

Unchanged images elsewhere in this directory remain evidence of their earlier
capture, rather than a claim that every historical browser check was rerun.

To run the application against a snapshot, use
`uv run python -m scripts.dev_demo --snapshot /absolute/path/to/snapshot.db`.
The source database is opened read-only; migrations and fixtures affect only the
local copy. See [`TESTING.md`](../../TESTING.md#mobile-browser-verification) for
browser checks. Available records, dates, and live timers depend on the database
and capture time. The editor previews use a fictional account and do not save
profile changes; the image is selected locally from the snapshot's existing media.

## Recovery, navigation and validation — 2026-10-24

These 26 captures show the implemented recovery and form states from optimized
frontend bundles. Mobile captures request 390×844 unless noted; the quick-register
capture requests 320×844 and the desktop directory requests 1280×900. Content-area
image dimensions may exclude scrollbars. They use the migrated snapshot copy and
fictional demo records; no data-changing request was forwarded during capture.
Failure messages are supplied by a loopback test harness. The invitation view uses
a fictional read-only team-session, roster and capacity response. All screenshots
are browser captures, not mockups. Earlier captures above retain their own scope.

| File | State and purpose |
| --- | --- |
| `account-retry-mobile.png` | Account check unavailable: one retry notice, with public directory content retained. |
| `events-filter-mobile.png` | Past event search restored from its URL. |
| `leagues-filter-mobile.png` | Active league search restored from its URL. |
| `players-search-mobile.png` | Submitted player query and matching results. |
| `teams-search-mobile.png` | Submitted team query with compact event counts. |
| `teams-search-desktop.png` | The same team search at a 1280×900 viewport. |
| `create-event-field-error-mobile.png` | Inline event-name validation and retained URL. |
| `create-league-field-error-mobile.png` | Equivalent inline league-name validation. |
| `create-event-save-error-mobile.png` | Creation service failure keeps the name and URL draft. |
| `app-validation-mobile.png` | Developer application homepage error associated with its field. |
| `profile-field-error-mobile.png` | Profile validation returns from Preview to the field needing correction. |
| `invitation-check-status-mobile.png` | Fictional team-session fixture: one uncertain request is guarded while another row remains usable. |
| `schedule-delete-confirm-mobile.png` | Named destructive confirmation with Cancel initially focused. |
| `schedule-delete-recovery-mobile.png` | Uncertain schedule deletion retains the editor and offers a read-only status check. |
| `side-event-delete-confirm-mobile.png` | Side-event deletion explains the affected registrations and results. |
| `side-event-delete-recovery-mobile.png` | Retained side event, guarded deletion, status check and 44px Edit action. |
| `side-event-load-error-mobile.png` | Failed side-event read offers retry without an empty editable form. |
| `side-event-edit-retry-mobile.png` | Save error remains beside the action; the draft is retained. |
| `side-event-registration-error-mobile.png` | Selected Staff remains available after an intercepted registration failure. |
| `side-event-results-recovery-mobile.png` | Failed validity change reloads authoritative results and explains finalization prerequisites. |
| `quick-register-search-retry-320.png` | 320×844 viewport: themed Players/Teams tabs, retained query and search retry. |
| `help-tasks-mobile.png` | Task-first Help entry with links into the detailed guide. |
| `help-retry-mobile.png` | Failed Help load offers a local retry. |
| `registration-help-mobile.png` | Shared native registration-help dialog. |
| `match-notes-mobile.png` | Shared native read-only match-notes dialog. |
| `match-review-table-mobile.png` | Result review table with contained horizontal scrolling. |

The checks covered rejected writes, failed/successful reconciliation, retained
drafts, account-check failure versus HTTP 401, search response ordering, encoded
query reloads, Back/Forward and pagination, actual rendered Help anchors, inline
field focus, and native-dialog Escape/return focus. Requested 320×568 layouts and
a desktop directory were also inspected. These captures do not establish full
assistive-technology, physical-device, integration or participant testing.
