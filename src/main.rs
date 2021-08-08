use std::{fs, io};
use comrak::{parse_document, format_html, Arena, ComrakOptions};

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

  for path in news_paths {
    let raw_content = fs::read(path)?;
    let content = String::from_utf8(raw_content).unwrap();
    let rendered = render_markdown(&content)?;
    println!("{}", rendered.to_owned());
  };

  Ok(())
}
