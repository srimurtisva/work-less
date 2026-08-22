// app/src/main.rs
// use iced_client;

fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt::init();

    #[cfg(feature = "iced")]
    {
        iced_client::run()?;
    }

    #[cfg(not(feature = "iced"))]
    {
        println!("No GUI feature enabled!");
    }

    Ok(())
}
