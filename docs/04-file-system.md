# 04 – File System

## Principles

1. **The filesystem is the source of media; the DB is the source of truth about it.** ContentOS indexes and links files — it renames/moves only inside folders it owns (staging, exports triage), never touches camera originals without an explicit action.
2. **Every generated filename carries a code** (`R0142`, `B012`, `HK0031`). Filenames are the contract between ContentOS, DaVinci, and you-at-2am.
3. **Everything lives under a storage root** so the PC → Unraid move is a re-point ([Architecture](02-architecture.md)).

## Canonical library layout

Under the `media` root (today `D:\ContentOS`, later `\\unraid\content`):

```
<root>/
  .contentos-root                 ← root marker (UUID)
  00_INBOX/                       ← dump zone: camera offloads land here (watched)
  01_RAW/
    B012_2026-07-14_studio/       ← one folder per shoot batch
      B012_HK0031_T01.mp4         ← batch_component_take
      B012_HK0031_T02.mp4
      B012_R0142-S03_T01.mp4      ← standalone block: reel-blockpos_take
      ...
  02_HANDOFF/                     ← DaVinci staging (app-generated, disposable)
    B012/                         ← see doc 05 for contents
  03_EXPORTS/                     ← DaVinci render target (watched)
    R0142_gym-myths_FINAL.mp4
  04_FINALS/                      ← verified finals, filed by month
    2026-07/
      R0142_gym-myths_FINAL.mp4
  05_ARCHIVE/                     ← killed/expired material (manual or rule-based)
```

## Naming grammar

```
raw take        B{batch}_{clipkey}_T{take}.mp4
  clipkey       component code (HK0031/BD0007/CT0002)
                or reel-block (R0142-S03) for standalone blocks
final render    R{reel}_{slug}_FINAL[_vN].mp4
handoff clip    {order}_{kind}_{clipkey}.mp4     e.g. 01_HOOK_HK0031.mp4
```

Parsing is forgiving: the export matcher only needs to find `R\d{4}` anywhere in a filename; everything else is convention for humans.

## Ingest flow (recording day → library)

1. Offload SD card / phone into `00_INBOX/` (or straight into the batch folder if you prefer).
2. Watcher picks up new files → ffprobe + hash + thumbnail → they appear in **Library ▸ Inbox** sorted by recorded time.
3. **Triage view:** the active shoot batch's shot list sits beside the inbox clips. Because you recorded in shot-list order, linking is mostly "select run of clips → assign to shot" (multi-take runs auto-number `T01..Tn`). Linking a clip moves it into `01_RAW/B###.../` with its canonical name.
4. Pick the selected take per shot (keyboard: 1–5 rating, Enter selects). When all of a reel's blocks have selected takes, the reel flips to `shot`.

Design intent: **triage of a 60-clip session should take under 15 minutes.** Every interaction is keyboard-first.

## Watch folders

| Folder | Watcher behavior |
|---|---|
| `00_INBOX` | index + thumbnail + surface in triage |
| `03_EXPORTS` | wait for file-size stability (render in progress), then run export matching: parse `R####` → propose match to a reel in `editing`/`assembled` → on confirm (or auto-confirm setting), set `final_asset_id`, move file to `04_FINALS/YYYY-MM/`, flip reel to `edited` |

Unmatched exports and ambiguous matches (two files claiming `R0142`: keep `_v2`? both?) go to the Problems feed — never silently dropped.

## Integrity

- `verify_files` job spot-checks linked assets exist (full sweep weekly, missing → flagged, hash-relink attempted per [Architecture](02-architecture.md)).
- Duplicate detection by blake3 on import (same clip offloaded twice → dedupe prompt).
- Disk-space watermark warning on the media root (raw 4K at 100/wk eats drives; the dashboard shows projected weeks-until-full).
- **Retention rules** (configurable, off by default): e.g. "unselected takes of `verified` reels older than 90 days → move to `05_ARCHIVE`" — surfaced as a review list, one-click apply, never automatic deletion.
