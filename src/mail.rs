use base64::Engine;
use lettre::{
    message::Message,
    transport::smtp::authentication::{Credentials, Mechanism},
    SmtpTransport, Transport,
};
use rsa::pkcs1::DecodeRsaPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MailErrcode {
    NoConfiguration,
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
            .map_err(MailErrcode::DecodeB64Pubkey)?;
        let pubkey = rsa::RsaPublicKey::from_pkcs1_der(&pubkey_data)
            .map_err(MailErrcode::ImportPublicKey)?;

        let mut rng = rand::thread_rng();
        let mut body = "".to_string();
        for d in data.chunks(50) {
            let body_bytes = pubkey
                .encrypt(&mut rng, rsa::Oaep::new::<sha2::Sha256>(), d)
                .map_err(MailErrcode::MailBodyEncrypt)?;
            body += base64::engine::general_purpose::STANDARD
                .encode(body_bytes)
                .as_str();
            body += "\n-----\n";
        }

        let email = Message::builder()
            .from(self.from.parse().map_err(MailErrcode::FromParse)?)
            .to(self
                .recipient
                .parse()
                .map_err(MailErrcode::RecipientParse)?)
            .subject(format!(
                "{}: {subject}",
                chrono::Local::now().format("%Y/%m/%d %H:%M:%S")
            ))
            .body(body.trim().to_string())
            .map_err(MailErrcode::BuildMessage)?;

        let sender = SmtpTransport::relay(&self.relay)
            .map_err(MailErrcode::InvalidSmtpRelay)?
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .authentication(vec![Mechanism::Plain])
            .build();

        let result = sender.send(&email).map_err(MailErrcode::SendError)?;
        Ok(())
    }
}
