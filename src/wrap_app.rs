#[cfg(target_arch = "wasm32")]
use core::any::Any;
use egui::{Id, Margin};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use std::str::FromStr;

use crate::{
    apps::{RayMarchingRenderer, SliceRenderer},
    io::VolumeDataFileType,
};

// Docking GUI

trait TabUi {
    fn ui(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
    fn show_settings_oberlay(&mut self, _show: bool) {}
}
struct RayMarchingFirstHit {
    renderer: Option<RayMarchingRenderer>,
}
impl RayMarchingFirstHit {
    fn new(
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        let renderer = crate::apps::RayMarchingRenderer::new(wgpu_render_state, volume_texture);

        Self { renderer }
    }
}
impl TabUi for RayMarchingFirstHit {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let renderer = self.renderer.as_mut().unwrap();
        renderer.custom_painting(ui);
    }
    fn title(&self) -> String {
        "First Hit".to_owned()
    }
    fn show_settings_oberlay(&mut self, show: bool) {
        self.renderer.as_mut().unwrap().show_settings_oberlay = show;
    }
}
struct SliceViewAxial {
    slice_renderer: Option<SliceRenderer>,
}
impl SliceViewAxial {
    fn new(
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        let slice_renderer = crate::apps::SliceRenderer::axial(wgpu_render_state, volume_texture);

        Self { slice_renderer }
    }
}
impl TabUi for SliceViewAxial {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let renderer = self.slice_renderer.as_mut().unwrap();
        renderer.custom_painting(ui);
    }
    fn title(&self) -> String {
        "Axial".to_owned()
    }
    fn show_settings_oberlay(&mut self, show: bool) {
        self.slice_renderer.as_mut().unwrap().show_settings_oberlay = show;
    }
}
struct SliceViewCoronal {
    slice_renderer: Option<SliceRenderer>,
}
impl SliceViewCoronal {
    fn new(
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        let slice_renderer = crate::apps::SliceRenderer::coronal(wgpu_render_state, volume_texture);

        Self { slice_renderer }
    }
}
impl TabUi for SliceViewCoronal {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let renderer = self.slice_renderer.as_mut().unwrap();
        renderer.custom_painting(ui);
    }
    fn title(&self) -> String {
        "Coronal".to_owned()
    }
    fn show_settings_oberlay(&mut self, show: bool) {
        self.slice_renderer.as_mut().unwrap().show_settings_oberlay = show;
    }
}
struct SliceViewSaggital {
    slice_renderer: Option<SliceRenderer>,
}
impl SliceViewSaggital {
    fn new(
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        let slice_renderer =
            crate::apps::SliceRenderer::saggital(wgpu_render_state, volume_texture);

        Self { slice_renderer }
    }
}
impl TabUi for SliceViewSaggital {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let renderer = self.slice_renderer.as_mut().unwrap();
        renderer.custom_painting(ui);
    }
    fn title(&self) -> String {
        "Saggital".to_owned()
    }
    fn show_settings_oberlay(&mut self, show: bool) {
        self.slice_renderer.as_mut().unwrap().show_settings_oberlay = show;
    }
}

struct Tab {
    node: NodeIndex,
    content: Box<dyn TabUi>,
}
impl Tab {
    fn ray_marching_first_hit(
        node_index: usize,
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        Self {
            node: NodeIndex(node_index),
            content: Box::new(RayMarchingFirstHit::new(wgpu_render_state, volume_texture)),
        }
    }

    fn slice_view_axial(
        node_index: usize,
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        Self {
            node: NodeIndex(node_index),
            content: Box::new(SliceViewAxial::new(wgpu_render_state, volume_texture)),
        }
    }

    fn slice_view_coronal(
        node_index: usize,
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        Self {
            node: NodeIndex(node_index),
            content: Box::new(SliceViewCoronal::new(wgpu_render_state, volume_texture)),
        }
    }

    fn slice_view_saggital(
        node_index: usize,
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Self {
        Self {
            node: NodeIndex(node_index),
            content: Box::new(SliceViewSaggital::new(wgpu_render_state, volume_texture)),
        }
    }

    fn title(&self) -> String {
        self.content.title()
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        self.content.ui(ui);
    }
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<Tab>,
    wgpu_render_state: &'a eframe::egui_wgpu::RenderState,
    volume_texture: &'a crate::apps::Texture,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.content(ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab.node)
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, node: NodeIndex) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        if ui.button("Axial").clicked() {
            self.added_nodes.push(Tab::slice_view_axial(
                node.0,
                self.wgpu_render_state,
                self.volume_texture,
            ));
        }

        if ui.button("Coronal").clicked() {
            self.added_nodes.push(Tab::slice_view_coronal(
                node.0,
                self.wgpu_render_state,
                self.volume_texture,
            ));
        }

        if ui.button("Saggital").clicked() {
            self.added_nodes.push(Tab::slice_view_saggital(
                node.0,
                self.wgpu_render_state,
                self.volume_texture,
            ));
        }

        if ui.button("First Hit").clicked() {
            self.added_nodes.push(Tab::ray_marching_first_hit(
                node.0,
                self.wgpu_render_state,
                self.volume_texture,
            ));
        }
    }
}

