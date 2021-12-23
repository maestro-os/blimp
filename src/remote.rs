//! A remote is a remote host from which packages can be downloaded.

/// Structure representing a remote host.
pub struct Remote {
    /// The host's address and port (optional).
    host: String,
}

impl Remote {
    /// Creates a new instance.
    pub fn new(host: String) -> Self {
        Self{
            host,
        }
    }

    /// Returns the list of remote hosts.
    pub fn list() -> Vec<Self> {
        let v = Vec::new();
        // TODO Read from file

        v
    }

    /// Returns the host for the remote.
    pub fn get_host(&self) -> &String {
        &self.host
    }

    /// Returns the remote's motd.
    pub fn get_motd(&self) -> Result<String, ()> {
        let url = "https://".to_owned() + &self.host + "/motd";
        let resp = reqwest::blocking::get(url).ok().ok_or(())?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let motd = resp.text().ok().ok_or(())?;
                Ok(motd)
            },

            _ => Err(()),
        }
    }

    // TODO

    // TODO serialize function
}
