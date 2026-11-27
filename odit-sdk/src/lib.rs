//! # ODIT Rust Plugin SDK (`odit-sdk`)
//!
//! Write sandboxed WebAssembly (`.wasm`) plugins for **Onyx Editor (`odit`)** in idiomatic Rust
//! with zero external dependencies and full Host ABI v1.0.0 coverage.
//!
//! ## Quick Start
//! ```ignore
//! use odit_sdk::prelude::*;
//!
//! #[odit_plugin(
//!     id = "user.my_plugin",
//!     title = "My Plugin",
//!     version = "0.1.0",
//!     slot = RightGrid,
//!     width = 360.0,
//!     height = 240.0,
//! )]
//! #[derive(Default)]
//! pub struct MyPlugin {
//!     clicks: u32,
//! }
//!
//! impl Plugin for MyPlugin {
//!     fn init(&mut self, _window_id: u32) -> i32 {
//!         host::register_command("my_plugin.hello", "My Plugin: Say Hello", "");
//!         host::register_setting("my_plugin.greeting", "My Plugin: Greeting", SettingKind::String, "Hello from Rust!");
//!         0
//!     }
//!
//!     fn on_event(&mut self, _window_id: u32, event: Event<'_>) {
//!         if let Event::Command(cmd) = event {
//!             if cmd == "my_plugin.hello" {
//!                 self.clicks += 1;
//!                 notify!(Info, "Hello from Rust! (clicks: {})", self.clicks);
//!             }
//!         }
//!     }
//!
//!     fn render(&mut self, _window_id: u32, ctx: &FrameContext) {
//!         let mut v = ui::VStack::new(12.0, 12.0, ctx.width - 24.0, 8.0);
//!         let greeting = host::get_setting("my_plugin.greeting").unwrap_or_else(|| "Hello!".into());
//!         v.text(&greeting, 15.0, ui::theme_color(ThemeColor::TextPrimary), TextFlags::BOLD);
//!         let by = v.next(30.0);
//!         ui::on_click(1, v.x, by, 140.0, 30.0, "Click Me", ButtonFlags::PRIMARY, || {
//!             self.clicks += 1;
//!         });
//!     }
//! }
//! ```

pub mod abi;
pub mod event;
pub mod ui;

pub use odit_sdk_macros::odit_plugin;

use abi::{AbiError, BuildFlags, BuildTarget, DebugCommand, EditorCursor, FrameContext, GutterMarkerKind, LogLevel, ProcState, SettingKind, StreamMask};
use event::Event;

/// Commonly used types, macros, and modules for ODIT plugins.
pub mod prelude {
    pub use crate::abi::{
        rgba, AbiError, BuildFlags, BuildTarget, ButtonFlags, DebugCommand, DockSlot,
        EditorCursor, EventType, FrameContext, GutterMarkerKind, KeyEvent, LogLevel, ProcState,
        SettingKind, StreamMask, TableRowFlags, TextFlags, ThemeColor, WidgetState,
    };
    pub use crate::event::Event;
    pub use crate::{debug, editor, host, lsp, process, storage, ui, workspace};
    pub use crate::{log_error, log_info, log_warn, notify, odit_plugin, text, Plugin};
}

/// Static manifest metadata generated automatically by `#[odit_plugin(...)]`.
pub trait PluginMeta {
    const MANIFEST_JSON: &'static str;
}

/// Core lifecycle trait implemented by every Rust ODIT plugin.
pub trait Plugin: PluginMeta + Default + 'static {
    /// Called once when the plugin pane is spawned. Return `0` on success.
    fn init(&mut self, _window_id: u32) -> i32 {
        0
    }

    /// Called when the plugin pane is closed or reloaded.
    fn shutdown(&mut self, _window_id: u32) {}

    /// Called when an event matching the plugin's `event_mask` occurs.
    fn on_event(&mut self, _window_id: u32, _event: Event<'_>) {}

    /// Called every frame to draw immediate-mode UI into the plugin pane.
    fn render(&mut self, window_id: u32, ctx: &FrameContext);
}

