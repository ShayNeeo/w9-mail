// Email service implementation using Microsoft SMTP/IMAP/POP3
// This module will handle email operations

use lettre::{
    message::{header::ContentType, Mailbox, Message, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

pub struct EmailService;

impl EmailService {
    pub fn new() -> Self {
        EmailService
    }

    pub async fn send_email(
        &self,
        from_email: &str,
        from_password: &str,
        to: &str,
        subject: &str,
        body: &str,
        cc: Option<&str>,
        bcc: Option<&str>,
    ) -> anyhow::Result<()> {
        // Parse email addresses
        let from_addr: Mailbox = from_email.parse()?;
        
        // Build recipients list
        let mut to_addresses = Vec::new();
        for addr in to.split(',') {
            let trimmed = addr.trim();
            if !trimmed.is_empty() {
                to_addresses.push(trimmed.parse::<Mailbox>()?);
            }
        }

        // Build CC list
        let mut cc_addresses = Vec::new();
        if let Some(cc_str) = cc {
            for addr in cc_str.split(',') {
                let trimmed = addr.trim();
                if !trimmed.is_empty() {
                    cc_addresses.push(trimmed.parse::<Mailbox>()?);
                }
            }
        }

        // Build BCC list
        let mut bcc_addresses = Vec::new();
        if let Some(bcc_str) = bcc {
            for addr in bcc_str.split(',') {
                let trimmed = addr.trim();
                if !trimmed.is_empty() {
                    bcc_addresses.push(trimmed.parse::<Mailbox>()?);
                }
            }
        }

        // Build email message
        let mut message_builder = Message::builder()
            .from(from_addr.clone())
            .subject(subject);

        // Add To recipients
        for addr in &to_addresses {
            message_builder = message_builder.to(addr.clone());
        }

        // Add CC recipients
        for addr in &cc_addresses {
            message_builder = message_builder.cc(addr.clone());
        }

        // Add BCC recipients (lettre doesn't support BCC in headers, we'll add them to the envelope)
        for addr in &bcc_addresses {
            message_builder = message_builder.bcc(addr.clone());
        }

        // Set body as plain text (can be extended to HTML later)
        let email = message_builder
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(body.to_string()),
            )?;

        // Create SMTP transport for Microsoft/Outlook
        let creds = Credentials::new(from_email.to_string(), from_password.to_string());
        
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp-mail.outlook.com")?
            .port(587)
            .credentials(creds)
            .build();

        // Send email
        mailer.send(email).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn fetch_inbox(&self, _account: &str, _limit: Option<u32>) -> anyhow::Result<Vec<serde_json::Value>> {
        // TODO: Implement IMAP inbox fetching
        Ok(vec![])
    }
}
