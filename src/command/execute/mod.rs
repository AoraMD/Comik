mod dmzj;

use crate::{
    context::Context,
    mail::{Mailbox, MailboxJson},
    util::{extension::ResultExtension, pdf::create_pdf_from_images},
    APP_NAME,
};
use const_format::formatcp;
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, fs::File, path::PathBuf};

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
    fn read(path: PathBuf) -> Result<ConfigJson, Box<dyn Error>> {
        return Ok(serde_json::from_reader(File::open(path)?)?);
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

pub(self) trait Source {
    fn tag(&self) -> &'static str;
    fn fetch(&self, learn: bool, value: &Value, context: &Context) -> Vec<Element>;
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

pub(crate) fn execute_main(learn: bool, scale: f64, config: PathBuf, context: &Context) {
    let config_json = {
        let config_json = ConfigJson::read(config);
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
    let mut elements: Vec<Element> = vec![];
    for tag in source_value.keys() {
        if let Some(source) = find_source(tag) {
            let result = source.fetch(learn, source_value.get(tag).unwrap(), context);
            elements.extend(result);
        }
    }
    let elements = elements; // Mark immutable.

    // Create and send document.
    let config = {
        let config = Config::try_from(&config_json);
        if let Err(error) = config {
            context.report_error(&format!("failed to create config instance: {}", error));
            return;
        }
        config.unwrap()
    };
    'element: for element in elements {
        if !learn {
            let file = create_pdf_from_images(
                &format!("{} {}.pdf", &element.comic_name, &element.chapter_name),
                context.document_repo_path(),
                &element.images,
                scale,
            );
            if let Err(error) = file {
                context.report_error(&format!("failed to create document: {}", error));
                continue 'element;
            }
            let file = file.unwrap();
            let mut success = 0;
            for receiver in &config.receivers {
                if let Err(error) =
                    config
                        .sender
                        .send_file(receiver, APP_NAME, &file)
                {
                    context
                        .report_error(&format!("failed to send mail to {}: {}", receiver, error));
                } else {
                    success += 1;
                }
            }
            let content = config_json
                .notify
                .clone()
                .unwrap_or(DEFAULT_NOTIFY_CONTENT_TEMPLATE.to_string())
                .replace(HOLDER_COMIC_NAME, &element.comic_name)
                .replace(HOLDER_CHAPTER_NAME, &element.chapter_name)
                .replace(HOLDER_SUCCESS_COUNT, &success.to_string())
                .replace(HOLDER_TOTAL_COUNT, &config.receivers.len().to_string());
            context.notify(NOTIFY_UPDATE_TITLE, &content);
        } else {
            context.report_info(&format!(
                "Skip creating document for {}:{} in learn mode",
                &element.comic_id, &element.chapter_id
            ));
        }

        // Mark document has been sent.
        if let Err(error) =
            context.mark(&element.source_tag, &element.comic_id, &element.chapter_id)
        {
            context.report_error(&format!(
                "failed to mark {}:{}: {}",
                &element.comic_id, &element.chapter_id, error
            ));
        }
    }
}
