use regex::Regex;
use std::path::PathBuf;

use super::VolumeDataFileType;

#[derive(Default)]
pub struct ImportItem {
    file_type: Option<VolumeDataFileType>,
    file_size: Option<u64>,
    pub path: Option<PathBuf>,
    // preview_image_path: Option<PathBuf>,
    bits: Option<u8>,
    // endianness: Option<Endianness>,
    pub dimensions: Option<(u32, u32, u32)>,
    pub spacing: Option<(f32, f32, f32)>,
    pub data: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct Importer {
    visible: bool,
    loading: bool,
    pub show_drag_and_drop: bool,
    pub new_data_available: bool,
    pub item: ImportItem,
}

impl Importer {
    #[cfg(not(target_arch = "wasm32"))]
    fn get_file_as_byte_vec(file_path: PathBuf) -> Vec<u8> {
        use std::fs::File;
        use std::io::Read;
        let mut file = File::open(&file_path).expect("no file found");
        let metadata = std::fs::metadata(&file_path).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read_exact(&mut buffer).expect("buffer overflow");

        buffer
    }
    pub fn load_dialog(&mut self, file_type: VolumeDataFileType) {
        self.item.file_type = Some(file_type);

        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.item.path.is_none() {
                self.item.path = rfd::FileDialog::new().pick_file();
            }

            // abort import when FileDialog was cloaed with "Cancel" instead of "Open"
            if self.item.path.is_none() {
                return;
            }

            if let Some(path) = self.item.path.as_ref() {
                self.item.file_size = Some(path.metadata().unwrap().len());
            }
        }

        Self::prefill_metadata_from_file_name(self);
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        match self.item.file_type {
            None => {}
            Some(ref file_type) => match file_type {
                VolumeDataFileType::RAW3D => Self::show_metadata_dialog_raw3d(self, ctx),
            },
        }
    }
    fn prefill_metadata_from_file_name(&mut self) {
        let bits_regex = Regex::new(r"(?i)(\d+)[\._-]?bit").unwrap();
        let dimensions_regex = Regex::new(r"(?i)(\d+)\D(\d+)\D(\d+)").unwrap();
        let spacing_regex = Regex::new(r"(?i)(\d+\.\d+)x(\d+\.\d+)x(\d+\.\d+)").unwrap();

        let filename = self
            .item
            .path
            .as_ref()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        if let Some(captures) = bits_regex.captures(filename) {
            let bits: u8 = captures.get(1).unwrap().as_str().parse().unwrap();
            self.item.bits = Some(bits);
        } else {
            self.item.bits = Some(16);
        }

        if let Some(captures) = dimensions_regex.captures(filename) {
            let dim1: u32 = captures.get(1).unwrap().as_str().parse().unwrap();
            let dim2: u32 = captures.get(2).unwrap().as_str().parse().unwrap();
            let dim3: u32 = captures.get(3).unwrap().as_str().parse().unwrap();
            self.item.dimensions = Some((dim1, dim2, dim3));
        } else {
            self.item.dimensions = Some((1, 1, 1));
        }

        if let Some(captures) = spacing_regex.captures(filename) {
            let spacing1: f32 = captures.get(1).unwrap().as_str().parse().unwrap();
            let spacing2: f32 = captures.get(2).unwrap().as_str().parse().unwrap();
            let spacing3: f32 = captures.get(3).unwrap().as_str().parse().unwrap();
            self.item.spacing = Some((spacing1, spacing2, spacing3));
        } else {
            self.item.spacing = Some((1.0, 1.0, 1.0));
        }
    }
    fn show_metadata_dialog_raw3d(&mut self, ctx: &egui::Context) {
        let mut visible = self.visible;

        // create temporary variables for getting UI inputs and set default values
        let mut bits: u32 = if self.item.bits.is_some() {
            self.item.bits.unwrap() as u32
        } else {
            16
        };
        let mut dimension_x: u32 = if self.item.dimensions.is_some() {
            self.item.dimensions.unwrap().0
        } else {
            1
        };
        let mut dimension_y: u32 = if self.item.dimensions.is_some() {
            self.item.dimensions.unwrap().1
        } else {
            1
        };
        let mut dimension_z: u32 = if self.item.dimensions.is_some() {
            self.item.dimensions.unwrap().2
        } else {
            1
        };
        let mut spacing_x: f32 = if self.item.dimensions.is_some() {
            self.item.spacing.unwrap().0
        } else {
            1.0
        };
        let mut spacing_y: f32 = if self.item.dimensions.is_some() {
            self.item.spacing.unwrap().1
        } else {
            1.0
        };
        let mut spacing_z: f32 = if self.item.dimensions.is_some() {
            self.item.spacing.unwrap().2
        } else {
            1.0
        };

        egui::Window::new("Import raw 3D volume data")
            .open(&mut visible)
            .resizable(false)
            .collapsible(false)
            .enabled(!self.loading)
            .show(ctx, |ui| {
                ui.label("Review and add missing metadata to continue:");

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        ui.label(self.item.path.as_ref().unwrap().to_str().unwrap());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bits per Voxel:");
                        ui.add(egui::DragValue::new(&mut bits));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Endianness:");
                        ui.label("Little Endian (most common)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Dimensions in Pixel (x,y,z):");
                        ui.add(egui::DragValue::new(&mut dimension_x));
                        ui.add(egui::DragValue::new(&mut dimension_y));
                        ui.add(egui::DragValue::new(&mut dimension_z));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Spacing in mm (x,y,z):");
                        ui.add(egui::DragValue::new(&mut spacing_x));
                        ui.add(egui::DragValue::new(&mut spacing_y));
                        ui.add(egui::DragValue::new(&mut spacing_z));
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        self.loading = true;

                        // TODO: Make asynchronous with pollster
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            self.item.data = Some(Self::get_file_as_byte_vec(
                                self.item.path.as_ref().unwrap().to_path_buf(),
                            ));
                        }

                        self.loading = false;
                        self.visible = false;
                        self.new_data_available = true;
                    }
                    if self.loading {
                        ui.label("Loading...");
                        ui.add(egui::Spinner::new());
                    }
                });
            });
        self.visible &= visible;
        self.item.bits = Some(bits as u8);
        self.item.dimensions = Some((dimension_x, dimension_y, dimension_z));
        self.item.spacing = Some((spacing_x, spacing_y, spacing_z));
    }
}
