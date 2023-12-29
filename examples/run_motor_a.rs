use nxtusb::{motor::*, *};

const POWER: i8 = 80;

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first()?;

    println!("Running motor A at {POWER}");
    nxt.set_output_state(
        OutPort::A,
        POWER,
        OutMode::ON | OutMode::REGULATED,
        RegulationMode::Speed,
        0,
        RunState::Running,
        RUN_FOREVER,
    )?;

    println!("beep");
    nxt.play_tone(440, 1000)?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    println!("Stop");
    nxt.set_output_state(
        OutPort::A,
        0,
        OutMode::IDLE,
        RegulationMode::default(),
        0,
        RunState::Running,
        RUN_FOREVER,
    )?;

    let bat = nxt.get_battery_level()?;
    println!("Battery level is {bat} mV");

    Ok(())
}
