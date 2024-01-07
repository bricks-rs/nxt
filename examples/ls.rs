use nxtusb::*;

#[tokio::main]
async fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first_usb().await?;

    println!("List files");

    let mut handle = nxt.file_find_first(".").await?;

    loop {
        println!("{handle:?}");
        handle = nxt.file_find_next(&handle).await?;
    }
}
