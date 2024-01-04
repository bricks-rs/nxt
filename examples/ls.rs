use nxtusb::*;

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first_usb()?;

    println!("List files");

    let mut handle = nxt.file_find_first(".")?;

    loop {
        println!("{handle:?}");
        handle = nxt.file_find_next(&handle)?;
    }
}