// =============================================================================
// Internal Buffer-Size Query Helper
// =============================================================================

fn query_string<F: Fn(u32, u32) -> i32>(f: F) -> Option<String> {
    let needed = f(0, 0);
    if needed <= 0 {
        return None;
    }
    let mut buf = vec![0u8; needed as usize];
    let got = f(buf.as_mut_ptr() as u32, buf.len() as u32);
    if got <= 0 {
        return None;
    }
    buf.truncate(got as usize);
    Some(String::from_utf8_lossy(&buf).into_owned())
}

// =============================================================================
// A. Host Core, Commands & Settings
// =============================================================================

pub mod host {
    use super::*;

    pub fn log(level: LogLevel, msg: &str) {
        unsafe { abi::host_log(level as u32, msg.as_ptr() as u32, msg.len() as u32) }
    }

    pub fn set_window_title(title: &str) {
        unsafe { abi::host_set_window_title(title.as_ptr() as u32, title.len() as u32) }
    }

    pub fn request_redraw() {
        unsafe { abi::host_request_redraw() }
    }

    pub fn notify(level: LogLevel, msg: &str) {
        unsafe { abi::host_notify(level as u32, msg.as_ptr() as u32, msg.len() as u32) }
    }

    pub fn register_command(id: &str, label: &str, shortcut: &str) -> u32 {
        unsafe {
            abi::host_register_command(
                id.as_ptr() as u32,
                id.len() as u32,
                label.as_ptr() as u32,
                label.len() as u32,
                shortcut.as_ptr() as u32,
                shortcut.len() as u32,
            )
        }
    }

    pub fn execute_command(id: &str, arg: &str) -> Result<(), AbiError> {
        let rc = unsafe {
            abi::host_execute_command(
                id.as_ptr() as u32,
                id.len() as u32,
                arg.as_ptr() as u32,
                arg.len() as u32,
            )
        };
        if rc >= 0 { Ok(()) } else { Err(AbiError::from_code(rc)) }
    }

    pub fn spawn_plugin(plugin_id: &str) -> u32 {
        unsafe { abi::host_spawn_plugin(plugin_id.as_ptr() as u32, plugin_id.len() as u32) }
    }

    pub fn set_clipboard(text: &str) {
        unsafe { abi::host_set_clipboard(text.as_ptr() as u32, text.len() as u32) }
    }

    pub fn get_clipboard() -> Option<String> {
        query_string(|ptr, cap| unsafe { abi::host_get_clipboard(ptr, cap) })
    }

    pub fn open_url(url: &str) -> bool {
        unsafe { abi::host_open_url(url.as_ptr() as u32, url.len() as u32) >= 0 }
    }

    pub fn register_setting(key: &str, label: &str, kind: SettingKind, default_val: &str) -> u32 {
        unsafe {
            abi::host_register_setting(
                key.as_ptr() as u32,
                key.len() as u32,
                label.as_ptr() as u32,
                label.len() as u32,
                kind as u32,
                default_val.as_ptr() as u32,
                default_val.len() as u32,
            )
        }
    }

    pub fn get_setting(key: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::host_get_setting(key.as_ptr() as u32, key.len() as u32, ptr, cap)
        })
    }

    pub fn get_setting_bool(key: &str, fallback: bool) -> bool {
        match get_setting(key).as_deref() {
            Some(s) if s.starts_with('t') || s.starts_with('T') || s.starts_with('1') => true,
            Some(s) if s.starts_with('f') || s.starts_with('F') || s.starts_with('0') => false,
            _ => fallback,
        }
    }

    pub fn get_setting_i32(key: &str, fallback: i32) -> i32 {
        get_setting(key).and_then(|s| s.trim().parse().ok()).unwrap_or(fallback)
    }

    pub fn get_setting_f32(key: &str, fallback: f32) -> f32 {
        get_setting(key).and_then(|s| s.trim().parse().ok()).unwrap_or(fallback)
    }

    pub fn set_setting(key: &str, val: &str) -> bool {
        unsafe {
            abi::host_set_setting(
                key.as_ptr() as u32,
                key.len() as u32,
                val.as_ptr() as u32,
                val.len() as u32,
            ) >= 0
        }
    }
}

