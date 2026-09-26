//! Minimal POM plugin boundary with an embedded UI contract.

use serde_json::{json, Value};
use std::ffi::{c_char, c_void};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ABI_VERSION: u32 = 1;
static VERSION: &[u8] = b"0.1.0\0";
const UI_MANIFEST: &str = include_str!("../ui/manifest.json");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ByteSlice {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ByteBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HostCallbacks {
    pub log: Option<unsafe extern "C" fn(message: ByteSlice)>,
}

pub type PluginHandle = *mut c_void;
pub type CreateFn = unsafe extern "C" fn(HostCallbacks, ByteSlice) -> PluginHandle;
pub type IngestFn = unsafe extern "C" fn(PluginHandle, ByteSlice) -> i32;
pub type QueryFn = unsafe extern "C" fn(PluginHandle, ByteSlice) -> ByteBuffer;
pub type FreeBufferFn = unsafe extern "C" fn(ByteBuffer);
pub type ShutdownFn = unsafe extern "C" fn(PluginHandle);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PluginApiV1 {
    pub abi_version: u32,
    pub plugin_version: *const c_char,
    pub capabilities: u64,
    pub create: Option<CreateFn>,
    pub ingest: Option<IngestFn>,
    pub query: Option<QueryFn>,
    pub free_buffer: Option<FreeBufferFn>,
    pub shutdown: Option<ShutdownFn>,
}

unsafe impl Sync for PluginApiV1 {}

struct PluginState {
    workspace: Arc<WorkspaceState>,
    upstream: Option<WorkspaceServer>,
}

struct WorkspaceState {
    root: RwLock<Option<PathBuf>>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            root: RwLock::new(None),
        }
    }
}

impl WorkspaceState {
    fn from_root(root: PathBuf) -> Self {
        Self {
            root: RwLock::new(Some(root)),
        }
    }

    fn root(&self) -> Option<PathBuf> {
        self.root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn set_root(&self, root: Option<PathBuf>) {
        *self
            .root
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = root;
    }
}

struct WorkspaceServer {
    port: u16,
    token: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkspaceServer {
    fn start(workspace: Arc<WorkspaceState>) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let port = listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        let token = upstream_token(port);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread_token = token.clone();
        let thread = thread::Builder::new()
            .name("pom-plugin-base-workspace".into())
            .spawn(move || serve_workspace(listener, workspace, thread_token, thread_stop))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            port,
            token,
            stop,
            thread: Some(thread),
        })
    }

    fn response(&self) -> Value {
        json!({"status": "ready", "port": self.port, "token": self.token})
    }
}

impl Drop for WorkspaceServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn upstream_token(port: u16) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("{nanos:032x}{:08x}{port:04x}", std::process::id())
}

fn serve_workspace(
    listener: TcpListener,
    workspace: Arc<WorkspaceState>,
    token: String,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => handle_workspace_request(stream, &workspace, &token),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}

fn handle_workspace_request(mut stream: TcpStream, workspace: &WorkspaceState, token: &str) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    while request.len() < 16 * 1024 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                request.extend_from_slice(&chunk[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return,
        }
    }

    let request = String::from_utf8_lossy(&request);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();
    let authorized = lines.any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("x-pom-plugin-token") && value.trim() == token
        })
    });

    let path = path.split('?').next().unwrap_or_default();
    if !authorized {
        write_json_response(
            &mut stream,
            "401 Unauthorized",
            &json!({"error": "unauthorized"}),
        );
    } else if method == "GET" && path == "/projects" {
        write_json_response(&mut stream, "200 OK", &workspace_snapshot(workspace));
    } else {
        write_json_response(&mut stream, "404 Not Found", &json!({"error": "not found"}));
    }
}

fn write_json_response(stream: &mut TcpStream, status: &str, body: &Value) {
    let Ok(body) = serde_json::to_vec(body) else {
        return;
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn configure_workspace(state: &WorkspaceState, request: &Value) -> Result<Value, String> {
    let root = match request.get("workspace_root") {
        None | Some(Value::Null) => None,
        Some(Value::String(path)) if !path.trim().is_empty() => Some(PathBuf::from(path)),
        Some(Value::String(_)) => None,
        Some(_) => return Err("workspace_root must be a string or null".into()),
    };
    state.set_root(root);
    Ok(json!({"status": "ok"}))
}

fn workspace_snapshot(state: &WorkspaceState) -> Value {
    let Some(root) = state.root() else {
        return json!({"status": "unavailable", "projects": []});
    };
    let Ok(entries) = fs::read_dir(root) else {
        return json!({"status": "unavailable", "projects": []});
    };
    let mut projects = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_owned();
            if name.starts_with('.') || !entry.file_type().ok()?.is_dir() {
                return None;
            }
            Some(name)
        })
        .collect::<Vec<_>>();
    projects.sort();
    if projects.is_empty() {
        json!({"status": "empty", "projects": []})
    } else {
        json!({"status": "ready", "projects": projects})
    }
}

