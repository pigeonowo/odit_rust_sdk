//! Safe immediate-mode UI widgets, scoped layout helpers, textures, and audio.

use crate::abi::{self, ButtonFlags, TableRowFlags, TextFlags, ThemeColor, WidgetState};

#[inline]
pub fn theme_color(role: ThemeColor) -> u32 {
    unsafe { abi::ui_get_theme_color(role as u32) }
}

#[inline]
pub fn measure_text(text: &str, font_size: f32, flags: TextFlags) -> f32 {
    unsafe { abi::ui_measure_text(text.as_ptr() as u32, text.len() as u32, font_size, flags.bits()) }
}

#[inline]
pub fn draw_rect(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill_rgba: u32,
    radius: f32,
    border_rgba: u32,
    border_width: f32,
) {
    unsafe { abi::ui_draw_rect(x, y, w, h, radius, fill_rgba, border_rgba, border_width) }
}

#[inline]
pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, rgba: u32) {
    unsafe { abi::ui_draw_line(x1, y1, x2, y2, thickness, rgba) }
}

#[inline]
pub fn draw_arrow(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    head_size: f32,
    rgba: u32,
) {
    unsafe { abi::ui_draw_arrow(x1, y1, x2, y2, thickness, head_size, rgba) }
}

#[inline]
pub fn draw_circle(
    cx: f32,
    cy: f32,
    radius: f32,
    fill_rgba: u32,
    border_rgba: u32,
    border_width: f32,
) {
    unsafe { abi::ui_draw_circle(cx, cy, radius, fill_rgba, border_rgba, border_width) }
}

#[inline]
pub fn draw_text(x: f32, y: f32, text: &str, font_size: f32, rgba: u32, flags: TextFlags) {
    if text.is_empty() {
        return;
    }
    unsafe {
        abi::ui_draw_text(
            x,
            y,
            text.as_ptr() as u32,
            text.len() as u32,
            font_size,
            rgba,
            flags.bits(),
        )
    }
}

#[inline]
pub fn pill(x: f32, y: f32, text: &str, bg_rgba: u32, fg_rgba: u32) -> f32 {
    unsafe { abi::ui_pill(x, y, text.as_ptr() as u32, text.len() as u32, bg_rgba, fg_rgba) }
}

#[inline]
pub fn button(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    style: ButtonFlags,
) -> WidgetState {
    WidgetState(unsafe {
        abi::ui_button(
            id,
            x,
            y,
            w,
            h,
            label.as_ptr() as u32,
            label.len() as u32,
            style.bits(),
        )
    })
}

/// Draws a button and runs `f` if clicked this frame.
#[inline]
pub fn on_click<F: FnOnce()>(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    style: ButtonFlags,
    f: F,
) {
    if button(id, x, y, w, h, label, style).contains(WidgetState::CLICKED) {
        f();
    }
}

/// Draws an editable text input backed by a `String`.
pub fn input_text(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &mut String,
    placeholder: &str,
) -> WidgetState {
    let cap = text.len().max(256).min(4096);
    let mut buf = vec![0u8; cap];
    let initial_len = text.len().min(cap);
    buf[..initial_len].copy_from_slice(&text.as_bytes()[..initial_len]);
    let mut out_len = initial_len as u32;

    let state = WidgetState(unsafe {
        abi::ui_input(
            id,
            x,
            y,
            w,
            h,
            buf.as_mut_ptr() as u32,
            initial_len as u32,
            cap as u32,
            placeholder.as_ptr() as u32,
            placeholder.len() as u32,
            &mut out_len as *mut u32 as u32,
        )
    });

    let new_len = (out_len as usize).min(cap);
    if new_len != initial_len || state.contains(WidgetState::CHANGED) {
        *text = String::from_utf8_lossy(&buf[..new_len]).into_owned();
    }
    state
}

#[inline]
pub fn dropdown(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    items_newline_sep: &str,
    selected: i32,
) -> i32 {
    unsafe {
        abi::ui_dropdown(
            id,
            x,
            y,
            w,
            h,
            items_newline_sep.as_ptr() as u32,
            items_newline_sep.len() as u32,
            selected,
        )
    }
}

/// Runs `body` inside a scissor-clipped region, automatically popping the clip.
#[inline]
pub fn clipped<F: FnOnce()>(x: f32, y: f32, w: f32, h: f32, body: F) {
    unsafe { abi::ui_push_clip(x, y, w, h) };
    body();
    unsafe { abi::ui_pop_clip() };
}

