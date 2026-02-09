
mod colors;

use std::rc::Rc;

use egui::Color32;
use rand::seq::IndexedRandom;

use crate::colors::{Color, Palette};


fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(Colordle::new(cc)))));
}


#[derive(Default)]
struct RenderColors{
    pub border: Color32,
    pub center: Color32
}


struct Guess {
    color: Rc<Color>,
    number: usize,
    similarity: f32
}


#[derive(Default)]
struct Colordle {
    colors: Vec<Rc<Color>>,
    secret_color: Rc<Color>,
    search_color: String,
    matched_color: Option<Rc<Color>>,
    guesses: Vec<Guess>,
    render_colors: RenderColors
}

impl Colordle {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        cc.egui_ctx.set_theme(egui::Theme::Dark);

        let palette = Palette::load().unwrap();
        let mut colors: Vec<Rc<Color>> = palette
            .colors
            .into_iter()
            .map(|raw| raw.color().into())
            .collect();

        colors.sort_by(|a, b| a.search_name.cmp(&b.search_name));

        let secret_color = colors.choose(&mut rand::rng()).unwrap().clone();

        println!("Secret {:?}", secret_color);

        Self {
            colors,
            secret_color: secret_color.into(),
            search_color: String::new(),
            matched_color: None,
            guesses: vec!(),
            render_colors: RenderColors {
                border: egui::Theme::Dark.default_visuals().window_fill.linear_multiply(1.4),
                center: egui::Theme::Dark.default_visuals().window_fill.linear_multiply(1.6)
            }
        }
    }

    fn guess(&mut self, color: Rc<Color>) {
        println!("Guessed {:?}", color);

        let similarity = self.secret_color.similarity(&color);

        self.guesses.sort_by(|a, b| a.similarity.partial_cmp(&b.similarity).unwrap());

        self.guesses.push(Guess {
            color: color,
            number: self.guesses.len() + 1,
            similarity
        });
    }

    fn labeled_rect(&self, ui: &mut egui::Ui, text: &str, color: Color32, text_color: Color32) {
        let center = ui.min_rect().center_bottom() + egui::Vec2::new(0.0, 10.0);
        let rect = egui::Rect::from_center_size(center, egui::Vec2::new(400.0, 30.0));
        
        ui.painter().rect_filled(
            rect,
            5.0,
            self.render_colors.border
        );
        
        ui.painter().rect_filled(
            rect.shrink(3.0),
            5.0,
            color
        );

        ui.colored_label(
            text_color,
            egui::RichText::new(text).heading()
        );
    }
}

impl eframe::App for Colordle {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Colordle").size(75.0).strong());
                ui.add_space(75.0);

                let center = ui.min_rect().center_bottom();

                ui.painter().circle_filled(
                    center,
                    40.0,
                    self.render_colors.border,
                );

                match &self.matched_color {
                    Some(color) => {
                        ui.painter().circle_filled(
                            center,
                            32.0,
                            Color32::from_hex(&color.hex).unwrap(),
                        );
                    },
                    None => {
                        ui.painter().circle_filled(
                            center,
                            32.0,
                            self.render_colors.center
                        );
                    }
                }

                ui.add_space(90.0);

                let response = ui.add(egui::text_edit::TextEdit::singleline(&mut self.search_color)
                    .hint_text("Guess a color")
                    .horizontal_align(egui::Align::Center)
                    .font(egui::TextStyle::Heading));

                // update color when text box is changed
                if response.changed() {
                    let search_color = self.search_color.to_lowercase().replace(" ", "");

                    // match the color using binary search (fast)
                    self.matched_color = self.colors
                        .binary_search_by(|a| a.search_name.cmp(&search_color))
                        .ok()
                        .map(|i| Rc::clone(&self.colors[i]));
                }

                // submit color
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(color) = &self.matched_color {
                        self.guess(Rc::clone(color));

                        self.search_color.clear();
                    }
                }

                ui.add_space(20.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for guess in self.guesses.iter().rev() {
                        self.labeled_rect(
                            ui,
                            &format!("#{} {} - {:.2}%", guess.number, guess.color.name, guess.similarity * 100.0),
                            Color32::from_hex(&guess.color.hex).unwrap(),
                            match guess.color.text_color {
                                colors::TextColor::White => Color32::WHITE,
                                colors::TextColor::Black => Color32::BLACK,
                            },
                        );

                        ui.add_space(20.0);
                    }

                    self.labeled_rect(
                        ui,
                        "#0 Guess!",
                        self.render_colors.center,
                        Color32::WHITE
                    );

                    ui.add_space(35.0);
                });
            });
        });
    }
}
