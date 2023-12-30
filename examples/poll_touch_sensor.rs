use nxtusb::{sensor::*, *};

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first()?;

    println!("Set input mode");
    nxt.set_input_mode(InPort::S1, SensorType::Switch, SensorMode::Raw)?;
    nxt.set_input_mode(InPort::S1, SensorType::Switch, SensorMode::Percent)?;

    println!("Start polling");

    // nxt.set_brick_name("a")?;
    // nxt.boot(true)?;

    // todo!();

    loop {
        let val = nxt.get_input_values(InPort::S1)?;
        println!("{val:?}");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
