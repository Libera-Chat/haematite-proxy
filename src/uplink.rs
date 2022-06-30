use super::*;

use std::{
    net::{
        IpAddr,
        TcpStream,
    },
    io::{
        Write,
        BufRead,
        BufReader,
    },
    sync::Arc,
};

pub struct Uplink
{
    _local_address: Option<IpAddr>,
    remote_address: String,
    remote_name: String,
    remote_port: u16,
    password: String,

    server_name: String,
    sid: String,

    ca: rustls::Certificate,
}

fn current_time() -> u64
{
    std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).expect("Couldn't get system time?").as_secs()
}

impl Uplink
{
    pub fn new(config: Config) -> Self
    {
        Self {
            _local_address: config.uplink_local_address,
            remote_name: config.uplink_remote_name.unwrap_or(config.uplink_remote_address.clone()),
            remote_address: config.uplink_remote_address,
            remote_port: config.uplink_remote_port,
            password: config.uplink_password,

            server_name: config.server_name,
            sid: config.sid,

            ca: rustls::Certificate(config.uplink_ca),
        }
    }

    pub fn run(&self, mut local_stream: impl Write) -> std::io::Result<()>
    {
        let mut uplink_tcpstream = TcpStream::connect((self.remote_address.as_str(), self.remote_port))?;

        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(&self.ca).expect("Error adding certificate to store");

        let client_config = Arc::new(rustls::ClientConfig::builder().with_safe_defaults()
                                                                    .with_root_certificates(root_store)
                                                                    .with_no_client_auth());

        let mut tls_conn = rustls::ClientConnection::new(client_config,
                                                     self.remote_name.as_str().try_into().expect("invalid remote server name")
                                                ).expect("failed to establish tls connection");

        let mut uplink_stream = rustls::Stream::new(&mut tls_conn, &mut uplink_tcpstream);

        uplink_stream.write_fmt(format_args!("PASS {} TS 6 :{}\r\n", self.password, self.sid))?;
        uplink_stream.write_all(b"CAPAB :BAN CHW CLUSTER ECHO ENCAP EOPMOD EUID EX IE KLN KNOCK MLOCK QS RSFNC SAVE SERVICES TB UNKLN\r\n")?;
        // SERVER args: <server name> <hop count> :<description>
        uplink_stream.write_fmt(format_args!("SERVER {} 1 :(H) haematite proxy service\r\n", self.server_name))?;
        // SVINFO args: <current TS version> <min supported TS version> 0 :<current unix time>
        uplink_stream.write_fmt(format_args!("SVINFO 6 6 0 :{}\r\n", current_time()))?;
        // Emulate EOB in both directions. We won't ever have anything to burst apart from the SERVER/SVINFO above.
        uplink_stream.write_fmt(format_args!("PING :{}\r\n", self.sid))?;

        let mut line_buffer = Vec::new();
        let mut uplink_reader = BufReader::new(uplink_stream);

        while let Ok(_len) = uplink_reader.read_until(b'\n', &mut line_buffer)
        {
            let first_six = &line_buffer[0..6];
            let ping = b"PING :";
            if line_buffer.len() >= 7 && first_six == ping
            {
                // We only need to implement enough to emulate EOB and avoid pinging out.
                // The ping we need to respond to for EOB is `PING :<sid>`, where <sid> is that of the
                // server we're linking to.
                let from_server = String::from_utf8_lossy(&line_buffer[6..]);
                let pong = format!(":{} PONG {} :{}\r\n", self.sid, self.server_name, from_server.trim());
                uplink_reader.get_mut().write_all(pong.as_bytes())?;
            }

            local_stream.write_all(&line_buffer)?;
            line_buffer.clear();
        }

        Ok(())
    }
}
