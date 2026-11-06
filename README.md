# SnakeMice

A local Mac app for the Razer Naga V3 Pro. Synapse on macOS doesn't talk to this mouse. SnakeMice does, over the USB cable (`1532:00E7`) or the wireless receiver (`1532:00E8`).

## Demo

![SnakeMice app](docs/demo.gif)

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

## Open source on GitHub

This repo is meant to be **cloned or downloaded from GitHub** and built with Rust, or to ship **pre-built binaries** from **GitHub Releases** when a maintainer publishes them.

Maintainer: [jetrotullroa](https://github.com/jetrotullroa) — once the repo exists, it will be at `https://github.com/jetrotullroa/snakemice` (adjust the repo name if yours differs).

- **License:** MIT (see `LICENSE` and `Cargo.toml`).
- **Issues & PRs:** Use GitHub for bugs, device quirks, and contributions.

### Build from source (macOS or Windows)

**Prerequisites**

- [Rust](https://rustup.rs/) (stable toolchain)
- On **Windows:** [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) is usually already installed on Windows 10/11. The Dioxus desktop shell uses it.
- **HID:** the `hidapi` crate may need extra setup on Windows (e.g. `hidapi.dll` next to the exe, or static linking) — if a release build fails, check [hidapi-rs](https://docs.rs/hidapi/) for your platform.

**Commands**

```bash
git clone https://github.com/jetrotullroa/snakemice.git
cd snakemice
cargo build --release
```

- **macOS:** run `./target/release/snakemice` (or `cargo run --release`). If `CARGO_TARGET_DIR` is set, look under that directory instead of `./target`.
- **Windows:** run `target\release\snakemice.exe`.

First build downloads dependencies and can take several minutes.

### Pre-built downloads (GitHub Releases)

Maintainers can attach binaries so users **don’t need Rust**:

| Platform | Typical artifact | Notes |
|----------|------------------|--------|
| **macOS** | `SnakeMice.app` in a `.zip`, or a raw binary | **Gatekeeper:** unsigned builds may require **Right-click → Open** the first time, or **System Settings → Privacy & Security → Open Anyway**. For a smooth experience without that, sign and notarize with an **Apple Developer** account. Build for **arm64** (Apple Silicon), **x86_64** (Intel), or offer both. |
| **Windows** | `snakemice.exe` (and any required DLLs) | Ensure WebView2 is available on the machine. Test HID access with the Razer dongle or cable. |

**Suggested release workflow**

1. Tag a version (`v0.1.0`) and push the tag.
2. Use **GitHub Actions** (or local builds) with `cargo build --release` on `macos-latest` and `windows-latest`.
3. Upload artifacts under **Releases → Assets**.
4. In the README, link the **latest release** URL once the repo exists.

Until releases exist, treat **build from source** as the supported install path.
