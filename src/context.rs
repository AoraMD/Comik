use crate::{util::extension::ResultExtension, APP_NAME_TITLEIZE};
use futures::executor::block_on;
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio::fs::File as TokioFile;

pub(crate) struct Context {
    debug: bool,
    cache: PathBuf,
    repo: PathBuf,
    mark: PathBuf,
    bark: Option<String>,
}

pub(self) const ICON_URL: &str = "https://comik-icon.aoramd.moe/icon.png";

impl Context {
    pub fn new(debug: bool, cache: PathBuf, repo: PathBuf, bark: Option<String>) -> Self {
        return Context {
            debug,
            cache: cache.clone(),
            repo: repo.clone(),
            mark: repo.clone().join("mark"),
            bark,
        };
    }

    pub fn report_debug(&self, message: &str) {
        if self.debug {
            println!("{}", message);
        }
    }

    pub fn report_info(&self, message: &str) {
        println!("{}", message);
    }

    pub fn report_error(&self, message: &str) {
        eprintln!("{}", message);
    }

    pub fn is_marked(&self, tag: &str, comic_id: &str, chapter_id: &str) -> bool {
        return self
            .mark
            .clone()
            .join(&format!("{}_{}_{}", tag, comic_id, chapter_id))
            .exists();
    }

    pub async fn mark(&self, tag: &str, comic_id: &str, chapter_id: &str) -> io::Result<()> {
        let path = self.mark.clone();
        tokio::fs::create_dir_all(path.clone()).await?;
        let path = path.join(&format!("{}_{}_{}", tag, comic_id, chapter_id));
        TokioFile::create(path).await?;
        return Ok(());
    }

    pub async fn create_image_cache(
        &self,
        tag: &str,
        comic_id: &str,
        chapter_id: &str,
        index: usize,
        extension: &str,
    ) -> io::Result<PathBuf> {
        let cache_image = self.cache.clone();
        tokio::fs::create_dir_all(&cache_image).await?;
        return cache_image
            .join(format!(
                "{}_{}_{}_{}.{}",
                tag, comic_id, chapter_id, index, extension
            ))
            .into_ok();
    }

    pub fn document_repo_path(&self) -> &Path {
        return &self.repo;
    }

    pub async fn notify(&self, title: &str, content: &str) {
        // Bark
        if let Some(bark) = &self.bark {
            let base = Path::new(bark)
                .join(url_escape::encode_component(title).to_string())
                .join(url_escape::encode_component(content).to_string())
                .display()
                .to_string();
            let url = format!("{}?icon={}&group={}", base, ICON_URL, APP_NAME_TITLEIZE);
            self.report_debug(&format!("notify Bark: {}", &url));
            if let Err(error) = reqwest::get(url).await {
                self.report_error(&format!("failed to notify Bark: {}", error));
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        block_on(async {
            self.report_debug("start clean up context");
            if self.cache.exists() {
                if let Err(error) = tokio::fs::remove_dir_all(&self.cache).await {
                    self.report_error(&format!("failed to clean up cache: {}", error));
                }
            }
            self.report_debug("complete clean up context");
        });
    }
}
