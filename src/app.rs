use std::thread;
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct MainApp {
    ai_selected: usize,
    ep_selected: usize,
    isapi_deployed: bool,
    isprocessing: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            ai_selected: 0,
            ep_selected: 0,
            isapi_deployed: false,
            isprocessing: false,
        }
    }
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for MainApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("About", |ui| {
                    ui.hyperlink_to("Website", "https://boquila.org/en");
                    ui.hyperlink_to("Donate", "https://boquila.org/donate");
                    ui.hyperlink_to("Model HUB", "https://boquila.org/hub");
                });
                ui.menu_button("Models", |ui| {
                    ui.hyperlink_to("Model HUB", "https://boquila.org/hub");
                }); 
                

                egui::widgets::global_theme_preference_switch(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            // TODO: define at runtime
            let ai_alternatives = [
                "boquilanet-gen 0.1",
                "boquilanet-cl 0.1",
                "MDV6-yolov9-e-1280",
            ];
            let ep_alternatives = ["CPU", "CUDA", "Remote BoquilaHUB"];

            ui.label("Select an AI ");
            egui::ComboBox::from_id_salt("AI").show_index(
                ui,
                &mut self.ai_selected,
                ai_alternatives.len(),
                |i| ai_alternatives[i],
            );

            ui.add_space(8.0);            
            ui.label("Select a processor");
            egui::ComboBox::from_id_salt("EP").show_index(
                ui,
                &mut self.ep_selected,
                ep_alternatives.len(),
                |i| ep_alternatives[i],
            );

            ui.add_space(8.0);
            ui.label("API ");
            if ui.button("Deploy").clicked() {
                tokio::spawn(async {
                    // API deplyoing logic
                    // Placeholder
                    thread::sleep(Duration::from_secs(2)); 
                    println!("time is done");
                });   
                println!("done");
                self.isapi_deployed = true;
            }

            if self.isapi_deployed {
                ui.label("API deployed");
            }

            ui.separator();

            ui.hyperlink_to("Source code", "https://github.com/boquila/boquilahub/");
        });

        egui::CentralPanel::default().show(ctx, |ui| {});
    }
}
