use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load the local .env file if it exists
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable must be set")?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable must be set")?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid 16-bit unsigned integer")?;

        Ok(Self {
            database_url,
            jwt_secret,
            port,
        })
    }
}
