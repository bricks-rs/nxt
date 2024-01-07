use nxt::{sensor::*, *};

#[tokio::main]
async fn main() -> nxt::Result<()> {
    let nxt = Nxt::first_usb().await?;

    println!("Set input mode");
    nxt.set_input_mode(InPort::S1, SensorType::Switch, SensorMode::Bool)
        .await?;

    println!("Start polling");

    loop {
        let val = nxt.get_input_values(InPort::S1).await?;
        println!("{val:?}");

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
