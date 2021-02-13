use super::Alert;
use super::Notification;
use anyhow::{Context, Result};
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Email {
    namepass: Option<Vec<String>>,
    smtp_relay: String,
    smtp_username: String,
    smtp_password: String,
    from: String,
    to: String,
}

impl Alert for Email {
    fn namepass(&self) -> Option<Vec<String>> {
        self.namepass.clone()
    }

    fn notify(&self, notif: &Notification) -> Result<()> {
        if !self.should_fire(&notif.name) {
            log::info!("should not fire email alert for {}", &notif.name);
            return Ok(());
        }

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
