use crate::util::extension::ResultExtension;
use reqwest::header::{REFERER, USER_AGENT};
use serde::Deserialize;
use std::{error::Error, fs::File, io::Write, path::Path};

#[derive(Deserialize)]
pub(self) struct ComicResp {
    pub data: ComicDataResp,
}

#[derive(Deserialize)]
pub(self) struct ComicDataResp {
    pub info: ComicDataInfoResp,
    pub list: Vec<ComicDataListResp>,
}

#[derive(Deserialize)]
pub(self) struct ComicDataInfoResp {
    pub title: String,
}

#[derive(Deserialize)]
pub(self) struct ComicDataListResp {
    pub id: String,
    pub chapter_name: String,
}

#[derive(Debug)]
pub(crate) struct ComicInfo {
    pub title: String,
    pub chapters: Vec<ComicInfoChapter>,
}

#[derive(Debug)]
pub(crate) struct ComicInfoChapter {
    pub id: String,
    pub title: String,
}

impl From<ComicResp> for ComicInfo {
    fn from(value: ComicResp) -> Self {
        let mut chapters: Vec<ComicInfoChapter> = vec![];
        for chapter_raw in value.data.list {
            let chapter = ComicInfoChapter {
                id: chapter_raw.id,
                title: chapter_raw.chapter_name,
            };
            chapters.push(chapter);
        }
        return ComicInfo {
            title: value.data.info.title,
            chapters,
        };
    }
}

pub(crate) fn search_comic(id: &str) -> Result<ComicInfo, Box<dyn Error>> {
    let response: ComicResp = {
        let response = reqwest::blocking::get(format!(
            "https://api.dmzj.com//dynamic/comicinfo/{}.json",
            id
        ))?
        .text()?;
        serde_json::from_str(&response)?
    };
    return ComicInfo::from(response).into_ok();
}

#[derive(Deserialize)]
pub(self) struct ChapterResp {
    pub page_url: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ChapterInfo {
    pub pages: Vec<String>,
}

impl From<ChapterResp> for ChapterInfo {
    fn from(value: ChapterResp) -> Self {
        return ChapterInfo {
            pages: value.page_url,
        };
    }
}

pub(crate) fn search_chapter(
    comic_id: &str,
    chapter_id: &str,
) -> Result<ChapterInfo, Box<dyn Error>> {
    let response: ChapterResp = {
        let response = reqwest::blocking::get(format!(
            "https://m.dmzj.com/chapinfo/{}/{}.html",
            comic_id, chapter_id
        ))?
        .text()?;
        serde_json::from_str(&response)?
    };
    return ChapterInfo::from(response).into_ok();
}

pub(crate) fn download_image(file: &Path, url: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url)
    .header(REFERER, "http://images.muwai.com/")
    .header(USER_AGENT, "%E5%8A%A8%E6%BC%AB%E4%B9%8B%E5%AE%B6%E7%A4%BE%E5%8C%BA/27 CFNetwork/1329 Darwin/21.3.0")
    .send()?.bytes()?;
    let mut file = File::create(file)?;
    file.write_all(&response)?;
    return Ok(());
}
