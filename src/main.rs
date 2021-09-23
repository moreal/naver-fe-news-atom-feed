mod git;
mod error;

use atom_syndication::{
  CategoryBuilder, Content, Entry, EntryBuilder, FeedBuilder, FixedDateTime, LinkBuilder,
};
use chrono::{FixedOffset, TimeZone};
use comrak::{format_html, parse_document, Arena, ComrakOptions};
use std::{
  fs::{self, File},
  io,
  path::Path,
  time::UNIX_EPOCH,
};

use crate::error::Error;
use crate::git::created_at;

const FENEWS_GIT_DIRECTORY: &str = "./fe-news";
const FENEWS_PATH: &str = "./fe-news/issues";
const FENEWS_GITHUB_URL: &str = "https://github.com/naver/fe-news";
const NAVER_GITHUB_LOGO_URL: &str = "https://avatars.githubusercontent.com/u/6589568?s=200&v=4";

fn render_markdown(content: &str) -> io::Result<String> {
    let arena = Arena::new();

    let root = parse_document(&arena, content, &ComrakOptions::default());

    let mut html = vec![];
    format_html(root, &ComrakOptions::default(), &mut html).unwrap();

    Ok(String::from_utf8(html).unwrap())
}

fn main() -> std::result::Result<(), Error> {
    let news_entries = fs::read_dir(FENEWS_PATH)?
        .map(|res| res.map(|e| (e.file_name(), e.path(), e.metadata().unwrap())))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let feed_file = File::create("index.xml")?;
    let mut feed_builder = FeedBuilder::default()
        .title("fe-news")
        .icon(NAVER_GITHUB_LOGO_URL.to_owned())
        .logo(NAVER_GITHUB_LOGO_URL.to_owned())
        .categories(vec![
            CategoryBuilder::default().term("technology").build(),
            CategoryBuilder::default().term("web").build(),
            CategoryBuilder::default().term("frontend").build(),
        ])
        .to_owned();
    let mut entries: Vec<Entry> = vec![];
    let mut latest_updated: Option<FixedDateTime> = None;
    for (filename, path, metadata) in news_entries {
        let raw_content = fs::read(path.to_owned())?;
        let markdown_content = String::from_utf8(raw_content).unwrap();
        let rendered = render_markdown(&markdown_content)?;

        let created: FixedDateTime = created_at(FENEWS_GIT_DIRECTORY, filename.to_str().unwrap());
        let duration = metadata.modified()?.duration_since(UNIX_EPOCH)?;
        let updated: FixedDateTime =
            FixedOffset::east(0).timestamp(duration.as_secs() as i64, duration.subsec_nanos());

        if let Some(latest) = latest_updated {
            if latest.cmp(&updated) == std::cmp::Ordering::Less {
                latest_updated = Some(updated);
            }
        } else {
            latest_updated = Some(updated);
        }

        let mut content = Content::default();
        content.set_content_type("html".to_owned());
        content.set_value(html_escape::encode_text(&rendered).to_string());

        let filename = filename.to_str().to_owned().unwrap();
        let title = Path::new(filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let news_github_url =
            format!("{}/blob/master/issues/{}", FENEWS_GITHUB_URL, filename).to_owned();
        let entry = EntryBuilder::default()
            .id(&news_github_url)
            .title(title)
            .content(content)
            .updated(updated)
            .published(created)
            .links(vec![LinkBuilder::default().href(&news_github_url).build()])
            .build();

        entries.push(entry);
    }

    feed_builder
        .entries(entries)
        .updated(latest_updated.unwrap())
        .build()
        .write_to(feed_file)?;

    Ok(())
}
