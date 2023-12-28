use nxtusb::Nxt;

fn main() -> nxtusb::Result<()> {
    let nxt = Nxt::first()?;

    dbg!(nxt);

    Ok(())
}
