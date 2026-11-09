//! 首次使用引导：外观模式、灵动岛形态、常用功能三屏。
//!
//! 视觉遵循 WinUI 3 / Fluent：留白、细边框、线性图标、真实岛体预览。
//! 引导只写本地设置，不接触账号、日历或 AI 凭据。

use crate::app::App;
use crate::model::{Dock, IslandShape};
use crate::settings_theme::{SettingsTheme, ThemeTokens};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, Vec2};

pub(crate) struct State {
    pub open: bool,
    pub step: usize,
    step_at: f64,
}

impl State {
    pub fn new(open: bool) -> Self {
        Self { open, step: 0, step_at: 0.0 }
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Auto, Sun, Moon, Notch, Pill, Top, Left, Right,
    Music, Focus, Performance, Shelf, Todo, Ai, Calendar,
}

impl App {
    pub(crate) fn start_onboarding(&mut self) {
        self.settings_open = false;
        self.onboarding.open = true;
        self.onboarding.step = 0;
        self.onboarding.step_at = 0.0;
    }

    pub(crate) fn finish_onboarding(&mut self) {
        self.settings.onboarded = true;
        self.settings.save();
        self.onboarding.open = false;
    }

    pub fn onboarding_ui(&mut self, ctx: &egui::Context) {
        if !self.onboarding.open {
            return;
        }
        let t = self.theme.tokens();
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("bartap-onboarding"),
            egui::ViewportBuilder::default()
                .with_title("BarTap 引导")
                .with_icon(crate::theme::app_icon())
                .with_inner_size([640.0, 600.0])
                .with_position(egui::pos2(533.0, 270.0))
                .with_resizable(false)
                .with_decorations(false)
                .with_active(true)
                .with_always_on_top(),
            |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.finish_onboarding();
                    return;
                }
                crate::settings_theme::ThemeManager::apply_tokens(ctx, t);
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if t.dark {
                    egui::SystemTheme::Dark
                } else {
                    egui::SystemTheme::Light
                }));
                let age = ctx.input(|i| i.time) - self.onboarding.step_at;
                let alpha = (age / 0.22).clamp(0.0, 1.0) as f32;
                if age < 0.3 {
                    ctx.request_repaint_after(std::time::Duration::from_millis(16));
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(t.window_bg).inner_margin(0))
                    .show(ctx, |ui| {
                        let full = ui.max_rect();
                        let drag = Rect::from_min_size(full.min, Vec2::new(full.width() - 54.0, 48.0));
                        let drag_response = ui.interact(drag, ui.id().with("onboarding-drag"), egui::Sense::drag());
                        if drag_response.drag_started() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.painter().rect_filled(Rect::from_min_size(full.min, Vec2::new(full.width(), 1.0)), 0.0, t.border);
                        let close = Rect::from_min_size(full.right_top() + Vec2::new(-42.0, 12.0), Vec2::splat(28.0));
                        let close_response = ui.interact(close, ui.id().with("onboarding-close"), egui::Sense::click());
                        if close_response.hovered() {
                            ui.painter().rect_filled(close, 7.0, t.control_hover);
                        }
                        let c = close.center();
                        let close_color = if close_response.hovered() { t.text } else { t.text_secondary };
                        for dir in [-1.0, 1.0] {
                            ui.painter().line_segment(
                                [c + Vec2::new(-4.0, -4.0 * dir), c + Vec2::new(4.0, 4.0 * dir)],
                                Stroke::new(1.4, close_color),
                            );
                        }
                        if close_response.clicked() {
                            self.finish_onboarding();
                            return;
                        }
                        let content = full.shrink2(Vec2::new(38.0, 24.0));
                        ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
                            ui.add_space(6.0 + (1.0 - alpha) * 5.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("BarTap").size(22.0).strong().color(t.text));
                                ui.add_space(3.0);
                                ui.label(RichText::new("设置引导").size(12.0).color(t.text_secondary));
                            });
                            ui.add_space(22.0);
                            let (title, hint) = match self.onboarding.step {
                                0 => ("外观模式", "选择你喜欢的主题。之后可以在设置里随时修改。"),
                                1 => ("灵动岛形态", "选择岛体形状和停靠位置。"),
                                _ => ("常用功能", "选择第一次展开时要看到的页面。"),
                            };
                            ui.label(RichText::new(title).size(27.0).strong().color(t.text));
                            ui.add_space(5.0);
                            ui.label(RichText::new(hint).size(13.0).color(t.text_secondary));
                            ui.add_space(20.0);
                            match self.onboarding.step {
                                0 => self.onboarding_theme(ui, t),
                                1 => self.onboarding_shape(ui, t),
                                _ => self.onboarding_pages(ui, t),
                            }
                            ui.add_space(24.0);
                            self.onboarding_footer(ui, ctx, t);
                        });
                    });
            },
        );
    }

    fn onboarding_footer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, t: ThemeTokens) {
        ui.horizontal(|ui| {
            let skip = egui::Button::new(RichText::new("跳过").size(12.5).color(t.text_secondary))
                .fill(Color32::TRANSPARENT).stroke(Stroke::NONE);
            if ui.add(skip).clicked() {
                self.finish_onboarding();
                return;
            }
            ui.add_space(12.0);
            for i in 0..3 {
                let active = i == self.onboarding.step;
                let (rect, _) = ui.allocate_exact_size(Vec2::new(if active { 18.0 } else { 6.0 }, 6.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 3.0, if active { t.accent } else { t.border });
                ui.add_space(3.0);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if self.onboarding.step < 2 { "下一步" } else { "开始使用" };
                let button = egui::Button::new(RichText::new(label).size(13.0).strong().color(if t.dark { t.window_bg } else { Color32::WHITE }))
                    .fill(t.accent).min_size(Vec2::new(112.0, 36.0));
                if ui.add(button).clicked() {
                    if self.onboarding.step < 2 {
                        self.onboarding.step += 1;
                        self.onboarding.step_at = ctx.input(|i| i.time);
                        ctx.request_repaint();
                    } else {
                        self.finish_onboarding();
                    }
                }
                if self.onboarding.step > 0 {
                    if ui.button(RichText::new("上一步").size(12.5).color(t.text_secondary)).clicked() {
                        self.onboarding.step -= 1;
                        self.onboarding.step_at = ctx.input(|i| i.time);
                        ctx.request_repaint();
                    }
                }
            });
        });
    }

    fn onboarding_theme(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.horizontal(|ui| {
            for (mode, icon, label, desc) in [
                (SettingsTheme::System, Icon::Auto, "跟随系统", "随 Windows 切换"),
                (SettingsTheme::Light, Icon::Sun, "浅色", "Fluent Light"),
                (SettingsTheme::Dark, Icon::Moon, "深色", "Fluent Dark"),
            ] {
                let selected = self.settings.settings_theme == mode;
                if choice_card(ui, t, icon, label, desc, selected, Vec2::new(170.0, 92.0)) {
                    self.settings.settings_theme = mode;
                    let system_dark = crate::settings_theme::system_dark(ui.ctx());
                    self.theme.sync(mode, system_dark);
                    crate::settings_theme::ThemeManager::apply_tokens(ui.ctx(), self.theme.tokens());
                }
                ui.add_space(10.0);
            }
        });
        ui.add_space(22.0);
        preview(ui, t, self.settings.island_shape, self.settings.dock, true);
    }

    fn onboarding_shape(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.label(RichText::new("形状").size(12.0).color(t.text_secondary));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (shape, icon, desc) in [
                (IslandShape::Notch, Icon::Notch, "贴着屏幕顶边"),
                (IslandShape::Pill, Icon::Pill, "悬浮胶囊"),
            ] {
                let selected = self.settings.island_shape == shape;
                if choice_card(ui, t, icon, shape.label(), desc, selected, Vec2::new(170.0, 64.0)) {
                    self.settings.island_shape = shape;
                }
                ui.add_space(10.0);
            }
        });
        ui.add_space(16.0);
        ui.label(RichText::new("停靠位置").size(12.0).color(t.text_secondary));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (dock, icon) in [(Dock::Top, Icon::Top), (Dock::Left, Icon::Left), (Dock::Right, Icon::Right)] {
                let selected = self.settings.dock == dock;
                if segmented_card(ui, t, icon, dock.label(), selected, Vec2::new(100.0, 42.0)) {
                    self.settings.dock = dock;
                }
                ui.add_space(8.0);
            }
        });
        ui.add_space(18.0);
        preview(ui, t, self.settings.island_shape, self.settings.dock, false);
    }

    fn onboarding_pages(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.label(RichText::new("至少保留一个页面。置物架与剪贴板共用一页。").size(12.0).color(t.text_secondary));
        ui.add_space(10.0);
        let pages = [
            (0usize, "音乐", "播放与歌词", Icon::Music),
            (1usize, "专注", "番茄钟", Icon::Focus),
            (2usize, "性能", "CPU、内存、网络", Icon::Performance),
            (3usize, "置物架", "文件与剪贴板", Icon::Shelf),
            (4usize, "待办", "任务与提醒", Icon::Todo),
            (6usize, "AI 额度", "额度与余额", Icon::Ai),
            (7usize, "日程", "日历与会议", Icon::Calendar),
        ];
        ui.columns(2, |cols| {
            for (i, (page, label, desc, icon)) in pages.iter().enumerate() {
                let col = &mut cols[i % 2];
                let selected = self.settings.visible_pages[*page];
                let width = col.available_width();
                let response = page_row(col, t, *icon, label, desc, selected, width);
                if response.clicked() {
                    self.set_page_visible(*page, !selected);
                }
                col.add_space(6.0);
            }
        });
    }
}

