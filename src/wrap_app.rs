#[cfg(target_arch = "wasm32")]
use core::any::Any;
use std::str::FromStr;

use crate::io::VolumeDataFileType;

/// The state that we persist (serialize).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    backend_panel: super::backend_panel::BackendPanel,
    #[cfg_attr(feature = "serde", serde(skip))]
    importer: super::io::Importer,
}

/// Wraps many rendering apps into one for grid views and shared memory.
pub struct WrapApp {
    state: State,
    
    volume_texture: crate::apps::Texture,
    slice_renderer: Option<crate::apps::SliceRenderer>,
}

impl WrapApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
            volume_texture: crate::apps::Texture::default(_cc).unwrap(),
            slice_renderer: None,
        };

        slf.slice_renderer = crate::apps::SliceRenderer::new(_cc.wgpu_render_state.as_ref().unwrap(), &slf.volume_texture);

        #[cfg(feature = "persistence")]
        if let Some(storage) = _cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        slf
    }

    pub fn update_volume_texture(&mut self, frame: &mut eframe::Frame){
        let wgpu_render_state = eframe::Frame::wgpu_render_state(frame).unwrap();
        let device = &wgpu_render_state.device;
        let queue = &wgpu_render_state.queue;

        let mut bytes = self.state.importer.item.data.as_mut().unwrap();
        let dimensions = self.state.importer.item.dimensions.unwrap();
        let label = Some("Volume Texture");
        
        self.volume_texture = crate::apps::Texture::from_u16_bytes(device, queue, &mut bytes, &dimensions, label).unwrap();
        self.slice_renderer = crate::apps::SliceRenderer::new(wgpu_render_state, &self.volume_texture);
        self.state.importer = crate::io::Importer::default();
    }
}

impl eframe::App for WrapApp {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            frame.set_fullscreen(!frame.info().window_info.fullscreen);
        }

        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            egui::trace!(ui);
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.menu_bar_contents(ui);
            });
        });

        self.state.backend_panel.update(ctx, frame);

        self.backend_panel(ctx, frame);

        self.show_renderer(ctx, frame);

        if self.state.importer.new_data_available {
            self.update_volume_texture(frame);
        }

        self.show_importer(ctx);

        self.state.backend_panel.end_of_frame(ctx);

        self.ui_file_drag_and_drop(ctx);

        // On web, the browser controls `pixels_per_point`.
        if !frame.is_web() {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(&mut *self)
    }
}

impl WrapApp {
    fn backend_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // The backend-panel can be toggled on/off.
        // We show a little animation when the user switches it.
        let is_open =
            self.state.backend_panel.open || ctx.memory(|mem| mem.everything_is_visible());

        egui::SidePanel::left("backend_panel")
            .resizable(false)
            .show_animated(ctx, is_open, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Backend");
                });

                ui.separator();
                self.backend_panel_contents(ui, frame);
            });
    }

    fn backend_panel_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.backend_panel.ui(ui, frame);

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button("Reset GUI")
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close_menu();
            }

            if ui.button("Reset everything").clicked() {
                self.state = Default::default();
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close_menu();
            }
        });
    }

    fn show_renderer(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let renderer = self.slice_renderer.as_mut().map(|app| app as &mut dyn eframe::App).unwrap();
        renderer.update(ctx, frame);
    }

    fn show_importer(&mut self, ctx: &egui::Context) {
        self.state.importer.show(ctx);
    }

    fn file_menus(&mut self, ui: &mut egui::Ui) {
        // TODO: disable file menu while import or export is in progress
        ui.menu_button("File", |ui| {
            // Native open file dialog is currently not supported on wasm and would require complex work around...
            // https://stackoverflow.com/questions/71017592/can-i-read-files-from-the-disk-by-using-webassembly-re-evaluated
            #[cfg(not(target_arch = "wasm32"))]
            {
                // if ui.button("Open *.vdc...").clicked() {
                //     ui.close_menu();
                // }
                ui.menu_button("Open...", |ui| {
                    // if ui.button("Volume Data Container (.vdc)").clicked() {
                    // }
                    if ui.button("3D Raw (.raw)").clicked() {
                        self.state.importer.load_dialog(VolumeDataFileType::RAW3D);
                        ui.close_menu();
                    }
                    // if ui.button("DICOM (.dcm)").clicked() {
                    // }
                    // if ui.button("Series of Bitmaps (.bmp)").clicked() {
                    // }
                    // if ui.button("Series of Raw Binary Slices (.*)").clicked() {
                    // }
                });
            }
            #[cfg(target_arch = "wasm32")]
            {
                if ui.button("Open file...").clicked() {
                    // show drag and drop overlay
                    ui.close_menu();
                }
            }
            
            // if ui.button("Load Example...").clicked() {
            //     ui.close_menu();
            // }
            // ui.separator();
            // ui.menu_button("Export as...", |ui| {
            //     if ui.button("3D Raw (.raw)").clicked() {
            //     }
            // });
        });
    }

    fn menu_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.backend_panel.open, "💻 Backend");

        ui.separator();

        ui.horizontal(|ui| {
            self.file_menus(ui);
        });
        ui.separator();

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;
        use std::fmt::Write as _;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Dropping file:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        // text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped file:
        // TODO: Implement support for file formats with multiple files
        ctx.input(|i| {
            if i.raw.dropped_files.len() == 1 {
                let file = i.raw.dropped_files.first().unwrap();

                self.state.importer.item.path = if let Some(path) = &file.path {
                    Some(path.to_path_buf())
                } else if !file.name.is_empty() {
                    Some(std::path::PathBuf::from_str(file.name.clone().as_str()).unwrap())
                } else {
                    Some(std::path::PathBuf::from_str("???").unwrap())
                };

                self.state.importer.item.data = match &file.bytes {
                    Some(bytes) => Some(bytes.to_owned().to_vec()),
                    None => None,
                };

                self.state.importer.load_dialog(VolumeDataFileType::RAW3D);
            }
        });
    }
}
