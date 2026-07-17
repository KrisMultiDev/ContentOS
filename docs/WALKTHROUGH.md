# ContentOS — Guided Walkthrough & Inspection

This is our testing script. We go through the app **one page at a time, in the order you'd really use it.** On each page you click *everything*, and write down anything that's confusing, broken, ugly, or missing. Then we fix and repeat.

Keep it simple: for every page, ask yourself four questions —
1. **Did I understand what this page is for without being told?**
2. **Did every button do something, and the right thing?**
3. **When I did something wrong, did it tell me why?**
4. **What felt clunky or slow?**

Write the answers in `docs/FEEDBACK.md` (there's a slot per page).

---

## The direction — how the app flows

The app is an assembly line. Work moves left to right. Each arrow is "this page hands off to the next":

```
   SETTINGS  (one-time setup: media folder · pillars · weekly target)
      │
      ▼
   IDEAS  ──▶  SCRIPTS  ──▶  CALENDAR  ──▶  SHOOT  ──▶  LIBRARY  ──▶  ASSEMBLE  ──▶  PUBLISH
   capture     write the      pick the      record     bring in      DaVinci        captions,
   the idea    script         posting day   the clips  footage,      handoff +      schedule,
                                                        pick takes    finals back    export, track
      ▲                                                                                   │
      └───────────────────  DASHBOARD watches the whole line the entire time  ────────────┘
```

**In one sentence each:**
1. **Settings** — point the app at a media folder, set up your content categories (pillars), set your weekly goal. Do this once.
2. **Ideas** — brain-dump reel ideas. Promote the good ones into real reels.
3. **Scripts** — write each reel's script from blocks (hook / body / CTA). Reuse pieces from the component library.
4. **Calendar** — decide which day each reel posts.
5. **Shoot** — group reels into a recording session and film them with the teleprompter.
6. **Library** — copy your footage in, and match clips to the shots you planned.
7. **Assemble** — send everything to DaVinci as ready-made timelines, and catch the finished videos coming back.
8. **Publish** — write captions, set times, export to Metricool, and track what actually went live.
9. **Dashboard** — the overview you glance at the whole time to see what's stuck.

---

## Page 1 — Settings  *(do this first)*

**What it's for:** one-time setup.

**Try everything:**
1. **Storage roots** → click **Choose folder…** → pick a folder → **Add root**. Then check that folder in Explorer — all 7 sub-folders + a `_READ_ME_FIRST.txt` should now exist.
2. Click **Open** on the root — Explorer should open that folder.
3. Note the **Free** space and **Online** badge.
4. (Don't do it now, but read) **Re-point…** = move the whole library elsewhere; **Remove** = forget it (files stay).
5. **Content pillars** → type a name → click a **color square** → set a target → **Add pillar**. Add two with different colors.
6. **Weekly target** → change the number → **Save**.
7. **Appearance** → toggle **Dark / Light**.
8. **Database & backups** → **Back up now** → confirm the path it reports.

**Inspect:** Is it obvious this is where you start? Are the colors clear now they're swatches? Anything you expected to set here that's missing?

---

## Page 2 — Ideas

**What it's for:** capture ideas fast; promote the good ones.

**Try everything:**
1. Type an idea, pick a pillar, **Add idea** (or just press Enter).
2. Add 3–4 ideas quickly.
3. In the backlog, change an idea's **pillar** with its dropdown.
4. **Promote to reel** on one → it should jump you to the Scripts editor with a new reel.
5. **Kill** another → it disappears from the open list.

**Inspect:** Is capture fast enough to not break your flow? Is it clear what Promote vs Kill do?

---

## Page 3 — Scripts

**What it's for:** write reels; manage reusable script pieces.

**Try everything — Reels tab:**
1. Type a title → **New**. It appears in the left list and opens.
2. Edit the **title**; pick a **pillar**.
3. Add a **Notes** line.
4. **+ Hook** → type text in the block.
5. **+ Body** → this time click **Link component…** → search → pick one (make one in the Library tab first if empty).
6. **+ CTA** → type text.
7. Use **↑ / ↓** to reorder a block; **✕** to delete one.
8. **Save script**. Watch the status change to `scripted`.
9. Try the **search** box (type a word from your script) and the **status filter**.

**Try everything — Component library tab:**
1. Pick a kind (hook/body/cta) → type text → add tags → **Add**.
2. **Search** and **filter by kind**.
3. **Edit** a component inline; **Archive** one.
4. Note the **Used** count (how many reels link it).

**Inspect:** Is the block editor obvious? Is the difference between "type text" and "link a component" clear? Does the status dropdown make sense (or should it be hidden)?

---

## Page 4 — Calendar

**What it's for:** decide which day each reel posts.

**Try everything:**
1. Reels you made sit in the **Unscheduled** tray (right).
2. **Drag** one onto a day. (If drag feels bad, use the little **date box** on the tray item instead — tell me which you prefer.)
3. **Drag** a placed reel to a different day.
4. **Drag** a placed reel back to the tray to unschedule it.
5. Watch the **number at the end of each week row** — grey → amber → green as it fills toward your target.
6. **Prev / next month** arrows.
7. **Double-click** a reel chip → opens it in the editor.

**Inspect:** Did drag work? Was the date-box fallback obvious? Are pillar colors helpful here?

---

## Page 5 — Shoot

**What it's for:** group reels into a recording session; film with the teleprompter.

**Try everything:**
1. Type a batch name → optional date → **New**.
2. Select the batch. Click **Add reels…** → tick your scripted reels → **Add to shot list**.
3. Look at the **shot list** table: each row is one thing to film. Note the **Clip** code, **Used by** (which reels share it), and **Status**.
4. Change the batch **status** dropdown (planning → ready → shooting → done).
5. Click **Record Mode**:
   - Read the big script text.
   - **Space** = mark recorded + go next. **K** = skip. **← →** = move. **Esc** = exit.
   - Watch the progress bar of ticks at the top.

**Inspect:** Was it clear a "batch" = a recording session? Did the shot-list de-duplication make sense (shared hook shows once)? Was Record Mode comfortable to read from?

---

## Page 6 — Library

**What it's for:** bring footage in and match clips to shots.

**Try everything:**
1. Copy a few small video files into your media folder's **00_INBOX**. (Use the **Open inbox** button to find it. Use 3 *different* clips — identical files get skipped as duplicates on purpose.)
2. **Scan inbox** → read the report (how many new / duplicates).
3. Clips appear on the left. Pick your **batch** on the right.
4. **Tick** a clip → click **← takes** on the shot it belongs to. The file gets renamed + filed automatically.
5. Do that for each shot. **Star-rate** the takes.
6. Click **Select** on the best take per shot.
7. When every shot of a reel has a selected take, that reel becomes `shot`.

**Inspect:** Was the "tick clips then click ← takes" flow clear? Did you understand stars vs Select? Any surprise?

---

## Page 7 — Assemble

**What it's for:** hand off to DaVinci and catch the finished videos.

**Try everything:**
1. **Handoff (top):** pick your batch → **Generate handoff**. Read the report.
2. **Open folder** → look at what was made: a folder per reel with renamed clips, a script `.txt`, and `_IMPORT_ME.fcpxml`.
3. (If you have Resolve) File ▸ Import ▸ Timeline → that fcpxml. Otherwise skip.
4. **Finals (bottom):** to fake a render, copy any video into **03_EXPORTS** and rename it with your reel code, e.g. `R0001_final.mp4`.
5. **Scan exports** → it matches the file to the reel → **File it**. The reel becomes `edited`.

**Inspect:** Was the two-step (out, then back in) clear? Did the report tell you enough? Anything about the DaVinci step you'd want explained?

---

## Page 8 — Publish

**What it's for:** captions, scheduling, export to Metricool, and tracking.

**Try everything:**
1. **Queue** panel: your `edited` reels appear → **Queue** one (or **Queue all**). Makes IG + TikTok + YouTube posts.
2. **Captions & schedule** panel: write a **caption** + **hashtags** for a reel; set a **time** per platform; or click **Fill times from calendar dates**. **Save**.
3. **Export** panel: **Export ready posts** → **Open folder** → see the CSV + videos named by slot.
4. **Tracking** panel: mark a post **Published**, then **Verify live**. Try the **Failed** and **Re-queue** buttons.

**Inspect:** Was the 4-step order clear? Did you know what "ready" means for export? Was the tracking board understandable?

---

## Page 9 — Dashboard

**What it's for:** the overview you check throughout the week.

**Try everything:**
1. Read the **tiles**: scheduled this week, edited/awaiting captions, verified, stuck, problems.
2. Click a **pipeline funnel** stage → jumps to that filtered list.
3. If anything's **stuck** (untouched 4+ days) it lists here — click to open.
4. Press **Ctrl+K** anywhere → the **command palette**: jump to any page, or search reels/components by typing.

**Inspect:** Does the dashboard answer "what should I do next?" at a glance? Is anything you'd want on it missing?

---

## After you've been through all 9

Fill in `docs/FEEDBACK.md` and paste it back to me (or just paste your notes in chat). Then I fix everything, you re-test, and we repeat until the list is empty. Be brutal — "this word confused me for 3 seconds" is exactly the kind of note that makes it great.
