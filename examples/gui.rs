use eframe::egui;
use nxtusb::{motor::*, sensor::*, system::*, *};
use std::{sync::mpsc, time::Duration};

const POLL_DELAY: Duration = Duration::from_millis(300);

fn main() {
    let opts = eframe::NativeOptions::default();
    eframe::run_native("NXT GUI", opts, Box::new(|cc| Box::new(App::new(cc))))
        .unwrap();
}

struct App {
    nxt_available: Vec<Nxt>,
    nxt_selected: Option<usize>,
    motors: Vec<Motor>,
    sensors: Vec<InputValues>,
    sensor_poll_handle: SensorPollHandle,
    display: Option<DisplayRaster>,
}

struct Motor {
    port: OutPort,
    power: i8,
}

enum Message {
    Sensors(Vec<InputValues>),
    Display(DisplayRaster),
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let spacing = egui::style::Spacing {
            slider_width: 200.0,
            ..Default::default()
        };
        cc.egui_ctx.set_style(egui::style::Style {
            spacing,
            ..Default::default()
        });

        Self {
            nxt_available: Vec::new(),
            nxt_selected: None,
            motors: [OutPort::A, OutPort::B, OutPort::C]
                .iter()
                .map(|&port| Motor { port, power: 0 })
                .collect(),
            sensors: Vec::new(),
            sensor_poll_handle: SensorPollHandle::new(cc.egui_ctx.clone()),
            display: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(message) = self.sensor_poll_handle.recv() {
                match message {
                    Message::Sensors(values) => self.sensors = values,
                    Message::Display(raster) => self.display = Some(raster),
                }
            }

            ui.heading("NXT GUI");

            ui.horizontal(|ui| {
                let old = self.nxt_selected;
                ui.label("Selected brick:");
                egui::ComboBox::from_id_source("nxt")
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

                if self.nxt_selected != old {
                    let nxt = self
                        .nxt_selected
                        .and_then(|idx| self.nxt_available.get(idx));
                    self.sensor_poll_handle.send(nxt.cloned());
                }
            });

            if let Some(nxt) = self
                .nxt_selected
                .and_then(|idx| self.nxt_available.get(idx))
            {
                ui.separator();
                motor_ui(ui, nxt, &mut self.motors);
                ui.separator();
                sensor_ui(ui, nxt, &mut self.sensors);
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
                    .suffix("%")
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

fn sensor_ui(ui: &mut egui::Ui, nxt: &Nxt, sensors: &mut Vec<InputValues>) {
    for sens in sensors {
        ui.horizontal(|ui| {
            let old_typ = sens.sensor_type;
            let old_mode = sens.sensor_mode;

            ui.label(format!("{:?}", sens.port));

            ui.label("Type:");
            egui::ComboBox::from_id_source((sens.port, "sensor_type"))
                .selected_text(format!("{:?}", sens.sensor_type))
                .show_ui(ui, |ui| {
                    for opt in SensorType::iter() {
                        ui.selectable_value(
                            &mut sens.sensor_type,
                            opt,
                            format!("{opt:?}"),
                        );
                    }
                });

            ui.label("Mode:");
            egui::ComboBox::from_id_source((sens.port, "sensor_mode"))
                .selected_text(format!("{:?}", sens.sensor_mode))
                .show_ui(ui, |ui| {
                    for opt in SensorMode::iter() {
                        ui.selectable_value(
                            &mut sens.sensor_mode,
                            opt,
                            format!("{opt:?}"),
                        );
                    }
                });

            ui.label(format!("Value: {sens}"));

            if sens.sensor_type != old_typ || sens.sensor_mode != old_mode {
                nxt.set_input_mode(
                    sens.port,
                    sens.sensor_type,
                    sens.sensor_mode,
                )
                .unwrap();
            }
        });
    }
}

struct SensorPollHandle {
    val_rx: mpsc::Receiver<Message>,
    nxt_tx: mpsc::Sender<Option<Nxt>>,
}

impl SensorPollHandle {
    pub fn new(ctx: egui::Context) -> Self {
        let (val_tx, val_rx) = mpsc::channel();
        let (nxt_tx, nxt_rx) = mpsc::channel();

        std::thread::spawn(move || Self::thread_loop(ctx, val_tx, nxt_rx));

        Self { val_rx, nxt_tx }
    }

    pub fn recv(&mut self) -> Option<Message> {
        self.val_rx.try_recv().ok()
    }

    pub fn send(&self, nxt: Option<Nxt>) {
        self.nxt_tx.send(nxt).unwrap();
    }

    fn thread_loop(
        ctx: egui::Context,
        val_tx: mpsc::Sender<Message>,
        nxt_rx: mpsc::Receiver<Option<Nxt>>,
    ) {
        let mut nxt = None;
        let mut old_values = Vec::new();
        let mut old_screen = [0u8; DISPLAY_DATA_LEN];
        loop {
            if let Ok(new) = nxt_rx.try_recv() {
                nxt = new;
                println!("Change nxt to {nxt:?}");
            }

            if let Some(nxt) = &nxt {
                let mut values = Vec::with_capacity(4);
                for port in InPort::iter() {
                    values.push(nxt.get_input_values(port).unwrap());
                }
                if values != old_values {
                    old_values = values.clone();
                    val_tx.send(Message::Sensors(values)).unwrap();
                    ctx.request_repaint();
                }

                let screen = nxt.get_display_data().unwrap();
                if screen != old_screen {
                    val_tx
                        .send(Message::Display(display_data_to_raster(&screen)))
                        .unwrap();
                    old_screen = screen;
                }
            }
            std::thread::sleep(POLL_DELAY);
        }
    }
}