fn round_rect(ui: &egui::Ui, rect: Rect, fill: Color32, stroke: Color32, radius: f32) {
    ui.painter().rect_filled(rect, radius, fill);
    ui.painter().rect_stroke(rect, radius, Stroke::new(1.0, stroke), egui::StrokeKind::Inside);
}

fn choice_card(ui: &mut egui::Ui, t: ThemeTokens, icon: Icon, title: &str, desc: &str, selected: bool, size: Vec2) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hover = ui.ctx().animate_bool_with_time(ui.id().with(("onboard-choice", title)), response.hovered(), 0.12);
    let active = selected || hover > 0.5;
    let fill = if selected { t.accent.gamma_multiply(0.10) } else if response.hovered() { t.control_hover } else { t.card_bg };
    let border = if selected { t.accent } else if response.hovered() { t.border } else { t.border };
    round_rect(ui, rect, fill, border, 10.0);
    let icon_rect = Rect::from_min_size(rect.min + Vec2::new(14.0, 14.0), Vec2::splat(22.0));
    line_icon(ui.painter(), icon_rect, icon, if active { t.accent } else { t.text_secondary });
    ui.painter().text(rect.min + Vec2::new(14.0, 54.0), Align2::LEFT_CENTER, title, FontId::proportional(13.0), t.text);
    ui.painter().text(rect.min + Vec2::new(14.0, 72.0), Align2::LEFT_CENTER, desc, FontId::proportional(10.5), t.text_secondary);
    response.clicked()
}

