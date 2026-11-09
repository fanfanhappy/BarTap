//! 首次使用引导：外观模式、灵动岛形态、常用功能三屏，不碰账号与凭据。

use crate::app::App;
use crate::model::{Dock, IslandShape};
use crate::settings_theme::{SettingsTheme, ThemeTokens};
use eframe::egui::{self, Color32, RichText};

pub(crate) struct State {
    pub open: bool,
    pub step: usize,
}

impl State {
    pub fn new(open: bool) -> Self {
        Self { open, step: 0 }
    }
}

impl App {
    pub(crate) fn start_onboarding(&mut self) {
        self.settings_open = false;
        self.onboarding.open = true;
        self.onboarding.step = 0;
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
                .with_inner_size([560.0, 420.0])
                .with_resizable(false)
                .with_active(true)
                .with_always_on_top(),
            |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    // 关闭窗口等同于跳过，避免下次启动继续打扰。
                    self.finish_onboarding();
                    return;
                }
                crate::settings_theme::ThemeManager::apply_tokens(ctx, t);
                ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if t.dark { egui::SystemTheme::Dark } else { egui::SystemTheme::Light }));
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(t.window_bg).inner_margin(24))
                    .show(ctx, |ui| {
                        ui.set_min_width(512.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("BarTap").size(23.0).strong().color(t.text));
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("第 {} / 3 步", self.onboarding.step + 1)).size(12.0).color(t.text_secondary));
                        });
                        ui.add_space(4.0);
                        let (title, hint) = match self.onboarding.step {
                            0 => ("外观模式", "先选一个看着舒服的主题，之后可以在设置里随时改。"),
                            1 => ("灵动岛形态", "选择岛体的形状和停靠位置；顶部停靠最省空间。"),
                            _ => ("常用功能", "选择第一次展开时要看到的页面，之后可以在页面管理里调整。"),
                        };
                        ui.label(RichText::new(title).size(19.0).strong().color(t.text));
                        ui.add_space(2.0);
                        ui.label(RichText::new(hint).size(12.0).color(t.text_secondary));
                        ui.add_space(14.0);
                        egui::Frame::new().fill(t.card_bg).corner_radius(14.0).inner_margin(16).show(ui, |ui| {
                            ui.set_min_height(185.0);
                            match self.onboarding.step {
                                0 => self.onboarding_theme(ui, t),
                                1 => self.onboarding_shape(ui, t),
                                _ => self.onboarding_pages(ui, t),
                            }
                        });
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("跳过").size(12.5).color(t.text_secondary)).clicked() {
                                self.finish_onboarding();
                                return;
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let next = if self.onboarding.step < 2 { "下一步" } else { "开始使用" };
                                let button = egui::Button::new(RichText::new(next).size(13.0).strong().color(if t.dark { t.window_bg } else { Color32::WHITE }))
                                    .fill(t.accent)
                                    .min_size(egui::vec2(104.0, 34.0));
                                if ui.add(button).clicked() {
                                    if self.onboarding.step < 2 {
                                        self.onboarding.step += 1;
                                    } else {
                                        self.finish_onboarding();
                                    }
                                }
                                if self.onboarding.step > 0 && ui.button(RichText::new("上一步").size(12.5)).clicked() {
                                    self.onboarding.step -= 1;
                                }
                            });
                        });
                    });
            },
        );
    }

    fn onboarding_theme(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.label(RichText::new("选择后会立即应用到引导窗口和岛体。").size(12.0).color(t.text_secondary));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            for (mode, label, desc) in [
                (SettingsTheme::System, "跟随系统", "随 Windows 深浅色切换"),
                (SettingsTheme::Light, "浅色", "始终使用 Fluent Light"),
                (SettingsTheme::Dark, "深色", "始终使用 Fluent Dark"),
            ] {
                let selected = self.settings.settings_theme == mode;
                let response = ui.add_sized(
                    [148.0, 92.0],
                    egui::Button::new(
                        RichText::new(format!("{label}\n{desc}"))
                            .size(12.0)
                            .color(if selected { t.text } else { t.text_secondary }),
                    )
                    .fill(if selected { t.control_pressed } else { t.control_bg })
                    .stroke(egui::Stroke::new(1.0, if selected { t.accent } else { t.border })),
                );
                if response.clicked() {
                    self.settings.settings_theme = mode;
                    let system_dark = crate::settings_theme::system_dark(ui.ctx());
                    self.theme.sync(mode, system_dark);
                    crate::settings_theme::ThemeManager::apply_tokens(ui.ctx(), self.theme.tokens());
                }
            }
        });
    }

    fn onboarding_shape(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.label(RichText::new("形状").size(12.0).color(t.text_secondary));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for shape in IslandShape::ALL {
                let selected = self.settings.island_shape == shape;
                let response = ui.add_sized(
                    [160.0, 44.0],
                    egui::Button::new(RichText::new(shape.label()).size(13.0).color(if selected { t.text } else { t.text_secondary }))
                        .fill(if selected { t.control_pressed } else { t.control_bg })
                        .stroke(egui::Stroke::new(1.0, if selected { t.accent } else { t.border })),
                );
                if response.clicked() {
                    self.settings.island_shape = shape;
                }
            }
        });
        ui.add_space(12.0);
        ui.label(RichText::new("停靠位置").size(12.0).color(t.text_secondary));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for dock in Dock::ALL {
                let selected = self.settings.dock == dock;
                let response = ui.add_sized(
                    [104.0, 38.0],
                    egui::Button::new(RichText::new(dock.label()).size(13.0).color(if selected { t.text } else { t.text_secondary }))
                        .fill(if selected { t.control_pressed } else { t.control_bg })
                        .stroke(egui::Stroke::new(1.0, if selected { t.accent } else { t.border })),
                );
                if response.clicked() {
                    self.settings.dock = dock;
                }
            }
        });
        ui.add_space(12.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 58.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 10.0, t.secondary_bg);
        let bar = match self.settings.dock {
            Dock::Top => {
                let h = if self.settings.island_shape == IslandShape::Pill { 22.0 } else { 14.0 };
                egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 12.0), egui::vec2(190.0, h))
            }
            Dock::Left => egui::Rect::from_center_size(egui::pos2(rect.left() + 12.0, rect.center().y), egui::vec2(14.0, 92.0)),
            Dock::Right => egui::Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), egui::vec2(14.0, 92.0)),
        };
        ui.painter().rect_filled(bar, bar.height().min(bar.width()) * 0.5, t.text_secondary);
        ui.painter().text(
            egui::pos2(rect.center().x, rect.bottom() - 7.0),
            egui::Align2::CENTER_BOTTOM,
            format!("{} · {}", self.settings.island_shape.label(), self.settings.dock.label()),
            egui::FontId::proportional(10.0),
            t.text_secondary,
        );
    }

    fn onboarding_pages(&mut self, ui: &mut egui::Ui, t: ThemeTokens) {
        ui.label(RichText::new("至少保留一个页面。置物架与剪贴板共用一页。").size(12.0).color(t.text_secondary));
        ui.add_space(8.0);
        let pages = [
            (0usize, "音乐", "播放控制与歌词"),
            (1usize, "专注", "番茄钟与休息计时"),
            (2usize, "性能", "CPU、内存、网络、存储"),
            (3usize, "置物架", "文件收集与剪贴板"),
            (4usize, "待办", "任务清单与提醒"),
            (6usize, "AI 额度", "账号额度与余额"),
            (7usize, "日程", "月历与会议提醒"),
        ];
        ui.columns(2, |cols| {
            for (i, (page, label, desc)) in pages.iter().enumerate() {
                let col = &mut cols[i % 2];
                col.horizontal(|ui| {
                    let mut enabled = self.settings.visible_pages[*page];
                    if ui.checkbox(&mut enabled, "").changed() {
                        self.set_page_visible(*page, enabled);
                    }
                    ui.vertical(|ui| {
                        ui.label(RichText::new(*label).size(12.5).color(t.text));
                        ui.label(RichText::new(*desc).size(10.5).color(t.text_secondary));
                    });
                });
                col.add_space(6.0);
            }
        });
    }
}
