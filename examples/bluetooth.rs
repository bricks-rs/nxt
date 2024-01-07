#[tokio::main]
async fn main() -> nxt::Result<()> {
    tracing_subscriber::fmt::init();

    let nxt = nxt::Bluetooth::wait_for_nxt().await?;

    let info = nxt.get_device_info().await?;
    dbg!(info);

    Ok(())
}
