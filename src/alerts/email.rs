use crate::{alerts::Alert, probes::Notification};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use lettre::{
    message::{header, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use prometheus::{register_counter_vec, CounterVec};
use serde_derive::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Email {
    namepass: Option<Vec<String>>,
    smtp_relay: String,
    smtp_username: String,
    smtp_password: String,
    from: String,
    to: String,
}

lazy_static! {
    static ref RUNS_TOTAL: CounterVec = register_counter_vec!(
        "alert_email_runs_total",
        "run counter for email alert plugin",
        &["plugin", "smtp_relay", "from", "to"]
    )
    .unwrap();
}

#[async_trait]
impl Alert for Email {
    fn new(namepass: Vec<&str>) -> Self {
        Email {
            namepass: Some(namepass.into_iter().map(String::from).collect()),
            ..Default::default()
        }
    }

    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    async fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire email alert for {}", &notif.name);
            return Ok(());
        }
        RUNS_TOTAL
            .with_label_values(&["alert.email", &self.smtp_relay, &self.from, &self.to])
            .inc();

        log::info!(
            "sending email alert to {} via smtp relay {}",
            self.to,
            self.smtp_relay
        );
        log::debug!("NOTIFICATION: {:?}", notif);

        let from: Mailbox = (&self.from)
            .parse()
            .with_context(|| format!("failed parsing email from address {}", self.from))?;
        let to: Mailbox = (&self.to)
            .parse()
            .with_context(|| format!("failed parising email to address {}", self.to))?;
        let message_builder = Message::builder()
            .from(from.clone())
            .reply_to(from)
            .to(to)
            .subject(format!("TRIGGERED [{}]: {}", notif.from, notif.title));
        let email = match notif.message_html.to_owned() {
            Some(message_html) => message_builder.multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType(
                                "text/plain; charset=utf8".parse().unwrap(),
                            ))
                            .body(format!("{}\n{}", notif.check, notif.message)),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType(
                                "text/html; charset=utf8".parse().unwrap(),
                            ))
                            .body(format!("<p>{}</p>{}", notif.check, message_html)),
                    ),
            ),
            None => message_builder.body(format!("{}\n{}", notif.check, notif.message)),
        }
        .with_context(|| "failed building email message")?;

        let creds = Credentials::new(self.smtp_username.to_owned(), self.smtp_password.to_owned());
        let mailer = SmtpTransport::relay(&self.smtp_relay)
            .with_context(|| format!("failed building SMTP transport {}", self.smtp_relay))?
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(err) => anyhow::bail!("failed to send email to {}: {}", self.to, err),
        }
    }
}
