# ContentOS Tutorial — from idea to ready-to-post

This is the complete "how do I actually use this thing" guide. Part 1 explains the big picture in plain words. Part 2 is a **worked example you can follow right now** using fake video files — no camera needed — that takes one reel through the entire pipeline. Part 3 explains the statuses and what to do when something looks wrong.

---

## Part 1 — The big picture

ContentOS moves every reel through the same assembly line:

```
IDEA → SCRIPT → SHOOT → EDIT → PUBLISH
(Ideas)  (Scripts)  (Shoot + Library)  (Assemble + DaVinci)  (Publish)
```

Each screen in the sidebar is one station on that line, in order. A reel's **status** tells you which station it's at, and it moves forward automatically as you work — you almost never set it by hand.

The one-sentence version of each screen:

| Screen | What you do there |
|---|---|
| **Dashboard** | See the whole pipeline at a glance; spot stuck reels and problems |
| **Ideas** | Dump every idea; promote the good ones into reels |
| **Calendar** | Drag reels onto days to plan the week |
| **Scripts** | Write each reel as blocks (hook/body/CTA); reuse components from the library |
| **Shoot** | Group reels into a recording session (batch); teleprompter for the shoot |
| **Library** | Bring your footage in; match clips to shots; pick the best takes |
| **Assemble** | Generate ready-made DaVinci timelines; receive finished renders back |
| **Publish** | Captions, schedule times, Metricool export, and posted tracking |
| **Settings** | Media folder, content pillars, weekly target, backups |

**Two words you'll see everywhere:**

- **Component** — a reusable piece of script (a hook, body, or CTA) that lives in the library. Link one body into 5 reels with 5 different hooks and you've made 5 reels while recording the body **once**.
- **Batch** — one recording session. You put many reels in a batch and the app builds a single de-duplicated shot list for the session.

---

## Part 2 — Worked example: one reel, start to finish

Follow this exactly and you'll touch every feature. You'll need: the app running, and any small `.mp4` file to copy around (a 5-second phone clip is perfect).

### Step 0 — One-time setup (Settings)

1. In Windows Explorer, create a folder for your media, e.g. `D:\ContentOS` (any drive is fine).
2. In the app: **Settings → Storage roots → Choose folder…** → pick that folder → **Add root**.
3. Under **Content pillars**, add one: name `Gym myths`, click a color square, target `20` → **Add pillar**.

### Step 1 — Capture an idea (Ideas)

1. Go to **Ideas**. Type `Stop doing cardio before weights` → pick the `Gym myths` pillar → **Add idea**.
2. Click **Promote to reel**. You land in the script editor with a new reel — note its code (e.g. `R0001`). That code follows this reel everywhere: filenames, timelines, exports.

### Step 2 — Write the script (Scripts)

1. First make a reusable hook: open the **Component library** tab → kind `hook` → text `You've been lied to about cardio` → **Add**. (It gets a code like `HK0001`.)
2. Back on the **Reels** tab, select your reel. Now build the script:
   - Click **+ Hook** → then **Link component…** on that block → pick `HK0001`.
   - Click **+ Body** → type directly: `Cardio first drains the strength you need for lifts. Flip the order: lift first, cardio after. Same calories, better gains.`
   - Click **+ CTA** → type: `Follow for part 2.`
3. Click **Save script**. The reel's status flips to `scripted` automatically.

### Step 3 — Plan it (Calendar)

1. Go to **Calendar**. Your reel is in the **Unscheduled** tray on the right.
2. Drag it onto a day this week. The week counter ticks up. That's the whole job here.

### Step 4 — The shoot (Shoot)

1. Go to **Shoot**. Type a batch name `Test shoot` → **New**.
2. Click **Add reels…** → tick your reel → **Add 1 reel to shot list**. Two shots appear: the hook (HK0001) and your body+CTA blocks.
3. Click **Record Mode**. Read the script off the screen like a teleprompter. Press **Space** after each shot (pretend to record). **Esc** to exit. The batch shows 3/3 recorded.

### Step 5 — Fake footage in (Library)