/// Plugin-scoped global storage (`~/.config/odit/plugin_data/<plugin_id>/`).
pub mod storage {
    use super::*;

    pub fn read(key: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::storage_read(key.as_ptr() as u32, key.len() as u32, ptr, cap)
        })
    }

    pub fn write(key: &str, data: &str) -> bool {
        unsafe {
            abi::storage_write(
                key.as_ptr() as u32,
                key.len() as u32,
                data.as_ptr() as u32,
                data.len() as u32,
            ) >= 0
        }
    }

    pub fn delete(key: &str) -> bool {
        unsafe { abi::storage_delete(key.as_ptr() as u32, key.len() as u32) >= 0 }
    }
}

// =============================================================================
// B. Workspace, Editor & LSP Queries
// =============================================================================

pub mod workspace {
    use super::*;

    pub fn root() -> Option<String> {
        query_string(|ptr, cap| unsafe { abi::workspace_get_root(ptr, cap) })
    }

    pub fn list_dir_json(rel_path: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::workspace_list_dir(rel_path.as_ptr() as u32, rel_path.len() as u32, ptr, cap)
        })
    }

    pub fn read_file(rel_path: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::workspace_read_file(rel_path.as_ptr() as u32, rel_path.len() as u32, ptr, cap)
        })
    }

    pub fn write_file(rel_path: &str, data: &str) -> bool {
        unsafe {
            abi::workspace_write_file(
                rel_path.as_ptr() as u32,
                rel_path.len() as u32,
                data.as_ptr() as u32,
                data.len() as u32,
            ) >= 0
        }
    }

    pub fn find_files_json(ext: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::workspace_find_files(ext.as_ptr() as u32, ext.len() as u32, ptr, cap)
        })
    }

    pub fn file_exists(rel_path: &str) -> bool {
        unsafe { abi::workspace_file_exists(rel_path.as_ptr() as u32, rel_path.len() as u32) != 0 }
    }

    pub fn delete_file(rel_path: &str) -> bool {
        unsafe { abi::workspace_delete_file(rel_path.as_ptr() as u32, rel_path.len() as u32) >= 0 }
    }

    pub fn mkdir(rel_path: &str) -> bool {
        unsafe { abi::workspace_mkdir(rel_path.as_ptr() as u32, rel_path.len() as u32) >= 0 }
    }
}

pub mod editor {
    use super::*;

    pub fn active_path() -> Option<String> {
        query_string(|ptr, cap| unsafe { abi::editor_get_active_path(ptr, cap) })
    }

    pub fn buffer() -> Option<String> {
        let len = unsafe { abi::editor_get_buffer_len() };
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        let got = unsafe { abi::editor_get_buffer(buf.as_mut_ptr() as u32, buf.len() as u32) };
        if got <= 0 {
            return None;
        }
        buf.truncate(got as usize);
        Some(String::from_utf8_lossy(&buf).into_owned())
    }

    pub fn cursor() -> EditorCursor {
        let mut c = EditorCursor::default();
        unsafe { abi::editor_get_cursor(&mut c as *mut EditorCursor as u32) };
        c
    }

    pub fn open_file(path: &str, line: i32, col: i32) -> Result<(), AbiError> {
        let rc = unsafe {
            abi::editor_open_file(path.as_ptr() as u32, path.len() as u32, line, col)
        };
        if rc >= 0 { Ok(()) } else { Err(AbiError::from_code(rc)) }
    }

    pub fn set_cursor(line: i32, col: i32) {
        unsafe { abi::editor_set_cursor(line, col) }
    }

    pub fn insert_text(text: &str) -> bool {
        unsafe { abi::editor_insert_text(text.as_ptr() as u32, text.len() as u32) >= 0 }
    }

    pub fn selection() -> Option<String> {
        query_string(|ptr, cap| unsafe { abi::editor_get_selection(ptr, cap) })
    }

