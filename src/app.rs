use super::localization::*;
use crate::api;
use crate::api::abstractions::AI;
use crate::api::bq::get_bqs;
use api::import::IMAGE_FORMATS;
use api::import::VIDEO_FORMATS;
use egui::{ColorImage, TextureHandle, TextureOptions};
use rfd::FileDialog;
use std::fs::{self};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub struct MainApp {
    ai_selected: usize,
    ep_selected: usize,
    isapi_deployed: bool,
    is_processing: bool,
    selected_files: Vec<PathBuf>,
    screen_texture: Option<TextureHandle>,
    image_texture_n: usize,
    lang: Lang,
    ais: Vec<AI>,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            ai_selected: 0,
            ep_selected: 0,
            isapi_deployed: false,
            is_processing: false,
            selected_files: Vec::new(), // Add this
            screen_texture: None,
            image_texture_n: 0,
            lang: Lang::EN,
            ais: get_bqs(),
        }
    }

    pub fn t(&self, key: Key) -> &'static str {
        translate(key, &self.lang)
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
                ui.menu_button(self.t(Key::about), |ui| {
                    ui.hyperlink_to("Website", "https://boquila.org/en");
                    ui.hyperlink_to(self.t(Key::donate), "https://boquila.org/donate");
                    ui.hyperlink_to(
                        self.t(Key::source_code),
                        "https://github.com/boquila/boquilahub/",
                    );
                });
                ui.menu_button(self.t(Key::models), |ui| {
                    ui.hyperlink_to("Model HUB", "https://boquila.org/hub");
                });

                ui.menu_button(self.t(Key::idiom), |ui| {
                    ui.radio_value(&mut self.lang, Lang::EN, "English");
                    ui.radio_value(&mut self.lang, Lang::ES, "Español");
                });

                egui::widgets::global_theme_preference_switch(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("💻 {}", self.t(Key::setup)));
            });
            ui.separator();

            ui.label(self.t(Key::select_ai));

            egui::ComboBox::from_id_salt("AI").show_index(
                ui,
                &mut self.ai_selected,
                self.ais.len(),
                |i| self.ais[i].name.as_str(),
            );

            ui.add_space(8.0);
            let ep_alternatives = ["CPU", "CUDA", "Remote BoquilaHUB"];
            ui.label(self.t(Key::select_ep));
            egui::ComboBox::from_id_salt("EP").show_index(
                ui,
                &mut self.ep_selected,
                ep_alternatives.len(),
                |i| ep_alternatives[i],
            );

            ui.add_space(8.0);
            ui.label("API ");

            if !self.isapi_deployed {
                if ui.button(self.t(Key::deploy)).clicked() {
                    tokio::spawn(async {
                        thread::sleep(Duration::from_secs(2));
                    });
                    self.isapi_deployed = true;
                }
            }

            if self.isapi_deployed {
                ui.label(self.t(Key::deployed_api));
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.heading(format!("📎 {}", self.t(Key::select_your_data)));
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
                    // FOLDER SELECTION SECTION
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::folder)))
                        .clicked()
                    {
                        match FileDialog::new().pick_folder() {
                            Some(folder_path) => {
                                // Read directory contents and filter for image files
                                match fs::read_dir(&folder_path) {
                                    Ok(entries) => {
                                        let mut image_files = Vec::new();

                                        for entry in entries {
                                            if let Ok(entry) = entry {
                                                let path = entry.path();

                                                // Only process files (not subdirectories)
                                                if path.is_file() {
                                                    // Check if file extension matches IMAGE_FORMATS
                                                    if let Some(extension) = path.extension() {
                                                        if let Some(ext_str) = extension.to_str() {
                                                            if IMAGE_FORMATS.iter().any(|&format| {
                                                                ext_str.to_lowercase()
                                                                    == format.to_lowercase()
                                                            }) {
                                                                image_files.push(path);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if !image_files.is_empty() {
                                            // Set the first image as the screen texture
                                            self.screen_texture = Some(file_path_to_texture(
                                                image_files[0].clone(),
                                                ctx,
                                            ));
                                            self.selected_files = image_files;
                                        } else {
                                            // Handle case where no image files were found
                                            println!("No image files found in the selected folder");
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error reading directory: {}", e);
                                    }
                                }
                            }
                            None => (), // No folder selected
                        }
                    }

                    // IMAGE FILE SELECTION SECTION
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::image)))
                        .clicked()
                    {
                        match FileDialog::new()
                            .add_filter("Image", &IMAGE_FORMATS)
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

                    // VIDEO FILE SELECTION SECTION
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::video_file)))
                        .clicked()
                    {
                        match FileDialog::new()
                            .add_filter("Video", &VIDEO_FORMATS)
                            .pick_files()
                        {
                            Some(paths) => {
                                todo!()
                            }
                            _ => (), // no selection, do nothing
                        }
                    }

                    // Camera feed
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::camera_feed)))
                        .clicked()
                    {
                        // Camera feed logic here
                    }
                });

            if self.selected_files.len() > 0 {
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.heading(format!("📋 {}", self.t(Key::analysis)));
                });
                ui.separator();
                // ANALYZE BUTTON SECTION

                ui.vertical_centered(|ui| {
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::analyze)))
                        .clicked()
                    {
                        // Analyze logic
                    }
                });

                ui.add_space(8.0);

                ui.vertical_centered(|ui| {
                    if ui
                        .add_sized([85.0, 40.0], egui::Button::new(self.t(Key::export)))
                        .clicked()
                    {
                        // Analyze logic
                    }
                });
            }
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
