use nxtusb::{sensor::*, *};

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first()?;

    println!("Set input mode");
    nxt.set_input_mode(InPort::S1, SensorType::Switch, SensorMode::Bool)?;

    println!("Start polling");

    loop {
        let val = nxt.get_input_values(InPort::S1)?;
        println!("{val:?}");

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
