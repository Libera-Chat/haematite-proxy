use std::{
    io::{
        BufReader,
        prelude::*,
    },
    fs::File,
    path::Path,
    net::{
        IpAddr,
        SocketAddr,
    },
};

#[derive(Debug)]
pub struct Config
{
    pub server_name: String,
    pub sid: String,

    pub uplink_remote_address: String,
    pub uplink_remote_name: Option<String>,
    pub uplink_remote_port: u16,
    pub uplink_local_address: Option<IpAddr>,
    pub uplink_password: String,
    pub uplink_ca: Vec<u8>,

    pub ro_listen_address: SocketAddr,
    pub ro_cert: Vec<u8>,
    pub ro_key: Vec<u8>,

    pub auth_ca: Vec<u8>,
    pub auth_fingerprint: String,
}

#[derive(Debug)]
pub enum ConfigError
{
    InvalidSid,
    InvalidServerName,
    MissingField(&'static str),
    InvalidField(String),
    IoError(std::io::Error),
    SyntaxError(String),
}

impl From<std::io::Error> for ConfigError
{
    fn from(e: std::io::Error) -> Self
    {
        Self::IoError(e)
    }
}

impl std::fmt::Display for ConfigError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for ConfigError { }


impl Config
{
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>>
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut server_name = Err(ConfigError::MissingField("server_name"));
        let mut sid = Err(ConfigError::MissingField("sid"));

        let mut uplink_remote_address = Err(ConfigError::MissingField("uplink_remote_address"));
        let mut uplink_remote_name = None;
        let mut uplink_remote_port = Err(ConfigError::MissingField("uplink_remote_port"));
        let mut uplink_local_address = None;
        let mut uplink_password = Err(ConfigError::MissingField("uplink_password"));
        let mut uplink_ca = Err(ConfigError::MissingField("uplink_ca"));

        let mut ro_listen_address = Err(ConfigError::MissingField("ro_listen_address"));
        let mut ro_cert = Err(ConfigError::MissingField("ro_cert"));
        let mut ro_key = Err(ConfigError::MissingField("ro_key"));

        let mut auth_ca = Err(ConfigError::MissingField("auth_ca"));
        let mut auth_fingerprint = Err(ConfigError::MissingField("auth_fingerprint"));

        for line in reader.lines()
        {
            let line = line?;

            if line.trim().len() == 0
            {
                continue;
            }

            let tokens = line.split('=').collect::<Vec<_>>();

            if tokens.len() != 2
            {
                return Err(ConfigError::SyntaxError(line).into());
            }

            let setting = tokens[0].trim();
            let value = tokens[1].trim().to_string();

            match setting
            {
                "server_name" => server_name = Ok(value),
                "sid" => sid = Ok(value),
                "uplink_remote_address" => uplink_remote_address = Ok(value),
                "uplink_remote_name" => uplink_remote_name = Some(value),
                "uplink_remote_port" => uplink_remote_port = Ok(value.parse()?),
                "uplink_local_address" => uplink_local_address = Some(value),
                "uplink_password" => uplink_password = Ok(value),
                "uplink_ca" => uplink_ca = std::fs::read(value).map_err(Into::into),

                "ro_listen_address" => ro_listen_address = Ok(value),
                "ro_cert" => ro_cert = std::fs::read(value).map_err(Into::into),
                "ro_key" => ro_key = std::fs::read(value).map_err(Into::into),

                "auth_ca" => auth_ca = std::fs::read(value).map_err(Into::into),
                "auth_fingerprint" => auth_fingerprint = Ok(value),

                _ => return Err(ConfigError::InvalidField(setting.to_string()).into()),
            }
        }

        Ok(Self {
            server_name: server_name?,
            sid: sid?,

            uplink_remote_address: uplink_remote_address?,
            uplink_remote_name,
            uplink_remote_port: uplink_remote_port?,
            uplink_local_address: uplink_local_address.map(|s| s.parse().unwrap()),
            uplink_password: uplink_password?,
            uplink_ca: uplink_ca?,

            ro_listen_address: ro_listen_address?.parse()?,
            ro_cert: ro_cert?,
            ro_key: ro_key?,

            auth_ca: auth_ca?,
            auth_fingerprint: auth_fingerprint?,
        }.validate()?)
    }

    fn validate(self) -> Result<Self, ConfigError>
    {
        let sid = self.sid.as_bytes();

        if sid.len() != 3 ||
            ! sid[0].is_ascii_digit() ||
            ! (sid[1].is_ascii_uppercase() || sid[1].is_ascii_digit()) ||
            ! (sid[2].is_ascii_uppercase() || sid[2].is_ascii_digit())
        {
            return Err(ConfigError::InvalidSid);
        }

        if ! self.server_name.chars().all(|c| c.is_ascii_alphabetic() || c == '.')
        {
            return Err(ConfigError::InvalidServerName);
        }

        Ok(self)
    }
}