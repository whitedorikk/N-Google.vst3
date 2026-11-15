# N-Google

A JUCE-based Windows VST3 utility plugin with an embedded browser that opens Google by default.

**Plugin metadata:** `TG: @vst_ntune`

## Build on GitHub (no local Visual Studio required)

1. Create a new GitHub repository.
2. Upload the contents of this folder to the repository (keep `.github/workflows/build-windows.yml`).
3. Open **Actions** → **Build N-Google VST3** → **Run workflow**.
4. When the workflow finishes, open the run and download the `N-Google_Windows_x64` artifact.
5. Extract the ZIP. Copy `N-Google.vst3` to:
   `C:\Program Files\Common Files\VST3`
6. Rescan plugins in FL Studio.

The workflow builds the plugin on a GitHub-hosted Windows runner. This source archive is not itself a compiled VST3 binary.

## Local build

Requires CMake, a C++17 compiler and the Windows SDK / Visual Studio Build Tools.

```powershell
cmake -S . -B build -A x64
cmake --build build --config Release --target NGoogle_VST3 --parallel
```

## Notes

- Audio passes through unchanged.
- Browser data is managed by the platform browser backend.
- The browser backend available to JUCE depends on the Windows environment. On Windows, install the Microsoft Edge WebView2 Runtime if the embedded browser does not initialize.
- JUCE is fetched from its upstream repository at build time. Review JUCE licensing terms for your distribution model.
