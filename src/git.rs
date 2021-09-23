use std::process::Command;

use atom_syndication::FixedDateTime;

pub fn created_at(git_directory: &str, filename: &str) -> FixedDateTime {
    let output = Command::new("git")
                .arg("log")
                .arg("--format=\"%aD\"")
                .arg(format!("issues/{}", filename))
                .current_dir(git_directory)
                .output()
                .expect("12");
    let output = String::from_utf8(output.stdout).unwrap();
    let datetime_string = output.rsplit('\n').nth(1).unwrap();

    convert_iso8601(datetime_string)
}

fn convert_iso8601(datetime: &str) -> FixedDateTime {
    let output = Command::new("date")
                .arg(format!(
                    "--date={}",
                    datetime[1..datetime.len() - 1].to_owned()
                ))
                .arg("--iso-8601=seconds")
                .arg("--utc")
                .output()
                .expect("12");
    let output = String::from_utf8(output.stdout).unwrap();
    output.parse::<FixedDateTime>().unwrap()
}
