// Email service implementation using Microsoft SMTP/IMAP/POP3
// This module will handle email operations

use lettre::{
    message::{header::ContentType, Attachment, Body, Mailbox, Message, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use regex::Regex;

// Simple HTML escape function
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub struct EmailService;

// Render email body with W9 Mail branding template (matching w9-tools design)
pub fn render_email_template(body: &str) -> String {
    // Check if body is already a complete HTML document
    let trimmed = body.trim();
    if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
        // Already a complete HTML document, return as-is
        return body.to_string();
    }
    
    // Escape HTML in the body content (plain text)
    let escaped_body = html_escape(body);
    // Convert newlines to <br> tags for HTML
    let html_body = escaped_body.replace("\n", "<br />");
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>W9 Mail</title>
</head>
<body style="background:#050505;padding:32px;font-family:'Courier New',Courier,monospace;">
  <table role="presentation" cellpadding="0" cellspacing="0" width="100%">
    <tr>
      <td align="center">
        <table role="presentation" cellpadding="0" cellspacing="0" width="100%" style="max-width:640px;border:2px solid #fdfdfd;padding:28px;background:#000;">
          <tr><td style="text-align:left;">
            <table role="presentation" cellpadding="0" cellspacing="0" style="margin-bottom:24px;">
              <tr>
                <td style="width:42px;height:42px;border:2px solid #fdfdfd;text-align:center;vertical-align:middle;font-weight:bold;color:#fdfdfd;line-height:42px;font-size:16px;padding:0;margin:0;">W9</td>
                <td style="padding-left:12px;vertical-align:middle;">
                  <div style="color:#fdfdfd;font-size:18px;letter-spacing:0.1em;text-transform:uppercase;">W9 Mail</div>
                  <div style="color:#9a9a9a;font-size:12px;">Open-source mail rail</div>
                </td>
              </tr>
            </table>
            <div style="color:#fdfdfd;font-size:15px;line-height:1.6;font-family:'Courier New',Courier,monospace;">
              {html_body}
            </div>
            <hr style="border:none;border-top:2px solid #1a1a1a;margin:32px 0;" />
            <p style="margin:0;color:#686868;font-size:11px;line-height:1.4;">Sent via W9 Mail. Open-source mail rail for teams.</p>
          </td></tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        html_body = html_body
    )
}

impl EmailService {
    pub fn new() -> Self {
        EmailService
    }

    pub async fn send_email(
        &self,
        header_from: &str,
        auth_email: &str,
        auth_password: &str,
        to: &str,
        subject: &str,
        body: &str,
        cc: Option<&str>,
        bcc: Option<&str>,
        as_html: bool,
    ) -> anyhow::Result<()> {
        // Parse email addresses
        let from_addr: Mailbox = header_from.parse()?;
        
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

        // Handle inline images: convert data URIs to CID attachments
        let (final_body, attachments) = if as_html {
            extract_inline_images(body)
        } else {
            (body.to_string(), Vec::new())
        };

        let content_type = if as_html {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        // Build email with or without attachments
        let email = if attachments.is_empty() {
            // Simple singlepart email
            message_builder.singlepart(
                SinglePart::builder()
                    .header(content_type)
                    .body(final_body),
            )?
        } else {
            // Multipart email with inline attachments
            let mut multipart = MultiPart::related()
                .singlepart(
                    SinglePart::builder()
                        .header(content_type)
                        .body(final_body),
                );

            for (cid, mime_type, data) in attachments {
                let content_type = ContentType::parse(&mime_type)
                    .unwrap_or(ContentType::TEXT_PLAIN);
                let attachment = Attachment::new_inline(cid.clone())
                    .body(data, content_type);
                multipart = multipart.singlepart(attachment);
            }

            message_builder.multipart(multipart)?
        };

        // Create SMTP transport for Microsoft/Outlook
        // Port 587 requires STARTTLS (not direct TLS)
        let creds = Credentials::new(auth_email.to_string(), auth_password.to_string());
        
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp-mail.outlook.com")?
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

// Extract data URIs from HTML and convert them to CID attachments
// Returns (modified_html, vec of (cid, mime_type, data))
fn extract_inline_images(html: &str) -> (String, Vec<(String, String, Vec<u8>)>) {
    // Pattern to match data URIs: data:image/type;base64,data
    let re = Regex::new(r#"data:([^;]+);base64,([^"'\s>]+)"#).unwrap();
    let mut attachments = Vec::new();
    let mut cid_counter = 0;
    let mut modified_html = html.to_string();

    // Find all matches and collect them
    for cap in re.captures_iter(html) {
        let full_match = cap.get(0).unwrap();
        let mime_type = cap.get(1).map(|m| m.as_str()).unwrap_or("image/png");
        let base64_data = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        // Decode base64 data
        if let Ok(data) = Base64.decode(base64_data) {
            cid_counter += 1;
            let cid = format!("image{}", cid_counter);
            
            // Store attachment info
            attachments.push((cid.clone(), mime_type.to_string(), data));
            
            // Replace in HTML
            let cid_ref = format!("cid:{}", cid);
            modified_html = modified_html.replacen(full_match.as_str(), &cid_ref, 1);
        }
    }

    (modified_html, attachments)
}
