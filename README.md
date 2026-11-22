# N-Google (Rust)

VST3 utility plugin (TG: @vst_ntune): embedded WebView2 browser, Google by default, toolbar with back/forward/reload/home/address bar.
Audio passes through unchanged. The last page is saved in the DAW project.

Built with nih-plug + wry. Windows only.

## Build on GitHub
Upload this folder, run **Actions -> Build N-Google VST3**, download the artifact,
copy `N-Google.vst3` to `C:\Program Files\Common Files\VST3`, rescan in FL Studio.

## Local build
```powershell
cargo xtask bundle n_google --release
```
Output: `target/bundled/N-Google.vst3`. Requires the Microsoft Edge WebView2 Runtime (preinstalled on Windows 11).

## License
nih-plug's VST3 export is GPL-3.0, so this plugin is GPL-3.0-or-later.
