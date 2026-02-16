
mod colors;

use std::rc::Rc;

use egui::{Color32, Vec2};

use crate::colors::{Color, Palettes};


fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 800.0])
            .with_min_inner_size([550.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(Colordle::new(cc)))));
}


#[derive(Default)]
struct RenderColors{
    pub border: Color32,
    pub center: Color32,
    pub text: Color32
}


#[derive(Default)]
enum Status {
    #[default]
    Playing,
    SelectMode,
    Won,
    Lost
}


#[derive(Default)]
enum Palette {
    #[default]
    Basic,
    Advanced,
    All,
    Wikipedia
}

#[derive(Default)]
enum SimilarityMode {
    #[default]
    Standard,
    Easy
} 


struct Guess {
    color: Rc<Color>,
    number: usize,
    similarity: f32
}


#[derive(Default)]
struct Colordle {
    palettes: Palettes,
    render_colors: RenderColors,

    current_palette: Palette,
    similarity_mode: SimilarityMode,
    secret_color: Rc<Color>,
    guess_color: String,
    matched_color: Option<Rc<Color>>,
    guesses: Vec<Guess>,
    hint: String,

    status: Status,
    wins: usize
}

impl Colordle {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        cc.egui_ctx.set_theme(egui::Theme::Dark);

        let palettes = Palettes::new().unwrap();

        let secret_color = palettes.basic.random();

        Self {
            palettes,
            secret_color,
            render_colors: RenderColors {
                border: egui::Theme::Dark.default_visuals().window_fill.gamma_multiply(1.35),
                center: egui::Theme::Dark.default_visuals().window_fill.gamma_multiply(1.7),
                text: egui::Theme::Dark.default_visuals().text_color()
            },
            ..Default::default()
        }
    }

    fn labeled_rect(&self, ui: &mut egui::Ui, text: &str, color: Color32, text_color: Color32, height: f32, pad: f32) {
        let center = ui.min_rect().center_bottom() + Vec2::new(0.0, height * 0.5);
        let rect = egui::Rect::from_center_size(center, Vec2::new(475.0, height));

        ui.add_space(5.0 + pad);

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

        ui.add_space(5.0);
    }

    fn randomize_color(&mut self) {
        self.secret_color = match self.current_palette {
            Palette::Basic     => self.palettes.basic.random(),
            Palette::Advanced  => self.palettes.advanced.random(),
            Palette::All       => self.palettes.all.random(),
            Palette::Wikipedia => self.palettes.wikipedia.random()
        };
    }

    fn match_guess(&mut self) {
        self.matched_color = self.palettes.all.match_name(&self.guess_color);
    }

    fn guess(&mut self, color: Rc<Color>) {
        let similarity = self.secret_color.similarity(&color);

        self.guesses.sort_by(|a, b| a.similarity.partial_cmp(&b.similarity).unwrap());

        self.guesses.push(Guess {
            color: color,
            number: self.guesses.len() + 1,
            similarity
        });

        if similarity >= 0.99995 {
            if self.hint.len() != self.secret_color.name.len() {
                self.status = Status::Won;

                self.wins += 1;
            }
            else {
                self.status = Status::Lost
            }
        }
    }

    fn get_hint(&mut self) {
        match self.secret_color.name.chars().nth(self.hint.len()) {
            Some(' ') => {
                self.hint.push(' ');
                self.get_hint();
            },
            Some(ch) => {
                self.hint.push(ch);
                self.guess_color = self.hint.clone();
                self.match_guess();
            }
            None => {
                self.status = Status::Lost;
            }
        }
    }
}

impl eframe::App for Colordle {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Colordle").size(75.0).strong());
                ui.add_space(52.0);

                ui.allocate_ui(Vec2::new(ui.available_width(), 90.0), |ui| {
                    let center = ui.min_rect().center_bottom() + Vec2::new(0.0, 23.0);

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

                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "?",
                                egui::FontId::new(40.0, egui::FontFamily::Proportional),
                                self.render_colors.text
                            );
                        }
                    }

                    ui.set_height(90.0);
                });

                ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 0.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space((ui.available_width() - 280.0) * 0.5);

                    let response = ui.add(egui::text_edit::TextEdit::singleline(&mut self.guess_color)
                        .hint_text("Guess a color")
                        .horizontal_align(egui::Align::Center)
                        .font(egui::TextStyle::Heading));

                    // update color when text box is changed
                    if response.changed() {
                        self.match_guess();
                    }

                    // submit color
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(color) = &self.matched_color {
                            self.guess(Rc::clone(color));

                            self.guess_color.clear();
                            self.hint.clear();
                            self.match_guess();

                            response.request_focus();
                        }
                    }

                    if ui.button("?").on_hover_text("Hint").clicked() {
                        self.get_hint();
                    }
                });

                ui.add_space(20.0);

                match self.status {
                    Status::Won | Status::Lost => {
                        ui.allocate_ui(Vec2::new(ui.available_width(), 90.0), |ui| {
                            self.labeled_rect(
                                ui,
                                &format!("Congrats, you guessed the color in {} guess{}!", self.guesses.len(), if self.guesses.len() > 1 { "es" } else { "" }),
                                self.render_colors.center,
                                self.render_colors.text,
                                90.0,
                                10.0
                            );

                            ui.add_space(8.0);

                            if ui.button(egui::RichText::new("Play again").heading()).clicked() {
                                self.status = Status::Playing;

                                self.randomize_color();
                                self.guesses.clear();
                            }

                            ui.add_space(20.0);
                        });
                    },
                    Status::Lost => {

                    },
                    _ => {}
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for guess in self.guesses.iter().rev() {
                        let text = match self.similarity_mode {
                            SimilarityMode::Standard => &format!("#{} {} - {:.2}%", guess.number, guess.color.name, guess.similarity * 100.0),
                            SimilarityMode::Easy => &format!("#{} {} - Brightness {:.2}%, Color {:.2}%", guess.number, guess.color.name, guess.similarity * 100.0, 0.0),
                        };

                        self.labeled_rect(
                            ui,
                            text,
                            Color32::from_hex(&guess.color.hex).unwrap(),
                            match guess.color.text_color {
                                colors::TextColor::White => self.render_colors.text,
                                colors::TextColor::Black => Color32::BLACK,
                            },
                            30.0,
                            0.0
                        );

                        ui.add_space(10.0);
                    }

                    self.labeled_rect(
                        ui,
                        "#0 Guess!",
                        self.render_colors.center,
                        self.render_colors.text,
                        30.0,
                        0.0
                    );

                    ui.add_space(10.0);
                });
            });
        });
    }
}
