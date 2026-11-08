"use strict";

// Electron main process for Project Axiom.
//
// Serves the bundled web/ build from a privileged custom scheme (axiom://app),
// not from file://. Reasons: module workers, fetch() of axiom.wasm, and
// SharedArrayBuffer (which needs COOP/COEP) all behave differently or fail on
// file://. Responses carry the isolation headers and a Content-Security-Policy.
//
// The shell does not talk to Steam. Keep Steamworks code out of this file.

const { app, BrowserWindow, Menu, net, protocol, session } = require("electron");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

const SCHEME = "axiom";
const ORIGIN = `${SCHEME}://app`;
const START_URL = `${ORIGIN}/index.html`;

// wasm-unsafe-eval is required for WebAssembly.compile under a CSP. No eval, no remote origins.
const CSP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "worker-src 'self' blob:",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "frame-ancestors 'none'",
].join("; ");

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".txt": "text/plain; charset=utf-8",
};

let mainWindow = null;

// Packaged builds read web files from resources/app-web (see electron-builder.yml).
// Development runs read ../web directly, so no copy step is needed to iterate.
function webRoot() {
  return app.isPackaged
    ? path.join(process.resourcesPath, "app-web")
    : path.resolve(__dirname, "..", "web");
}

// Must run before the app is ready.
protocol.registerSchemesAsPrivileged([
  {
    scheme: SCHEME,
    privileges: { standard: true, secure: true, supportFetchAPI: true, stream: true },
  },
]);

function notFound() {
  return new Response("not found", { status: 404, headers: { "Content-Type": "text/plain; charset=utf-8" } });
}

async function serveWeb(request) {
  const root = webRoot();
  let rel;
  try {
    rel = decodeURIComponent(new URL(request.url).pathname);
  } catch {
    return notFound();
  }
  if (rel === "" || rel === "/") rel = "/index.html";

  // Resolve and confine to the web root. Blocks ../ traversal.
  const file = path.join(root, rel);
  if (file !== root && !file.startsWith(root + path.sep)) {
    return new Response("forbidden", { status: 403 });
  }

  let upstream;
  try {
    upstream = await net.fetch(pathToFileURL(file).toString());
  } catch {
    return notFound();
  }
  if (!upstream.ok) return notFound();

  const headers = new Headers();
  headers.set("Content-Type", MIME[path.extname(file).toLowerCase()] || "application/octet-stream");
  headers.set("Cross-Origin-Opener-Policy", "same-origin");
  headers.set("Cross-Origin-Embedder-Policy", "require-corp");
  headers.set("Cross-Origin-Resource-Policy", "same-origin");
  headers.set("Content-Security-Policy", CSP);
  headers.set("X-Content-Type-Options", "nosniff");
  headers.set("Cache-Control", "no-cache");
  return new Response(upstream.body, { status: 200, headers });
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 960,
    minHeight: 600,
    resizable: true,
    fullscreenable: true,
    show: false,
    backgroundColor: "#1E222A",
    title: "Project Axiom",
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      sandbox: true,
      nodeIntegration: false,
      webSecurity: true,
      // Keep the simulation loop running when the window is not focused.
      backgroundThrottling: false,
      spellcheck: false,
    },
  });

  mainWindow.once("ready-to-show", () => mainWindow.show());
  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  // F11 and Alt+Enter toggle fullscreen. Escape is left to the game (it is the cancel key).
  mainWindow.webContents.on("before-input-event", (event, input) => {
    if (input.type !== "keyDown") return;
    if (input.key === "F11" || (input.key === "Enter" && input.alt)) {
      event.preventDefault();
      mainWindow.setFullScreen(!mainWindow.isFullScreen());
    }
  });

  mainWindow.loadURL(START_URL);
}

// Page navigation and new windows stay inside the app.
app.on("web-contents-created", (_event, contents) => {
  contents.on("will-navigate", (event, url) => {
    if (!url.startsWith(`${ORIGIN}/`)) event.preventDefault();
  });
  contents.setWindowOpenHandler(() => ({ action: "deny" }));
  contents.on("will-attach-webview", (event) => event.preventDefault());
});

if (!app.requestSingleInstanceLock()) {
  app.quit();
} else {
  app.on("second-instance", () => {
    if (!mainWindow) return;
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
  });

  app.whenReady().then(() => {
    // The game needs no network. Allow the app's own origin, in-memory URLs, and file: reads
    // that this process makes itself through net.fetch in serveWeb(). The page cannot use file:
    // URLs, because its CSP is default-src 'self' and its origin is axiom://app.
    session.defaultSession.webRequest.onBeforeRequest((details, callback) => {
      const url = details.url;
      const allowed =
        url.startsWith(`${ORIGIN}/`) ||
        url.startsWith("file:") ||
        url.startsWith("blob:") ||
        url.startsWith("data:") ||
        url.startsWith("devtools:");
      callback({ cancel: !allowed });
    });

    // Deny every permission prompt. The game uses no camera, microphone, location or notifications.
    session.defaultSession.setPermissionRequestHandler((_wc, _permission, callback) => callback(false));
    session.defaultSession.setPermissionCheckHandler(() => false);

    protocol.handle(SCHEME, serveWeb);

    // Windows and Linux use the window without a menu bar. macOS keeps its standard app menu (Quit, etc.).
    if (process.platform !== "darwin") Menu.setApplicationMenu(null);

    createWindow();

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
  });

  app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
  });
}
