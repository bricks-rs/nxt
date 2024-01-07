#[tokio::main]
async fn main() -> nxtusb::Result<()> {
    tracing_subscriber::fmt::init();

    let nxt = nxtusb::Bluetooth::wait_for_nxt().await?;

    let info = nxt.get_device_info().await?;
    dbg!(info);

    Ok(())
}