impl PluginState {
    fn new() -> Self {
        let workspace = Arc::new(WorkspaceState::default());
        let upstream = WorkspaceServer::start(Arc::clone(&workspace)).ok();
        Self {
            workspace,
            upstream,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));

unsafe fn input_bytes<'a>(input: ByteSlice) -> Result<&'a [u8], String> {
    if input.len == 0 {
        return Ok(&[]);
    }
    if input.ptr.is_null() {
        return Err("null byte slice".into());
    }
    Ok(std::slice::from_raw_parts(input.ptr, input.len))
}

fn ui_asset(path: &str) -> Option<&'static (&'static str, &'static str, &'static [u8])> {
    UI_ASSETS.iter().find(|(name, _, _)| *name == path)
}

fn ui_manifest() -> Result<Value, String> {
    let manifest: Value = serde_json::from_str(UI_MANIFEST).map_err(|error| error.to_string())?;
    let assets = manifest["assets"]
        .as_array()
        .ok_or("manifest has no assets")?;
    for asset in assets.iter().filter_map(Value::as_str) {
        if ui_asset(asset).is_none() {
            return Err(format!("asset is not embedded: {asset}"));
        }
    }
    Ok(manifest)
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for index in 0..4 {
            if index <= chunk.len() {
                output.push(TABLE[((value >> (18 - 6 * index)) & 63) as usize] as char);
            } else {
                output.push('=');
            }
        }
    }
    output
}

fn query_inner(state: &PluginState, request: &[u8]) -> Result<Value, String> {
    let request: Value = serde_json::from_slice(request).map_err(|error| error.to_string())?;
    match request["operation"].as_str().unwrap_or_default() {
        "host.configure" => configure_workspace(&state.workspace, &request),
        "workspace.projects" => Ok(workspace_snapshot(&state.workspace)),
        "ui.upstream" => state
            .upstream
            .as_ref()
            .map(WorkspaceServer::response)
            .ok_or_else(|| "workspace upstream is unavailable".into()),
        "ui.manifest" => ui_manifest(),
        "ui.asset" => {
            let path = request["path"].as_str().ok_or("asset path is missing")?;
            let (_, content_type, bytes) = ui_asset(path).ok_or("unknown asset")?;
            Ok(json!({"content_type": content_type, "base64": encode_base64(bytes)}))
        }
        _ => Err("unknown operation".into()),
    }
}

fn buffer_from_bytes(bytes: Vec<u8>) -> ByteBuffer {
    let mut bytes = bytes.into_boxed_slice();
    let buffer = ByteBuffer {
        ptr: bytes.as_mut_ptr(),
        len: bytes.len(),
    };
    std::mem::forget(bytes);
    buffer
}

unsafe extern "C" fn create(_: HostCallbacks, config: ByteSlice) -> PluginHandle {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let config = input_bytes(config)?;
        if !config.is_empty() {
            serde_json::from_slice::<Value>(config).map_err(|error| error.to_string())?;
        }
        Ok::<_, String>(Box::into_raw(Box::new(PluginState::new())).cast::<c_void>())
    }));
    match result {
        Ok(Ok(handle)) => handle,
        _ => std::ptr::null_mut(),
    }
}

unsafe extern "C" fn ingest(handle: PluginHandle, event: ByteSlice) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if handle.is_null() {
            return false;
        }
        input_bytes(event).is_ok()
    }));
    matches!(result, Ok(true)).then_some(0).unwrap_or(-1)
}

unsafe extern "C" fn query(handle: PluginHandle, request: ByteSlice) -> ByteBuffer {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err("plugin handle is null".to_owned());
        }
        query_inner(&*handle.cast::<PluginState>(), input_bytes(request)?)
            .and_then(|value| serde_json::to_vec(&value).map_err(|error| error.to_string()))
    }));
    match result {
        Ok(Ok(bytes)) => buffer_from_bytes(bytes),
        _ => ByteBuffer {
            ptr: std::ptr::null_mut(),
            len: 0,
        },
    }
}

