use base64::Engine;
use rsa::pkcs1::DecodeRsaPublicKey;
use serde::{Deserialize, Serialize};
use lettre::{message::Message, transport::smtp::authentication::{Credentials, Mechanism}, SmtpTransport, Transport};

#[derive(Debug)]
pub enum MailErrcode {
    DecodeB64Pubkey(base64::DecodeError),
    ImportPublicKey(rsa::pkcs1::Error),
    MailBodyEncrypt(rsa::Error),
    FromParse(lettre::address::AddressError),
    RecipientParse(lettre::address::AddressError),
    BuildMessage(lettre::error::Error),
    InvalidSmtpRelay(lettre::transport::smtp::Error),
    SendError(lettre::transport::smtp::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailConfig {
    relay: String,
    username: String,
    password: String,

    from: String,
    recipient: String,
    pubkey: String,
}

impl MailConfig {
    pub fn send_data(&self, subject: &str, data: &[u8]) -> Result<(), MailErrcode> {
        let pubkey_data = base64::engine::general_purpose::STANDARD
            .decode(&self.pubkey)
            .map_err(|e| MailErrcode::DecodeB64Pubkey(e))?;
        let pubkey = rsa::RsaPublicKey::from_pkcs1_der(&pubkey_data)
            .map_err(|e| MailErrcode::ImportPublicKey(e))?;

        let mut rng = rand::thread_rng();
        let mut body = "".to_string();
        for d in data.chunks(50) {
            let body_bytes = pubkey
                .encrypt(&mut rng, rsa::Oaep::new::<sha2::Sha256>(), d)
                .map_err(|e| MailErrcode::MailBodyEncrypt(e))?;
            body += base64::engine::general_purpose::STANDARD.encode(body_bytes).as_str();
            body += "\n-----\n";
        }

        let email = Message::builder()
            .from(
                self.from.parse()
                    .map_err(|e| MailErrcode::FromParse(e))?
            )
            .to(
                self.recipient.parse()
                .map_err(|e| MailErrcode::RecipientParse(e))?
            )
            .subject(format!("{}: {subject}", chrono::Local::now().format("%Y/%m/%d %H:%M:%S")))
            .body(body.trim().to_string())
            .map_err(|e| MailErrcode::BuildMessage(e))?;

        let sender = SmtpTransport::relay(&self.relay)
            .map_err(|e| MailErrcode::InvalidSmtpRelay(e))?
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .authentication(vec![Mechanism::Plain])
            .build();


        let result = sender.send(&email)
            .map_err(|e| MailErrcode::SendError(e))?;
        Ok(())
    }
}
