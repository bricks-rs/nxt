use nxtusb::{system::*, *};

#[tokio::main]
async fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first_usb().await?;

    println!("Read display");

    let screen = nxt.get_display_data().await?;

    dbg!(screen);

    let raster = display_data_to_raster(&screen);
    let rendered = raster_to_string(&raster);
    println!("{rendered}");

    Ok(())
}