fn segmented_card(ui: &mut egui::Ui, t: ThemeTokens, icon: Icon, title: &str, selected: bool, size: Vec2) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    round_rect(ui, rect, if selected { t.accent.gamma_multiply(0.10) } else if response.hovered() { t.control_hover } else { t.card_bg }, if selected { t.accent } else { t.border }, 9.0);
    let icon_rect = Rect::from_min_size(rect.min + Vec2::new(12.0, 11.0), Vec2::splat(19.0));
    line_icon(ui.painter(), icon_rect, icon, if selected { t.accent } else { t.text_secondary });
    ui.painter().text(rect.min + Vec2::new(40.0, rect.height() / 2.0), Align2::LEFT_CENTER, title, FontId::proportional(12.0), if selected { t.text } else { t.text_secondary });
    response.clicked()
}

fn page_row(ui: &mut egui::Ui, t: ThemeTokens, icon: Icon, title: &str, desc: &str, selected: bool, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 40.0), egui::Sense::click());
    let fill = if selected { t.accent.gamma_multiply(0.09) } else if response.hovered() { t.control_hover } else { Color32::TRANSPARENT };
    round_rect(ui, rect, fill, if selected { t.accent.gamma_multiply(0.7) } else { t.border }, 8.0);
    line_icon(ui.painter(), Rect::from_min_size(rect.min + Vec2::new(11.0, 11.0), Vec2::splat(18.0)), icon, if selected { t.accent } else { t.text_secondary });
    ui.painter().text(rect.min + Vec2::new(38.0, 13.0), Align2::LEFT_CENTER, title, FontId::proportional(12.0), t.text);
    ui.painter().text(rect.min + Vec2::new(38.0, 28.0), Align2::LEFT_CENTER, desc, FontId::proportional(9.5), t.text_secondary);
    let sw = Rect::from_center_size(rect.right_center() + Vec2::new(-21.0, 0.0), Vec2::new(28.0, 16.0));
    ui.painter().rect_filled(sw, 8.0, if selected { t.accent } else { t.control_pressed });
    let knob_x = if selected { sw.right() - 8.0 } else { sw.left() + 8.0 };
    ui.painter().circle_filled(Pos2::new(knob_x, sw.center().y), 5.5, Color32::WHITE);
    response
}

