
mod config;
use config::Config;

mod listener;
use listener::Listener;

mod uplink;
use uplink::*;

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let mut args = std::env::args();
    args.next(); // skip the executable name
    let arg = args.next().unwrap();

    let config = Config::load_from_file(arg)?;

    println!("{:?}", config);

    let listener = Listener::new(config);

    listener.run()?;

    Ok(())
}