unsafe extern "C" fn free_buffer(buffer: ByteBuffer) {
    if !buffer.ptr.is_null() {
        let slice = std::ptr::slice_from_raw_parts_mut(buffer.ptr, buffer.len);
        drop(Box::from_raw(slice));
    }
}

unsafe extern "C" fn shutdown(handle: PluginHandle) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(Box::from_raw(handle.cast::<PluginState>()));
        }
    }));
}

static API: PluginApiV1 = PluginApiV1 {
    abi_version: ABI_VERSION,
    plugin_version: VERSION.as_ptr().cast(),
    capabilities: 0,
    create: Some(create),
    ingest: Some(ingest),
    query: Some(query),
    free_buffer: Some(free_buffer),
    shutdown: Some(shutdown),
};

#[no_mangle]
pub unsafe extern "C" fn pom_base_plugin_v1() -> *const PluginApiV1 {
    &API
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_workspace() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("pom-plugin-base-{suffix}"));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn entry_exposes_the_generic_v1_boundary() {
        let api = unsafe { &*pom_base_plugin_v1() };
        assert_eq!(api.abi_version, ABI_VERSION);
        assert_eq!(api.capabilities, 0);
        assert!(api.create.is_some());
        assert!(api.ingest.is_some());
        assert!(api.query.is_some());
        assert!(api.free_buffer.is_some());
        assert!(api.shutdown.is_some());
    }

    #[test]
    fn buffer_encoding_uses_standard_padding() {
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
    }

    #[test]
    fn manifest_resolves_all_built_assets() {
        let manifest: Value = serde_json::from_str(UI_MANIFEST).unwrap();
        if ui_asset("ui/screens.js").is_some() {
            assert!(ui_manifest().is_ok());
            for asset in manifest["assets"].as_array().unwrap() {
                assert!(ui_asset(asset.as_str().unwrap()).is_some());
            }
        } else {
            assert!(ui_manifest().is_err());
        }
    }

    #[test]
    fn host_configure_reads_and_clears_the_top_level_workspace_root() {
        let root = temporary_workspace();
        let state = WorkspaceState::default();

        assert_eq!(
            configure_workspace(
                &state,
                &json!({"operation": "host.configure", "workspace_root": root})
            )
            .unwrap(),
            json!({"status": "ok"})
        );
        assert_eq!(state.root(), Some(root.clone()));

        configure_workspace(&state, &json!({"operation": "host.configure"})).unwrap();
        assert_eq!(state.root(), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn host_configure_rejects_a_non_string_workspace_root() {
        let state = WorkspaceState::default();
        let error = configure_workspace(
            &state,
            &json!({"operation": "host.configure", "workspace_root": 42}),
        )
        .unwrap_err();

        assert!(error.contains("workspace_root"));
        assert_eq!(state.root(), None);
    }

    #[test]
    fn workspace_snapshot_lists_visible_directories_only() {
        let root = temporary_workspace();
        fs::create_dir(root.join("alpha")).unwrap();
        fs::create_dir(root.join("beta")).unwrap();
        fs::create_dir(root.join(".plugin-state")).unwrap();
        fs::write(root.join("notes.txt"), "not a project").unwrap();
        let state = WorkspaceState::from_root(root.clone());

        assert_eq!(
            workspace_snapshot(&state),
            json!({
                "status": "ready",
                "projects": ["alpha", "beta"]
            })
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_snapshot_reports_empty_and_unavailable_roots() {
        let empty = temporary_workspace();
        assert_eq!(
            workspace_snapshot(&WorkspaceState::from_root(empty.clone())),
            json!({"status": "empty", "projects": []})
        );
        fs::remove_dir_all(empty).unwrap();

        let missing = std::env::temp_dir().join("pom-plugin-base-root-that-does-not-exist");
        let file = temporary_workspace().join("not-a-directory");
        fs::write(&file, "file").unwrap();
        assert_eq!(
            workspace_snapshot(&WorkspaceState::from_root(missing)),
            json!({"status": "unavailable", "projects": []})
        );
        assert_eq!(
            workspace_snapshot(&WorkspaceState::from_root(file.clone())),
            json!({"status": "unavailable", "projects": []})
        );
        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }
}
