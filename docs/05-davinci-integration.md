# 05 – DaVinci Resolve Integration

Deep integration, three parts: **staged handoff folders out → pre-built timelines out → rendered exports matched back in.** The goal: when you sit down to edit, every reel's timeline already exists with clips in order — editing starts at "tighten and polish", not "find and assemble".

## Part 1 — Handoff staging (`generate_handoff`)

Run per shoot batch (or per selected set of reels) once reels are `shot`. The app builds a disposable staging tree under `02_HANDOFF/`:

```
02_HANDOFF/B012/
  _IMPORT_ME.fcpxml               ← all timelines for this batch (see part 2)
  _BATCH_SHEET.html               ← printable/openable summary: every reel,
                                    its blocks, selected takes, durations
  R0142_gym-myths/
    01_HOOK_HK0031.mp4            ← hardlink (same volume) or copy of the
    02_BODY_BD0007.mp4              selected take, renamed to edit order
    03_CTA_CT0002.mp4
    R0142_script.txt              ← full script text for reference/captions
  R0143_gym-myths-2/
    ...
```

- **Hardlinks, not copies**, when staging and raw live on the same NTFS volume (instant, zero extra disk). Falls back to copy across volumes; the NAS phase uses copies to a local scratch staging root for editing speed.
- Shared components appear in each reel folder that uses them — hardlinks make this free.
- Staging is **regenerable and disposable**: change a selected take → regenerate; a "clean handoff" action deletes staging for batches whose reels are all `edited`.
- Reels flip to `assembled` on generation.

## Part 2 — Generated timelines (FCPXML)

The app writes one FCPXML file per batch containing **one timeline per reel**, each timeline being the reel's selected takes laid end-to-end in block order (hook → body → CTA / segment order for standalone).

In Resolve: `File ▸ Import ▸ Timeline ▸ _IMPORT_ME.fcpxml` → every reel appears as a ready-made timeline with media auto-linked (FCPXML carries file paths pointing into the staging folders).

Technical notes:

- Target **FCPXML 1.8** — the dialect Resolve imports most reliably. Generation lives in a small, heavily-tested Rust module (`davinci::fcpxml`); the format is XML with `<resource>`/`<asset>` declarations and `<spine>` sequences of `<asset-clip>` elements.
- **Timeline settings:** 1080×1920 vertical, frame rate taken from the clips (mismatched-fps clips in one reel are flagged at handoff time, before you're in Resolve wondering why audio drifts).
- Clip durations come from ffprobe; clips are laid in **full length** — trimming is the human's job in Resolve.
- Timeline naming = `R0142_gym-myths` so Resolve's render filename inherits the reel code automatically (which part 3 depends on).
- **Validation harness in CI:** golden-file tests for the XML, plus a documented manual smoke-test checklist per Resolve version (Resolve's importer quirks change between versions; we pin what we've verified in `docs/compat/resolve-versions.md` as we go).

### Considered and rejected (for now)

- **Resolve's Python scripting API** (auto-create projects/bins/timelines inside Resolve, trigger renders): more power, but it's fragile across versions, requires Resolve running with scripting enabled, and external scripting needs Studio. FCPXML is inspectable, versionable, and testable offline. The `davinci` module keeps a clean seam so a scripting backend can be added later without touching the rest of the app.
- **EDL:** too lossy (single track, no clip names worth having).

## Part 3 — Exports back in

Covered in [File System](04-file-system.md#watch-folders): render from Resolve into `03_EXPORTS/`; the watcher matches `R####` in the filename to the reel, files the final into `04_FINALS/`, links it as `final_asset_id`, and flips the reel to `edited`. Recommended Resolve render preset (documented in-app): filename = timeline name, so zero manual naming ever happens.

**Preflight before a reel can leave `edited`:** duration within platform limits, vertical aspect, audio present, codec/container in the allowlist (H.264/H.265 MP4). Failures land in the Problems feed with the reason.

## The editing-day loop (what it feels like)

1. Assemble screen → select batch B012 → **Generate handoff** (10 s).
2. Open Resolve → import `_IMPORT_ME.fcpxml` → 25 timelines appear, media linked.
3. Edit down the list; render all to `03_EXPORTS/` with the preset.
4. Alt-tab to ContentOS: finals matched, reels `edited`, Publish queue already filling.
