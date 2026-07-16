# Setting up ContentOS on your Windows laptop (step by step)

Follow these in order. Total time: ~30–45 minutes, most of it waiting for installers and the first build. You only do this once — after that, starting the app is one command.

## Part 1 — Install the tools

### 1. Git (if you don't have it)

1. Download from https://git-scm.com/download/win
2. Run the installer. **Click Next through everything** — the defaults are fine.
3. Verify: open **PowerShell** (press `Win`, type "powershell", Enter) and run:
   ```powershell
   git --version
   ```
   You should see something like `git version 2.x`.

### 2. Node.js

1. Download the **LTS** version from https://nodejs.org
2. Run the installer, defaults are fine. (No need to check the "native modules tools" box.)
3. Verify in a **new** PowerShell window:
   ```powershell
   node --version
   ```
   Should print `v20` or higher.

### 3. Visual Studio Build Tools (needed by Rust — this is the big one)

1. Download **Build Tools for Visual Studio** from https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run it. In the window that opens, check exactly one box: **"Desktop development with C++"**.
3. Click **Install** and let it finish (several GB — go make coffee).

### 4. Rust

1. Download `rustup-init.exe` (64-bit) from https://rustup.rs
2. Run it. When it asks in the terminal, press **Enter** to accept the default installation.
   (If it complains about missing Visual Studio components, finish step 3 first, then run it again.)
3. Verify in a **new** PowerShell window:
   ```powershell
   cargo --version
   ```

### 5. WebView2 (probably already installed)

Windows 11 and updated Windows 10 already have it. Skip this. If the app later opens as a blank window, install "WebView2 Runtime — Evergreen Bootstrapper" from https://developer.microsoft.com/en-us/microsoft-edge/webview2/.

## Part 2 — Get the project

Open PowerShell and run these one at a time (adjust the folder if you keep code somewhere else):

```powershell
cd ~\Documents
git clone https://github.com/KrisMultiDev/ContentOS.git
cd ContentOS
git checkout claude/content-planning-app-design-yixfy6
```

> The last command switches to the development branch where all the work lives. If GitHub asks you to log in, follow the prompts (it opens a browser).

## Part 3 — Open in VS Code and run

1. Open VS Code → **File → Open Folder…** → pick the `ContentOS` folder.
2. When VS Code suggests extensions, the useful ones are **rust-analyzer** and **Tauri**. (Optional — the app runs without them.)
3. Open the built-in terminal: **Terminal → New Terminal** (or `` Ctrl+` ``).
4. Install the JavaScript dependencies (one-time, ~1 minute):
   ```powershell
   npm install
   ```
5. Start the app:
   ```powershell
   npm run tauri dev
   ```

   **The first run compiles all the Rust code — expect 5–15 minutes.** Later runs start in seconds. When it finishes, the ContentOS window opens by itself.

## Part 4 — First 5 minutes inside the app

1. Create your media folder in Explorer, e.g. `D:\ContentOS` (any drive with space for video).
2. In the app: **Settings → Storage roots** → name it `media`, paste `D:\ContentOS`, kind `local` → **Add root**.
3. **Settings → Content pillars** → add your content categories (each gets a color).
4. **Ideas** → dump in a few reel ideas → **Promote** the best one.
5. **Scripts** → write its blocks (hook/body/CTA — link reusable components from the Library tab).
6. **Calendar** → drag it onto a day.
7. When you shoot: **Shoot** → new batch → add scripted reels → **Record Mode** (Space = recorded, K = skip).
8. After the shoot: copy your clips into `D:\ContentOS\00_INBOX`, then **Library → Scan inbox** and link clips to shots as takes.

## If something goes wrong

| Symptom | Fix |
|---|---|
| `cargo` or `node` "is not recognized" | Close and reopen PowerShell/VS Code (installers update PATH only for new windows). |
| Rust errors mentioning `link.exe` or MSVC | Part 1 step 3 wasn't completed — install the C++ Build Tools, then retry. |
| `npm run tauri dev` fails in the Rust part | Copy the whole error text and paste it to Claude in the ContentOS session — the code was written on Linux and the first Windows compile may need a one-line fix. |
| App opens but the window is blank | Install the WebView2 runtime (Part 1 step 5), restart the app. |
| Port 5173 already in use | Close other dev servers or reboot, then rerun. |

## Daily use after setup

```powershell
cd ~\Documents\ContentOS
npm run tauri dev
```

To get the latest changes Claude has pushed since you cloned:

```powershell
git pull
npm install
npm run tauri dev
```

To build a real installer you can pin to the taskbar (produces an `.msi` under `src-tauri/target/release/bundle/`):

```powershell
npm run tauri build
```