fn preview(ui: &mut egui::Ui, t: ThemeTokens, shape: IslandShape, dock: Dock, show_theme: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 142.0), egui::Sense::hover());
    round_rect(ui, rect, t.secondary_bg, t.border, 12.0);
    let p = ui.painter();
    let island_color = if t.dark { Color32::from_rgb(18, 18, 20) } else { Color32::from_rgb(248, 248, 250) };
    let ink = if t.dark { Color32::from_rgb(236, 236, 238) } else { Color32::from_rgb(24, 24, 26) };
    let muted = if t.dark { Color32::from_rgb(158, 158, 164) } else { Color32::from_rgb(104, 104, 110) };
    let label = if show_theme { "主题实时预览" } else { "岛体位置预览" };
    p.text(rect.left_top() + Vec2::new(14.0, 12.0), Align2::LEFT_TOP, label, FontId::proportional(10.0), muted);
    let area = Rect::from_min_max(rect.left_top() + Vec2::new(20.0, 34.0), rect.right_bottom() - Vec2::new(20.0, 16.0));
    let (w, h, x, y) = match dock {
        Dock::Top => {
            let h = if shape == IslandShape::Pill { 28.0 } else { 22.0 };
            (250.0, h, area.center().x - 125.0, area.top() + 4.0)
        }
        Dock::Left => (44.0, 86.0, area.left() + 12.0, area.center().y - 43.0),
        Dock::Right => (44.0, 86.0, area.right() - 56.0, area.center().y - 43.0),
    };
    let island = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
    p.rect_filled(island, if shape == IslandShape::Pill { h / 2.0 } else { 9.0 }, island_color);
    p.rect_stroke(island, if shape == IslandShape::Pill { h / 2.0 } else { 9.0 }, Stroke::new(1.0, if t.dark { Color32::from_white_alpha(22) } else { Color32::from_black_alpha(18) }), egui::StrokeKind::Inside);
    // Real product cues: clock, page label, battery and a music glyph.
    p.text(island.center_top() + Vec2::new(0.0, 9.0), Align2::CENTER_TOP, "09:41", FontId::proportional(10.0), ink);
    p.text(island.center_bottom() + Vec2::new(0.0, -8.0), Align2::CENTER_BOTTOM, if show_theme { "BarTap" } else { "音乐 · 岛体" }, FontId::proportional(9.0), muted);
    let dot = Pos2::new(island.center().x, island.bottom() - 5.0);
    p.circle_filled(dot, 1.5, muted);
    if dock == Dock::Top {
        for i in 0..4 {
            p.circle_filled(dot + Vec2::new(-12.0 + i as f32 * 8.0, 0.0), 1.0, if i == 0 { ink } else { muted });
        }
    }
    // Expanded panel preview: same island palette, enough real product texture to judge the theme.
    let panel_w = 270.0;
    let panel = match dock {
        Dock::Top => Rect::from_min_size(Pos2::new(area.center().x - panel_w / 2.0, area.top() + 34.0), Vec2::new(panel_w, 58.0)),
        Dock::Left => Rect::from_min_size(Pos2::new(area.left() + 68.0, area.center().y - 29.0), Vec2::new(210.0, 58.0)),
        Dock::Right => Rect::from_min_size(Pos2::new(area.right() - 68.0 - 210.0, area.center().y - 29.0), Vec2::new(210.0, 58.0)),
    };
    p.rect_filled(panel, 10.0, island_color);
    p.rect_stroke(panel, 10.0, Stroke::new(1.0, if t.dark { Color32::from_white_alpha(18) } else { Color32::from_black_alpha(14) }), egui::StrokeKind::Inside);
    let art = Rect::from_min_size(panel.min + Vec2::new(10.0, 10.0), Vec2::splat(30.0));
    p.rect_filled(art, 7.0, t.accent.gamma_multiply(0.85));
    line_icon(p, art.shrink(7.0), Icon::Music, Color32::WHITE);
    p.text(panel.min + Vec2::new(48.0, 16.0), Align2::LEFT_CENTER, "夜航", FontId::proportional(10.0), ink);
    p.text(panel.min + Vec2::new(48.0, 30.0), Align2::LEFT_CENTER, "BarTap · 正在播放", FontId::proportional(8.0), muted);
    let track = Rect::from_min_size(panel.left_bottom() + Vec2::new(10.0, -12.0), Vec2::new(panel.width() - 20.0, 3.0));
    p.rect_filled(track, 1.5, t.control_pressed);
    p.rect_filled(Rect::from_min_size(track.min, Vec2::new(track.width() * 0.42, 3.0)), 1.5, t.accent);
    for i in 0..5 {
        p.circle_filled(panel.center_bottom() + Vec2::new(-16.0 + i as f32 * 8.0, -5.0), 1.2, if i == 0 { ink } else { muted });
    }
}

