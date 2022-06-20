use lettre::smtp::authentication::Credentials;
use lettre::smtp::authentication::Mechanism;
use lettre::ClientSecurity;
use lettre::ClientTlsParameters;
use lettre::SmtpClient;
use lettre::SmtpTransport;
use lettre::Transport;
use lettre_email::mime::Mime;
use lettre_email::EmailBuilder;
use native_tls::Protocol;
use native_tls::TlsConnector;
use serde::Deserialize;
use std::cell::RefCell;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize)]
pub(crate) struct MailboxJson {
    address: String,
    host: String,
    port: u16,
    password: String,
}

pub(crate) struct Mailbox {
    address: String,
    transport: RefCell<SmtpTransport>,
}

impl Mailbox {
    pub fn send_file(
        &self,
        to: &str,
        subject: &str,
        file: &Path,
        mine: &Mime,
    ) -> Result<(), Box<dyn Error>> {
        let mail = EmailBuilder::new()
            .from(self.address.as_str())
            .to(to)
            .subject(subject)
            .attachment_from_file(file, file.file_name().and_then(|x| x.to_str()), mine)?
            .build()?;

        self.transport.borrow_mut().send(mail.into())?;
        return Ok(());
    }
}

impl TryFrom<&MailboxJson> for Mailbox {
    type Error = Box<dyn Error>;

    fn try_from(json: &MailboxJson) -> Result<Self, Box<dyn Error>> {
        let tls_connector = TlsConnector::builder()
            .min_protocol_version(Some(Protocol::Tlsv12))
            .build()?;
        let tls_parameters = ClientTlsParameters::new(json.host.clone(), tls_connector);
        let transport = SmtpClient::new(
            (json.host.as_str(), json.port),
            ClientSecurity::Required(tls_parameters),
        )?
        .authentication_mechanism(Mechanism::Login)
        .credentials(Credentials::new(
            json.address.clone(),
            json.password.clone(),
        ))
        .transport();
        return Ok(Mailbox {
            address: json.address.clone(),
            transport: RefCell::new(transport),
        });
    }
}
