use clap::Parser;
use eframe::{
    egui::{self, Widget},
    emath::TSTransform,
};
use eyre::Result;
use store::Store;

mod download;
mod store;
mod util;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Tier {
    title: &'static str,
    subtitle: &'static str,
    color: egui::Color32,
}

const TIERS: &[Tier] = &[
    Tier {
        title: "U",
        subtitle: "Unsorted",
        color: egui::Color32::TRANSPARENT,
    },
    Tier {
        title: "S+",
        subtitle: "Cliche",
        color: egui::Color32::from_rgb(0x66, 0x00, 0x66),
    },
    Tier {
        title: "S",
        subtitle: "Superb",
        color: egui::Color32::from_rgb(0x88, 0x22, 0x00),
    },
    Tier {
        title: "A",
        subtitle: "Very good",
        color: egui::Color32::from_rgb(0x88, 0x44, 0x00),
    },
    Tier {
        title: "B",
        subtitle: "Good",
        color: egui::Color32::from_rgb(0x88, 0x88, 0x00),
    },
    Tier {
        title: "C",
        subtitle: "Mediocre",
        color: egui::Color32::from_rgb(0x22, 0x88, 0x22),
    },
    Tier {
        title: "D",
        subtitle: "Obscure",
        color: egui::Color32::from_rgb(0x22, 0x88, 0x88),
    },
    Tier {
        title: "E",
        subtitle: "Bad",
        color: egui::Color32::from_rgb(0x22, 0x44, 0x88),
    },
    Tier {
        title: "F",
        subtitle: "N/A",
        color: egui::Color32::from_rgb(0x22, 0x00, 0x66),
    },
];

/// Xkcd downloader and tier list
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Whether to download all comics instead of showing the UI.
    #[arg(short, long)]
    download: bool,

    /// Whether to redownload comics that we have already downloaded.
    #[arg(short, long)]
    redownload: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.download {
        crate::download::download_all_comics(args.redownload)?;
    } else {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "xkcd Rank",
            native_options,
            Box::new(|cc| Ok(Box::new(App::new(cc)))),
        )?;
    }
    Ok(())
}

#[derive(Default)]
struct App {
    data: Store,

