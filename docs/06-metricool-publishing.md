# 06 – Metricool Publishing

ContentOS pushes finished reels to **Metricool** for scheduling on Instagram Reels, TikTok, and YouTube Shorts, then reads back posted status and metrics. Metricool is the only publishing integration — no direct platform APIs.

## Flow

```
reel `edited`
  → Publish screen: captions/hashtags per platform (templates + overrides)
  → assign schedule times (from calendar target date + per-platform time defaults)
  → PUSH: upload video + create scheduled posts via Metricool API   → `scheduled`
  → metricool_sync job (every 30 min): poll statuses               → `published`
  → verification: post has live URL / appears in published list     → `verified`
  → metrics pulls (daily): views/likes/comments/shares/saves per post
```

Per-platform `posts` rows track everything independently — TikTok can be `published` while YouTube is still `scheduled`; the reel itself flips `posted` when the first platform publishes and `verified` when **all** its non-canceled posts verify.

## API client (Rust `metricool` module)

- Auth: `X-Mc-Auth` API token + user/blog id, stored in **Windows Credential Manager**, entered once in Settings with a "test connection" button.
- Core operations: upload media, create/update/delete scheduled posts, list scheduled + published posts, fetch post metrics.
- All calls behind a trait (`PublishingProvider`) so the CSV fallback (below) and any future provider implement the same interface.
- Rate limiting + retries with backoff; every push is a `metricool_push` job → visible progress, resumable, failures land in the Problems feed with the API error text and a retry button.
- **Idempotency:** `posts.metricool_post_id` is recorded before status flips; a re-push updates the existing Metricool post instead of duplicating it. Sync reconciles by that id.
- Edits after push (caption fix, time change) → app marks the post `dirty` and offers "update on Metricool".

## ⚠ API access is plan-gated — verify before building

Metricool's public API is available on **Advanced/Enterprise tiers**, and its documented surface has historically centered on analytics + scheduling for connected brands. *Current status: the account is on the free plan; the subscription upgrade happens when Phase 4 reaches this limitation. Until then, development targets the CSV fallback path first, with the API client built behind the same `PublishingProvider` trait.* **Phase-4 kickoff task #1 is a spike:** confirm with the current Metricool API docs/account that (a) the plan has API access, (b) scheduling with **video upload** is supported for IG Reels + TikTok + YT Shorts, (c) posted-status and metrics reads cover all three. The `PublishingProvider` trait exists precisely so findings here don't ripple through the app.

## Fallback: CSV bulk scheduling

Metricool supports bulk-scheduling via CSV import in its planner. If API access is unavailable (or breaks), ContentOS exports a **Metricool-format CSV** for any selected set of posts plus a folder of the corresponding finals for manual upload. Status then comes from the sync-read side (if readable) or a manual "mark as posted" sweep in the Publish board. Not the dream, but 100/week survives an API outage.

## Scheduling intelligence (app-side, not Metricool-side)

- **Time-slot defaults** per platform per weekday (e.g. IG 09:00/13:00/19:00), editable in Settings; bulk-fill assigns `edited` reels to open slots in target-date order.
- **Capacity view:** calendar shows posts/day per platform against your target; gaps and pile-ups are visually loud.
- **Collision guard:** two posts on one platform within N minutes → warning before push.
- **Cross-platform stagger** (optional): auto-offset TikTok/YT times ±30 min from the IG time.

## Verification & the Posted board

The Publish screen's board answers the only question that matters at this volume — *"did everything actually go out?"*:

- Columns: Needs captions → Ready to push → Scheduled → Published → Verified, plus a red **Failed** lane.
- Daily digest tile on the Dashboard: "Yesterday: 14/14 scheduled posts verified live. Today: 15 scheduled, 2 reels still missing captions."
- A `scheduled` post whose time is >2 h past with no `published` status → escalated to Problems (this is the "Metricool silently didn't post it" catch).
