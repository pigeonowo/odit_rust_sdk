//! Low-level ODIT Host <-> WASM Plugin ABI v1.0.0 definitions and raw FFI imports.

use core::ops::{BitAnd, BitOr, BitOrAssign};

pub const ABI_VERSION_MAJOR: u32 = 1;
pub const ABI_VERSION_MINOR: u32 = 0;
pub const ABI_VERSION_PATCH: u32 = 0;
pub const ABI_VERSION: u32 =
    (ABI_VERSION_MAJOR << 16) | (ABI_VERSION_MINOR << 8) | ABI_VERSION_PATCH;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiError {
    Ok = 0,
    Unknown = -1,
    NotFound = -2,
    BufferTooSmall = -3,
    InvalidArg = -4,
    PermissionDenied = -5,
    Busy = -6,
    NotReady = -7,
    ProcessFailed = -8,
    Unsupported = -9,
}

impl AbiError {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            -2 => Self::NotFound,
            -3 => Self::BufferTooSmall,
            -4 => Self::InvalidArg,
            -5 => Self::PermissionDenied,
            -6 => Self::Busy,
            -7 => Self::NotReady,
            -8 => Self::ProcessFailed,
            -9 => Self::Unsupported,
            _ => Self::Unknown,
        }
    }
}

/// Packed RGBA8 color (`0xRRGGBBAA`).
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    BgBase = 0,
    BgPanel = 1,
    BgHeader = 2,
    BgSurface = 3,
    BgHover = 4,
    BgSelection = 5,
    Border = 6,
    BorderFocused = 7,
    TextPrimary = 8,
    TextSecondary = 9,
    TextDisabled = 10,
    Accent = 11,
    Success = 12,
    Warning = 13,
    Error = 14,
    Info = 15,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    String = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSlot {
    RightGrid = 0,
    LeftSide = 1,
    Bottom = 2,
    Floating = 3,
    MainArea = 4,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None = 0,
    ActiveFileChanged = 1,
    BufferModified = 2,
    CursorMoved = 3,
    DiagnosticsUpdated = 4,
    SymbolsUpdated = 5,
    ProcessOutput = 6,
    ProcessExited = 7,
    CommandInvoked = 8,
    DebugStateChanged = 9,
    KeyInput = 10,
    WorkspaceChanged = 11,
}

impl EventType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::ActiveFileChanged,
            2 => Self::BufferModified,
            3 => Self::CursorMoved,
            4 => Self::DiagnosticsUpdated,
            5 => Self::SymbolsUpdated,
            6 => Self::ProcessOutput,
            7 => Self::ProcessExited,
            8 => Self::CommandInvoked,
            9 => Self::DebugStateChanged,
            10 => Self::KeyInput,
            11 => Self::WorkspaceChanged,
            _ => Self::None,
        }
    }
}

macro_rules! define_bitflags {
    ($name:ident { $($const_name:ident = $val:expr;)* }) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct $name(pub u32);

        impl $name {
            pub const NONE: Self = Self(0);
            $(pub const $const_name: Self = Self($val);)*

            #[inline]
            pub const fn bits(self) -> u32 { self.0 }

            #[inline]
            pub const fn contains(self, other: Self) -> bool {
                (self.0 & other.0) == other.0 && other.0 != 0
            }
        }

        impl BitOr for $name {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
        }

        impl BitAnd for $name {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
        }
    };
}

define_bitflags!(TextFlags {
    MONO = 1 << 0;
    BOLD = 1 << 1;
    ALIGN_CENTER = 1 << 2;
    ALIGN_RIGHT = 1 << 3;
    ELLIPSIS = 1 << 4;
});

define_bitflags!(ButtonFlags {
    PRIMARY = 1 << 0;
    DANGER = 1 << 1;
    GHOST = 1 << 2;
    DISABLED = 1 << 3;
    ACTIVE = 1 << 4;
});

define_bitflags!(WidgetState {
    HOVERED = 1 << 0;
    ACTIVE = 1 << 1;
    CLICKED = 1 << 2;
    DOUBLE_CLICKED = 1 << 3;
    CHANGED = 1 << 4;
    SUBMITTED = 1 << 5;
    FOCUSED = 1 << 6;
});

