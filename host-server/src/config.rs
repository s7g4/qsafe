use std::env;

/// HS256 signs with the secret as an HMAC key; NIST SP 800-107 recommends a
/// key at least as long as the hash output (32 bytes for SHA-256). Below
/// that, the secret becomes brute-forceable and the entire dual-JWT auth
/// model's security guarantee is void.
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub hsm_mock: bool,
    pub hsm_port: Option<String>,
    pub cors_origin: String,
    pub db_max_connections: u32,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable must be set")?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable must be set")?;
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN} bytes long (got {}); a short secret can be brute-forced, breaking all token signing",
                jwt_secret.len()
            )
            .into());
        }

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid 16-bit unsigned integer")?;

        let hsm_mock = env::var("HSM_MOCK")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let hsm_port = env::var("HSM_PORT").ok();

        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|_| "DB_MAX_CONNECTIONS must be a valid unsigned integer")?;

        let tls_cert_path = env::var("TLS_CERT_PATH").ok();
        let tls_key_path = env::var("TLS_KEY_PATH").ok();

        Ok(Self {
            database_url,
            jwt_secret,
            port,
            hsm_mock,
            hsm_port,
            cors_origin,
            db_max_connections,
            tls_cert_path,
            tls_key_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Config::load() reads process-wide env vars, so these tests must not
    // run concurrently with each other (they'd stomp on each other's
    // JWT_SECRET/DATABASE_URL). A single mutex serializes just this module.
    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        let _guard = ENV_TEST_LOCK.lock().unwrap();
        for (k, v) in vars {
            std::env::set_var(k, v);
        }
        f();
        for (k, _) in vars {
            std::env::remove_var(k);
        }
    }

    #[test]
    fn rejects_short_jwt_secret() {
        with_env(
            &[
                ("DATABASE_URL", "postgres://localhost/doesnotmatter"),
                ("JWT_SECRET", "too-short"),
            ],
            || {
                let err = Config::load().expect_err("short JWT_SECRET must be rejected");
                assert!(err.to_string().contains("JWT_SECRET must be at least"));
            },
        );
    }

    #[test]
    fn accepts_jwt_secret_at_the_minimum_length() {
        with_env(
            &[
                ("DATABASE_URL", "postgres://localhost/doesnotmatter"),
                ("JWT_SECRET", &"a".repeat(MIN_JWT_SECRET_LEN)),
            ],
            || {
                let config = Config::load().expect("32-byte JWT_SECRET must be accepted");
                assert_eq!(config.jwt_secret.len(), MIN_JWT_SECRET_LEN);
            },
        );
    }
}
