use nxtusb::{system::*, *};

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first_usb()?;

    println!("Read display");

    let screen = nxt.get_display_data()?;

    dbg!(screen);

    let raster = display_data_to_raster(&screen);
    let rendered = raster_to_string(&raster);
    println!("{rendered}");

    Ok(())
}
