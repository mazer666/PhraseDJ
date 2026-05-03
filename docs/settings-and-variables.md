# Settings and Variables Reference

**Audience:** Developers, QA, maintainers  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Source files
- `config/defaults.toml` (shipped defaults)
- `config/schema.json` (validation contract)
- `config/keymap.toml` (default key bindings)

## Runtime settings groups

### `audio`
- `sample_rate` (int enum): output sample rate.
- `buffer_size` (int enum): callback frames; latency/CPU tradeoff.
- `output_device` (string): output device name.
- `cue_device` (string): cue/headphone output.
- `pitch_range_pct` (number): tempo fader range.

### `library`
- `music_root` (string): root for relative path handling.
- `follow_symlinks` (bool): include symlinks while scanning.
- `ignore_files` (string[]): skip patterns.
- `backup_count` (int): DB backup retention.
- `stem_cache_gb` (number): cache cap for stems.

### `stems`
- `model` (string): stem separation model id.
- `stem_count` (int enum): 4 or 6 stems.
- `auto_analyse` (bool): analyse on import.
- `max_parallel_jobs` (int): analysis concurrency.

### `lyrics`
- `online_lookup` (bool): internet lyric lookups opt-in.
- `online_service_url` (uri): provider endpoint.
- `whisper_model` (string): local align model.
- `online_timeout_secs` (int): network timeout.

### `ui`
- `target_fps` (int enum): waveform redraw rate.
- `default_mode` (enum): `classic`, `stem`, or `macro`.

### `network`
- `update_check` (bool): update check opt-in.

## Keymap variables
The keymap maps key combos to intent strings (e.g., `deck.A.toggle_play`, `macro.toggle_record`).

## App-level variables exposed via commands
- version/status via `app_version`, `app_status`
- key bindings via `keymap_load`
- settings read/write via `settings_load`, `settings_save`
