mod dmzj;

use crate::{
    context::Context,
    mail::{Mailbox, MailboxJson},
    util::{extension::ResultExtension, pdf::create_pdf_from_images},
    APP_NAME_TITLEIZE,
};
use async_trait::async_trait;
use const_format::formatcp;
use futures::future::join_all;
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, path::PathBuf};

pub(self) const NOTIFY_UPDATE_TITLE: &str = "Comic Update";
pub(self) const HOLDER_COMIC_NAME: &str = "%comic%";
pub(self) const HOLDER_CHAPTER_NAME: &str = "%chapter%";
pub(self) const HOLDER_SUCCESS_COUNT: &str = "%success%";
pub(self) const HOLDER_TOTAL_COUNT: &str = "%total%";
pub(self) const DEFAULT_NOTIFY_CONTENT_TEMPLATE: &str = formatcp!(
    "Comic {} has been updated to chapter {} ({}/{}).",
    HOLDER_COMIC_NAME,
    HOLDER_CHAPTER_NAME,
    HOLDER_SUCCESS_COUNT,
    HOLDER_TOTAL_COUNT
);

#[derive(Deserialize)]
pub(self) struct ConfigJson {
    sender: MailboxJson,
    receivers: Vec<String>,
    notify: Option<String>,
    source: Value,
}

impl ConfigJson {
    async fn read(path: PathBuf) -> Result<ConfigJson, Box<dyn Error>> {
        let json = tokio::fs::read_to_string(path).await?;
        return serde_json::from_str::<ConfigJson>(json.as_str())?.into_ok();
    }
}

pub(self) struct Config {
    sender: Mailbox,
    receivers: Vec<String>,
}

impl TryFrom<&ConfigJson> for Config {
    type Error = Box<dyn Error>;

    fn try_from(json: &ConfigJson) -> Result<Self, Box<dyn Error>> {
        return Config {
            sender: Mailbox::try_from(&json.sender)?,
            receivers: json.receivers.clone(),
        }
        .into_ok();
    }
}

#[async_trait]
pub(self) trait Source {
    fn tag(&self) -> &'static str;
    async fn fetch(&self, learn: bool, value: &Value, context: &Context) -> Vec<Element>;
}

fn find_source(tag: &str) -> Option<Box<dyn Source>> {
    return match tag {
        "dmzj" => Some(Box::new(dmzj::DmzjSource {})),
        _ => None,
    };
}

pub(self) struct Element {
    source_tag: &'static str,
    comic_id: String,
    comic_name: String,
    chapter_id: String,
    chapter_name: String,
    images: Vec<PathBuf>,
}

pub(crate) async fn main(learn: bool, scale: f64, config: PathBuf, context: &Context) {
    let config_json = {
        let config_json = ConfigJson::read(config).await;
        if let Err(error) = config_json {
            context.report_error(&format!("failed to parse config file: {}", error));
            return;
        }
        config_json.unwrap()
    };

    if !config_json.source.is_object() {
        context.report_error("the source property parsed from config file is not an object");
        return;
    }
    let source_value = config_json.source.as_object().unwrap();

    // Collect elements will be sent.
    let elements = {
        let futures = source_value.keys().map(|tag| async {
            if let Some(source) = find_source(tag) {
                return Some(
                    source
                        .fetch(learn, source_value.get(tag).unwrap(), context)
                        .await,
                );
            } else {
                return None;
            }
        });
        join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<Element>>()
    };

    // Create and send document.
    let config = {
        let config = Config::try_from(&config_json);
        if let Err(error) = config {
            context.report_error(&format!("failed to create config instance: {}", error));
            return;
        }
        config.unwrap()
    };
    let sender = &config.sender;
    let receivers = &config.receivers;
    let notify = &config_json.notify;
    let futures = elements.into_iter().map(|element| async move {
        if !learn {
            let file = create_pdf_from_images(
                &format!("{} {}.pdf", &element.comic_name, &element.chapter_name),
                context.document_repo_path(),
                &element.images,
                scale,
            )
            .await;
            if let Err(error) = file {
                context.report_error(&format!("failed to create document: {}", error));
                return;
            }
            let file = file.unwrap();
            let mut success = 0;
            for receiver in receivers {
                if let Err(error) = sender.send_file(receiver, APP_NAME_TITLEIZE, &file).await {
                    context
                        .report_error(&format!("failed to send mail to {}: {}", receiver, error));
                } else {
                    success += 1;
                }
            }
            let content = notify
                .clone()
                .unwrap_or(DEFAULT_NOTIFY_CONTENT_TEMPLATE.to_string())
                .replace(HOLDER_COMIC_NAME, &element.comic_name)
                .replace(HOLDER_CHAPTER_NAME, &element.chapter_name)
                .replace(HOLDER_SUCCESS_COUNT, &success.to_string())
                .replace(HOLDER_TOTAL_COUNT, receivers.len().to_string().as_str());
            context.notify(NOTIFY_UPDATE_TITLE, &content).await;
        } else {
            context.report_info(&format!(
                "Skip creating document for {}:{} in learn mode",
                &element.comic_id, &element.chapter_id
            ));
        }

        // Mark document has been sent.
        if let Err(error) = context
            .mark(&element.source_tag, &element.comic_id, &element.chapter_id)
            .await
        {
            context.report_error(&format!(
                "failed to mark {}:{}: {}",
                &element.comic_id, &element.chapter_id, error
            ));
        }
    });
    join_all(futures).await;
}