define_bitflags!(TableRowFlags {
    SELECTED = 1 << 0;
    MUTED = 1 << 1;
    HIGHLIGHTED = 1 << 2;
});

define_bitflags!(StreamMask {
    STDOUT = 1 << 0;
    STDERR = 1 << 1;
    BOTH = (1 << 0) | (1 << 1);
});

define_bitflags!(BuildFlags {
    DEBUG_INFO = 1 << 0;
    NAME_SECTION = 1 << 1;
    FUNCTION_MAPPINGS = 1 << 2;
    STACK_TRACE = 1 << 3;
    MULTI_THREADED = 1 << 4;
    RUN_AFTER_BUILD = 1 << 5;
});

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterMarkerKind {
    None = 0,
    Breakpoint = 1,
    CurrentLine = 2,
    DiagnosticErr = 3,
    DiagnosticWarn = 4,
    Custom = 5,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTarget {
    WasmOnyx = 0,
    WasmWasi = 1,
    WasmJs = 2,
    WasmCustom = 3,
    NativeExe = 4,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcState {
    Invalid = 0,
    Running = 1,
    Exited = 2,
    Killed = 3,
}

impl ProcState {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::Running,
            2 => Self::Exited,
            3 => Self::Killed,
            _ => Self::Invalid,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCommand {
    Continue = 1,
    Pause = 2,
    StepOver = 3,
    StepInto = 4,
    StepOut = 5,
    StepInstruction = 6,
    Stop = 7,
}

pub const FRAME_FLAG_FOCUSED: u32 = 1 << 0;
pub const FRAME_FLAG_HOVERED: u32 = 1 << 1;

/// Per-frame context passed to `Plugin::render`. Exact size: 48 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameContext {
    pub window_id: u32,
    pub width: f32,
    pub height: f32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub wheel_y: f32,
    pub mouse_down: u32,
    pub mouse_pressed: u32,
    pub mouse_released: u32,
    pub key_mods: u32,
    pub flags: u32,
    pub delta_time: f32,
}

impl FrameContext {
    #[inline]
    pub const fn is_focused(&self) -> bool {
        (self.flags & FRAME_FLAG_FOCUSED) != 0
    }

    #[inline]
    pub const fn is_hovered(&self) -> bool {
        (self.flags & FRAME_FLAG_HOVERED) != 0
    }
}

/// Active editor cursor snapshot. Exact size: 28 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EditorCursor {
    pub line: u32,
    pub column: u32,
    pub byte_offset: u32,
    pub sel_start: u32,
    pub sel_end: u32,
    pub buffer_version: u32,
    pub is_dirty: u32,
}

/// Delivered with `Event::KeyInput`. Exact size: 16 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyEvent {
    pub key_code: u32,
    pub char_code: u32,
    pub mods: u32,
    pub pressed: u32,
}

/// Prefix header for `EventType::ProcessOutput` / `ProcessExited`. Exact size: 16 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessEventHeader {
    pub handle: u32,
    pub state: u32,
    pub exit_code: i32,
    pub stream: u32,
}

// Compile-time ABI POD layout assertions matching `abi/abi.onyx`
const _: () = assert!(core::mem::size_of::<FrameContext>() == 48);
const _: () = assert!(core::mem::size_of::<EditorCursor>() == 28);
const _: () = assert!(core::mem::size_of::<KeyEvent>() == 16);
const _: () = assert!(core::mem::size_of::<ProcessEventHeader>() == 16);

