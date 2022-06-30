use super::*;

use std::{
    net::{
        SocketAddr,
        TcpListener,
    },
    sync::Arc,
    io::Write,
};

use sha1::{
    Sha1,
    Digest,
};

pub struct Listener
{
    address: SocketAddr,
    cert: Vec<u8>,
    key: Vec<u8>,

    auth_ca: Vec<u8>,
    auth_fingerprint: String,

    uplink: Uplink,
}

impl Listener
{
    pub fn new(config: Config) -> Self
    {
        Self {
            address: config.ro_listen_address.clone(),
            cert: config.ro_cert.clone(),
            key: config.ro_key.clone(),
            auth_ca: config.auth_ca.clone(),
            auth_fingerprint: config.auth_fingerprint.clone(),
            uplink: Uplink::new(config),
        }
    }

    pub fn run(&self) -> std::io::Result<()>
    {
        let listener = TcpListener::bind(self.address)?;

        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(&rustls::Certificate(self.auth_ca.clone())).expect("Error adding auth ca to store");

        let server_config = Arc::new(rustls::ServerConfig::builder().with_safe_defaults()
                                                                    .with_client_cert_verifier(
                                                                        rustls::server::AllowAnyAuthenticatedClient::new(
                                                                            root_store
                                                                        )
                                                                    )
                                                                    .with_single_cert(
                                                                        vec!(rustls::Certificate(self.cert.clone())),
                                                                        rustls::PrivateKey(self.key.clone())
                                                                    )
                                                                    .expect("invalid tls server config"));

        while let Ok((mut stream, addr)) = listener.accept()
        {
            println!("Received proxy connection from {}", addr);

            let mut tls_conn = rustls::ServerConnection::new(Arc::clone(&server_config)).expect("failed to setup tls connection");
            let mut tls_stream = rustls::Stream::new(&mut tls_conn, &mut stream);

            // This is a bit of a hack: we need to complete TLS negotiation before the client's cert will be
            // available to read, and there's no method to do that explicitly. flush() will tell it to complete
            // any outstanding i/o including negotiation, though, without needing to write anything.
            if let Err(e) = tls_stream.flush()
            {
                eprintln!("Error on proxy connection: {}", e);
                continue;
            }

            match tls_stream.conn.peer_certificates()
            {
                None =>
                {
                    eprintln!("No peer certificates?");
                    continue;
                }
                Some(certs) =>
                {
                    let mut hasher = Sha1::new();
                    hasher.update(&certs[0].0);
                    let fingerprint = hex::encode(hasher.finalize());
                    if fingerprint != self.auth_fingerprint
                    {
                        eprintln!("Wrong client cert fingerprint: {} != {}", fingerprint, self.auth_fingerprint);
                        continue;
                    }
                }
            }

            if let Err(e) = self.uplink.run(tls_stream)
            {
                eprintln!("Error handling proxy connection: {}", e);
            }
        }

        Ok(())
    }
}