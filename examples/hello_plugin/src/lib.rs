//! Minimal example ODIT WASM plugin authored in Rust using `odit-sdk`.
//!
//! Build & install:
//! ```bash
//! cargo build -p hello_plugin --target wasm32-unknown-unknown --release
//! cp target/wasm32-unknown-unknown/release/hello_plugin.wasm ~/.config/odit/plugins/
//! ```

use odit_sdk::prelude::*;

#[odit_plugin(
    id = "user.rust_hello",
    title = "Rust Hello",
    version = "0.1.0",
    slot = RightGrid,
    width = 360.0,
    height = 240.0,
    event_mask = 0xFFFFFFFF,
    allow_multiple = false,
)]
#[derive(Default)]
pub struct HelloPlugin {
    clicks: u32,
    note: String,
    active_file: String,
}

impl Plugin for HelloPlugin {
    fn init(&mut self, _window_id: u32) -> i32 {
        host::register_command("rust_hello.greet", "Rust Hello: Say Hello", "");
        host::register_setting(
            "rust_hello.greeting",
            "Rust Hello: Greeting",
            SettingKind::String,
            "Hello from Rust WASM!",
        );
        self.active_file = editor::active_path().unwrap_or_default();
        log_info!("Rust Hello plugin initialized!");
        0
    }

    fn on_event(&mut self, _window_id: u32, event: Event<'_>) {
        match event {
            Event::ActiveFileChanged(path) => {
                self.active_file = path.to_string();
            }
            Event::Command(cmd) if cmd == "rust_hello.greet" => {
                self.clicks += 1;
                let msg = host::get_setting("rust_hello.greeting")
                    .unwrap_or_else(|| "Hello from Rust!".into());
                notify!(Info, "{} (#{})", msg, self.clicks);
            }
            _ => {}
        }
    }

    fn render(&mut self, _window_id: u32, ctx: &FrameContext) {
        let pad = 12.0;
        let mut v = ui::VStack::new(pad, pad, ctx.width - pad * 2.0, 8.0);

        let greeting = host::get_setting("rust_hello.greeting")
            .unwrap_or_else(|| "Hello from Rust WASM!".into());
        v.text(
            &greeting,
            15.0,
            ui::theme_color(ThemeColor::TextPrimary),
            TextFlags::BOLD,
        );

        let sub = if self.active_file.is_empty() {
            format!("Clicks: {}", self.clicks)
        } else {
            format!("Clicks: {}  |  File: {}", self.clicks, self.active_file)
        };
        v.text(
            &sub,
            12.0,
            ui::theme_color(ThemeColor::TextSecondary),
            TextFlags::ELLIPSIS,
        );

        let iy = v.next(28.0);
        ui::input_text(10, v.x, iy, v.w, 28.0, &mut self.note, "Type a note...");

        let by = v.next(30.0);
        ui::on_click(1, v.x, by, 130.0, 30.0, "Click Me", ButtonFlags::PRIMARY, || {
            self.clicks += 1;
            notify!(Info, "Rust click #{}", self.clicks);
        });
        ui::on_click(
            2,
            v.x + 138.0,
            by,
            110.0,
            30.0,
            "Reload All",
            ButtonFlags::NONE,
            || {
                let _ = host::execute_command("plugin.reload_all", "");
            },
        );
    }
}
