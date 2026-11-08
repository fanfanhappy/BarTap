//! BarTap 全局主题：WinUI 3 / Fluent 2 的 Light、Dark 与跟随系统。
//!
//! 主题偏好在设置中保存，`ThemeManager` 每帧解析系统状态并生成一份
//! `ThemeTokens`。所有 egui 窗口、菜单、输入框和自绘岛体都从同一份
//! Token 取色，不再按页面各写一套深浅色。

use eframe::egui::{self, Color32};
use std::cell::Cell;

/// 用户选择的主题模式。`System` 跟随 Windows 的应用主题。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsTheme {
    #[default]
    System,
    Light,
    Dark,
}

pub type ThemeMode = SettingsTheme;

impl SettingsTheme {
    pub fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
    pub fn parse(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
    pub fn dark(self, system_dark: bool) -> bool {
        match self {
            Self::System => system_dark,
            Self::Light => false,
            Self::Dark => true,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "跟随系统",
            Self::Light => "浅色",
            Self::Dark => "深色",
        }
    }
}

/// 一套 Fluent 颜色角色。页面代码只应引用这些角色，而不是具体色值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeTokens {
    pub dark: bool,
    pub window_bg: Color32,
    pub card_bg: Color32,
    pub secondary_bg: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    pub control_bg: Color32,
    pub control_hover: Color32,
    pub control_pressed: Color32,
    pub control_disabled: Color32,
    pub text_disabled: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
}

impl ThemeTokens {
    /// Windows 11 设置应用的浅色基准。
    pub const LIGHT: Self = Self {
        dark: false,
        window_bg: Color32::from_rgb(0xF3, 0xF3, 0xF3),
        card_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        secondary_bg: Color32::from_rgb(0xF8, 0xF8, 0xF8),
        text: Color32::from_rgb(0x1A, 0x1A, 0x1A),
        text_secondary: Color32::from_rgb(0x60, 0x60, 0x60),
        border: Color32::from_rgb(0xE5, 0xE5, 0xE5),
        accent: Color32::from_rgb(0x00, 0x67, 0xC0),
        accent_hover: Color32::from_rgb(0x19, 0x75, 0xC5),
        accent_pressed: Color32::from_rgb(0x00, 0x5A, 0x9E),
        control_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        control_hover: Color32::from_rgb(0xF8, 0xF8, 0xF8),
        control_pressed: Color32::from_rgb(0xED, 0xED, 0xED),
        control_disabled: Color32::from_rgb(0xF3, 0xF3, 0xF3),
        text_disabled: Color32::from_rgb(0x9A, 0x9A, 0x9A),
        danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
        success: Color32::from_rgb(0x0F, 0x7B, 0x0F),
        warning: Color32::from_rgb(0x9D, 0x5D, 0x00),
    };

    /// Windows 11 设置应用的深色基准。
    pub const DARK: Self = Self {
        dark: true,
        window_bg: Color32::from_rgb(0x20, 0x20, 0x20),
        card_bg: Color32::from_rgb(0x2B, 0x2B, 0x2B),
        secondary_bg: Color32::from_rgb(0x32, 0x32, 0x32),
        text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        text_secondary: Color32::from_rgb(0xC5, 0xC5, 0xC5),
        border: Color32::from_rgb(0x45, 0x45, 0x45),
        accent: Color32::from_rgb(0x60, 0xCD, 0xFF),
        accent_hover: Color32::from_rgb(0x7B, 0xD7, 0xFF),
        accent_pressed: Color32::from_rgb(0x4C, 0xB8, 0xE8),
        control_bg: Color32::from_rgb(0x32, 0x32, 0x32),
        control_hover: Color32::from_rgb(0x3A, 0x3A, 0x3A),
        control_pressed: Color32::from_rgb(0x45, 0x45, 0x45),
        control_disabled: Color32::from_rgb(0x2A, 0x2A, 0x2A),
        text_disabled: Color32::from_rgb(0x8A, 0x8A, 0x8A),
        danger: Color32::from_rgb(0xFF, 0x99, 0xA4),
        success: Color32::from_rgb(0x6C, 0xCB, 0x5F),
        warning: Color32::from_rgb(0xFC, 0xE1, 0x00),
    };

