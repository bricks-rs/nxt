use nxtusb::Nxt;

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first()?;

    let bat = nxt.get_battery_level()?;
    println!("Battery level is {bat} mV");

    Ok(())
}
