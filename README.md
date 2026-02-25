# LockPilot (Mac)

A macOS desktop app to schedule multiple one-time timers with actions:
- popup message
- lock screen
- shut down
- restart

You can add any number of timers, see active timers with due times/countdown, and cancel any timer.


## Install on Mac
If you distribute an unsigned or non-notarized `.app`, macOS may block it with a "damaged and can't be opened" message.

You can remove the quarantine flag manually:

```bash
sudo xattr -dr com.apple.quarantine "/Applications/LockPilot.app"
```

WARNING: This bypasses a Gatekeeper safety check and allows the app to run without notarization.

## Project Layout
- `src-tauri/`: Rust backend + Tauri app config
- `ui/`: static frontend (HTML/CSS/JS)

## macOS behavior notes
- `Lock` uses a fallback chain: lock shortcut (`Ctrl+Cmd+Q`), then screen saver, then `pmset displaysleepnow`.
- `Shutdown` and `Reboot` use AppleScript (`System Events`) and may require macOS permissions.
- `Popup` uses AppleScript dialog.

## Dev Run (no JS framework required)
1. Install Rust and cargo.
2. Install Tauri CLI:
   - `cargo install tauri-cli`
3. In one terminal, serve the `ui/` folder:
   - `python3 -m http.server 1420 --directory ui`
4. In another terminal, run:
   - `cd src-tauri`
   - `cargo tauri dev`

## Build .app
From `src-tauri/`:
- `cargo tauri build`

Output app bundle will be under `src-tauri/target/release/bundle/`.


