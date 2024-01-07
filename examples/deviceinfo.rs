use nxt::Nxt;

#[tokio::main]
async fn main() -> nxt::Result<()> {
    let nxt = Nxt::all_usb().await?;

    println!("Found {} NXT bricks", nxt.len());

    for (idx, nxt) in nxt.iter().enumerate() {
        println!("## Brick {idx}:");

        let bat = nxt.get_battery_level().await?;
        println!("Battery level is {bat} mV");

        let info = nxt.get_device_info().await?;
        println!("Device info:\n{info:?}");
        println!(
            "BT address: {}",
            info.bt_addr
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .reduce(|acc, ele| format!("{acc}:{ele}"))
                .unwrap_or_default()
        );

        let versions = nxt.get_firmware_version().await?;
        println!("Versions:\n{versions:?}");
    }

    Ok(())
}
