
use egui;

pub struct CodeExample {
    name: String,
    age: u32,
}

impl CodeExample {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Saves us from writing `&mut self.name` etc
        let Self { name, age } = self;

        ui.heading("Example");
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(name);
        });

        ui.add(
            egui::DragValue::new(age)
                .range(0..=120)
                .suffix(" years"),
        );
        if ui.button("Increment").clicked() {
            *age += 1;
        }
        ui.label(format!("{name} is {age}"));
    }
}
