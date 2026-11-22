# N-Google (Rust)

VST3 utility plugin (TG: @vst_ntune): embedded WebView2 browser, Google by default, toolbar with back/forward/reload/home/address bar.
On YouTube watch pages a **В FL** button appears: the audio is downloaded as WAV (yt-dlp + ffmpeg are fetched once into `%APPDATA%\N-Google\bin`, needs internet),
then drag the green chip into the FL Studio playlist / Channel Rack. Files go to `Documents\Image-Line\FL Studio\Data\Patches\Packs\N-Google`.
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
