use nxtusb::Nxt;

#[tokio::main]
async fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::all_usb().await?;

    println!("Found {} NXT bricks", nxt.len());

    for (idx, nxt) in nxt.iter().enumerate() {
        println!("## Brick {idx}:");

        let bat = nxt.get_battery_level().await?;
        println!("Battery level is {bat} mV");

        let info = nxt.get_device_info().await?;
        println!("Device info:\n{info:?}");

        let versions = nxt.get_firmware_version().await?;
        println!("Versions:\n{versions:?}");
    }

    Ok(())
}
