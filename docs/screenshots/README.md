# Application screenshots

Visual references for the implemented interface. Phone captures use widths from
320px to 390px; desktop captures use 1280px. Full-page images extend below the
viewport. Landscape images demonstrate short-height scrolling. These are browser
captures, not device screenshots or a record of test coverage.

Public pages use a local copy of existing event/profile data. Editing, pending,
and failure examples use fictional demo accounts or intercepted responses; no
real registration, score, signature, or private record was changed for those
captures. Delay and error examples use a local proxy that blocks data writes. Fixture
error wording is illustrative. Available records, dates, and
timers depend on the database and capture time.

Use the [local setup guide](../../README.md#running-the-app-locally) to explore
the current app and [TESTING.md](../../TESTING.md#mobile-browser-verification)
for verification guidance. Screenshots do not establish keyboard, screen-reader,
Safari, physical-device, or third-party integration support.

## Discovery, appearance, and detail pages

| Image | What it shows |
| --- | --- |
| [home-mobile.png](home-mobile.png) | 360px Events page with date filters and contextual Organize control. |
| [home-dismissed-mobile.png](home-dismissed-mobile.png) | 360px viewport after dismissing the Events introduction. |
| [home-desktop.png](home-desktop.png) | 1280px full page. |
| [organize-mobile.png](organize-mobile.png) | 360px Events page with its tournament creation action open. |
| [leagues-mobile.png](leagues-mobile.png) | 360px league directory with Active/Past filters and season dates. |
| [event-desktop.png](event-desktop.png) | 1280px event banner, navigation, and information. |
| [event-mobile.png](event-mobile.png) | 390px full page with event banner and navigation tabs. |
| [event-schedule-mobile.png](event-schedule-mobile.png) | 390px field schedule within the shared event layout. |
| [league-mobile.png](league-mobile.png) | 390px league banner, navigation, and information. |
| [league-event-mobile.png](league-event-mobile.png) | 390px league event with compact registered-team rows. |
| [player-profile-mobile.png](player-profile-mobile.png) | 360px Reid Dye profile with cover, avatar, and information/history tabs. |
| [team-profile-mobile.png](team-profile-mobile.png) | 360px Ursae Majoris profile with cover, avatar, tabs, and a compact past-event count. |
| [players-mobile.png](players-mobile.png) | 360px player directory with past-event counts. |
| [home-registration-mobile.png](home-registration-mobile.png) | pending approval and unpaid badges for a fictional player registration. |
| [organizer-tools-mobile.png](organizer-tools-mobile.png) | compact organizer menu in the event banner. |
| [organizer-settings-mobile.png](organizer-settings-mobile.png) | flat settings with expanded referee options, a Jump to save action, and an explicit independent banner-save boundary. |
| [penalty-types-mobile.png](penalty-types-mobile.png) | Add beside the penalty heading, with labeled fields stacked for phones. |
| [profile-crop-mobile.png](profile-crop-mobile.png) | team image positioning with one Zoom control and visible guidance that the profile Save stores the draft. |
| [profile-edit-mobile.png](profile-edit-mobile.png) | editor with unchanged-state feedback beside Save and Cancel. |
| [profile-preview-mobile.png](profile-preview-mobile.png) | unsaved Markdown preview with save-state feedback on a phone. |
| [profile-edit-desktop.png](profile-edit-desktop.png) | live Markdown preview beside the editing fields and shared save toolbar. |
| [profile-history-mobile.png](profile-history-mobile.png) | a plain history table with contained horizontal scrolling. |
| [stones-mobile.png](stones-mobile.png) | 360px playback and sound selection controls. |
| [match-mobile.png](match-mobile.png) | 390px viewport with Correct count, one phase label, primary Start point and secondary Review result. |
| [appearance-mobile.png](appearance-mobile.png) | 360px viewport with the Appearance panel open in California Jugger/Light. |
| [home-dark-mobile.png](home-dark-mobile.png) | 360px viewport showing discovery in California Jugger/Dark. |
| [home-field-mobile.png](home-field-mobile.png) | 360px viewport showing discovery in Field/Light. |
| [home-neutral-mobile.png](home-neutral-mobile.png) | 360px viewport showing discovery in Neutral/Light. |

## Forms, actions, and recovery

| Image | What it shows |
| --- | --- |
| [registration-edit-mobile.png](registration-edit-mobile.png) | Save changes and Cancel stay together; withdrawal has a separate section. |
| [registration-withdraw-mobile.png](registration-withdraw-mobile.png) | Nested withdrawal confirmation, with Keep registration initially focused. |
| [registration-saving-mobile.png](registration-saving-mobile.png) | In-flight save disables duplicate actions and dismissal. |
| [registration-save-error-mobile.png](registration-save-error-mobile.png) | Rejected save retains the draft and shows feedback in the footer. |
| [registration-waiver-mobile.png](registration-waiver-mobile.png) | Read-only response fixture demonstrates the readable waiver reminder beside Edit registration. |
| [settings-actions-mobile.png](settings-actions-mobile.png) | Single in-flow Save settings destination. |
| [settings-save-mobile.png](settings-save-mobile.png) | Failed event settings save focuses the action group and retains the draft. |
| [league-settings-save-mobile.png](league-settings-save-mobile.png) | Equivalent feedback and focus on league settings. |
| [profile-discard-mobile.png](profile-discard-mobile.png) | Changed profile asks whether to discard; Keep editing is the safe initial focus. |
| [profile-save-error-mobile.png](profile-save-error-mobile.png) | Player profile retains edits and exposes the save error beside the action. |
| [team-profile-edit-error-mobile.png](team-profile-edit-error-mobile.png) | The same behavior with team-specific fields and a read-only team-session fixture. |
| [match-start-blocked-mobile.png](match-start-blocked-mobile.png) | Named-team roster-limit explanations beside disabled Start match. |
| [match-start-ready-mobile.png](match-start-ready-mobile.png) | Start match becomes available after both rosters meet the limit. |
| [match-start-pending-mobile.png](match-start-pending-mobile.png) | Starting state locks the roster and duplicate submission. |
| [stone-correction-mobile.png](stone-correction-mobile.png) | Explicit count draft with Save correction and Cancel; focus loss does not save. |
| [stone-correction-error-mobile.png](stone-correction-error-mobile.png) | Retained count draft with Retry and Check current count; conflicting actions remain blocked. |
| [match-winner-mobile.png](match-winner-mobile.png) | Large, named winner choices with a visible selected state. |
| [match-finalize-requirements-mobile.png](match-finalize-requirements-mobile.png) | Visible missing winner/signature requirements explain the disabled final action. |
| [match-finalize-ready-mobile.png](match-finalize-ready-mobile.png) | Deliberate Confirm and finalize match action beside Back to scoring. |
| [match-finalize-pending-mobile.png](match-finalize-pending-mobile.png) | Pending finalization locks winner, signature, and submit controls. |
| [account-retry-mobile.png](account-retry-mobile.png) | Account check unavailable: one retry notice, with public directory content retained. |
| [events-filter-mobile.png](events-filter-mobile.png) | Past event search restored from its URL. |
| [leagues-filter-mobile.png](leagues-filter-mobile.png) | Active league search restored from its URL. |
| [players-search-mobile.png](players-search-mobile.png) | Submitted player query and matching results. |
| [teams-search-mobile.png](teams-search-mobile.png) | Submitted team query with compact event counts. |
| [teams-search-desktop.png](teams-search-desktop.png) | The same team search at a 1280×900 viewport. |
| [create-event-field-error-mobile.png](create-event-field-error-mobile.png) | Inline event-name validation and retained URL. |
| [create-league-field-error-mobile.png](create-league-field-error-mobile.png) | Equivalent inline league-name validation. |
| [create-event-save-error-mobile.png](create-event-save-error-mobile.png) | Creation service failure keeps the name and URL draft. |
| [app-validation-mobile.png](app-validation-mobile.png) | Developer application homepage error associated with its field. |
| [profile-field-error-mobile.png](profile-field-error-mobile.png) | Profile validation returns from Preview to the field needing correction. |
| [invitation-check-status-mobile.png](invitation-check-status-mobile.png) | Fictional team-session fixture: one uncertain request is guarded while another row remains usable. |
| [schedule-delete-confirm-mobile.png](schedule-delete-confirm-mobile.png) | Named destructive confirmation with Cancel initially focused. |
| [schedule-delete-recovery-mobile.png](schedule-delete-recovery-mobile.png) | Uncertain schedule deletion retains the editor and offers a read-only status check. |
| [side-event-delete-confirm-mobile.png](side-event-delete-confirm-mobile.png) | Side-event deletion explains the affected registrations and results. |
| [side-event-delete-recovery-mobile.png](side-event-delete-recovery-mobile.png) | Retained side event, guarded deletion, status check and 44px Edit action. |
| [side-event-load-error-mobile.png](side-event-load-error-mobile.png) | Failed side-event read offers retry without an empty editable form. |
| [side-event-edit-retry-mobile.png](side-event-edit-retry-mobile.png) | Save error remains beside the action; the draft is retained. |
| [side-event-registration-error-mobile.png](side-event-registration-error-mobile.png) | Selected Staff remains available after an intercepted registration failure. |
| [side-event-results-recovery-mobile.png](side-event-results-recovery-mobile.png) | Failed validity change reloads authoritative results and explains finalization prerequisites. |
| [quick-register-search-retry-320.png](quick-register-search-retry-320.png) | 320×844 viewport: themed Players/Teams tabs, retained query and search retry. |
| [help-tasks-mobile.png](help-tasks-mobile.png) | Task-first Help entry with links into the detailed guide. |
| [help-retry-mobile.png](help-retry-mobile.png) | Failed Help load offers a local retry. |
| [registration-help-mobile.png](registration-help-mobile.png) | Native registration-help dialog. |
| [match-notes-mobile.png](match-notes-mobile.png) | Native read-only match-notes dialog. |
| [match-review-table-mobile.png](match-review-table-mobile.png) | Result review table with contained horizontal scrolling. |
| [account-check-pending-mobile.png](account-check-pending-mobile.png) | 390×844 account check in progress, without showing a false signed-out state. |
| [profile-load-pending-mobile.png](profile-load-pending-mobile.png) | 390×844 profile editor loading the current record before showing editable fields. |
| [profile-load-retry-mobile.png](profile-load-retry-mobile.png) | 390×844 profile read failure with an explicit Retry action. |
| [sign-in-request-retry-mobile.png](sign-in-request-retry-mobile.png) | 390×844 sign-in request failure with fields retained for retry. |
| [match-link-mobile.png](match-link-mobile.png) | 390×844 match view reached through its permanent link. |
| [forms-login-form-mobile.png](forms-login-form-mobile.png) | Sign-in fields and a single, comfortably sized account-switch link. |
| [forms-register-form-mobile.png](forms-register-form-mobile.png) | Full account form with labeled inputs, hints and the matching sign-in link. |
| [forms-account-setup-retry-mobile.png](forms-account-setup-retry-mobile.png) | Account setup read failure offers Retry without exposing an empty editable record. |
| [forms-account-profile-validation-mobile.png](forms-account-profile-validation-mobile.png) | Focused display-name error with the newer username result retained. |
| [forms-injury-load-retry-mobile.png](forms-injury-load-retry-mobile.png) | Fictional injury record read failure offers Retry; no editable blank record. |
| [forms-injury-save-recovery-mobile.png](forms-injury-save-recovery-mobile.png) | Rejected fictional injury save retains the note, date and privacy setting. |
| [forms-league-event-validation-mobile.png](forms-league-event-validation-mobile.png) | League event creation keeps the URL draft and identifies the invalid name. |
| [forms-event-delete-mobile.png](forms-event-delete-mobile.png) | Native event deletion confirmation starts focus on Cancel. |
| [forms-event-delete-recovery-mobile.png](forms-event-delete-recovery-mobile.png) | Rejected event deletion preserves the typed URL and focuses feedback. |
| [forms-league-delete-mobile.png](forms-league-delete-mobile.png) | League deletion uses the same confirmation structure. |
| [forms-dialog-actions-landscape.png](forms-dialog-actions-landscape.png) | Scrolled short-height dialog shows reachable bottom actions; title is above the scroll position. |
| [forms-match-start-explanation-mobile.png](forms-match-start-explanation-mobile.png) | Match-start explanation in a native dialog. |
| [forms-match-setup-retry-mobile.png](forms-match-setup-retry-mobile.png) | Force Start setup failure leaves Cancel and a read-only Retry available. |
| [forms-force-start-mobile.png](forms-force-start-mobile.png) | Loaded Force Start form with named team and referee fields. |
| [forms-force-start-recovery-mobile.png](forms-force-start-recovery-mobile.png) | Rejected Force Start retains choices and displays feedback beside its actions. |
| [forms-profile-penalty-dialog-mobile.png](forms-profile-penalty-dialog-mobile.png) | Player penalty description uses a named native dialog; description is fictional. |
| [forms-match-penalty-dialog-mobile.png](forms-match-penalty-dialog-mobile.png) | The same penalty-description behavior on a match; description is fictional. |
| [forms-schedule-fields-mobile.png](forms-schedule-fields-mobile.png) | Field management in a native dialog with an explicit add-field form. |
| [forms-schedule-fields-recovery-mobile.png](forms-schedule-fields-recovery-mobile.png) | Rejected field creation preserves the entered field name. |
| [forms-schedule-tags-recovery-mobile.png](forms-schedule-tags-recovery-mobile.png) | Rejected tag creation preserves the entered tag name. |
| [forms-schedule-warnings-retry-mobile.png](forms-schedule-warnings-retry-mobile.png) | Warnings read failure settles into a retryable state instead of remaining loading. |
| [forms-schedule-refresh-recovery-mobile.png](forms-schedule-refresh-recovery-mobile.png) | Open script drafts survive failed setup refresh; edits remain locked until Retry succeeds. |
| [forms-schedule-import-recovery-mobile.png](forms-schedule-import-recovery-mobile.png) | Failed TOML import keeps the file picker available for an explicit retry. |
| [forms-schedule-import-warning-mobile.png](forms-schedule-import-warning-mobile.png) | Synthetic successful-import warning; no stored schedule was changed. |
| [forms-schedule-expression-help-mobile.png](forms-schedule-expression-help-mobile.png) | Scrollable expression help can close without dismissing the underlying match editor. |
| [forms-bracket-match-dialog-mobile.png](forms-bracket-match-dialog-mobile.png) | Searchable match picker with contained scrolling and a reachable Close action. |
| [forms-bracket-image-dialog-mobile.png](forms-bracket-image-dialog-mobile.png) | Bracket image dialog with a labeled file picker and visible limit. |
| [forms-bracket-text-dialog-mobile.png](forms-bracket-text-dialog-mobile.png) | Provisional text editor; cancellation rolls back only its fictional new node. |
| [forms-bracket-team-dialog-mobile.png](forms-bracket-team-dialog-mobile.png) | Provisional team-label editor with associated labels and help. |
| [forms-bracket-legacy-recovery-mobile.png](forms-bracket-legacy-recovery-mobile.png) | Readable legacy-bracket failure and wrapping phone rows; all records are fictional fixtures. |
| [forms-calibration-dialog-mobile.png](forms-calibration-dialog-mobile.png) | Calibration shares the dialog layout while retaining snapshot restoration on Cancel. |
| [forms-side-event-keypad-small-phone.png](forms-side-event-keypad-small-phone.png) | 320×568 scorer with every keypad row visible. |
| [forms-side-event-keypad-landscape.png](forms-side-event-keypad-landscape.png) | 844×390 scorer scrolled to its reachable last row. |
