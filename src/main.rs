use atom_syndication::{
    CategoryBuilder, Content, Entry, EntryBuilder, FeedBuilder, FixedDateTime, LinkBuilder,
};
use chrono::{FixedOffset, TimeZone};
use comrak::{format_html, parse_document, Arena, ComrakOptions};
use std::process::Command;
use std::{
    fs::{self, File},
    io,
    path::Path,
    time::{SystemTimeError, UNIX_EPOCH},
};

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

        let created: FixedDateTime = {
            let output = Command::new("git")
                .arg("log")
                .arg("--format=\"%aD\"")
                .arg(format!("issues/{}", filename.to_str().to_owned().unwrap()))
                .current_dir(FENEWS_GIT_DIRECTORY)
                .output()
                .expect("12");

            let output = String::from_utf8(output.stdout).unwrap();

            let datetime_string = output.rsplit('\n').nth(1).unwrap();

            let output = Command::new("date")
                .arg(format!(
                    "--date={}",
                    datetime_string[1..datetime_string.len() - 1].to_owned()
                ))
                .arg("--iso-8601=seconds")
                .arg("--utc")
                .output()
                .expect("12");
            let datetime_string = String::from_utf8(output.stdout).unwrap();

            datetime_string.parse::<FixedDateTime>().unwrap()
        };
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

struct Error {
    reason: String,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("reason", &self.reason)
            .finish()
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}

impl From<atom_syndication::Error> for Error {
    fn from(e: atom_syndication::Error) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}
