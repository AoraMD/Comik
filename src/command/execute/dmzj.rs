use super::{Element, Source};
use crate::{context::Context, model};
use async_trait::async_trait;
use futures::future::join_all;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) struct DmzjSource {}

#[derive(Deserialize)]
pub(self) struct DmzjChannel {
    pub id: String,
}

#[async_trait]
impl Source for DmzjSource {
    fn tag(&self) -> &'static str {
        return "dmzj";
    }

    async fn fetch(&self, learn: bool, value: &Value, context: &Context) -> Vec<Element> {
        if !value.is_array() {
            context.report_error("the source parsed from config file is not an array");
            return vec![];
        }
        let channels: Vec<DmzjChannel> = match serde_json::from_value(value.clone()) {
            Ok(channels) => channels,
            Err(error) => {
                context.report_error(&format!("failed to parse source: {}", error));
                return vec![];
            }
        };
        let futures = channels.into_iter().map(|channel| async move {
            // Fetch comic information.
            let comic_id = &channel.id;
            context.report_debug(&format!("fetching comic {}", comic_id));
            let comic_info = match model::dmzj::search_comic(comic_id).await {
                Ok(comic_info) => comic_info,
                Err(error) => {
                    context.report_error(&format!("failed to search comic: {}", error));
                    return None; // 'channel
                }
            };
            context.report_debug(&format!(
                "found comic {} from {}",
                &comic_info.title, comic_id
            ));

            // Fetch chapters.
            let comic_name = comic_info.title.as_str();
            let futures = comic_info.chapters.into_iter().map(|chapter| async move {
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
                    return None; // 'chapter
                }

                let images = if !learn {
                    // Only fetch and download image if not in learn mode.
                    context.report_debug(&format!("fetching chapter {}:{}", comic_id, chapter_id));
                    let chapter_info = match model::dmzj::search_chapter(comic_id, chapter_id).await {
                        Ok(chapter_info) => chapter_info,
                        Err(error) => {
                            context.report_error(&format!("failed to search chapter: {}", error));
                            return None; // 'chapter
                        }
                    };

                    // Download comic images.
                    let futures =
                        chapter_info
                            .pages
                            .iter()
                            .enumerate()
                            .map(|(index, url)| async move {
                                let extension =
                                    match Path::new(&url).extension().and_then(|e| e.to_str()) {
                                        Some(extension) => extension,
                                        None => {
                                            context.report_error(&format!(
                                                "cannot parse extension from url {}",
                                                &url
                                            ));
                                            return None; // 'page
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
                                        return None; // 'page
                                    }
                                };
                                if let Err(error) =
                                    model::dmzj::download_image(path.as_path(), &url).await
                                {
                                    context.report_error(&format!(
                                        "failed to download image: {}",
                                        error
                                    ));
                                    return None; // 'page
                                } else {
                                    return Some(path); // 'page
                                }
                            });
                    join_all(futures)
                        .await
                        .into_iter()
                        .flatten()
                        .collect::<Vec<PathBuf>>()
                } else {
                    vec![]
                };
                return Some(Element {
                    source_tag: self.tag(),
                    comic_id: comic_id.clone(),
                    comic_name: comic_name.to_string(),
                    chapter_id: chapter_id.clone(),
                    chapter_name: chapter.title,
                    images,
                });
            });
            return Some(
                join_all(futures)
                    .await
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Element>>(),
            ); // 'channel
        });
        return join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect();
    }
}
