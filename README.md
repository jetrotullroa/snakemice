# SnakeMice

A local Mac app for the Razer Naga V3 Pro. Synapse on macOS doesn't talk to this mouse. SnakeMice does, over the USB cable (`1532:00E7`) or the wireless receiver (`1532:00E8`).

**Landing page (share this URL):** [jetrotullroa.github.io/snakemice](https://jetrotullroa.github.io/snakemice/) — same project; includes an FAQ and metadata so search and “AI overview” tools can cite a clear answer when people ask whether **Naga V3 Pro works with Synapse on Mac** or how to **remap side buttons without Windows**.

## Demo

![SnakeMice app](docs/demo.gif)

### Download (no Rust needed)

**For most users (no Terminal):** use **GitHub Releases** and get the **app bundle**, not the raw executable.

1. Open **[Releases](https://github.com/jetrotullroa/snakemice/releases)** for this repo.
2. Pick the **latest** release.
3. Under **Assets**, download **`SnakeMice-macos-arm64.app.zip`** (Apple Silicon) or **`SnakeMice-macos-x86_64.app.zip`** (Intel), depending on what the maintainer published.
4. Double-click the zip → you get **`SnakeMice.app`**. Drag it to **Applications** (optional), open from **Finder**.
5. If macOS blocks it the first time: **Control-click** the app → **Open** → **Open** (unsigned apps often need this once).

**macOS says “SnakeMice.app is damaged and can’t be opened”?** For downloads from the web, that usually means the app is **quarantined** and/or **not signed with a Developer ID** — it is often **not** a bad zip. Try **Control-click → Open** again. If it still refuses, clear the quarantine flag once (then open normally):

```bash
xattr -dr com.apple.quarantine /path/to/SnakeMice.app
```

A **Developer ID signed + notarized** `.app` is what makes first launch smooth for everyone without that step.

**Looks “empty”?** GitHub always shows **Source code (zip)** and **Source code (tar.gz)** automatically. Those are not the Mac app. If there is no **`.app.zip`** under **Assets**, nobody has uploaded that build yet — see [publishing a release asset](#pre-built-downloads-github-releases) (maintainers) or [build from source](#build-from-source-macos) (developers).

**Optional:** some releases also attach a zip with only the **`snakemice`** command-line binary (for power users). Prefer **`SnakeMice-…-app.zip`** for normal installs.

Windows users with a Naga V3 Pro can use **Razer Synapse** on PC; this project targets **Mac** users who cannot rely on Synapse for this mouse.

**Before you use SnakeMice:** plug in the **USB cable** or **2.4 GHz dongle**, and **quit Razer Synapse** so it does not fight this app for the mouse.

If there is **no release yet**, install [Rust](https://rustup.rs/) and see **[Run](#run)** below, or ask in Issues for a published build.

**Simple landing page (for sharing):** after you enable [GitHub Pages](#github-pages) below, share **`https://jetrotullroa.github.io/snakemice/`** — it links to the latest release and the source repo.

Button changes are written into the mouse. They keep working when this app is closed, including in games.

The 12-button plate and the 6-button plate have separate buttons in the mouse:

| Plate | Slots | Factory keys |
|---|---|---|
| 12-button, labels 1–12 | `0x40`–`0x4B` | 1 2 3 4 5 6 7 8 9 0 - = |
| 6-button, labels 1–6 | `0x50`–`0x55` | 1 2 3 4 5 6 |

Snap the 6-button plate on and the mouse uses those six slots by itself. You don't copy the 12-button layout over, and you don't replug.

### Top of mouse (not the side plate)

These are on the **main top shell** (and the right grip strip), not on the magnetic 12/6 thumb plate:

| What you feel | Slot | Factory |
|---|---|---|
| Top edge, front (left of left-click) | 11 | DPI stage up |
| Top edge, rear | 12 | DPI stage down |
| Top, in front of scroll wheel | 106 | Scroll mode |
| Top, behind scroll wheel | 128 | Off |
| Ring-finger strip (right grip) | 57 | HyperShift |

In the app, open **Top of mouse (not the side plate)**. **Apply: Ctrl + Alt edges, DPI under wheel, Shift on ring** sets: top edges → Control and Option, wheel pair → DPI up/down, ring strip → Shift. The underside lock still blocks the ring button when flipped.

The firmware does not report which plate is attached. When a press can only belong to one plate (factory 7, 8, 9, 0, `-`, and `=` are 12-button keys), the matching plate is highlighted.

Bluetooth isn't supported. Use the cable or the receiver. Quit Razer Synapse first so the two programs aren't configuring the mouse at once.

## Run

```bash
cargo run --release
```

The first build takes a few minutes. It compiles a desktop window with Dioxus.

## What you can change

- Each side button: factory key, another key with ⌃ ⌥ ⇧ ⌘, a mouse button, or off
- DPI, from 100 to 50,000, with separate X and Y
- Polling rate: 125, 500, or 1000 Hz
- Battery level

This project is not affiliated with Razer. It does not use Razer Synapse.

### Disclaimer

**Use at your own risk.** SnakeMice is unofficial software: it sends configuration commands to your mouse over USB. Authors and contributors are **not** responsible for any device issues, lost settings, or other harm. If something looks wrong, **quit the app**, **quit Synapse**, and reconnect the mouse on **cable or dongle**. The MIT license also limits liability; see `LICENSE`.

## Open source on GitHub

This repo is meant to be **cloned or downloaded from GitHub** and built with Rust, or to ship **pre-built binaries** from **GitHub Releases** when a maintainer publishes them.

Repo: **[github.com/jetrotullroa/snakemice](https://github.com/jetrotullroa/snakemice)** · Maintainer: [jetrotullroa](https://github.com/jetrotullroa)

- **License:** MIT (see `LICENSE` and `Cargo.toml`).
- **Issues & PRs:** Use GitHub for bugs, device quirks, and contributions.

### GitHub Pages

This repo includes a small static site at [`docs/index.html`](docs/index.html) (demo GIF + download + source links).

1. On GitHub: repo **Settings** → **Pages** → **Build and deployment** → Source: **Deploy from a branch**.
2. Branch: **`main`** (or your default branch), folder: **`/docs`** → **Save**.
3. After a minute, the site is live at **`https://jetrotullroa.github.io/snakemice/`**.
4. Publish a [Release](#pre-built-downloads-github-releases) with **`SnakeMice-macos-….app.zip`** so the **Download for Mac** button on the [Pages site](https://jetrotullroa.github.io/snakemice/) matches what non-developers expect.

### Build from source (macOS)

**Prerequisites:** [Rust](https://rustup.rs/) (stable toolchain) on a Mac.

```bash
git clone https://github.com/jetrotullroa/snakemice.git
cd snakemice
cargo build --release
```

Run `./target/release/snakemice` (or `cargo run --release`). If `CARGO_TARGET_DIR` is set, look under that directory instead of `./target`.

The first build downloads dependencies and can take several minutes.

### Package as a double-clickable app (macOS)

Downloads feel **much** friendlier as **`SnakeMice.app`** (open from Finder, drag to **Applications**) than a naked Unix binary.

From the repo root:

```bash
./packaging/macos-bundle.sh
```

That runs `cargo build --release`, writes **`target/release/SnakeMice.app`**, **ad-hoc signs** it (`codesign -s -`), and creates **`target/release/SnakeMice-macos-$(uname -m).app.zip`** ready to upload. For end users you still want **Developer ID + notarization** when possible.

To rebuild only the zip after tweaking the app folder (optional):

```bash
cd target/release
ditto -c -k --sequesterRsrc --keepParent SnakeMice.app "SnakeMice-macos-$(uname -m).app.zip"
```

Attach **`SnakeMice-macos-arm64.app.zip`** (or **x86_64**) under **Assets**.

**Gatekeeper:** unsigned builds usually need **Control-click → Open** once, or **System Settings → Privacy & Security → Open Anyway**. Signing and notarizing with an **Apple Developer** account is what makes it seamless for strangers.

### Pre-built downloads (GitHub Releases)

Maintainers can attach a **Mac** build so users **don’t need Rust**:

| Artifact | Notes |
|----------|--------|
| **`SnakeMice.app` in a `.zip`** (recommended) | Users unzip, double-click the app, drag to **Applications**. |
| `snakemice` only (Unix executable in a zip) | Works, but Finder shows a generic “exec”; less obvious how to run. Same Gatekeeper caveats. |

**Suggested release workflow**

1. Tag a version (`v0.1.0`) and push the tag.
2. On **macOS**: `./packaging/macos-bundle.sh` (produces the **`.app.zip`** in `target/release/`). Build **arm64** and **x86_64** separately if you want both (or use GitHub Actions `macos-latest` runners).
3. On GitHub: **Releases → your draft → Attach binaries**, upload the **`.app.zip`** — it must appear as its **own** line under **Assets**, not only “Source code”.
4. Click **Publish release** (or **Update release**). The [Pages site](https://jetrotullroa.github.io/snakemice/) “Latest release” link should show that zip under **Assets**.

Until a release exists, **build from source** is the install path.
