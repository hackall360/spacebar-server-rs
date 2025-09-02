use anyhow::{anyhow, Result};
use config::EmailConfiguration;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// Email helper built on top of `lettre`.
pub struct Email {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl Email {
    /// Initialise an SMTP transport based on configuration.
    pub async fn init(cfg: &EmailConfiguration) -> Result<Self> {
        let provider = cfg.provider.as_deref().unwrap_or("").to_lowercase();
        if provider != "smtp" {
            return Err(anyhow!("unsupported email provider"));
        }

        let host = cfg
            .smtp
            .host
            .clone()
            .ok_or_else(|| anyhow!("smtp.host missing"))?;
        let port = cfg.smtp.port.unwrap_or(587);

        // Choose secure or plain connection
        let mut builder = if cfg.smtp.secure.unwrap_or(true) {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                .map_err(|e| anyhow!("{e}"))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
        };

        builder = builder.port(port);

        if let (Some(user), Some(pass)) = (&cfg.smtp.username, &cfg.smtp.password) {
            let creds = Credentials::new(user.clone(), pass.clone());
            builder = builder.credentials(creds);
        }

        Ok(Self {
            transport: builder.build(),
        })
    }

    /// Send an email message using the configured transport.
    pub async fn send(&self, msg: Message) -> Result<()> {
        self.transport
            .send(msg)
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }
}
