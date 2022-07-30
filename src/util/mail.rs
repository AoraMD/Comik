use crate::APP_NAME_TITLEIZE;
use lettre::{
    address::Address,
    message::{header::ContentType, Attachment, Mailbox as LettreMailBox, Message},
    transport::smtp::authentication::{Credentials, Mechanism},
    SmtpTransport, Transport,
};
use serde::Deserialize;
use std::{cell::RefCell, error::Error, path::Path};

#[derive(Deserialize)]
pub(crate) struct MailboxJson {
    address: String,
    host: String,
    password: String,
}

pub(crate) struct Mailbox {
    address: String,
    transport: RefCell<SmtpTransport>,
}

impl Mailbox {
    pub async fn send_file(
        &self,
        to: &str,
        subject: &str,
        file: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let from = LettreMailBox::new(
            Some(APP_NAME_TITLEIZE.to_string()),
            self.address.as_str().parse::<Address>()?,
        );
        let to = LettreMailBox::new(None, to.parse::<Address>()?);
        let mail = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .singlepart(
                Attachment::new(file.file_name().unwrap().to_string_lossy().to_string()).body(
                    tokio::fs::read(file).await?,
                    ContentType::parse("application/pdf").unwrap(),
                ),
            )?;

        self.transport.borrow_mut().send(&mail)?;
        return Ok(());
    }
}

impl TryFrom<&MailboxJson> for Mailbox {
    type Error = Box<dyn Error>;

    fn try_from(json: &MailboxJson) -> Result<Self, Box<dyn Error>> {
        let transport = SmtpTransport::starttls_relay(json.host.as_str())?
            .credentials(Credentials::new(
                json.address.clone(),
                json.password.clone(),
            ))
            .authentication(vec![Mechanism::Login])
            .build();
        return Ok(Mailbox {
            address: json.address.clone(),
            transport: RefCell::new(transport),
        });
    }
}