    pub fn replace_range(
        start_line: i32,
        start_col: i32,
        end_line: i32,
        end_col: i32,
        text: &str,
    ) -> bool {
        unsafe {
            abi::editor_replace_range(
                start_line,
                start_col,
                end_line,
                end_col,
                text.as_ptr() as u32,
                text.len() as u32,
            ) >= 0
        }
    }

    pub fn save() -> bool {
        unsafe { abi::editor_save() >= 0 }
    }

    pub fn set_gutter_marker(path: &str, line: i32, kind: GutterMarkerKind, rgba: u32) {
        unsafe {
            abi::editor_set_gutter_marker(
                path.as_ptr() as u32,
                path.len() as u32,
                line,
                kind as u32,
                rgba,
            )
        }
    }

    pub fn clear_gutter_markers(kind: GutterMarkerKind) {
        unsafe { abi::editor_clear_gutter_markers(kind as u32) }
    }
}

pub mod lsp {
    use super::*;

    pub fn symbols_json(rel_path: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::ast_get_symbols(rel_path.as_ptr() as u32, rel_path.len() as u32, ptr, cap)
        })
    }

    pub fn references_json(rel_path: &str, line: u32, col: u32) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::ast_get_references(
                rel_path.as_ptr() as u32,
                rel_path.len() as u32,
                line,
                col,
                ptr,
                cap,
            )
        })
    }

    pub fn diagnostics_json(rel_path: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::ast_get_diagnostics(rel_path.as_ptr() as u32, rel_path.len() as u32, ptr, cap)
        })
    }

    pub fn docs_search_json(query: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::docs_search(query.as_ptr() as u32, query.len() as u32, ptr, cap)
        })
    }
}

// =============================================================================
// C. Process Execution, Compiler & Live Debugger
// =============================================================================

pub mod process {
    use super::*;

    pub fn spawn(cmd: &str, args: &[&str], cwd: &str) -> Option<u32> {
        let joined = args.join("\n");
        let h = unsafe {
            abi::proc_spawn(
                cmd.as_ptr() as u32,
                cmd.len() as u32,
                joined.as_ptr() as u32,
                joined.len() as u32,
                cwd.as_ptr() as u32,
                cwd.len() as u32,
            )
        };
        if h != 0 { Some(h) } else { None }
    }

    pub fn poll(handle: u32) -> (ProcState, i32) {
        let mut exit_code: i32 = 0;
        let st = unsafe { abi::proc_poll_status(handle, &mut exit_code as *mut i32 as u32) };
        (ProcState::from_u32(st), exit_code)
    }

    pub fn read(handle: u32, stream: StreamMask, buf: &mut [u8]) -> usize {
        let n = unsafe {
            abi::proc_read_output(handle, stream.bits(), buf.as_mut_ptr() as u32, buf.len() as u32)
        };
        if n > 0 { n as usize } else { 0 }
    }

    pub fn write(handle: u32, data: &str) -> i32 {
        unsafe { abi::proc_write_input(handle, data.as_ptr() as u32, data.len() as u32) }
    }

    pub fn kill(handle: u32) -> i32 {
        unsafe { abi::proc_kill(handle) }
    }

    /// Starts an async HTTP GET request; poll & read the body using [`poll`] and [`read`].
    pub fn http_fetch(url: &str) -> Option<u32> {
        let h = unsafe { abi::http_fetch(url.as_ptr() as u32, url.len() as u32) };
        if h != 0 { Some(h) } else { None }
    }

    pub fn compiler_build(entry: &str, target: BuildTarget, flags: BuildFlags) -> u32 {
        unsafe {
            abi::compiler_build(
                entry.as_ptr() as u32,
                entry.len() as u32,
                target as u32,
                flags.bits(),
            )
        }
    }

    pub fn function_wat_json(file: &str, func: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::compiler_get_function_wat(
                file.as_ptr() as u32,
                file.len() as u32,
                func.as_ptr() as u32,
                func.len() as u32,
                ptr,
                cap,
            )
        })
    }
}

pub mod debug {
    use super::*;

