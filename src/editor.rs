//! Editor: two WebView2 child views inside the host window: a toolbar (HTML) on top, the page below.
//! On YouTube watch pages the toolbar shows "В FL": the audio is downloaded as WAV (yt-dlp + ffmpeg),
//! then the result can be dragged straight into FL Studio (native file drag-and-drop).
use crate::HOME;
use nih_plug::prelude::*;
use raw_window_handle::{HandleError, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::num::NonZeroIsize;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{Rect, WebContext, WebView, WebViewBuilder};

const W: u32 = 1000;
const H: u32 = 680;
const BAR: u32 = 42;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DRAG_ICON: &[u8] = include_bytes!("../assets/drag.png");

const TOOLBAR_HTML: &str = r##"<!doctype html><meta charset="utf-8"><style>
*{box-sizing:border-box}html,body{margin:0;height:100%;overflow:hidden;background:#f8f9fa;font:14px Segoe UI,sans-serif;border-bottom:1px solid #dadce0;user-select:none}
body{display:flex;align-items:center;gap:4px;padding:4px 5px}
button{height:32px;min-width:34px;border:1px solid #dadce0;background:#fff;border-radius:6px;cursor:pointer;font-size:15px}
button:hover{background:#e8eaed}input{flex:1;min-width:80px;height:32px;border:1px solid #dadce0;border-radius:16px;padding:0 14px;font-size:14px;outline:none;user-select:text}
input:focus{border-color:#4285f4}
#imp{display:none;background:#e8f0fe;border-color:#4285f4;color:#174ea6;font-size:13px;font-weight:600;padding:0 10px}
#st{font-size:12px;color:#5f6368;max-width:190px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
#chip{display:none;height:32px;line-height:30px;padding:0 12px;border:1px solid #34a853;background:#e6f4ea;color:#137333;border-radius:16px;font-size:13px;font-weight:600;cursor:grab;max-width:230px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
</style>
<button onclick="send('back')">&#8592;</button><button onclick="send('forward')">&#8594;</button>
<button onclick="send('reload')">&#8635;</button><button onclick="send('home')">G</button>
<input id="a" spellcheck="false" autocomplete="off"><span id="st"></span>
<div id="chip" title="Перетащи в плейлист или Channel Rack FL Studio" onmousedown="send('drag')"></div>
<button id="imp" onclick="send('import')">В FL</button>
<button onclick="go()" style="min-width:48px">Go</button>
<script>
const a=document.getElementById('a'),st=document.getElementById('st'),chip=document.getElementById('chip'),imp=document.getElementById('imp');
function send(m){window.ipc.postMessage(m)}
function go(){send('go:'+a.value)}
function setAddr(u){if(document.activeElement!==a)a.value=u}
function setImport(v){imp.style.display=v?'inline-block':'none'}
function setStatus(t,file){st.textContent=t;st.title=t;if(file){chip.textContent='\u283F '+file;chip.style.display='block'}else{chip.style.display='none'}}
a.addEventListener('keydown',e=>{if(e.key==='Enter')go()});
setInterval(()=>send('poll'),400);
</script>"##;

// runs inside every page: reports URL changes (YouTube navigates without page loads)
const URL_WATCH_JS: &str = r#"(function(){var l='';setInterval(function(){if(location.href!==l){l=location.href;try{window.ipc.postMessage('url:'+l)}catch(e){}}},600)})();"#;

struct ParentHandle(*mut c_void);

impl HasWindowHandle for ParentHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let hwnd = NonZeroIsize::new(self.0 as isize).ok_or(HandleError::Unavailable)?;
        let raw = RawWindowHandle::Win32(Win32WindowHandle::new(hwnd));
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
    Rect {
        position: PhysicalPosition::new(x, y).into(),
        size: PhysicalSize::new(w, h).into(),
    }
}

fn js_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// address bar text -> URL (same rules as the old C++ version)
fn resolve_target(input: &str) -> Option<String> {
    let t = input.trim();
    if t.is_empty() {
        return None;
    }
    let low = t.to_ascii_lowercase();
    Some(if low.starts_with("http://") || low.starts_with("https://") {
        t.to_string()
    } else if t.contains('.') && !t.contains(' ') {
        format!("https://{t}")
    } else {
        format!("https://www.google.com/search?q={}", urlencoding::encode(t))
    })
}

fn is_video_url(u: &str) -> bool {
    let l = u.to_ascii_lowercase();
    (l.contains("youtube.com/watch?") || l.contains("youtube.com/shorts/") || l.contains("youtu.be/"))
        && (l.starts_with("https://"))
}

// ------------------------------------------------------------------------------------------------
// import (download to WAV)
#[derive(Default)]
struct ImportState {
    busy: bool,
    status: String,
    file: Option<PathBuf>,
    dirty: bool,
}
type Shared = Arc<Mutex<ImportState>>;

fn set_state(s: &Shared, busy: bool, msg: &str, file: Option<PathBuf>) {
    if let Ok(mut g) = s.lock() {
        g.busy = busy;
        g.status = msg.to_string();
        g.file = file;
        g.dirty = true;
    }
}

fn hidden(program: impl AsRef<std::ffi::OsStr>, args: &[&std::ffi::OsStr]) -> std::io::Result<Output> {
    Command::new(program).args(args).creation_flags(CREATE_NO_WINDOW).output()
}

fn os(s: &str) -> &std::ffi::OsStr {
    std::ffi::OsStr::new(s)
}

/// yt-dlp.exe and ffmpeg.exe live in %APPDATA%\N-Google\bin; downloaded on first use (curl.exe and tar.exe ship with Windows 10/11)
fn ensure_tools(bin: &Path, state: &Shared) -> Result<(), String> {
    std::fs::create_dir_all(bin).map_err(|e| e.to_string())?;
    let ytdlp = bin.join("yt-dlp.exe");
    if !ytdlp.exists() {
        set_state(state, true, "Скачиваю yt-dlp…", None);
        let ok = hidden("curl.exe", &[os("-L"), os("--fail"), os("-o"), ytdlp.as_os_str(),
            os("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe")])
            .map(|o| o.status.success()).unwrap_or(false);
        if !ok || !ytdlp.exists() {
            let _ = std::fs::remove_file(&ytdlp);
            return Err("Не удалось скачать yt-dlp".into());
        }
    }
    let ffmpeg = bin.join("ffmpeg.exe");
    if !ffmpeg.exists() {
        set_state(state, true, "Скачиваю ffmpeg (один раз)…", None);
        let zip = bin.join("ffmpeg.zip");
        let ok = hidden("curl.exe", &[os("-L"), os("--fail"), os("-o"), zip.as_os_str(),
            os("https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip")])
            .map(|o| o.status.success()).unwrap_or(false);
        if !ok {
            let _ = std::fs::remove_file(&zip);
            return Err("Не удалось скачать ffmpeg".into());
        }
        let _ = hidden("tar.exe", &[os("-xf"), zip.as_os_str(), os("-C"), bin.as_os_str()]);
        let extracted = bin.join("ffmpeg-master-latest-win64-gpl");
        let _ = std::fs::copy(extracted.join("bin").join("ffmpeg.exe"), &ffmpeg);
        let _ = std::fs::remove_dir_all(&extracted);
        let _ = std::fs::remove_file(&zip);
        if !ffmpeg.exists() {
            return Err("Не удалось распаковать ffmpeg".into());
        }
    }
    Ok(())
}

fn output_dir() -> PathBuf {
    // FL Studio's Browser shows this folder under "Packs" (not verified on every FL version)
    if let Some(p) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(p).join("Documents").join("Image-Line").join("FL Studio")
            .join("Data").join("Patches").join("Packs").join("N-Google");
    }
    std::env::temp_dir().join("N-Google")
}

fn run_import(url: String, state: Shared) {
    let Some(appdata) = std::env::var_os("APPDATA") else {
        return set_state(&state, false, "APPDATA не найден", None);
    };
    let bin = PathBuf::from(appdata).join("N-Google").join("bin");
    if let Err(e) = ensure_tools(&bin, &state) {
        return set_state(&state, false, &e, None);
    }
    let out = output_dir();
    let _ = std::fs::create_dir_all(&out);
    set_state(&state, true, "Скачиваю аудио в WAV…", None);

    let result = hidden(bin.join("yt-dlp.exe"), &[
        os("--no-playlist"), os("--no-warnings"), os("--no-simulate"),
        os("-x"), os("--audio-format"), os("wav"),
        os("--ffmpeg-location"), bin.as_os_str(),
        os("--windows-filenames"),
        os("-P"), out.as_os_str(),
        os("-o"), os("%(title).80B [%(id)s].%(ext)s"),
        os("--print"), os("after_move:filepath"),
        os(&url),
    ]);
    match result {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            let path = text.lines().map(str::trim).filter(|l| !l.is_empty()).last().map(PathBuf::from);
            match path {
                Some(p) if p.exists() => set_state(&state, false, "Готово — перетащи в FL", Some(p)),
                _ => set_state(&state, false, "Файл не найден после загрузки", None),
            }
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            let first = err.lines().rev().find(|l| l.contains("ERROR")).unwrap_or("yt-dlp: ошибка");
            set_state(&state, false, first.trim(), None);
        }
        Err(e) => set_state(&state, false, &format!("Не запустился yt-dlp: {e}"), None),
    }
}

// ------------------------------------------------------------------------------------------------
/// the URL changed (navigation or SPA route): save it, update the address bar and the import button
fn report_url(toolbar: &WebView, saved: &Mutex<String>, u: &str) {
    if !u.starts_with("http") {
        return;
    }
    if let Ok(mut s) = saved.lock() {
        *s = u.to_string();
    }
    let _ = toolbar.evaluate_script(&format!("setAddr({});setImport({})", js_str(u), is_video_url(u)));
}

struct Views {
    _content: Rc<RefCell<Option<WebView>>>,
    _toolbar: Rc<WebView>,
    _ctx: WebContext, // dropped last
}

// WebView2 objects live on the host's UI thread only; the handle is just kept alive and dropped there.
struct EditorHandle(#[allow(dead_code)] Option<Views>);
unsafe impl Send for EditorHandle {}

fn build(hwnd: *mut c_void, url: Arc<Mutex<String>>) -> Result<Views, Box<dyn std::error::Error>> {
    let parent = ParentHandle(hwnd);
    // WebView2 profile goes to %APPDATA%\N-Google (the DAW folder is not writable)
    let data_dir = std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("N-Google").join("WebView2"));
    let mut ctx = WebContext::new(data_dir);
    let slot: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));
    let tb_weak: Rc<RefCell<Weak<WebView>>> = Rc::new(RefCell::new(Weak::new()));
    let import: Shared = Arc::new(Mutex::new(ImportState::default()));

    // ---- toolbar: all commands come from here
    let slot_ipc = slot.clone();
    let tb_ipc = tb_weak.clone();
    let url_ipc = url.clone();
    let import_ipc = import.clone();
    let hwnd_ipc = hwnd as usize;
    let toolbar = Rc::new(
        WebViewBuilder::with_web_context(&mut ctx)
            .with_bounds(rect(0, 0, W, BAR))
            .with_html(TOOLBAR_HTML)
            .with_ipc_handler(move |req| {
                let msg = req.body().as_str();
                match msg {
                    "poll" => {
                        let update = import_ipc.lock().ok().and_then(|mut s| {
                            if s.dirty {
                                s.dirty = false;
                                let name = s.file.as_ref().and_then(|f| f.file_name()).map(|n| n.to_string_lossy().to_string());
                                Some((s.status.clone(), name))
                            } else {
                                None
                            }
                        });
                        if let (Some((status, name)), Some(tb)) = (update, tb_ipc.borrow().upgrade()) {
                            let _ = tb.evaluate_script(&format!("setStatus({},{})", js_str(&status),
                                name.map(|n| js_str(&n)).unwrap_or_else(|| "null".into())));
                        }
                    }
                    "import" => {
                        let current = url_ipc.lock().map(|u| u.clone()).unwrap_or_default();
                        let busy = import_ipc.lock().map(|s| s.busy).unwrap_or(true);
                        if busy || !is_video_url(&current) {
                            return;
                        }
                        set_state(&import_ipc, true, "Запускаю…", None);
                        let st = import_ipc.clone();
                        std::thread::spawn(move || run_import(current, st));
                    }
                    "drag" => {
                        let file = import_ipc.lock().ok().and_then(|s| s.file.clone());
                        if let Some(f) = file.filter(|f| f.exists()) {
                            let handle = ParentHandle(hwnd_ipc as *mut c_void);
                            let _ = drag::start_drag(
                                &handle,
                                drag::DragItem::Files(vec![f]),
                                drag::Image::Raw(DRAG_ICON.to_vec()),
                                |_result, _pos| {},
                                drag::Options::default(),
                            );
                        }
                    }
                    "back" | "forward" | "reload" | "home" => {
                        if let Some(view) = slot_ipc.borrow().as_ref() {
                            let _ = match msg {
                                "back" => view.evaluate_script("history.back()"),
                                "forward" => view.evaluate_script("history.forward()"),
                                "reload" => view.evaluate_script("location.reload()"),
                                _ => view.load_url(HOME),
                            };
                        }
                    }
                    _ => {
                        if let Some(target) = msg.strip_prefix("go:").and_then(resolve_target) {
                            if let Some(view) = slot_ipc.borrow().as_ref() {
                                let _ = view.load_url(&target);
                            }
                        }
                    }
                }
            })
            .build_as_child(&parent)?,
    );
    *tb_weak.borrow_mut() = Rc::downgrade(&toolbar);

    let start = {
        let saved = url.lock().map(|u| u.clone()).unwrap_or_default();
        if saved.starts_with("http") { saved } else { HOME.to_string() }
    };

    // ---- page view
    let tb_nav = toolbar.clone();
    let url_nav = url.clone();
    let tb_msg = toolbar.clone();
    let url_msg = url.clone();
    let slot_new = slot.clone();
    let content = WebViewBuilder::with_web_context(&mut ctx)
        .with_bounds(rect(0, BAR as i32, W, H - BAR))
        .with_url(&start)
        .with_initialization_script(URL_WATCH_JS)
        .with_navigation_handler(move |u: String| {
            report_url(&tb_nav, &url_nav, &u);
            true
        })
        // pages can only report their URL; every other message is ignored
        .with_ipc_handler(move |req| {
            if let Some(u) = req.body().strip_prefix("url:") {
                report_url(&tb_msg, &url_msg, u);
            }
        })
        // links that want a new window open in the same view
        .with_new_window_req_handler(move |u: String| {
            if let Some(v) = slot_new.borrow().as_ref() {
                let _ = v.load_url(&u);
            }
            false
        })
        .build_as_child(&parent)?;

    report_url(&toolbar, &url, &start);
    *slot.borrow_mut() = Some(content);
    Ok(Views { _content: slot, _toolbar: toolbar, _ctx: ctx })
}

pub struct GoogleEditor {
    pub url: Arc<Mutex<String>>,
}

impl Editor for GoogleEditor {
    fn spawn(&self, parent: ParentWindowHandle, _context: Arc<dyn GuiContext>) -> Box<dyn Any + Send> {
        let ParentWindowHandle::Win32Hwnd(hwnd) = parent else {
            return Box::new(EditorHandle(None));
        };
        match build(hwnd, self.url.clone()) {
            Ok(v) => Box::new(EditorHandle(Some(v))),
            Err(e) => {
                nih_error!("N-Google: WebView2 init failed (is the WebView2 Runtime installed?): {e}");
                Box::new(EditorHandle(None))
            }
        }
    }

    fn size(&self) -> (u32, u32) {
        (W, H)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false // the OS scales the window
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}
}
