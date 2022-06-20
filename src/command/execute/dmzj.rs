use super::{Element, Source};
use crate::{context::Context, model};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) struct DmzjSource {}

#[derive(Deserialize)]
pub(self) struct DmzjChannel {
    pub id: String,
}

impl Source for DmzjSource {
    fn tag(&self) -> &'static str {
        return "dmzj";
    }

    fn fetch(&self, learn: bool, value: &Value, context: &Context) -> Vec<Element> {
        if !value.is_array() {
            context.report_error("the source parsed from config file is not an array");
            return vec![];
        }
        let mut result: Vec<Element> = vec![];
        let channels: Vec<DmzjChannel> = match serde_json::from_value(value.clone()) {
            Ok(channels) => channels,
            Err(error) => {
                context.report_error(&format!("failed to parse source: {}", error));
                return vec![];
            }
        };
        'channel: for channel in channels {
            // Fetch comic information.
            let comic_id = &channel.id;
            context.report_debug(&format!("fetching comic {}", comic_id));
            let comic_info = match model::dmzj::search_comic(comic_id) {
                Ok(comic_info) => comic_info,
                Err(error) => {
                    context.report_error(&format!("failed to search comic: {}", error));
                    continue 'channel;
                }
            };
            context.report_debug(&format!(
                "found comic {} from {}",
                &comic_info.title, comic_id
            ));

            // Fetch chapters.
            'chapter: for chapter in comic_info.chapters {
                let chapter_id = &chapter.id;
                context.report_debug(&format!(
                    "found chapter {} from {}:{}",
                    &chapter.title, comic_id, chapter_id
                ));
                if context.is_marked(self.tag(), comic_id, chapter_id) {
                    context.report_debug(&format!(
                        "skip chapter {}:{} because it is marked",
                        comic_id, chapter_id
                    ));
                    continue 'chapter;
                }

                let mut images: Vec<PathBuf> = vec![];
                if !learn {
                    // Only fetch and download image if not in learn mode.
                    context.report_debug(&format!("fetching chapter {}:{}", comic_id, chapter_id));
                    let chapter_info = match model::dmzj::search_chapter(comic_id, chapter_id) {
                        Ok(chapter_info) => chapter_info,
                        Err(error) => {
                            context.report_error(&format!("failed to search chapter: {}", error));
                            continue 'chapter;
                        }
                    };

                    // Download comic images.
                    for (index, url) in chapter_info.pages.iter().enumerate() {
                        let extension = match Path::new(&url).extension().and_then(|e| e.to_str()) {
                            Some(extension) => extension,
                            None => {
                                context.report_error(&format!(
                                    "cannot parse extension from url {}",
                                    &url
                                ));
                                continue 'chapter;
                            }
                        };
                        let path = match context.create_image_cache(
                            self.tag(),
                            comic_id,
                            chapter_id,
                            index,
                            extension,
                        ) {
                            Ok(path) => path,
                            Err(error) => {
                                context.report_error(&format!(
                                    "failed to create image file: {}",
                                    error
                                ));
                                continue 'chapter;
                            }
                        };
                        if let Err(error) = model::dmzj::download_image(path.as_path(), &url) {
                            context.report_error(&format!("failed to download image: {}", error));
                            continue 'chapter;
                        } else {
                            images.push(path);
                        }
                    }
                }
                result.push(Element {
                    source_tag: self.tag(),
                    comic_id: comic_id.clone(),
                    comic_name: comic_info.title.clone(),
                    chapter_id: chapter_id.clone(),
                    chapter_name: chapter.title,
                    images,
                });
            }
        }
        return result;
    }
}