    pub fn toggle_breakpoint(path: &str, line: u32) -> bool {
        unsafe { abi::debug_toggle_breakpoint(path.as_ptr() as u32, path.len() as u32, line) != 0 }
    }

    pub fn breakpoints_json(path: &str) -> Option<String> {
        query_string(|ptr, cap| unsafe {
            abi::debug_get_breakpoints(path.as_ptr() as u32, path.len() as u32, ptr, cap)
        })
    }

    pub fn start(entry: &str) -> i32 {
        unsafe { abi::debug_start(entry.as_ptr() as u32, entry.len() as u32) }
    }

    pub fn command(cmd: DebugCommand) -> i32 {
        unsafe { abi::debug_command(cmd as u32) }
    }

    pub fn state_json() -> Option<String> {
        query_string(|ptr, cap| unsafe { abi::debug_get_state(ptr, cap) })
    }
}

// =============================================================================
// Ergonomic Declarative Macros
// =============================================================================

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::host::log($crate::abi::LogLevel::Info, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::host::log($crate::abi::LogLevel::Warn, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::host::log($crate::abi::LogLevel::Error, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! notify {
    ($level:ident, $($arg:tt)*) => {
        $crate::host::notify($crate::abi::LogLevel::$level, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! text {
    ($x:expr, $y:expr, $size:expr, $color:expr, $($arg:tt)*) => {
        $crate::ui::draw_text($x, $y, &format!($($arg)*), $size, $color, $crate::abi::TextFlags::NONE)
    };
}

// =============================================================================
// Internal Runtime Plumbing Used by `#[odit_plugin]`
// =============================================================================

#[doc(hidden)]
pub mod __rt {
    use crate::abi::{AbiError, FrameContext};
    use crate::{event, Plugin};
    use core::alloc::Layout;

    /// Allocates `size` bytes with an 8-byte header storing the size so `odit_free`
    /// can deallocate safely without needing the size argument.
    pub unsafe fn alloc(size: u32) -> u32 {
        if size == 0 {
            return 0;
        }
        let total = (size as usize).saturating_add(8);
        let Ok(layout) = Layout::from_size_align(total, 8) else {
            return 0;
        };
        let raw = std::alloc::alloc(layout);
        if raw.is_null() {
            return 0;
        }
        *(raw as *mut usize) = total;
        raw.add(8) as u32
    }

    pub unsafe fn free(ptr: u32) {
        if ptr == 0 {
            return;
        }
        let base = (ptr as *mut u8).sub(8);
        let total = *(base as *const usize);
        if let Ok(layout) = Layout::from_size_align(total, 8) {
            std::alloc::dealloc(base, layout);
        }
    }

    pub unsafe fn write_manifest(json: &str, out_ptr: u32, out_cap: u32) -> i32 {
        let bytes = json.as_bytes();
        if out_ptr == 0 || out_cap == 0 {
            return bytes.len() as i32;
        }
        if bytes.len() > out_cap as usize {
            return AbiError::BufferTooSmall as i32;
        }
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_ptr as *mut u8, bytes.len());
        bytes.len() as i32
    }

    pub unsafe fn dispatch_event<P: Plugin>(
        state: &mut P,
        window_id: u32,
        event_type: u32,
        payload_ptr: u32,
        payload_len: u32,
    ) {
        let slice = if payload_ptr != 0 && payload_len > 0 {
            core::slice::from_raw_parts(payload_ptr as *const u8, payload_len as usize)
        } else {
            &[]
        };
        let ev = event::decode_event(event_type, slice);
        state.on_event(window_id, ev);
    }

    pub unsafe fn dispatch_render<P: Plugin>(state: &mut P, window_id: u32, ctx_ptr: u32) {
        if ctx_ptr == 0 {
            return;
        }
        let ctx = &*(ctx_ptr as *const FrameContext);
        state.render(window_id, ctx);
    }
}

/// Internal macro emitted by `#[odit_plugin(...)]` to export all 9 required WASM ABI symbols.
#[doc(hidden)]
#[macro_export]
macro_rules! __export_plugin {
    ($plugin_ty:ident) => {
        mod __odit_exports {
            use super::$plugin_ty;
            use ::core::cell::UnsafeCell;
            use ::odit_sdk::{Plugin, PluginMeta};

            struct PluginCell(UnsafeCell<Option<$plugin_ty>>);
            unsafe impl Sync for PluginCell {}

            static STATE: PluginCell = PluginCell(UnsafeCell::new(None));

            #[inline]
            unsafe fn with_state<R>(f: impl FnOnce(&mut $plugin_ty) -> R) -> R {
                let slot = &mut *STATE.0.get();
                if slot.is_none() {
                    *slot = Some(<$plugin_ty as Default>::default());
                }
                f(slot.as_mut().unwrap())
            }

            #[no_mangle]
            pub extern "C" fn _initialize() {}

            #[no_mangle]
            pub extern "C" fn odit_abi_version() -> u32 {
                ::odit_sdk::abi::ABI_VERSION
            }

            #[no_mangle]
            pub extern "C" fn odit_alloc(size: u32) -> u32 {
                unsafe { ::odit_sdk::__rt::alloc(size) }
            }

            #[no_mangle]
            pub extern "C" fn odit_free(ptr: u32) {
                unsafe { ::odit_sdk::__rt::free(ptr) }
            }

            #[no_mangle]
            pub extern "C" fn odit_plugin_manifest(out_ptr: u32, out_cap: u32) -> i32 {
                unsafe {
                    ::odit_sdk::__rt::write_manifest(
                        <$plugin_ty as PluginMeta>::MANIFEST_JSON,
                        out_ptr,
                        out_cap,
                    )
                }
            }

            #[no_mangle]
            pub extern "C" fn odit_plugin_init(window_id: u32) -> i32 {
                unsafe { with_state(|s| s.init(window_id)) }
            }

            #[no_mangle]
            pub extern "C" fn odit_plugin_shutdown(window_id: u32) {
                unsafe {
                    let slot = &mut *STATE.0.get();
                    if let Some(s) = slot.as_mut() {
                        s.shutdown(window_id);
                    }
                    *slot = None;
                }
            }

            #[no_mangle]
            pub extern "C" fn odit_plugin_event(
                window_id: u32,
                event_type: u32,
                payload_ptr: u32,
                payload_len: u32,
            ) {
                unsafe {
                    with_state(|s| {
                        ::odit_sdk::__rt::dispatch_event(
                            s,
                            window_id,
                            event_type,
                            payload_ptr,
                            payload_len,
                        )
                    });
                }
            }

            #[no_mangle]
            pub extern "C" fn odit_plugin_render(window_id: u32, ctx_ptr: u32) {
                unsafe {
                    with_state(|s| ::odit_sdk::__rt::dispatch_render(s, window_id, ctx_ptr));
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pod_sizes_match_odit_abi_v1() {
        assert_eq!(core::mem::size_of::<abi::FrameContext>(), 48);
        assert_eq!(core::mem::size_of::<abi::EditorCursor>(), 28);
        assert_eq!(core::mem::size_of::<abi::KeyEvent>(), 16);
        assert_eq!(core::mem::size_of::<abi::ProcessEventHeader>(), 16);
    }

    #[test]
    fn json_event_decoders_work() {
        let ev = event::decode_event(
            abi::EventType::CommandInvoked as u32,
            br#"{"id":"my_plugin.hello","arg":""}"#,
        );
        match ev {
            Event::Command(id) => assert_eq!(id, "my_plugin.hello"),
            other => panic!("unexpected event: {:?}", other),
        }

        let bm = event::decode_event(
            abi::EventType::BufferModified as u32,
            br#"{"path":"src/main.onyx","version":42,"saved":true}"#,
        );
        match bm {
            Event::BufferModified { path, version, saved } => {
                assert_eq!(path, "src/main.onyx");
                assert_eq!(version, 42);
                assert!(saved);
            }
            other => panic!("unexpected event: {:?}", other),
        }
    }
}
