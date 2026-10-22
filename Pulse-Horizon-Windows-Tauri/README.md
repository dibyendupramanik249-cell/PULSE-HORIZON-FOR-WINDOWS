PULSE HORIZON FOR WINDOWS
=========================

A Windows build of the browser, in Rust (Tauri) + the HTML you already have.
Nothing runs on your PC - GitHub builds the EXE for you.


============================================================
WHAT YOU GET
============================================================

  src/index.html        the chrome: tab strip, address bar, menu
  src/home.html         YOUR home page - carried over unchanged
  src-tauri/            the Rust: window, two webviews, the bridge
  .github/workflows/    the build, on GitHub's Windows machines


============================================================
HOW TO GET THE .EXE  (no tools on your PC)
============================================================

  1. Put these files at the root of a GitHub repository.

  2. That is it. Pushing to main starts the build, or you can press
     Actions -> "Build Windows EXE" -> Run workflow.

  3. When it finishes, open the run and download the artifact
     "pulse-horizon-windows". Inside is the installer .exe.

  Nothing is installed on your computer. A power cut cannot stop it.


============================================================
WHAT WAS TRANSLATED
============================================================

  Android (Kotlin)                     Windows (this)
  ---------------------------------    --------------------------------
  Activity + WebView                   a window + two webviews
  the Kotlin-built top bar,            index.html - the same look,
    address bar and tab strip            now in HTML
  home.html                            home.html, unchanged
  evaluateJavascript()                 the page_eval command
  shouldOverrideUrlLoading +           on_navigation, catching pulse://
    the pulse:// scheme
  the tab list in Kotlin               the tab list in the chrome's JS
  SharedPreferences                    a config file (not wired yet)

  THE UI DID NOT NEED TRANSLATING. It is a web page, and web pages do
  not care what operating system they are on. That is why this is a
  small app instead of a rewrite of a 341 KB file.


============================================================
WHAT IS NOT DONE YET
============================================================

  - the media notification and background music (Windows media controls)
  - settings that persist between runs
  - bookmarks and downloads
  - the theme switch (the app's Theme menu item)

  These are the next stages. The window, the tabs, the address bar and
  the home page are the first stage, and they are here.



============================================================
PAIRING - A QR TO DISPLAY, NOT A CAMERA TO SCAN
============================================================

There is NO camera scanner on Windows, and there should not be. A desktop
has no camera pointed at anything. It is the wrong way round.

The Windows side SHOWS a code. The phone SCANS it.

  Menu (the 3 dots)  ->  Pair a device

That opens a sheet with:

  - a real QR code, generated in Rust by the qrcode crate
  - this machine's own address on the local network
  - a short one-time code

The code carries:   pulse://pair/<address>:<port>#<token>

So the phone learns where to come back to. NOTHING goes through any server -
the address is this machine's own LAN address, read straight from the OS.

HONEST: the code is real and scannable today. What it leads TO - the actual
file transfer between phone and PC - is not built yet. So this is the pairing
half, done properly, waiting for the other half.

============================================================
HOW IT IS PUT TOGETHER
============================================================

  Two webviews inside one window:

    +-------------------------------------------+
    |  chrome   (index.html)   <- tab strip, url |
    +-------------------------------------------+
    |                                           |
    |  page     (home.html, or a website)       |
    |                                           |
    +-------------------------------------------+

  The chrome drives the page with Rust commands:
    set_url, page_back, page_forward, page_reload, page_eval, page_url,
    open_outside

  The page talks back the way it always did: it navigates to a
  pulse:// link, Rust catches it, and sends it to the chrome.

  Tauri uses the WebView2 engine that is already on Windows, so no
  Chromium is bundled and the installer stays small.

  NOTE: Tauri's multi-webview support is behind an "unstable" Cargo
  feature. It is enabled in src-tauri/Cargo.toml, because a browser
  needs a second webview for the page. It works - Tauri ships an
  official example of exactly this - but Tauri labels it unstable.
