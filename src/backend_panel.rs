/// How often we repaint
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    /// This is the default.
    ///
    /// If this is selected, egui is only updated if are input events
    /// (like mouse movements) or there are some animations in the GUI.
    ///
    /// Reactive mode saves CPU.
    ///
    /// The downside is that the UI can become out-of-date if something it is supposed to monitor changes.
    /// For instance, a GUI for a thermostat need to repaint each time the temperature changes.
    /// To ensure the UI is up to date you need to call `egui::Context::request_repaint()` each
    /// time such an event happens. You can also chose to call `request_repaint()` once every second
    /// or after every single frame - this is called [`Continuous`](RunMode::Continuous) mode,
    /// and for games and interactive tools that need repainting every frame anyway, this should be the default.
    Reactive,

    /// This will call `egui::Context::request_repaint()` at the end of each frame
    /// to request the backend to repaint as soon as possible.
    ///
    /// On most platforms this will mean that egui will run at the display refresh rate of e.g. 60 Hz.
    ///
    /// For this demo it is not any reason to do so except to
    /// demonstrate how quickly egui runs.
    ///
    /// For games or other interactive apps, this is probably what you want to do.
    /// It will guarantee that egui is always up-to-date.
    Continuous,
}

/// Default is Reactive since
/// 1) We want to use minimal CPU
/// 2) There are no external events that could invalidate the UI
///    so there are no events to miss.
impl Default for RunMode {
    fn default() -> Self {
        RunMode::Reactive
    }
}

// ----------------------------------------------------------------------------

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct BackendPanel {
    pub open: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    // go back to [`RunMode::Reactive`] mode each time we start
    run_mode: RunMode,

    /// current slider value for current gui scale
    #[cfg_attr(feature = "serde", serde(skip))]
    pixels_per_point: Option<f32>,

    #[cfg_attr(feature = "serde", serde(skip))]
    frame_history: crate::frame_history::FrameHistory,

    egui_windows: EguiWindows,
}