1. In Explorer, copy your small `.mp4` **three times** into `D:\ContentOS\00_INBOX\`, named anything — BUT make each file's content unique or the duplicate detector will (correctly!) skip them. Easiest trick: copy three *different* small videos, or open each copy's name in a text editor — actually simplest: record three 2-second clips on your phone.
2. In the app: **Library → Scan inbox**. The clips appear on the left.
3. Pick your batch `B001` on the right. Tick clip 1 → click **← takes** on the first shot. Tick clip 2 → **← takes** on the second shot. Tick clip 3 → the third. Watch the files vanish from the inbox — check Explorer: they've been renamed to things like `B001_HK0001_T01.mp4` and filed under `01_RAW\B001\`.
4. On each shot, give the take a star rating and click **Select**. When the last one is selected, the reel's status becomes `shot`.

### Step 6 — The DaVinci handoff (Assemble)

1. Go to **Assemble**. Your batch is in the dropdown → **Generate handoff**.
2. Click **Open folder**: you'll see a folder per reel with clips renamed in edit order (`01_HOOK_HK0001.mp4`…), a script text file, and `_IMPORT_ME.fcpxml`.
3. If you have Resolve: **File ▸ Import ▸ Timeline** → pick that fcpxml → your reel appears as a ready-made timeline. Edit it, then render to `D:\ContentOS\03_EXPORTS\` keeping the timeline name.
   **No Resolve? Fake it:** copy any `.mp4` into `03_EXPORTS\` and rename it `R0001_final.mp4` (use YOUR reel's code).
4. Back in Assemble: **Scan exports** → the file matches your reel → click **File it**. It moves to `04_FINALS\` and the reel becomes `edited`.

### Step 7 — Publish (Publish)

1. Go to **Publish**. Your reel is in panel 1 → **Queue** it (creates IG + TikTok + YouTube posts).
2. Panel 2: write a caption (`You've been lied to about cardio 🏋️`), hashtags (`gym fitness cardio`), then click **Fill times from calendar dates** — all three posts get times on the day you planned. **Save**.
3. Panel 3: **Export 3 ready posts**. Click **Open folder**: there's `metricool_import.csv` plus your video named by slot. (With a real Metricool account: Planner → bulk import the CSV → attach the videos by their matching names.)
4. Panel 4: as posts go live on Metricool, click **Published** → then **Verify live** on each. When all three are verified, the reel's status reaches `verified`.

**Done.** Check the Dashboard: your reel walked the whole funnel. That's the loop you'll run 100× a week — except in bulk: 30 ideas at a time, one batch for 50 clips, one handoff for 25 timelines, one CSV for the whole week.

---

## Part 3 — Statuses & when something looks wrong

What each status means (they move automatically):

| Status | Means | It advances when… |
|---|---|---|
| `idea` | Just created | you save a script |
| `scripted` | Script written | you add it to a shoot batch |
| `shotlisted` | On a batch's shot list | every block has a selected take |
| `shot` | All material recorded & picked | you generate the DaVinci handoff |
| `assembled` | Timeline staged for editing | its final render is filed from 03_EXPORTS |
| `edited` | Final video exists | all its posts are scheduled (export) |
| `scheduled` | Pushed to Metricool | you mark a post published |
| `posted` | Live on at least one platform | all posts verified |
| `verified` | Confirmed live everywhere | — finished! |

Common "why isn't this working":

- **Reel doesn't show in "Add reels…"** → it has no script yet (status `idea`), or it's already in this batch. Save a script first.
- **"Add … to shot list" button grayed out** → tick at least one reel's checkbox first.
- **Batch says 0 shots after adding a reel** → the reel's components already have master takes from an earlier batch (nothing new to record — that's the reuse system working).
- **Clips don't appear after Scan inbox** → they're not in `00_INBOX`, aren't video files, or are byte-identical duplicates of already-imported clips (the scan report says how many were skipped and why).
- **Reel stuck at `shotlisted`** → some shot has takes but none **selected** — stars are just notes; the **Select** button is what counts.
- **Generate handoff says a reel was skipped** → the message names the exact block with no selected take.
- **Scan exports doesn't match a file** → the filename must contain the reel code (`R0001`). Render with the timeline name and it's automatic.
- **Can't export a post** → it needs all three: caption, schedule time, and a filed final render. The export lists exactly what each skipped post is missing.

Still stuck? The Dashboard's **Stuck reels** panel shows anything that hasn't moved in 4+ days, and every error in the app says what went wrong — if you hit one that doesn't, that's a bug: report it.
