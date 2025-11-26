use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([240.0, 160.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Color viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<ColorViewerApp>::default())
        }),
    )
}

struct ColorViewerApp {
    color_count: usize,
    seed: f32,
    step: f32,
}

impl Default for ColorViewerApp {
    fn default() -> Self {
        Self {
            color_count: 20,
            seed: rand::random(),
            step: 1.618033988749895,
        }
    }
}

impl eframe::App for ColorViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let (num_rows, num_cols) = rows_and_cols_for_count(self.color_count);
                let mut angle = self.seed % 1.;
                for row in 0..num_rows {
                    ui.horizontal(|ui| {
                        for col in 0..num_cols {
                            let cell_idx = col + row * num_rows;
                            if cell_idx < self.color_count {
                                let img = generate_image(angle);
                                ui.image(img);
                                angle = (angle + self.step) % 1.;
                            }
                        }
                    });
                }
            });
            if ui.button("Reseed").clicked() {
                self.reseed();
            }
            ui.horizontal(|ui| {
                ui.label("Step by:");
                let mut changed = false;
                if ui.button("phi").clicked() {
                    self.step = 1.618033988749895;
                    changed = true;
                }
                if ui.button("sqrt2").clicked() {
                    self.step = std::f32::consts::SQRT_2;
                    changed = true;
                }
                if ui.button("pi").clicked() {
                    self.step = std::f32::consts::PI;
                    changed = true;
                }
                if ui.button("e").clicked() {
                    self.step = std::f32::consts::E;
                    changed = true;
                }
                if ui.button("rand").clicked() {
                    self.step = rand::random();
                    changed = true;
                }
                if changed {
                    log::info!("New step: {}", self.step);
                }
            })
        });
    }
}

impl ColorViewerApp {
    fn reseed(&mut self) {
        self.seed = rand::random();
    }
}

fn generate_image(angle: f32) -> egui::ImageSource<'static> {
    let image = image::ImageBuffer::from_pixel(256, 256, rgb_for_angle(angle));
    let mut buffer = vec![0; 1 << 16];
    let write_len = {
        let mut writer = std::io::Cursor::new(buffer.as_mut_slice());
        image
            .write_to(&mut writer, image::ImageFormat::Jpeg)
            .unwrap();
        writer.position() as usize
    };
    buffer.drain(write_len..);
    egui::ImageSource::Bytes {
        uri: format!("bytes://texture_{angle:.02}.jpeg").into(),
        bytes: buffer.into(),
    }
}

fn rgb_for_angle(angle: f32) -> image::Rgb<u8> {
    let h_prime = angle * 6.;
    let sector = h_prime as u8;
    let x = ((1. - ((h_prime % 2.) - 1.).abs()) * 255.) as u8;
    match sector {
        0 => image::Rgb([255, x, 0]),
        1 => image::Rgb([x, 255, 0]),
        2 => image::Rgb([0, 255, x]),
        3 => image::Rgb([0, x, 255]),
        4 => image::Rgb([x, 0, 255]),
        5 => image::Rgb([255, 0, x]),
        _ => unreachable!(),
    }
}

fn rows_and_cols_for_count(count: usize) -> (usize, usize) {
    (1..=count.div_ceil(2))
        .map(|a| (a, count.div_ceil(a)))
        .min_by_key(|&(a, b)| {
            const EXCESS_CELL_PENALTY: usize = 5;
            const ASPECT_RATIO_2_PENALTY: usize = 1;

            let excess_cells = a * b - count;
            let aspect_ratio = a.abs_diff(b);

            EXCESS_CELL_PENALTY.strict_mul(excess_cells).strict_add(
                aspect_ratio
                    .strict_pow(2)
                    .strict_mul(ASPECT_RATIO_2_PENALTY),
            )
        })
        .unwrap()
}