/// Runs `body` inside a themed surface card with an optional title header.
#[inline]
pub fn card<F: FnOnce()>(x: f32, y: f32, w: f32, h: f32, title: &str, body: F) {
    draw_rect(
        x,
        y,
        w,
        h,
        theme_color(ThemeColor::BgSurface),
        6.0,
        theme_color(ThemeColor::Border),
        1.0,
    );
    if !title.is_empty() {
        draw_text(
            x + 10.0,
            y + 8.0,
            title,
            13.0,
            theme_color(ThemeColor::TextPrimary),
            TextFlags::BOLD,
        );
    }
    body();
}

/// Runs `body` inside a scrollable clipped viewport, automatically updating `scroll_y`.
#[inline]
pub fn scroll_area<F: FnOnce()>(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    content_h: f32,
    scroll_y: &mut f32,
    body: F,
) {
    unsafe {
        abi::ui_scroll_area_begin(id, x, y, w, h, content_h, scroll_y as *mut f32 as u32);
    }
    body();
    unsafe { abi::ui_scroll_area_end() };
}

#[inline]
pub fn table_row(row_id: u32, cells_tab_sep: &str, flags: TableRowFlags) -> WidgetState {
    WidgetState(unsafe {
        abi::ui_table_row(
            row_id,
            cells_tab_sep.as_ptr() as u32,
            cells_tab_sep.len() as u32,
            flags.bits(),
        )
    })
}

/// Runs `body` inside a structured table, automatically calling `ui_table_end`.
#[inline]
pub fn table<F: FnOnce()>(
    id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    headers_tab_sep: &str,
    col_widths: &[f32],
    body: F,
) {
    unsafe {
        abi::ui_table_begin(
            id,
            x,
            y,
            w,
            h,
            col_widths.len() as u32,
            headers_tab_sep.as_ptr() as u32,
            headers_tab_sep.len() as u32,
            col_widths.as_ptr() as u32,
        );
    }
    body();
    unsafe { abi::ui_table_end() };
}

/// Simple vertical flow layout helper.
#[derive(Debug, Clone, Copy)]
pub struct VStack {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub gap: f32,
}

impl VStack {
    #[inline]
    pub const fn new(x: f32, y: f32, w: f32, gap: f32) -> Self {
        Self { x, y, w, gap }
    }

    #[inline]
    pub fn next(&mut self, h: f32) -> f32 {
        let cur = self.y;
        self.y += h + self.gap;
        cur
    }

    #[inline]
    pub fn spacer(&mut self, h: f32) {
        self.y += h;
    }

    #[inline]
    pub fn text(&mut self, text: &str, font_size: f32, color: u32, flags: TextFlags) {
        let y = self.next(font_size + 4.0);
        draw_text(self.x, y, text, font_size, color, flags);
    }

    #[inline]
    pub fn divider(&mut self) {
        let y = self.next(1.0);
        draw_line(
            self.x,
            y,
            self.x + self.w,
            y,
            1.0,
            theme_color(ThemeColor::Border),
        );
    }
}

/// Host-managed GPU texture handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture(pub u32);

impl Texture {
    pub fn load(rel_path: &str) -> Option<Self> {
        let h = unsafe { abi::ui_texture_load(rel_path.as_ptr() as u32, rel_path.len() as u32) };
        if h != 0 { Some(Self(h)) } else { None }
    }

    pub fn draw(self, x: f32, y: f32, w: f32, h: f32, tint_rgba: u32) {
        unsafe { abi::ui_draw_texture(self.0, x, y, w, h, tint_rgba) }
    }

    pub fn unload(self) {
        unsafe { abi::ui_texture_unload(self.0) }
    }
}

/// Host-managed audio sound handle (`.wav`, `.mp3`, `.ogg`, `.flac`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sound(pub u32);

impl Sound {
    pub fn load(rel_path: &str) -> Option<Self> {
        let h = unsafe { abi::audio_load(rel_path.as_ptr() as u32, rel_path.len() as u32) };
        if h != 0 { Some(Self(h)) } else { None }
    }

    /// Loads a sound directly from embedded bytes (e.g. `include_bytes!("sfx.mp3")`).
    /// `ext` may be with or without a leading dot (`".mp3"` or `"mp3"`).
    pub fn load_memory(bytes: &[u8], ext: &str) -> Option<Self> {
        let h = unsafe {
            abi::audio_load_memory(
                bytes.as_ptr() as u32,
                bytes.len() as u32,
                ext.as_ptr() as u32,
                ext.len() as u32,
            )
        };
        if h != 0 { Some(Self(h)) } else { None }
    }

    pub fn play(self, volume: f32) {
        unsafe { abi::audio_play(self.0, volume) }
    }

    pub fn unload(self) {
        unsafe { abi::audio_unload(self.0) }
    }
}