    n: usize,
    img_transform: TSTransform,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(1.5);
        Self {
            data: Store::load(),

            n: 1,
            img_transform: TSTransform::IDENTITY,
        }
    }

    fn show_comic_selector(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button("➡").clicked() {
                self.n += 1;
                self.reset_img_transform();
            }
            if ui.button("⬅").clicked() {
                self.n -= 1;
                self.reset_img_transform();
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.spacing_mut().slider_width = ui.available_width()
                    - ui.spacing().interact_size.x
                    - ui.spacing().item_spacing.x;
                let r = egui::Slider::new(&mut self.n, 1..=self.data.comics.len() - 1)
                    .trailing_fill(true)
                    .drag_value_speed(0.25)
                    .ui(ui);
                if r.changed() {
                    self.reset_img_transform();
                }
            });
        });
    }

    fn show_comic_img(&mut self, ui: &mut egui::Ui) {
        let Some(Some(comic)) = self.data.comics.get(self.n) else {
            return;
        };

        if !comic.has_image_downloaded() {
            ui.label(&comic.transcript);
            return;
        }

        let img_uri = format!("file://{}", comic.img_path().to_string_lossy());

        let (id, rect) = ui.allocate_space(ui.available_size());
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        // Allow dragging the background.
        if response.dragged() {
            self.img_transform.translation += response.drag_delta();
        }

        // Plot-like reset
        if response.double_clicked() {
            self.reset_img_transform();
        }

        let img_transform =
            TSTransform::from_translation(ui.min_rect().left_top().to_vec2()) * self.img_transform;

        if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            if response.hovered() {
                let pointer_in_layer = img_transform.inverse() * pointer;
                let zoom_delta = ui
                    .ctx()
                    .input(|i| 2.0_f32.powf(i.smooth_scroll_delta.y / 500.0));

                // Zoom in on pointer
                self.img_transform = self.img_transform
                    * TSTransform::from_translation(pointer_in_layer.to_vec2())
                    * TSTransform::from_scaling(zoom_delta)
                    * TSTransform::from_translation(-pointer_in_layer.to_vec2());
            }
        }

        let window_layer = ui.layer_id();
        let id = egui::Area::new(ui.auto_id_with("comic_img"))
            .default_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Middle)
            .constrain(false)
            .show(ui.ctx(), |ui| {
                ui.set_clip_rect(self.img_transform.inverse() * rect);
                let img = egui::Image::from_uri(img_uri);
                let size = img
                    .load_and_calc_size(ui, rect.size())
                    .unwrap_or(rect.size());
                img.paint_at(ui, egui::Rect::from_center_size(rect.center(), size));
            })
            .response
            .layer_id;
        ui.ctx().set_transform_layer(id, self.img_transform);
        ui.ctx().set_sublayer(window_layer, id);
    }

    fn show_comic_column(&mut self, ui: &mut egui::Ui) {
        let Some(Some(comic)) = self.data.comics.get(self.n).cloned() else {
            ui.colored_label(ui.visuals().error_fg_color, "Error fetching comic");
            if ui.button("Try again").clicked() {
                if let Err(e) = self.data.fetch_comic(self.n) {
                    eprintln!("error fetching comic: {e}");
                }
            }
            return;
        };

        ui.group(|ui| {
            egui::Sides::new().show(
                ui,
                |ui| ui.heading(&comic.title),
                |ui| ui.heading(format!("#{}", comic.num)),
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(&comic.alt);
                });
                egui::Frame::group(ui.style())
                    .inner_margin(1.0)
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.set_min_size(ui.available_size());
                                self.show_comic_img(ui)
                            },
                        );
                    });
            });
        });
    }

    fn reset_img_transform(&mut self) {
        self.img_transform = TSTransform::IDENTITY;
    }

    fn show_tier_list(&mut self, ui: &mut egui::Ui) {
        self.data.ensure_tiers_exist();

        ui.group(|ui| {
            ui.columns(TIERS.len(), |uis| {
                for i in 0..TIERS.len() {
                    self.display_tier(&mut uis[i], i)
                }
            });
        });
    }

    fn display_tier(&mut self, ui: &mut egui::Ui, i: usize) {
        let tier = TIERS[i];

        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                let w = ui.max_rect().width();
                let h = 45.0;
                let colored_rect = egui::Rect::from_center_size(
                    ui.max_rect().left_top() + egui::vec2(w, h) / 2.0,
                    egui::vec2(w, h),
                );
                ui.painter().rect(
                    colored_rect,
                    5.0,
                    tier.color,
                    egui::Stroke {
                        width: 1.0,
                        color: egui::Color32::WHITE,
                    },
                );

                let text_format = egui::TextFormat {
                    color: egui::Color32::WHITE,
                    font_id: egui::FontId::proportional(40.0),
                    ..Default::default()
                };
                ui.label(egui::text::LayoutJob::single_section(
                    tier.title.to_owned(),
                    text_format,
                ));
                ui.strong(tier.subtitle);

                let comic_numbers: Vec<usize> = self
                    .data
                    .tier_assignments
                    .iter()
                    .enumerate()
                    .skip(1)
                    .filter(|(_, &comic_tier)| comic_tier == i as u8)
                    .map(|(i, _)| i)
                    .collect();

                ui.label(format!("({})", comic_numbers.len()));
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt(i)
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show_rows(
                        ui,
                        ui.spacing().interact_size.y,
                        comic_numbers.len(),
                        |ui, range| {
                            for &c in &comic_numbers[range] {
                                let mut r = ui.selectable_label(self.n == c, format!("#{c}"));
                                if let Some(Some(comic)) = self.data.comics.get(c) {
                                    r = r.on_hover_text(&comic.title);
                                }
                                if r.clicked() {
                                    self.n = c;
                                    self.reset_img_transform();
                                }
                                r.context_menu(|ui| self.comic_context_menu_contents(ui, c));
                            }
                        },
                    );
            },
        );
    }

    fn show_summary(&mut self, ui: &mut egui::Ui) {
        const TOTAL: usize = 3000;
        const H: usize = 30;
        const W: usize = TOTAL / H;

        let scale = ui.available_width() / W as f32;

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(W as f32, H as f32) * scale, egui::Sense::click());

        let get_colored_rect = |x, y| {
            egui::Rect::from_min_size(
                rect.min + egui::vec2(x as f32, y as f32) * scale,
                egui::vec2(scale, scale),
            )
        };

        for y in (0..H).rev() {
            for x in 0..W {
                let i = y * W + x + 1;
                let colored_rect = get_colored_rect(x, y);
                let color = TIERS
                    .get(self.data.get_tier_of_comic(i) as usize)
                    .unwrap_or(&TIERS[0])
                    .color;
                ui.painter().rect_filled(colored_rect, 0.0, color);
                if response.clicked()
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|pos| colored_rect.contains(pos))
                {
                    self.n = i;
                    self.reset_img_transform();
                }
            }
        }

        ui.painter().rect_stroke(
            get_colored_rect((self.n - 1) % W, (self.n - 1) / W),
            0.0,
            egui::Stroke {
                width: 1.0,
                color: egui::Color32::WHITE,
            },
        );

        ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowUp) && self.n > W {
                self.n -= W;
                self.reset_img_transform();
            }
            if input.key_pressed(egui::Key::ArrowDown) && self.n + W < self.data.comics.len() {
                self.n += W;
                self.reset_img_transform();
            }
            if input.key_pressed(egui::Key::ArrowLeft) && self.n > 1 {
                self.n -= 1;
                self.reset_img_transform();
            }
            if input.key_pressed(egui::Key::ArrowRight) && self.n + 1 < self.data.comics.len() {
                self.n += 1;
                self.reset_img_transform();
            }
        });
    }

    fn comic_context_menu_contents(&self, ui: &mut egui::Ui, i: usize) {
        ui.hyperlink_to(format!("xkcd.com/{i}"), format!("https://xkcd.com/{i}"));
        ui.hyperlink_to(
            format!("explainxkcd.com/{i}"),
            format!("https://explainxkcd.com/{i}"),
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin {
                    left: 25.0,
                    right: 25.0,
                    top: 25.0,
                    bottom: 100.0,
                }),
            )
            .show(ctx, |ui| {
                self.show_comic_selector(ui);
                ui.add_space(20.0);
                ui.columns(2, |uis| {
                    self.show_comic_column(&mut uis[0]);
                    uis[1].with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.group(|ui| {
                            self.show_summary(ui);
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            self.show_tier_list(ui);
                        })
                    })
                });
                if self.data.unsaved {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label("Unsaved changes");
                    });
                }

                ui.input(|input| {
                    if input.modifiers.command_only() && input.key_pressed(egui::Key::S) {
                        self.data.save();
                    }

                    if input.modifiers.is_none() {
                        for (key, tier) in [
                            (egui::Key::U, 0),
                            (egui::Key::W, 1),
                            (egui::Key::S, 2),
                            (egui::Key::A, 3),
                            (egui::Key::B, 4),
                            (egui::Key::C, 5),
                            (egui::Key::D, 6),
                            (egui::Key::E, 7),
                            (egui::Key::F, 8),
                        ] {
                            if input.key_pressed(key) {
                                self.data.set_tier_of_comic(self.n, tier);
                            }
                        }
                        if input.key_pressed(egui::Key::Space) {
                            self.n += 1;
                            self.reset_img_transform();
                        }
                    }
                })
            });
    }
}