fn line_icon(p: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let c = rect.center();
    let s = rect.width().min(rect.height()) / 22.0;
    let stroke = Stroke::new(1.4, color);
    let seg = |a: (f32, f32), b: (f32, f32)| p.line_segment([c + Vec2::new(a.0, a.1) * s, c + Vec2::new(b.0, b.1) * s], stroke);
    let circle = |r: f32| p.circle_stroke(c, r * s, stroke);
    match icon {
        Icon::Sun => {
            circle(4.2);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                seg((a.cos() * 7.0, a.sin() * 7.0), (a.cos() * 9.0, a.sin() * 9.0));
            }
        }
        Icon::Moon => {
            let mut pts = Vec::new();
            for i in 0..18 {
                let a = std::f32::consts::PI * (0.55 + i as f32 / 17.0 * 1.25);
                pts.push(c + Vec2::new(a.cos(), a.sin()) * 7.0 * s);
            }
            for i in 0..18 {
                let a = std::f32::consts::PI * (1.8 + i as f32 / 17.0 * 1.2);
                pts.push(c + Vec2::new(a.cos(), a.sin()) * 9.0 * s);
            }
            p.add(egui::Shape::line(pts, stroke));
        }
        Icon::Auto => {
            circle(6.8);
            seg((0.0, -4.4), (0.0, 4.4));
            seg((-4.4, 0.0), (4.4, 0.0));
            circle(2.0);
        }
        Icon::Notch => {
            let path = vec![
                c + Vec2::new(-9.0, 4.0) * s,
                c + Vec2::new(-7.0, -3.0) * s,
                c + Vec2::new(7.0, -3.0) * s,
                c + Vec2::new(9.0, 4.0) * s,
            ];
            p.add(egui::Shape::line(path, stroke));
        }
        Icon::Pill => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(19.0, 9.0) * s), 4.5 * s, stroke, egui::StrokeKind::Inside);
        }
        Icon::Top => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(16.0, 12.0) * s), 2.0 * s, stroke, egui::StrokeKind::Inside);
            seg((-7.0, -5.0), (7.0, -5.0));
        }
        Icon::Left => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(16.0, 12.0) * s), 2.0 * s, stroke, egui::StrokeKind::Inside);
            seg((-7.0, -4.0), (-7.0, 4.0));
        }
        Icon::Right => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::new(16.0, 12.0) * s), 2.0 * s, stroke, egui::StrokeKind::Inside);
            seg((7.0, -4.0), (7.0, 4.0));
        }
        Icon::Music => {
            circle(2.0);
            seg((2.0, 0.0), (2.0, -8.0));
            seg((2.0, -8.0), (8.0, -9.5));
            p.circle_filled(c + Vec2::new(8.0, -7.0) * s, 2.0 * s, color);
        }
        Icon::Focus => {
            circle(7.0);
            seg((0.0, 0.0), (0.0, -4.5));
            seg((0.0, 0.0), (3.8, 2.0));
        }
        Icon::Performance => {
            p.add(egui::Shape::line(vec![
                c + Vec2::new(-9.0, 1.0) * s,
                c + Vec2::new(-5.0, 1.0) * s,
                c + Vec2::new(-3.0, -6.0) * s,
                c + Vec2::new(1.0, 7.0) * s,
                c + Vec2::new(4.0, -2.0) * s,
                c + Vec2::new(9.0, -2.0) * s,
            ], stroke));
        }
        Icon::Shelf => {
            p.add(egui::Shape::line(vec![
                c + Vec2::new(-8.0, 5.0) * s,
                c + Vec2::new(-8.0, -5.0) * s,
                c + Vec2::new(-2.0, -5.0) * s,
                c + Vec2::new(0.0, -2.5) * s,
                c + Vec2::new(8.0, -2.5) * s,
                c + Vec2::new(8.0, 5.0) * s,
                c + Vec2::new(-8.0, 5.0) * s,
            ], stroke));
        }
        Icon::Todo => {
            for y in [-5.0, 0.0, 5.0] {
                p.circle_stroke(c + Vec2::new(-6.5, y) * s, 1.5 * s, stroke);
                seg((-3.0, y), (8.0, y));
            }
        }
        Icon::Ai => {
            seg((0.0, -9.0), (0.0, 9.0));
            seg((-9.0, 0.0), (9.0, 0.0));
            seg((-6.0, -6.0), (6.0, 6.0));
            seg((6.0, -6.0), (-6.0, 6.0));
        }
        Icon::Calendar => {
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, 1.0) * s, Vec2::new(16.0, 14.0) * s), 2.0 * s, stroke, egui::StrokeKind::Inside);
            seg((-8.0, -3.0), (8.0, -3.0));
            seg((-5.0, -8.0), (-5.0, -4.0));
            seg((5.0, -8.0), (5.0, -4.0));
        }
    }
}
