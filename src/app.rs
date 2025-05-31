use eframe::CreationContext;
use egui::{Color32, ColorImage, ImageData, TextureHandle, TextureOptions};
use image::{open, GenericImageView, Rgb, RgbImage}; // For dimensions()
use rfd::FileDialog;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct MainApp {
    ai_selected: usize,
    ep_selected: usize,
    isapi_deployed: bool,
    isprocessing: bool,
    selected_files: Vec<PathBuf>,
    screen_texture: Option<TextureHandle>,
}

impl MainApp {
    pub fn new(cc: &CreationContext) -> Self {
        Self {
            ai_selected: 0,
            ep_selected: 0,
            isapi_deployed: false,
            isprocessing: false,            
            selected_files: Vec::new(), // Add this
            screen_texture: None,
        }
    }
}

impl eframe::App for MainApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui_extras::install_image_loaders(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("About", |ui| {
                    ui.hyperlink_to("Website", "https://boquila.org/en");
                    ui.hyperlink_to("Donate", "https://boquila.org/donate");
                    ui.hyperlink_to("Source code", "https://github.com/boquila/boquilahub/");
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

            ui.vertical_centered(|ui| {
                ui.heading("💻 Setup");
            });
            ui.separator();

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
                    thread::sleep(Duration::from_secs(2));
                });
                self.isapi_deployed = true;
            }

            if self.isapi_deployed {
                ui.label("API deployed");
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.heading("📋 Select your data");
            });
            ui.separator();

            // File selection logic

            // Option 1: Grid layout with consistent sizing and spacing
            ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            egui::Grid::new("file_selection_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    // Folder selection
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new("Folder"))
                        .clicked()
                    {
                        // Folder selection logic here
                    }

                    // Image file selection
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new("Image"))
                        .clicked()
                    {
                        match FileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg", "gif", "bmp"])
                            .pick_files()
                        {
                            Some(paths) => {
                                self.screen_texture =
                                    Some(file_path_to_texture(paths[0].clone(), ctx));
                                self.selected_files = paths;

                                
                            }
                            _ => (), // no selection, do nothing
                        }
                    }
                    ui.end_row();

                    // Video file selection
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new("Video"))
                        .clicked()
                    {
                        // Video selection logic here
                    }

                    // Camera feed
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new("Feed"))
                        .clicked()
                    {
                        // Camera feed logic here
                    }
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.image("https://i.pinimg.com/736x/a3/f5/d9/a3f5d95d519315eb158c867d7121dd3a.jpg");
            egui::ScrollArea::vertical().show(ui, |ui| {
                // This should obviously not be here, but it's just a test
                match &self.screen_texture {
                    Some(texture) => {
                        ui.add(
                            egui::Image::new(texture)
                                .max_height(800.0)
                                .corner_radius(10.0),
                        );
                    }
                    None => {

                        // no image is here to be deployed
                    }
                }
            });
        });
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn file_path_to_texture(path: PathBuf, ctx: &egui::Context) -> TextureHandle {
    let a = fs::read(path).unwrap();
    let b = load_image_from_memory(&a).unwrap();

    let screen_texture = ctx.load_texture(
        "current_img", // name for the texture
        b,
        TextureOptions::default(),
    );

    return screen_texture;
}
