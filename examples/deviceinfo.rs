use nxtusb::Nxt;

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::all()?;

    println!("Found {} NXT bricks", nxt.len());

    for (idx, nxt) in nxt.iter().enumerate() {
        println!("## Brick {idx}:");

        let bat = nxt.get_battery_level()?;
        println!("Battery level is {bat} mV");

        let info = nxt.get_device_info()?;
        println!("Device info:\n{info:?}");

        let versions = nxt.get_firmware_version()?;
        println!("Versions:\n{versions:?}");
    }

    Ok(())
}