    pub fn for_dark(dark: bool) -> Self {
        if dark {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }

    pub fn alpha(self, color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }
}

thread_local! {
    static CURRENT: Cell<ThemeTokens> = const { Cell::new(ThemeTokens::DARK) };
}

/// 当前线程正在使用的主题。绘制代码用它取色，避免每个函数都传 Token。
pub fn current() -> ThemeTokens {
    CURRENT.with(Cell::get)
}

fn publish(tokens: ThemeTokens) {
    CURRENT.with(|slot| slot.set(tokens));
}

/// 解析主题、监听系统变化，并把 Fluent Visuals 应用到 egui。
#[derive(Clone, Copy, Debug)]
pub struct ThemeManager {
    mode: SettingsTheme,
    system_dark: bool,
    tokens: ThemeTokens,
}

impl ThemeManager {
    pub fn new(mode: SettingsTheme, system_dark: bool) -> Self {
        let tokens = ThemeTokens::for_dark(mode.dark(system_dark));
        publish(tokens);
        Self { mode, system_dark, tokens }
    }

    pub fn mode(self) -> SettingsTheme {
        self.mode
    }

    pub fn system_dark(self) -> bool {
        self.system_dark
    }

    pub fn tokens(self) -> ThemeTokens {
        self.tokens
    }

    /// 返回主题是否发生了变化；同时刷新线程内 Token。
    pub fn sync(&mut self, mode: SettingsTheme, system_dark: bool) -> bool {
        let next = ThemeTokens::for_dark(mode.dark(system_dark));
        let changed = next != self.tokens || mode != self.mode;
        self.mode = mode;
        self.system_dark = system_dark;
        self.tokens = next;
        publish(next);
        changed
    }

    /// 只刷新当前线程 Token，不重设 egui Visuals（用于每帧取色）。
    pub fn activate(self) {
        publish(self.tokens);
    }

    /// 把一套已知 Token 应用到另一个 viewport（邮件全文、更新弹窗等）。
    pub fn apply_tokens(ctx: &egui::Context, tokens: ThemeTokens) {
        let manager = Self { mode: SettingsTheme::System, system_dark: tokens.dark, tokens };
        manager.apply(ctx);
    }

    /// 应用一套完整的 Fluent 控件样式。
    pub fn apply(self, ctx: &egui::Context) {
        publish(self.tokens);
        let t = self.tokens;
        let mut v = if t.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.dark_mode = t.dark;
        v.override_text_color = Some(t.text);
        v.panel_fill = t.window_bg;
        v.window_fill = t.card_bg;
        v.faint_bg_color = t.secondary_bg;
        v.extreme_bg_color = t.secondary_bg;
        v.code_bg_color = t.secondary_bg;
        v.window_stroke = egui::Stroke::new(1.0_f32, t.border);
        v.window_shadow = egui::epaint::Shadow {
            offset: [0, 10],
            blur: 28,
            spread: 0,
            color: Color32::from_black_alpha(if t.dark { 130 } else { 36 }),
        };
        v.popup_shadow = egui::epaint::Shadow {
            offset: [0, 6],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(if t.dark { 150 } else { 42 }),
        };
        v.hyperlink_color = t.accent;
        v.selection.bg_fill = t.accent.gamma_multiply(0.32);
        v.selection.stroke = egui::Stroke::new(1.0_f32, t.accent);
        v.text_cursor.stroke = egui::Stroke::new(1.5_f32, t.accent);
        v.widgets.noninteractive.bg_fill = t.card_bg;
        v.widgets.noninteractive.weak_bg_fill = t.card_bg;
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, t.text);
        v.widgets.inactive.bg_fill = t.control_bg;
        v.widgets.inactive.weak_bg_fill = t.control_bg;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, t.text);
        v.widgets.hovered.bg_fill = t.control_hover;
        v.widgets.hovered.weak_bg_fill = t.control_hover;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
        v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, t.text);
        v.widgets.active.bg_fill = t.control_pressed;
        v.widgets.active.weak_bg_fill = t.control_pressed;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, t.accent);
        v.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, t.text);
        v.widgets.open.bg_fill = t.secondary_bg;
        v.widgets.open.weak_bg_fill = t.secondary_bg;
        v.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, t.border);
        v.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, t.text);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = egui::CornerRadius::same(8);
        }
        ctx.set_visuals(v);
    }
}

