use gilrs::{Button, Event, EventType::AxisChanged, Gilrs};
use std::time::{Duration, Instant};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

const MIN_TIME_BETWEEN_UPDATES: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct Robot {}

impl Robot {
    pub fn set_speed(&mut self, speed: f32) {
        let speed = (speed * 100.0).trunc() as i8;
        dbg!(speed);
    }
    pub fn set_steering(&mut self, steering: f32) {}
    pub fn changed(&self) -> bool {
        true
    }
    pub fn send(&mut self) -> Result {
        Ok(())
    }
}

fn main() -> Result {
    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;
    let mut robot = Robot {};
    let mut last_update = Instant::now();

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            #[cfg(debug_assertions)]
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
            if let AxisChanged(axis, pos, _) = event {
                match axis {
                    gilrs::Axis::LeftStickX => robot.set_steering(pos),
                    gilrs::Axis::LeftStickY => robot.set_speed(pos),
                    gilrs::Axis::LeftZ => (),
                    gilrs::Axis::RightStickX => (),
                    gilrs::Axis::RightStickY => (),
                    gilrs::Axis::RightZ => (),
                    gilrs::Axis::DPadX => (),
                    gilrs::Axis::DPadY => (),
                    gilrs::Axis::Unknown => (),
                }
            }
        }

        if last_update.elapsed() > MIN_TIME_BETWEEN_UPDATES {
            last_update = Instant::now();
            if robot.changed() {
                println!("{robot:?}");
                robot.send()?;
            }
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
                break;
            }
        }
    }

    Ok(())
}
