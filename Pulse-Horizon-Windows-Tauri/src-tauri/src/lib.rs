// Pulse Horizon - Windows
// ============================================================
// This is the translation of the Android MainActivity.
//
// Android had:  an Activity, a WebView, Kotlin-built views for the
//               chrome, SharedPreferences, a foreground service.
// Windows has:  a window, two webviews, HTML for the chrome, a config
//               file, and the OS media controls.
//
// The chrome (tab strip, address bar, menu) is drawn in index.html.
// The page area is a second webview called "page", which shows the
// home page when there is no site, and the site otherwise.
// ============================================================

use std::sync::Mutex;

use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent};

/// Height of the chrome, in logical pixels. The page sits under it.
const CHROME_H: f64 = 96.0;

/// The label of the home page inside the app bundle.
const HOME: &str = "home.html";

struct Layout {
    w: f64,
    h: f64,
}

#[derive(Default)]
struct State {
    layout: Mutex<Layout>,
}

/// Navigate the page webview. If the url is empty, go back to the home page.
#[tauri::command]
async fn set_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let Some(page) = app.get_webview("page") else {
        return Err("no page webview".into());
    };

    if url.trim().is_empty() {
        page.navigate(tauri::Url::parse(&format!("tauri://localhost/{HOME}"))
            .map_err(|e| e.to_string())?);
        return Ok(());
    }

    let parsed = tauri::Url::parse(&url).map_err(|e| e.to_string())?;
    page.navigate(parsed).map_err(|e| e.to_string())
}

#[tauri::command]
async fn page_back(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(page) = app.get_webview("page") {
        let _ = page.eval("history.back()");
    }
    Ok(())
}

#[tauri::command]
async fn page_forward(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(page) = app.get_webview("page") {
        let _ = page.eval("history.forward()");
    }
    Ok(())
}

#[tauri::command]
async fn page_reload(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(page) = app.get_webview("page") {
        let _ = page.eval("location.reload()");
    }
    Ok(())
}

/// Open a link in the user's normal browser, rather than in here.
#[tauri::command]
async fn open_outside(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// ============================================================
/// PAIRING - a QR to DISPLAY, not a camera to scan.
///
/// The Windows side shows a code. The phone scans it. That is the way
/// round it has to be: a desktop has no camera pointed at anything, and
/// the phone is the thing that can read a screen.
///
/// The code carries this machine's own address on the local network and a
/// one-time token, so the phone knows where to come back to. Nothing goes
/// through any server.
/// ============================================================
#[derive(serde::Serialize)]
struct PairInfo {
    payload: String,
    svg: String,
    host: String,
    token: String,
}

/// This machine's address on the local network, without pulling in a crate.
fn lan_ip() -> String {
    let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") else {
        return "127.0.0.1".into();
    };
    // connecting a UDP socket sends nothing - it only asks the OS which
    // interface would be used to reach the outside, which is the LAN one.
    if sock.connect("8.8.8.8:80").is_err() {
        return "127.0.0.1".into();
    }
    sock.local_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".into())
}

fn make_token() -> String {
    // a short, readable one-time code. Not a security boundary on its own -
    // the phone still has to be on the same network.
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut x = n as u64;
    let mut out = String::new();
    let alphabet = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    for _ in 0..6 {
        out.push(alphabet[(x % alphabet.len() as u64) as usize] as char);
        x = x / alphabet.len() as u64 + 7919;
    }
    out
}

#[tauri::command]
async fn pair_info() -> Result<PairInfo, String> {
    let host = lan_ip();
    let token = make_token();
    let port = 8787u16;

    // pulse://pair/<host>:<port>#<token>
    let payload = format!("pulse://pair/{host}:{port}#{token}");

    let code = qrcode::QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?;
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(260, 260)
        .dark_color(qrcode::render::svg::Color("#04121C"))
        .light_color(qrcode::render::svg::Color("#FFFFFF"))
        .build();

    Ok(PairInfo { payload, svg, host, token })
}

/// Run a line of JavaScript inside the page webview.
/// This is the Windows version of Android's evaluateJavascript - it is how the
/// chrome pushes the tab list into the home page, exactly like phSetTabs did.
#[tauri::command]
async fn page_eval(app: tauri::AppHandle, js: String) -> Result<(), String> {
    if let Some(page) = app.get_webview("page") {
        let _ = page.eval(&js);
    }
    Ok(())
}

/// Ask the page webview what it is showing, so the address bar can keep up.
#[tauri::command]
async fn page_url(app: tauri::AppHandle) -> Result<String, String> {
    if let Some(page) = app.get_webview("page") {
        if let Ok(u) = page.url() {
            return Ok(u.to_string());
        }
    }
    Ok(String::new())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(State::default())
        .invoke_handler(tauri::generate_handler![
            set_url,
            page_back,
            page_forward,
            page_reload,
            page_url,
            page_eval,
            pair_info,
            open_outside
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let win = tauri::window::WindowBuilder::new(app, "main")
                .title("Pulse Horizon")
                .inner_size(1180.0, 800.0)
                .min_inner_size(520.0, 420.0)
                .build()?;

            let sf = win.scale_factor().unwrap_or(1.0);
            let sz = win.inner_size()?;
            let w = sz.width as f64 / sf;
            let h = sz.height as f64 / sf;

            *app.state::<State>().layout.lock().unwrap() = Layout { w, h };

            // ---------- the chrome: tab strip, address bar, menu ----------
            win.add_child(
                tauri::webview::WebviewBuilder::new("chrome", WebviewUrl::App("index.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(w, CHROME_H),
            )?;

            // ---------- the page: the home page, or a website ----------
            let nav_handle = handle.clone();
            win.add_child(
                tauri::webview::WebviewBuilder::new("page", WebviewUrl::App(HOME.into()))
                    .on_navigation(move |url| {
                        // The page talks to the browser with pulse:// links, the
                        // same scheme the Android app used. Catch them here and
                        // pass them to the chrome instead of trying to load them.
                        if url.scheme() == "pulse" {
                            let _ = nav_handle.emit("pulse", url.as_str().to_string());
                            return false;
                        }
                        true
                    }),
                LogicalPosition::new(0.0, CHROME_H),
                LogicalSize::new(w, (h - CHROME_H).max(0.0)),
            )?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Resized(_) = event {
                let app = window.app_handle();
                let (Some(chrome), Some(page)) = (app.get_webview("chrome"), app.get_webview("page"))
                else {
                    return;
                };
                let Ok(sz) = window.inner_size() else { return };
                let sf = window.scale_factor().unwrap_or(1.0);
                let w = sz.width as f64 / sf;
                let h = sz.height as f64 / sf;

                let _ = chrome.set_position(LogicalPosition::new(0.0, 0.0));
                let _ = chrome.set_size(LogicalSize::new(w, CHROME_H));
                let _ = page.set_position(LogicalPosition::new(0.0, CHROME_H));
                let _ = page.set_size(LogicalSize::new(w, (h - CHROME_H).max(0.0)));
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Pulse Horizon");
}