impl BackendPanel {
    pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        match self.run_mode {
            RunMode::Continuous => {
                // Tell the backend to repaint as soon as possible
                ctx.request_repaint();
            }
            RunMode::Reactive => {
                // let the computer rest for a bit
            }
        }
    }

    pub fn end_of_frame(&mut self, ctx: &egui::Context) {
        self.egui_windows.windows(ctx);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::trace!(ui);

        self.run_mode_ui(ui);

        ui.separator();

        self.frame_history.ui(ui);

        ui.separator();

        self.egui_windows.checkboxes(ui);

        ui.separator();

        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.separator();
            if ui.button("Quit").clicked() {
                frame.close();
            }
        }

        if cfg!(debug_assertions) && cfg!(target_arch = "wasm32") {
            ui.separator();
            // For testing panic handling on web:
            #[allow(clippy::manual_assert)]
            if ui.button("panic!()").clicked() {
                panic!("intentional panic!");
            }
        }

        ui.separator();

        self.integration_ui(ui, frame);
    }

    fn integration_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // On web, the browser controls `pixels_per_point`.
        let integration_controls_pixels_per_point = frame.is_web();
        if !integration_controls_pixels_per_point {
            self.pixels_per_point_ui(ui, &frame.info());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.horizontal(|ui| {
                {
                    let mut fullscreen = frame.info().window_info.fullscreen;
                    if ui
                        .checkbox(&mut fullscreen, "🗖 Fullscreen (F11)")
                        .on_hover_text("Fullscreen the window")
                        .changed()
                    {
                        frame.set_fullscreen(fullscreen);
                    }
                }
            });

            ui.button("Native window info (hover me)")
                .on_hover_ui(|ui| {
                    window_info_ui(ui, &frame.info().window_info);
                });
        }
    }

    fn pixels_per_point_ui(&mut self, ui: &mut egui::Ui, info: &eframe::IntegrationInfo) {
        let pixels_per_point = self
            .pixels_per_point
            .get_or_insert_with(|| ui.ctx().pixels_per_point());

        let mut reset = false;

        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 90.0;

            let response = ui
                .add(
                    egui::Slider::new(pixels_per_point, 0.5..=5.0)
                        .logarithmic(true)
                        .clamp_to_range(true)
                        .text("Scale"),
                )
                .on_hover_text("Physical pixels per point.");

            if response.drag_released() {
                // We wait until mouse release to activate:
                ui.ctx().set_pixels_per_point(*pixels_per_point);
                reset = true;
            } else if !response.is_pointer_button_down_on() {
                // When not dragging, show the current pixels_per_point so others can change it.
                reset = true;
            }

            if let Some(native_pixels_per_point) = info.native_pixels_per_point {
                let enabled = ui.ctx().pixels_per_point() != native_pixels_per_point;
                if ui
                    .add_enabled(enabled, egui::Button::new("Reset"))
                    .on_hover_text(format!(
                        "Reset scale to native value ({:.1})",
                        native_pixels_per_point
                    ))
                    .clicked()
                {
                    ui.ctx().set_pixels_per_point(native_pixels_per_point);
                }
            }
        });

        if reset {
            self.pixels_per_point = None;
        }
    }

    fn run_mode_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let run_mode = &mut self.run_mode;
            ui.label("Mode:");
            ui.radio_value(run_mode, RunMode::Reactive, "Reactive")
                .on_hover_text("Repaint when there are animations or input (e.g. mouse movement)");
            ui.radio_value(run_mode, RunMode::Continuous, "Continuous")
                .on_hover_text("Repaint everything each frame");
        });

        if self.run_mode == RunMode::Continuous {
            ui.label("Repainting the UI each frame.");
            ui.label(format!(
                "FPS: {:.1}",
                self.frame_history.fps()
            ));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn window_info_ui(ui: &mut egui::Ui, window_info: &eframe::WindowInfo) {
    let eframe::WindowInfo {
        position,
        fullscreen,
        minimized,
        maximized,
        focused,
        size,
        monitor_size,
    } = window_info;

    egui::Grid::new("window_info_grid")
        .num_columns(2)
        .show(ui, |ui| {
            if let Some(egui::Pos2 { x, y }) = position {
                ui.label("Position:");
                ui.monospace(format!("{x:.0}, {y:.0}"));
                ui.end_row();
            }

            ui.label("Fullscreen:");
            ui.label(fullscreen.to_string());
            ui.end_row();

            ui.label("Minimized:");
            ui.label(minimized.to_string());
            ui.end_row();

            ui.label("Maximized:");
            ui.label(maximized.to_string());
            ui.end_row();

            ui.label("Focused:");
            ui.label(focused.to_string());
            ui.end_row();

            ui.label("Window size:");
            ui.monospace(format!("{x:.0} x {y:.0}", x = size.x, y = size.y));
            ui.end_row();

            if let Some(egui::Vec2 { x, y }) = monitor_size {
                ui.label("Monitor size:");
                ui.monospace(format!("{x:.0} x {y:.0}"));
                ui.end_row();
            }
        });
}

// ----------------------------------------------------------------------------

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct EguiWindows {
    // egui stuff:
    settings: bool,
    inspection: bool,
    memory: bool,
    output_events: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    output_event_history: std::collections::VecDeque<egui::output::OutputEvent>,
}

impl Default for EguiWindows {
    fn default() -> Self {
        EguiWindows::none()
    }
}

impl EguiWindows {
    fn none() -> Self {
        Self {
            settings: false,
            inspection: false,
            memory: false,
            output_events: false,
            output_event_history: Default::default(),
        }
    }

    fn checkboxes(&mut self, ui: &mut egui::Ui) {
        let Self {
            settings,
            inspection,
            memory,
            output_events,
            output_event_history: _,
        } = self;

        ui.checkbox(settings, "🔧 Settings");
        ui.checkbox(inspection, "🔍 Inspection");
        ui.checkbox(memory, "📝 Memory");
        ui.checkbox(output_events, "📤 Output Events");
    }

    fn windows(&mut self, ctx: &egui::Context) {
        let Self {
            settings,
            inspection,
            memory,
            output_events,
            output_event_history,
        } = self;

        ctx.output(|o| {
            for event in &o.events {
                output_event_history.push_back(event.clone());
            }
        });
        while output_event_history.len() > 1000 {
            output_event_history.pop_front();
        }

        egui::Window::new("🔧 Settings")
            .open(settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });

        egui::Window::new("📤 Output Events")
            .open(output_events)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    "Recent output events from egui. \
            These are emitted when you interact with widgets, or move focus between them with TAB. \
            They can be hooked up to a screen reader on supported platforms.",
                );

                ui.separator();

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for event in output_event_history {
                            ui.label(format!("{:?}", event));
                        }
                    });
            });
    }
}

// ----------------------------------------------------------------------------
