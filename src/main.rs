use std::{fs::{self, File}, io};
use comrak::{parse_document, format_html, Arena, ComrakOptions};
use atom_syndication::{Content, Entry, EntryBuilder, FeedBuilder, Text};

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
  let news_paths = fs::read_dir(FENEWS_PATH)?
    .map(|res| res.map(|e| e.path()))
    .collect::<Result<Vec<_>, io::Error>>()?;

  let feed_file = File::create("index.xml")?;
  let mut feed_builder = FeedBuilder::default().title("fe-news").to_owned();
  let mut entries: Vec<Entry> = vec![];
  for path in news_paths {
    let raw_content = fs::read(path.to_owned())?;
    let markdown_content = String::from_utf8(raw_content).unwrap();
    let rendered = render_markdown(&markdown_content)?;

    let title = Text::plain(path.to_str().to_owned().unwrap());
    let mut content = Content::default();
    content.set_content_type("xhtml".to_owned());
    content.set_value(rendered);

    let entry = EntryBuilder::default()
      .title(title)
      .content(content)
      .build();

    entries.push(entry);
  };

  feed_builder.entries(entries);
  feed_builder.build().write_to(feed_file);

  Ok(())
}
