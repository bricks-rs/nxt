use eframe::egui;
use nxtusb::{motor::*, sensor::*, system::*, *};
use std::{sync::mpsc, time::Duration};
use tokio::runtime::Runtime;

const POLL_DELAY: Duration = Duration::from_millis(300);
const DISPLAY_PX_SCALE: usize = 4;

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
    rt: Runtime,
}

struct Motor {
    port: OutPort,
    power: i8,
}

enum Message {
    Sensors(Vec<InputValues>),
    Display(Box<DisplayRaster>),
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
            rt: Runtime::new().unwrap(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(message) = self.sensor_poll_handle.recv() {
                match message {
                    Message::Sensors(values) => self.sensors = values,
                    Message::Display(raster) => self.display = Some(*raster),
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
                    let all = self.rt.block_on(Nxt::all_usb());
                    match all {
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
                motor_ui(ui, &self.rt, nxt, &mut self.motors);
                ui.separator();
                sensor_ui(ui, &self.rt, nxt, &mut self.sensors);
                if let Some(display) = &self.display {
                    ui.separator();
                    display_ui(ui, display);
                }
            }
        });
    }
}

fn motor_ui(
    ui: &mut egui::Ui,
    rt: &Runtime,
    nxt: &Nxt,
    motors: &mut Vec<Motor>,
) {
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
                rt.block_on(nxt.set_output_state(
                    mot.port,
                    mot.power,
                    OutMode::ON | OutMode::REGULATED,
                    RegulationMode::Speed,
                    0,
                    RunState::Running,
                    RUN_FOREVER,
                ))
                .unwrap();
            }
        });
    }
}

fn sensor_ui(
    ui: &mut egui::Ui,
    rt: &Runtime,
    nxt: &Nxt,
    sensors: &mut Vec<InputValues>,
) {
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
                rt.block_on(nxt.set_input_mode(
                    sens.port,
                    sens.sensor_type,
                    sens.sensor_mode,
                ))
                .unwrap();
            }
        });
    }
}

fn display_ui(ui: &mut egui::Ui, display: &DisplayRaster) {
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let position = ui.available_rect_before_wrap().min.to_vec2();

        #[allow(clippy::needless_range_loop)]
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                let x1 = row * DISPLAY_PX_SCALE;
                let y1 = col * DISPLAY_PX_SCALE;
                let x2 = x1 + DISPLAY_PX_SCALE;
                let y2 = y1 + DISPLAY_PX_SCALE;
                let fill = if display[row][col] == 0 { 0xff } else { 0x00 };
                ui.painter().rect_filled(
                    egui::Rect::from_two_pos(
                        egui::pos2(y1 as f32, x1 as f32),
                        egui::pos2(y2 as f32, x2 as f32),
                    )
                    .translate(position),
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgb(fill, fill, fill),
                );
            }
        }
    });
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        loop {
            if let Ok(new) = nxt_rx.try_recv() {
                nxt = new;
                println!("Change nxt to {nxt:?}");
            }

            if let Some(nxt) = &nxt {
                let mut values = Vec::with_capacity(4);
                for port in InPort::iter() {
                    values
                        .push(rt.block_on(nxt.get_input_values(port)).unwrap());
                }
                if values != old_values {
                    old_values = values.clone();
                    val_tx.send(Message::Sensors(values)).unwrap();
                    ctx.request_repaint();
                }

                let screen = rt.block_on(nxt.get_display_data()).unwrap();
                if screen != old_screen {
                    val_tx
                        .send(Message::Display(Box::new(
                            display_data_to_raster(&screen),
                        )))
                        .unwrap();
                    old_screen = screen;
                    ctx.request_repaint();
                }
            }
            std::thread::sleep(POLL_DELAY);
        }
    }
}
