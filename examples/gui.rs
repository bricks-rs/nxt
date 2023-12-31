use eframe::egui;
use nxtusb::{motor::*, *};

fn main() {
    let opts = eframe::NativeOptions::default();
    eframe::run_native("NXT GUI", opts, Box::new(|cc| Box::new(App::new(cc))))
        .unwrap();
}

#[derive(Default)]
struct App {
    nxt_available: Vec<Nxt>,
    nxt_selected: Option<usize>,
    motors: Vec<Motor>,
}

struct Motor {
    port: OutPort,
    power: i8,
}

impl App {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            motors: [OutPort::A, OutPort::B, OutPort::C]
                .iter()
                .map(|&port| Motor { port, power: 0 })
                .collect(),
            ..Self::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NXT GUI");

            ui.horizontal(|ui| {
                ui.label("Selected brick:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.nxt_selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.nxt_selected,
                            None,
                            "None",
                        );
                        for (idx, nxt) in self.nxt_available.iter().enumerate()
                        {
                            ui.selectable_value(
                                &mut self.nxt_selected,
                                Some(idx),
                                nxt.name(),
                            );
                        }
                    });
                if ui.button("Refresh").clicked() {
                    self.nxt_selected = None;
                    self.nxt_available.clear();
                    match Nxt::all() {
                        Ok(avail) => self.nxt_available = avail,
                        Err(e) => println!("Error: {e}"),
                    }
                    if self.nxt_available.len() == 1 {
                        self.nxt_selected = Some(0);
                    }
                }
            });

            ui.separator();
            if let Some(nxt) = self
                .nxt_selected
                .and_then(|idx| self.nxt_available.get(idx))
            {
                motor_ui(ui, nxt, &mut self.motors);
            }
        });
    }
}

fn motor_ui(ui: &mut egui::Ui, nxt: &Nxt, motors: &mut Vec<Motor>) {
    for mot in motors {
        ui.horizontal(|ui| {
            let old = mot.power;
            ui.label(format!("{:?}", mot.port));
            ui.add(
                egui::Slider::new(&mut mot.power, -100..=100)
                    .text("Power")
                    .clamp_to_range(true),
            );
            if ui.button("Stop").clicked() {
                mot.power = 0;
            }

            if mot.power != old {
                // it has changed
                nxt.set_output_state(
                    mot.port,
                    mot.power,
                    OutMode::ON | OutMode::REGULATED,
                    RegulationMode::Speed,
                    0,
                    RunState::Running,
                    RUN_FOREVER,
                )
                .unwrap();
            }
        });
    }
}
