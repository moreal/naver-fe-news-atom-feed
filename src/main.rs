use std::{fs::{self, File}, io, time::UNIX_EPOCH};
use comrak::{parse_document, format_html, Arena, ComrakOptions};
use atom_syndication::{Content, Entry, EntryBuilder, FeedBuilder, FixedDateTime, Text};
use chrono::{FixedOffset, TimeZone};

const FENEWS_PATH: &str = "./fe-news/issues";

fn render_markdown(content: &str) -> io::Result<String> {
  let arena = Arena::new();

  let root = parse_document(
    &arena,
    content,
    &ComrakOptions::default());

  let mut html = vec![];
  format_html(root, &ComrakOptions::default(), &mut html).unwrap();

  Ok(String::from_utf8(html).unwrap().to_owned())
}

fn main() -> io::Result<()> {
  let news_entries = fs::read_dir(FENEWS_PATH)?
    .map(|res| res.map(|e| (e.path(), e.metadata().unwrap())))
    .collect::<Result<Vec<_>, io::Error>>()?;

  let feed_file = File::create("index.xml")?;
  let mut feed_builder = FeedBuilder::default().title("fe-news").to_owned();
  let mut entries: Vec<Entry> = vec![];
  let mut latest_updated: Option<FixedDateTime> = None;
  for (path, metadata) in news_entries {
    let raw_content = fs::read(path.to_owned())?;
    let markdown_content = String::from_utf8(raw_content).unwrap();
    let rendered = render_markdown(&markdown_content)?;

    let title = Text::plain(path.to_str().to_owned().unwrap());
    let updated: FixedDateTime = match metadata.modified()?.duration_since(UNIX_EPOCH) {
      Ok(duration) => FixedOffset::east(0).timestamp(duration.as_secs() as i64, duration.subsec_nanos()),
      Err(_) => {
        return Err(io::Error::new(io::ErrorKind::Other, "error"));
      }
    };

    if let Some(latest) = latest_updated {
      match latest.cmp(&updated) {
        std::cmp::Ordering::Greater => latest_updated = Some(updated),
        _ => ()
      };
    } else {
      latest_updated = Some(updated);
    }

    let mut content = Content::default();
    content.set_content_type("xhtml".to_owned());
    content.set_value(rendered);

    let entry = EntryBuilder::default()
      .title(title)
      .content(content)
      .updated(updated)
      .build();

    entries.push(entry);
  };

  feed_builder
    .entries(entries)
    .updated(latest_updated.unwrap())
    .build()
    .write_to(feed_file);

  Ok(())
}
