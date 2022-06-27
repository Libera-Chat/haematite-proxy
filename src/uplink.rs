use super::*;

use std::{
    net::{
        IpAddr,
        SocketAddr,
        TcpStream,
    },
    io::{
        Write,
        BufRead,
        BufReader,
    },
};

pub struct Uplink
{
    local_address: Option<IpAddr>,
    remote_address: SocketAddr,
    password: String,

    server_name: String,
    sid: String,
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
            local_address: config.uplink_local_address,
            remote_address: config.uplink_remote_address,
            password: config.uplink_password,

            server_name: config.server_name,
            sid: config.sid,
        }
    }

    pub fn run(&self, mut local_stream: impl Write) -> std::io::Result<()>
    {
        let mut uplink_stream = TcpStream::connect(self.remote_address)?;
        let mut uplink_reader = BufReader::new(uplink_stream.try_clone()?);

        uplink_stream.write_fmt(format_args!("PASS {} TS 6 :{}\r\n", self.password, self.sid))?;
        uplink_stream.write_all(b"CAPAB :BAN CHW CLUSTER ECHO ENCAP EOPMOD EUID EX IE KLN KNOCK MLOCK QS RSFNC SAVE SERVICES TB UNKLN\r\n")?;
        // SERVER args: <server name> <hop count> :<description>
        uplink_stream.write_fmt(format_args!("SERVER {} 1 :(H) haematite proxy service\r\n", self.server_name))?;
        // SVINFO args: <current TS version> <min supported TS version> 0 :<current unix time>
        uplink_stream.write_fmt(format_args!("SVINFO 6 6 0 :{}\r\n", current_time()))?;
        // Emulate EOB in both directions. We won't ever have anything to burst apart from the SERVER/SVINFO above.
        uplink_stream.write_fmt(format_args!("PING :{}", self.sid))?;

        let mut line_buffer = Vec::new();

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
                let pong = format!(":{} PONG {} :{}\r\n", self.sid, self.server_name, from_server);
                uplink_stream.write_all(pong.as_bytes())?;
            }

            local_stream.write_all(&line_buffer)?;
            line_buffer.clear();
        }

        Ok(())
    }
}