/// 读取 Windows 当前的应用主题（`AppsUseLightTheme`）。
pub fn system_dark(ctx: &egui::Context) -> bool {
    let id = egui::Id::new("barta-system-theme");
    if let Some((at, dark)) = ctx.data(|d| d.get_temp::<(std::time::Instant, bool)>(id)) {
        if at.elapsed() < std::time::Duration::from_secs(1) {
            return dark;
        }
    }
    #[cfg(windows)]
    let dark = {
        use windows::{core::w, Win32::System::Registry::*};
        let mut value = 1u32;
        let mut size = 4u32;
        unsafe {
            let _ = RegGetValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
                w!("AppsUseLightTheme"),
                RRF_RT_REG_DWORD,
                None,
                Some((&mut value as *mut u32).cast()),
                Some(&mut size),
            );
        }
        value == 0
    };
    #[cfg(not(windows))]
    let dark = ctx.input(|i| i.raw.system_theme) == Some(egui::Theme::Dark);
    ctx.data_mut(|d| d.insert_temp(id, (std::time::Instant::now(), dark)));
    dark
}

/// 兼容旧调用：根据当前主题从一对深浅值中取一个。
pub fn color(_ctx: &egui::Context, dark: Color32, light: Color32) -> Color32 {
    if current().dark {
        dark
    } else {
        light
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_modes_resolve_light_dark_and_system() {
        assert!(!SettingsTheme::Light.dark(true));
        assert!(SettingsTheme::Dark.dark(false));
        assert!(SettingsTheme::System.dark(true));
        assert!(!SettingsTheme::System.dark(false));
        assert_eq!(SettingsTheme::parse("system"), SettingsTheme::System);
        assert_eq!(SettingsTheme::parse("light"), SettingsTheme::Light);
        assert_eq!(SettingsTheme::parse("dark"), SettingsTheme::Dark);
        assert_eq!(SettingsTheme::parse("unknown"), SettingsTheme::System);
    }

    #[test]
    fn fluent_palette_matches_winui_reference_values() {
        assert_eq!(ThemeTokens::LIGHT.window_bg, Color32::from_rgb(0xF3, 0xF3, 0xF3));
        assert_eq!(ThemeTokens::LIGHT.card_bg, Color32::from_rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(ThemeTokens::LIGHT.text, Color32::from_rgb(0x1A, 0x1A, 0x1A));
        assert_eq!(ThemeTokens::LIGHT.text_secondary, Color32::from_rgb(0x60, 0x60, 0x60));
        assert_eq!(ThemeTokens::LIGHT.border, Color32::from_rgb(0xE5, 0xE5, 0xE5));
        assert_eq!(ThemeTokens::LIGHT.accent, Color32::from_rgb(0x00, 0x67, 0xC0));
        assert_eq!(ThemeTokens::DARK.window_bg, Color32::from_rgb(0x20, 0x20, 0x20));
        assert_eq!(ThemeTokens::DARK.card_bg, Color32::from_rgb(0x2B, 0x2B, 0x2B));
        assert_eq!(ThemeTokens::DARK.text, Color32::WHITE);
        assert_eq!(ThemeTokens::DARK.text_secondary, Color32::from_rgb(0xC5, 0xC5, 0xC5));
        assert_eq!(ThemeTokens::DARK.border, Color32::from_rgb(0x45, 0x45, 0x45));
        assert_eq!(ThemeTokens::DARK.accent, Color32::from_rgb(0x60, 0xCD, 0xFF));
    }

    #[test]
    fn manager_syncs_system_changes_and_publishes_tokens() {
        let mut manager = ThemeManager::new(SettingsTheme::System, false);
        assert!(!manager.tokens().dark);
        assert_eq!(current().window_bg, ThemeTokens::LIGHT.window_bg);
        assert!(manager.sync(SettingsTheme::System, true));
        assert!(manager.tokens().dark);
        assert_eq!(current().window_bg, ThemeTokens::DARK.window_bg);
        assert!(!manager.sync(SettingsTheme::System, true));
        manager.sync(SettingsTheme::Dark, false);
        assert!(manager.tokens().dark);
        manager.sync(SettingsTheme::Light, true);
        assert!(!manager.tokens().dark);
    }

    #[test]
    fn apply_sets_fluent_widget_visuals() {
        let ctx = egui::Context::default();
        let manager = ThemeManager::new(SettingsTheme::Dark, false);
        manager.apply(&ctx);
        let visuals = &ctx.style().visuals;
        assert!(visuals.dark_mode);
        assert_eq!(visuals.panel_fill, ThemeTokens::DARK.window_bg);
        assert_eq!(visuals.window_fill, ThemeTokens::DARK.card_bg);
        assert_eq!(visuals.widgets.inactive.weak_bg_fill, ThemeTokens::DARK.control_bg);
        assert_eq!(visuals.widgets.hovered.weak_bg_fill, ThemeTokens::DARK.control_hover);
        assert_eq!(visuals.selection.stroke.color, ThemeTokens::DARK.accent);
    }
}
