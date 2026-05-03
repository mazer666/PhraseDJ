# Function & Command Reference (Current Surface)

**Audience:** UI and backend contributors  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Tauri command functions

### Deck commands (`src-tauri/src/commands/deck.rs`)
- `deck_load`
- `deck_play`
- `deck_pause`
- `deck_seek`
- `deck_state`
- `deck_set_tempo_ratio`
- `deck_sync`
- `deck_nudge_tempo`
- `deck_waveform`

### Mixer commands (`src-tauri/src/commands/mixer.rs`)
- `mixer_set_fader`
- `mixer_set_crossfader`
- `mixer_set_master_gain`
- `mixer_set_stem_gain`

### Library commands (`src-tauri/src/commands/library.rs`)
- `library_import_file`
- `library_scan_folder`
- `library_recent`
- `library_search`

### App/settings commands (`src-tauri/src/commands/app.rs`)
- `app_version`
- `app_status`
- `keymap_load`
- `settings_load`
- `settings_save`

## Notes
- This page is an index/reference, not an API contract.
- Keep it in sync when commands are added/renamed/removed.
