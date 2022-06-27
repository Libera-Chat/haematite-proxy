use super::*;

use std::{
    net::{
        SocketAddr,
        TcpListener,
    },
};

pub struct Listener
{
    address: SocketAddr,
    uplink: Uplink,
}

impl Listener
{
    pub fn new(config: Config) -> Self
    {
        Self {
            address: config.ro_listen_address.clone(),
            uplink: Uplink::new(config),
        }
    }

    pub fn run(&self) -> std::io::Result<()>
    {
        let listener = TcpListener::bind(self.address)?;

        for stream in listener.incoming()
        {
            let stream = stream?;

            self.uplink.run(stream)?;
        }

        Ok(())
    }
}