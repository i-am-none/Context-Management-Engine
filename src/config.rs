use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub host: [u8; 4],
    pub port: u16,
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: [127, 0, 0, 1],
            port: 3000,
            database_url: "sqlite://nexus.db".to_string(),
            max_connections: 5,
        }
    }
}

impl Config {
    /// Loads configuration with optional overrides from environment variables.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(db_url) = std::env::var("NEXUS_DATABASE_URL") {
            let trimmed = db_url.trim();
            if !trimmed.is_empty() {
                config.database_url = trimmed.to_string();
            }
        }

        if let Ok(conns_str) = std::env::var("NEXUS_MAX_CONNECTIONS") {
            if let Ok(conns) = conns_str.parse::<u32>() {
                if conns > 0 {
                    config.max_connections = conns;
                }
            }
        }

        config
    }

    /// Validates configuration parameters.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_connections == 0 {
            return Err("max_connections must be at least 1");
        }
        if self.database_url.trim().is_empty() {
            return Err("database_url must not be empty");
        }
        Ok(())
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.host, [127, 0, 0, 1]);
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "sqlite://nexus.db");
        assert_eq!(config.max_connections, 5);
        assert_eq!(
            config.socket_addr(),
            SocketAddr::from(([127, 0, 0, 1], 3000))
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_max_connections() {
        let mut config = Config::default();
        config.max_connections = 0;
        assert_eq!(config.validate(), Err("max_connections must be at least 1"));
    }

    #[test]
    fn test_config_validation_empty_database_url() {
        let mut config = Config::default();
        config.database_url = "   ".to_string();
        assert_eq!(config.validate(), Err("database_url must not be empty"));
    }
}
