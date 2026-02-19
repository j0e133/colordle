
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

    let _ = eframe::run_native("Colordle", native_options, Box::new(|cc| Ok(Box::new(Colordle::new(cc)))));
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
    status: Status,
    hints: usize
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
        // let secret_color = palettes.all.match_name(&"black".into()).unwrap();

        Self {
            palettes,
            secret_color,
            render_colors: RenderColors {
                border: egui::Theme::Dark.default_visuals().window_fill.gamma_multiply(1.35),
                center: egui::Theme::Dark.default_visuals().window_fill.gamma_multiply(1.7),
                text:   egui::Theme::Dark.default_visuals().text_color()
            },
            ..Default::default()
        }
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
        // sort by similarity
        self.guesses.sort_by(|a, b| a.similarity.partial_cmp(&b.similarity).unwrap());

        // add new guess
        self.guesses.push(Guess {
            color: Rc::clone(&color),
            number: self.guesses.len() + 1,
            similarity: self.secret_color.similarity(&color)
        });

        // check for win
        if Rc::ptr_eq(&self.secret_color, &color) {
            self.status = if self.hints == self.secret_color.name.len() { Status::Lost } else { Status::Won };

            // shows the correct color in the circle
            self.guess_color = self.secret_color.name.clone();
        }
        else {
            self.guess_color.clear();
        }

        self.match_guess();
    }

    fn get_hint(&mut self) {
        // if guess isn't already hint, clone that shi and update the color dot thingy
        if self.guess_color != self.secret_color.name[..self.hints] {
            self.update_with_hint();
            return;
        }

        self.hints += match self.secret_color.name.chars().nth(self.hints) {
            Some(' ') => 2,
            Some(_) => 1,
            None => 0
        };

        self.update_with_hint();
    }

    fn update_with_hint(&mut self) {
        self.secret_color.name[..self.hints].clone_into(&mut self.guess_color);
        self.match_guess();
    }

    fn reset(&mut self) {
        self.hints = 0;

        self.status = Status::Playing;

        self.guess_color.clear();
        self.guesses.clear();
        self.matched_color = None;

        self.randomize_color();
    }

    fn labeled_rect(&self, ui: &mut egui::Ui, width: f32, text: &str, color: Color32, text_color: Color32, height: f32, pad: f32) {
        let center = ui.min_rect().center_bottom() + Vec2::new(0.0, height * 0.5);
        let rect = egui::Rect::from_center_size(center, Vec2::new(width, height));

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

    fn render_color_circle(&self, ui: &mut egui::Ui) {
        ui.allocate_ui(Vec2::new(ui.available_width(), 90.0), |ui| {
            let center = ui.min_rect().center_bottom() + Vec2::new(0.0, 23.0);

            // outline circle
            ui.painter().circle_filled(
                center,
                40.0,
                self.render_colors.border,
            );
            
            // fill circle
            ui.painter().circle_filled(
                center,
                32.0,
                match &self.matched_color {
                    Some(color) => Color32::from_hex(&color.hex).unwrap(),
                    None => self.render_colors.center
                },
            );

            // question mark if empty
            if self.matched_color.is_none() {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "?",
                    egui::FontId::new(40.0, egui::FontFamily::Proportional),
                    self.render_colors.text
                );
            }

            ui.set_height(90.0);
        });
    }

    fn render_guess_box(&mut self, ui: &mut egui::Ui) {
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

                    response.request_focus();
                }
            }

            // hint button
            if ui.button("?").on_hover_text("Hint").clicked() {
                self.get_hint();
            }
        });

        ui.add_space(20.0);
    }

    fn render_game_over(&mut self, ui: &mut egui::Ui, message: String) {
        self.labeled_rect(
            ui,
            525.0,
            &message,
            self.render_colors.center,
            self.render_colors.text,
            90.0,
            10.0
        );

        ui.add_space(8.0);

        if ui.button(egui::RichText::new("Play again").heading()).clicked() {
            self.reset();
        }

        ui.add_space(35.0);
    }

    fn render_guess(&self, ui: &mut egui::Ui, message: &str, color: Color32, text_color: Color32) {
        self.labeled_rect(
            ui,
            450.0,
            message,
            color,
            text_color,
            30.0,
            0.0
        );

        ui.add_space(10.0);
    }

    fn render_guesses(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for guess in self.guesses.iter().rev() {
                let message = match self.similarity_mode {
                    SimilarityMode::Standard => &format!("#{} {} - {:.2}%", guess.number, guess.color.name, guess.similarity * 100.0),
                    SimilarityMode::Easy => &format!("#{} {} - Brightness {:.2}%, Color {:.2}%", guess.number, guess.color.name, guess.similarity * 100.0, 0.0),
                };
                
                self.render_guess(
                    ui,
                    message,
                    Color32::from_hex(&guess.color.hex).unwrap(),
                    match guess.color.text_color {
                        colors::TextColor::Black => Color32::BLACK,
                        colors::TextColor::White => Color32::WHITE
                    }
                );
            }
                
            self.render_guess(
                ui,
                "#0 Guess!",
                self.render_colors.center,
                Color32::WHITE
            );
        });
    }
}

impl eframe::App for Colordle {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                
                ui.label(egui::RichText::new("Colordle").size(75.0).strong());
                
                ui.add_space(55.0);
                
                self.render_color_circle(ui);

                match self.status {
                    Status::Playing => 
                        self.render_guess_box(ui),
                    Status::Won => 
                        self.render_game_over(ui, format!("Congrats, you guessed the color in {} guess{} with {} hint{}!",
                        self.guesses.len(),                                                        if self.guesses.len() == 1 { "" } else { "es" },
                        self.hints - self.secret_color.name.chars().filter(|&c| c == ' ').count(), if self.hints         == 1 { "" } else { "s" })),
                    Status::Lost => 
                        self.render_game_over(ui, format!("Uh oh, you used all the hints. The color was {}!", self.secret_color.name)),
                    _ => {}
                }

                self.render_guesses(ui);
            });
        });
    }
}
