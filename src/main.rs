#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 180.0]),
        ..Default::default()
    };
    eframe::run_native(
        "",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<HexViewer>::default()
        }),
    )
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InputBase {
    Hex,
    Dec,
}

struct HexViewer {
    value: u64,
    input: String,
    base: InputBase,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    is_always_on_top: bool,
    show_bit_fields: bool,
    bit_field_config: String,
}

impl Default for HexViewer {
    fn default() -> Self {
        Self {
            value: 0,
            input: "".to_string(),
            base: InputBase::Hex,
            selection_start: None,
            selection_end: None,
            is_always_on_top: false,
            show_bit_fields: false,
            bit_field_config: "4:20:8".to_string(),
        }
    }
}

impl HexViewer {
    fn format_input_for_base(&self, value: u64) -> String {
        match self.base {
            InputBase::Hex => format!("{:X}", value),
            InputBase::Dec => format!("{}", value),
        }
    }

    fn ui_header(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let prev_base = self.base;

            egui::ComboBox::from_id_source("base_selector")
                .selected_text(match self.base {
                    InputBase::Hex => "HEX",
                    InputBase::Dec => "DEC",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.base, InputBase::Hex, "HEX");
                    ui.selectable_value(&mut self.base, InputBase::Dec, "DEC");
                });

            if self.base != prev_base {
                self.input = self.format_input_for_base(self.value);
            }

            let resp = ui.text_edit_singleline(&mut self.input);
            if resp.changed() {
                let parsed = match self.base {
                    InputBase::Hex => parse_hex_to_u64(&self.input),
                    InputBase::Dec => parse_dec_to_u64(&self.input),
                };
                if let Some(v) = parsed {
                    self.value = v;
                    self.input = self.format_input_for_base(self.value);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pin_text = egui::RichText::new("📌").size(18.0);
                let mut button =
                    egui::Button::new(pin_text).min_size(egui::vec2(28.0, 28.0));
                if self.is_always_on_top {
                    button = button.fill(egui::Color32::from_rgb(0, 122, 255));
                }
                if ui.add(button).clicked() {
                    self.is_always_on_top = !self.is_always_on_top;
                    let level = if self.is_always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }

                ui.add_space(8.0);

                let toggle_text = if self.show_bit_fields { "Bits" } else { "Fields" };
                if ui.button(toggle_text).clicked() {
                    self.show_bit_fields = !self.show_bit_fields;
                }
            });
        });
    }

    fn ui_field_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Config:");
            ui.text_edit_singleline(&mut self.bit_field_config);
        });
        ui.add_space(4.0);

        let raw_widths: Vec<u32> = self
            .bit_field_config
            .split(&[':', ',', ' '][..])
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        let mut widths: Vec<u32> = Vec::new();
        let mut used_bits: u32 = 0;
        for w in raw_widths {
            if w == 0 {
                continue;
            }
            if used_bits + w > 32 {
                break;
            }
            widths.push(w);
            used_bits += w;
        }

        let total_bits: u32 = used_bits;
        let start_bit = if total_bits > 0 { total_bits - 1 } else { 0 };
        let non_zero_count = widths.len().max(1) as f32;

        let colors = [
            egui::Color32::from_rgb(200, 200, 255),
            egui::Color32::from_rgb(200, 255, 200),
            egui::Color32::from_rgb(255, 200, 200),
            egui::Color32::from_rgb(255, 255, 200),
            egui::Color32::from_rgb(200, 255, 255),
            egui::Color32::from_rgb(255, 200, 255),
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let total_width = ui.available_width();
            let rect_width = total_width / non_zero_count;
            let mut current_high = start_bit;

            for (i, &w) in widths.iter().enumerate() {
                let current_low = if current_high >= w {
                    current_high - w + 1
                } else {
                    0
                };

                let mask = if w == 64 {
                    u64::MAX
                } else {
                    (1u64 << w) - 1
                };
                let val = (self.value >> current_low) & mask;

                let bg_color = colors[i % colors.len()];
                let text_color = egui::Color32::BLACK;

                let rect_size = egui::vec2(rect_width, 72.0);
                ui.allocate_ui(rect_size, |ui| {
                    egui::Frame::none()
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                        .inner_margin(2.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let range_text = if w == 1 {
                                    format!("[{}]", current_high)
                                } else {
                                    format!("[{}:{}]", current_high, current_low)
                                };
                                ui.label(
                                    egui::RichText::new(range_text)
                                        .color(text_color)
                                        .strong(),
                                );

                                let font_size = if rect_width < 40.0 { 12.0 } else { 16.0 };
                                ui.label(
                                    egui::RichText::new(format!("0x{:X}", val))
                                        .color(text_color)
                                        .size(font_size),
                                );

                                if rect_width >= 30.0 {
                                    ui.label(
                                        egui::RichText::new(format!("{}", val))
                                            .color(text_color)
                                            .size(font_size - 2.0),
                                    );
                                }
                            });
                        });
                });

                if current_high >= w {
                    current_high -= w;
                } else {
                    break;
                }
            }
        });
    }

    fn ui_bit_view(&mut self, ui: &mut egui::Ui) {
        for row in 0..2 {
            ui.horizontal(|ui| {
                for col in 0..32 {
                    let index = row * 32 + col;
                    let bit_pos = 63 - index;

                    ui.allocate_ui(egui::Vec2 { x: 28.0, y: 64.0 }, |ui| {
                        ui.vertical_centered(|ui| {
                            let bit = ((self.value >> bit_pos) & 1) == 1;
                            let text = if bit { "1" } else { "0" };

                            let selected =
                                is_index_selected(index, self.selection_start, self.selection_end);
                            let mut button =
                                egui::Button::new(text).min_size(egui::vec2(24.0, 32.0));
                            if bit {
                                button = button.fill(egui::Color32::from_rgb(0, 122, 255));
                            }
                            if selected {
                                button = button.fill(ui.visuals().selection.bg_fill);
                            }
                            let response =
                                ui.add(button.sense(egui::Sense::click_and_drag()));

                            if response.clicked() {
                                self.value ^= 1u64 << bit_pos;
                                self.input = self.format_input_for_base(self.value);
                            }

                            let label = format!("{}", bit_pos);
                            ui.label(egui::RichText::new(label).size(14.0));
                        });
                    });
                }
            });
            if row == 0 {
                ui.add_space(10.0);
            }
        }
    }
}

impl eframe::App for HexViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_header(ctx, ui);

            ui.add_space(6.0);
            ui.spacing_mut().item_spacing.x = 4.0;

            if self.show_bit_fields {
                self.ui_field_view(ui);
            } else {
                self.ui_bit_view(ui);
            }
        });
    }
}

fn parse_hex_to_u64(s: &str) -> Option<u64> {
    let mut t = s.trim().to_string();
    if let Some(stripped) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        t = stripped.to_string();
    }
    let filtered: String = t.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if filtered.is_empty() { return None; }
    let truncated = if filtered.len() > 16 { filtered[filtered.len()-16..].to_string() } else { filtered };
    u64::from_str_radix(&truncated, 16).ok()
}

fn parse_dec_to_u64(s: &str) -> Option<u64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let filtered: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    if filtered.is_empty() {
        return None;
    }
    u64::from_str_radix(&filtered, 10).ok()
}

fn is_index_selected(index: usize, start: Option<usize>, end: Option<usize>) -> bool {
    match (start, end) {
        (Some(s), Some(e)) => {
            let (a, b) = if s <= e { (s, e) } else { (e, s) };
            index >= a && index <= b
        }
        _ => false,
    }
}
