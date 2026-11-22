//! Editor: two WebView2 child views inside the host window: a small toolbar (HTML) on top, the page below.
use crate::HOME;
use nih_plug::prelude::*;
use raw_window_handle::{HandleError, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::num::NonZeroIsize;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{Rect, WebContext, WebView, WebViewBuilder};

const W: u32 = 1000;
const H: u32 = 680;
const BAR: u32 = 42;

const TOOLBAR_HTML: &str = r##"<!doctype html><meta charset="utf-8"><style>
*{box-sizing:border-box}html,body{margin:0;height:100%;overflow:hidden;background:#f8f9fa;font:14px Segoe UI,sans-serif;border-bottom:1px solid #dadce0}
body{display:flex;align-items:center;gap:4px;padding:4px 5px}
button{height:32px;min-width:34px;border:1px solid #dadce0;background:#fff;border-radius:6px;cursor:pointer;font-size:15px}
button:hover{background:#e8eaed}input{flex:1;height:32px;border:1px solid #dadce0;border-radius:16px;padding:0 14px;font-size:14px;outline:none}
input:focus{border-color:#4285f4}
</style>
<button onclick="send('back')">&#8592;</button><button onclick="send('forward')">&#8594;</button>
<button onclick="send('reload')">&#8635;</button><button onclick="send('home')">G</button>
<input id="a" spellcheck="false" autocomplete="off"><button onclick="go()" style="min-width:48px">Go</button>
<script>
const a=document.getElementById('a');
function send(m){window.ipc.postMessage(m)}
function go(){send('go:'+a.value)}
function setAddr(u){if(document.activeElement!==a)a.value=u}
a.addEventListener('keydown',e=>{if(e.key==='Enter')go()});
</script>"##;

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

fn handle_command(view: &WebView, msg: &str) {
    match msg {
        "back" => { let _ = view.evaluate_script("history.back()"); }
        "forward" => { let _ = view.evaluate_script("history.forward()"); }
        "reload" => { let _ = view.evaluate_script("location.reload()"); }
        "home" => { let _ = view.load_url(HOME); }
        _ => {
            if let Some(target) = msg.strip_prefix("go:").and_then(resolve_target) {
                let _ = view.load_url(&target);
            }
        }
    }
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

    let slot_ipc = slot.clone();
    let toolbar = Rc::new(
        WebViewBuilder::with_web_context(&mut ctx)
            .with_bounds(rect(0, 0, W, BAR))
            .with_html(TOOLBAR_HTML)
            .with_ipc_handler(move |req| {
                if let Some(view) = slot_ipc.borrow().as_ref() {
                    handle_command(view, req.body());
                }
            })
            .build_as_child(&parent)?,
    );

    let start = {
        let saved = url.lock().map(|u| u.clone()).unwrap_or_default();
        if saved.starts_with("http") { saved } else { HOME.to_string() }
    };

    let tb = toolbar.clone();
    let url_nav = url.clone();
    let slot_new = slot.clone();
    let content = WebViewBuilder::with_web_context(&mut ctx)
        .with_bounds(rect(0, BAR as i32, W, H - BAR))
        .with_url(&start)
        .with_navigation_handler(move |u: String| {
            if u.starts_with("http") {
                if let Ok(mut s) = url_nav.lock() {
                    *s = u.clone();
                }
                let js = format!("setAddr({})", serde_json::to_string(&u).unwrap_or_default());
                let _ = tb.evaluate_script(&js);
            }
            true
        })
        // links that want a new window open in the same view
        .with_new_window_req_handler(move |u: String| {
            if let Some(v) = slot_new.borrow().as_ref() {
                let _ = v.load_url(&u);
            }
            false
        })
        .build_as_child(&parent)?;

    let _ = toolbar.evaluate_script(&format!("setAddr({})", serde_json::to_string(&start).unwrap_or_default()));
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