/// The state that we persist (serialize).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    backend_panel: super::backend_panel::BackendPanel,
    #[cfg_attr(feature = "serde", serde(skip))]
    importer: super::io::Importer,
    hide_settings_oberlay: bool,
}

/// Wraps many rendering apps into one for grid views and shared memory.
pub struct WrapApp {
    state: State,
    tree: Tree<Tab>,
    node_counter: usize,

    volume_texture: crate::apps::Texture,
}

impl WrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let volume_texture = crate::apps::Texture::default(cc).unwrap();
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let tree = Self::default_dock(wgpu_render_state, &volume_texture);

        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
            tree,
            node_counter: 4,
            volume_texture,
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        slf
    }

    fn default_dock(
        wgpu_render_state: &eframe::egui_wgpu::RenderState,
        volume_texture: &crate::apps::Texture,
    ) -> Tree<Tab> {
        let mut tree = Tree::new(vec![Tab::slice_view_axial(
            0,
            wgpu_render_state,
            volume_texture,
        )]);

        // Modify the tree before constructing the dock
        let [a, b] = tree.split_right(
            NodeIndex::root(),
            0.5,
            vec![Tab::slice_view_coronal(
                1,
                wgpu_render_state,
                volume_texture,
            )],
        );
        let [_, _] = tree.split_below(
            a,
            0.5,
            vec![Tab::slice_view_saggital(
                2,
                wgpu_render_state,
                volume_texture,
            )],
        );
        let [_, _] = tree.split_below(
            b,
            0.5,
            vec![Tab::ray_marching_first_hit(
                3,
                wgpu_render_state,
                volume_texture,
            )],
        );

        tree
    }

    pub fn update_volume_texture(&mut self, frame: &mut eframe::Frame) {
        let wgpu_render_state = eframe::Frame::wgpu_render_state(frame).unwrap();
        let device = &wgpu_render_state.device;
        let queue = &wgpu_render_state.queue;

        let bytes = self.state.importer.item.data.as_mut().unwrap();
        let dimensions = self.state.importer.item.dimensions.unwrap();
        let spacing = self.state.importer.item.spacing.unwrap();
        let label: Option<&str> = Some("Volume Texture");

        self.volume_texture =
            crate::apps::Texture::from_u16_bytes(device, queue, bytes, dimensions, spacing, label)
                .unwrap();

        self.tree = Self::default_dock(wgpu_render_state, &self.volume_texture);

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

        self.show_dock(ctx, frame);

        if self.state.importer.new_data_available {
            self.update_volume_texture(frame);
        }

        self.show_importer(ctx);

        self.ui_file_drag_and_drop(ctx);

        self.state.backend_panel.end_of_frame(ctx);

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

    fn show_dock(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut added_nodes = Vec::new();
        let wgpu_render_state = eframe::Frame::wgpu_render_state(frame).unwrap();
        let volume_texture = &self.volume_texture;

        let mut style = Style::from_egui(ctx.style().as_ref());
        style.tabs.inner_margin = Margin::same(0.0);
        // style.tabs.bg_fill = Color32::BLACK;

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .style(style)
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                    wgpu_render_state,
                    volume_texture,
                },
            );

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node.node);
            self.tree.push_to_focused_leaf(Tab {
                node: NodeIndex(self.node_counter),
                content: node.content,
            });
            self.node_counter += 1;
        });
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
                    self.state.importer.show_drag_and_drop = true;
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

    fn view_menus(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            let hide_settings_oberlay = if self.state.hide_settings_oberlay {
                "Show Settings Overlays"
            } else {
                "Hide Settings Overlays"
            };
            if ui.button(hide_settings_oberlay).clicked() {
                self.state.hide_settings_oberlay = !self.state.hide_settings_oberlay;

                for node in self.tree.iter_mut() {
                    if let egui_dock::Node::Leaf { tabs, .. } = node {
                        for tab in tabs {
                            tab.content
                                .show_settings_oberlay(!self.state.hide_settings_oberlay);
                        }
                    }
                }

                ui.close_menu();
            }
        });
    }

    fn menu_bar_contents(&mut self, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.backend_panel.open, "💻 Backend");

        ui.separator();

        ui.horizontal(|ui| {
            self.file_menus(ui);
            ui.separator();
            self.view_menus(ui);
        });
        ui.separator();

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::*;

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.state.importer.show_drag_and_drop = false;
        }

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) || self.state.importer.show_drag_and_drop
        {
            #[cfg(target_arch = "wasm32")]
            let text = "Simply drag and drop your file here to import.\n              Press the escape key to abort.".to_owned();

            #[cfg(not(target_arch = "wasm32"))]
            let text = ctx.input(|i| {
                use std::fmt::Write as _;
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

                self.state.importer.item.data =
                    file.bytes.as_ref().map(|bytes| bytes.to_owned().to_vec());

                self.state.importer.load_dialog(VolumeDataFileType::RAW3D);

                self.state.importer.show_drag_and_drop = false;
            }
        });
    }
}