// =============================================================================
// Raw `"odit_host"` WASM Imports
// =============================================================================

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "odit_host")]
extern "C" {
    // A. Host Core, Logging, Command Palette & Settings
    pub fn host_log(level: u32, msg_ptr: u32, msg_len: u32);
    pub fn host_set_window_title(title_ptr: u32, title_len: u32);
    pub fn host_request_redraw();
    pub fn host_notify(severity: u32, msg_ptr: u32, msg_len: u32);
    pub fn host_register_command(
        id_ptr: u32,
        id_len: u32,
        label_ptr: u32,
        label_len: u32,
        shortcut_ptr: u32,
        shortcut_len: u32,
    ) -> u32;
    pub fn host_execute_command(id_ptr: u32, id_len: u32, arg_ptr: u32, arg_len: u32) -> i32;
    pub fn host_spawn_plugin(plugin_id_ptr: u32, plugin_id_len: u32) -> u32;
    pub fn host_set_clipboard(text_ptr: u32, text_len: u32);
    pub fn host_register_setting(
        key_ptr: u32,
        key_len: u32,
        label_ptr: u32,
        label_len: u32,
        kind: u32,
        def_ptr: u32,
        def_len: u32,
    ) -> u32;
    pub fn host_get_setting(key_ptr: u32, key_len: u32, out_ptr: u32, out_cap: u32) -> i32;
    pub fn host_set_setting(key_ptr: u32, key_len: u32, val_ptr: u32, val_len: u32) -> i32;

    // B. UI Drawing Primitives, Immediate Widgets, Textures & Audio
    pub fn ui_push_clip(x: f32, y: f32, w: f32, h: f32);
    pub fn ui_pop_clip();
    pub fn ui_measure_text(text_ptr: u32, text_len: u32, font_size: f32, flags: u32) -> f32;
    pub fn ui_get_theme_color(role: u32) -> u32;
    pub fn ui_draw_rect(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        fill_rgba: u32,
        border_rgba: u32,
        border_width: f32,
    );
    pub fn ui_draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, rgba: u32);
    pub fn ui_draw_arrow(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        head_size: f32,
        rgba: u32,
    );
    pub fn ui_draw_circle(
        cx: f32,
        cy: f32,
        radius: f32,
        fill_rgba: u32,
        border_rgba: u32,
        border_width: f32,
    );
    pub fn ui_draw_text(
        x: f32,
        y: f32,
        text_ptr: u32,
        text_len: u32,
        font_size: f32,
        rgba: u32,
        flags: u32,
    );
    pub fn ui_pill(
        x: f32,
        y: f32,
        text_ptr: u32,
        text_len: u32,
        bg_rgba: u32,
        fg_rgba: u32,
    ) -> f32;
    pub fn ui_button(
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label_ptr: u32,
        label_len: u32,
        style_flags: u32,
    ) -> u32;
    pub fn ui_input(
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        buf_ptr: u32,
        buf_len: u32,
        buf_cap: u32,
        placeholder_ptr: u32,
        placeholder_len: u32,
        out_len_ptr: u32,
    ) -> u32;
    pub fn ui_dropdown(
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        items_ptr: u32,
        items_len: u32,
        selected_idx: i32,
    ) -> i32;
    pub fn ui_scroll_area_begin(
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        content_h: f32,
        scroll_y_ptr: u32,
    );
    pub fn ui_scroll_area_end();
    pub fn ui_table_begin(
        id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        col_count: u32,
        headers_ptr: u32,
        headers_len: u32,
        col_widths_ptr: u32,
    );
    pub fn ui_table_row(row_id: u32, cells_ptr: u32, cells_len: u32, flags: u32) -> u32;
    pub fn ui_table_end();
    pub fn ui_texture_load(path_ptr: u32, path_len: u32) -> u32;
    pub fn ui_texture_unload(handle: u32);
    pub fn ui_draw_texture(handle: u32, x: f32, y: f32, w: f32, h: f32, tint_rgba: u32);
    pub fn audio_load(path_ptr: u32, path_len: u32) -> u32;
    pub fn audio_load_memory(data_ptr: u32, data_len: u32, ext_ptr: u32, ext_len: u32) -> u32;
    pub fn audio_play(handle: u32, volume: f32);
    pub fn audio_unload(handle: u32);

    // C. Workspace, Editor, AST/LSP & Docs
    pub fn workspace_get_root(out_ptr: u32, out_cap: u32) -> i32;
    pub fn workspace_list_dir(
        rel_path_ptr: u32,
        rel_path_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> i32;
    pub fn workspace_read_file(
        rel_path_ptr: u32,
        rel_path_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> i32;
    pub fn workspace_write_file(
        rel_path_ptr: u32,
        rel_path_len: u32,
        data_ptr: u32,
        data_len: u32,
    ) -> i32;
    pub fn workspace_find_files(ext_ptr: u32, ext_len: u32, out_ptr: u32, out_cap: u32) -> i32;

    pub fn editor_get_active_path(out_ptr: u32, out_cap: u32) -> i32;
    pub fn editor_get_buffer_len() -> i32;
    pub fn editor_get_buffer(out_ptr: u32, out_cap: u32) -> i32;
    pub fn editor_get_cursor(out_cursor_ptr: u32) -> i32;
    pub fn editor_open_file(path_ptr: u32, path_len: u32, line: i32, col: i32) -> i32;
    pub fn editor_set_cursor(line: i32, col: i32);
    pub fn editor_set_gutter_marker(
        path_ptr: u32,
        path_len: u32,
        line: i32,
        kind: u32,
        rgba: u32,
    );
    pub fn editor_clear_gutter_markers(kind: u32);

    pub fn ast_get_symbols(path_ptr: u32, path_len: u32, out_ptr: u32, out_cap: u32) -> i32;
    pub fn ast_get_references(
        path_ptr: u32,
        path_len: u32,
        line: u32,
        col: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> i32;
    pub fn ast_get_diagnostics(path_ptr: u32, path_len: u32, out_ptr: u32, out_cap: u32) -> i32;
    pub fn docs_search(query_ptr: u32, query_len: u32, out_ptr: u32, out_cap: u32) -> i32;

    // D. Process Execution & Compiler / WAT Disassembly
    pub fn proc_spawn(
        cmd_ptr: u32,
        cmd_len: u32,
        args_ptr: u32,
        args_len: u32,
        cwd_ptr: u32,
        cwd_len: u32,
    ) -> u32;
    pub fn proc_poll_status(handle: u32, out_exit_code_ptr: u32) -> u32;
    pub fn proc_read_output(handle: u32, stream_mask: u32, out_ptr: u32, out_cap: u32) -> i32;
    pub fn proc_write_input(handle: u32, data_ptr: u32, data_len: u32) -> i32;
    pub fn proc_kill(handle: u32) -> i32;
    pub fn compiler_build(entry_ptr: u32, entry_len: u32, target: u32, flags: u32) -> u32;
    pub fn compiler_get_function_wat(
        file_ptr: u32,
        file_len: u32,
        func_ptr: u32,
        func_len: u32,
        out_ptr: u32,
        out_cap: u32,
    ) -> i32;

    // E. Live Debugger
    pub fn debug_toggle_breakpoint(path_ptr: u32, path_len: u32, line: u32) -> u32;
    pub fn debug_get_breakpoints(path_ptr: u32, path_len: u32, out_ptr: u32, out_cap: u32) -> i32;
    pub fn debug_start(entry_ptr: u32, entry_len: u32) -> i32;
    pub fn debug_command(cmd: u32) -> i32;
    pub fn debug_get_state(out_ptr: u32, out_cap: u32) -> i32;
}

// Host-target no-op stubs so `cargo test` works natively on x86_64/arm64.
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables, clippy::too_many_arguments)]
pub mod stubs {
    pub unsafe fn host_log(_: u32, _: u32, _: u32) {}
    pub unsafe fn host_set_window_title(_: u32, _: u32) {}
    pub unsafe fn host_request_redraw() {}
    pub unsafe fn host_notify(_: u32, _: u32, _: u32) {}
    pub unsafe fn host_register_command(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn host_execute_command(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn host_spawn_plugin(_: u32, _: u32) -> u32 { 0 }
    pub unsafe fn host_set_clipboard(_: u32, _: u32) {}
    pub unsafe fn host_register_setting(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn host_get_setting(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn host_set_setting(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn ui_push_clip(_: f32, _: f32, _: f32, _: f32) {}
    pub unsafe fn ui_pop_clip() {}
    pub unsafe fn ui_measure_text(_: u32, len: u32, font_size: f32, _: u32) -> f32 { len as f32 * font_size * 0.6 }
    pub unsafe fn ui_get_theme_color(_: u32) -> u32 { 0xCDD6_F4FF }
    pub unsafe fn ui_draw_rect(_: f32, _: f32, _: f32, _: f32, _: f32, _: u32, _: u32, _: f32) {}
    pub unsafe fn ui_draw_line(_: f32, _: f32, _: f32, _: f32, _: f32, _: u32) {}
    pub unsafe fn ui_draw_arrow(_: f32, _: f32, _: f32, _: f32, _: f32, _: f32, _: u32) {}
    pub unsafe fn ui_draw_circle(_: f32, _: f32, _: f32, _: u32, _: u32, _: f32) {}
    pub unsafe fn ui_draw_text(_: f32, _: f32, _: u32, _: u32, _: f32, _: u32, _: u32) {}
    pub unsafe fn ui_pill(_: f32, _: f32, _: u32, len: u32, _: u32, _: u32) -> f32 { len as f32 * 8.0 + 14.0 }
    pub unsafe fn ui_button(_: u32, _: f32, _: f32, _: f32, _: f32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn ui_input(_: u32, _: f32, _: f32, _: f32, _: f32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn ui_dropdown(_: u32, _: f32, _: f32, _: f32, _: f32, _: u32, _: u32, sel: i32) -> i32 { sel }
    pub unsafe fn ui_scroll_area_begin(_: u32, _: f32, _: f32, _: f32, _: f32, _: f32, _: u32) {}
    pub unsafe fn ui_scroll_area_end() {}
    pub unsafe fn ui_table_begin(_: u32, _: f32, _: f32, _: f32, _: f32, _: u32, _: u32, _: u32, _: u32) {}
    pub unsafe fn ui_table_row(_: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn ui_table_end() {}
    pub unsafe fn ui_texture_load(_: u32, _: u32) -> u32 { 0 }
    pub unsafe fn ui_texture_unload(_: u32) {}
    pub unsafe fn ui_draw_texture(_: u32, _: f32, _: f32, _: f32, _: f32, _: u32) {}
    pub unsafe fn audio_load(_: u32, _: u32) -> u32 { 0 }
    pub unsafe fn audio_load_memory(_: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn audio_play(_: u32, _: f32) {}
    pub unsafe fn audio_unload(_: u32) {}
    pub unsafe fn workspace_get_root(_: u32, _: u32) -> i32 { 0 }
    pub unsafe fn workspace_list_dir(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn workspace_read_file(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn workspace_write_file(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn workspace_find_files(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn editor_get_active_path(_: u32, _: u32) -> i32 { 0 }
    pub unsafe fn editor_get_buffer_len() -> i32 { 0 }
    pub unsafe fn editor_get_buffer(_: u32, _: u32) -> i32 { 0 }
    pub unsafe fn editor_get_cursor(_: u32) -> i32 { 0 }
    pub unsafe fn editor_open_file(_: u32, _: u32, _: i32, _: i32) -> i32 { 0 }
    pub unsafe fn editor_set_cursor(_: i32, _: i32) {}
    pub unsafe fn editor_set_gutter_marker(_: u32, _: u32, _: i32, _: u32, _: u32) {}
    pub unsafe fn editor_clear_gutter_markers(_: u32) {}
    pub unsafe fn ast_get_symbols(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn ast_get_references(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn ast_get_diagnostics(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn docs_search(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn proc_spawn(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn proc_poll_status(_: u32, _: u32) -> u32 { 0 }
    pub unsafe fn proc_read_output(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn proc_write_input(_: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn proc_kill(_: u32) -> i32 { 0 }
    pub unsafe fn compiler_build(_: u32, _: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn compiler_get_function_wat(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn debug_toggle_breakpoint(_: u32, _: u32, _: u32) -> u32 { 0 }
    pub unsafe fn debug_get_breakpoints(_: u32, _: u32, _: u32, _: u32) -> i32 { 0 }
    pub unsafe fn debug_start(_: u32, _: u32) -> i32 { 0 }
    pub unsafe fn debug_command(_: u32) -> i32 { 0 }
    pub unsafe fn debug_get_state(_: u32, _: u32) -> i32 { 0 }
}

#[cfg(not(target_arch = "wasm32"))]
pub use stubs::*;
