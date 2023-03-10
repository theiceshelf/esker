// use crate::util;
use crate::{site::Site, util};
use chrono::prelude::{DateTime, Local, NaiveDateTime};
use chrono::ParseError;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub title: String,
    filepath: PathBuf,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub publish: bool,
    pub date_created: NaiveDateTime,
    pub date_created_timestamp: i64,
    pub date_updated: NaiveDateTime,
    pub date_updated_timestamp: i64,
    pub template: String,
    pub in_sitemap: bool,
}

impl Frontmatter {
    pub fn new(site: &mut Site, md_file_path: &PathBuf) -> Option<Frontmatter> {
        let metadata = fs::metadata(md_file_path).unwrap();
        let mut has_valid_fm = true;

        let date_created = metadata.created().expect("failed to get created time.");
        let date_created: DateTime<Local> = date_created.into();
        let date_created: NaiveDateTime = date_created.naive_local();

        let date_updated = metadata.modified().expect("failed to get modified time.");
        let date_updated: DateTime<Local> = date_updated.into();
        let date_updated: NaiveDateTime = date_updated.naive_local();

        let mut fm = Frontmatter {
            title: std::ffi::OsString::into_string(
                md_file_path.file_stem().unwrap().to_os_string(),
            )
            .unwrap(),
            filepath: md_file_path.clone(),
            date_created,
            date_created_timestamp: date_created.timestamp(),
            date_updated,
            date_updated_timestamp: date_updated.timestamp(),
            summary: None,
            publish: true,
            tags: Vec::new(),
            template: String::from(""),
            in_sitemap: true,
        };

        let mut capturing = false;

        if let Ok(lines) = util::read_lines(md_file_path) {
            for line in lines.flatten() {
                if line != "---" && !capturing {
                    has_valid_fm = false;
                    break;
                }

                if line == "---" && !capturing {
                    capturing = true;
                    continue;
                }

                fm.get_key_value_from_line(&line, site);

                if line == "---" && capturing {
                    break;
                }
            }
        }

        if has_valid_fm {
            Some(fm)
        } else {
            None
        }
    }

    fn match_possible_dates(date_str: &str) -> Result<NaiveDateTime, ParseError> {
        if let Ok(dc) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            return Ok(dc);
        }

        if let Ok(dc) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M") {
            return Ok(dc);
        }

        match NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(dc) => Ok(dc),
            Err(_e) => {
                // try adding extra HH/MM to datestring to see if that works, otherwise giveup.
                let mut new_date_str_attempt = String::from(date_str);
                new_date_str_attempt.push_str(" 17:00");
                match NaiveDateTime::parse_from_str(&new_date_str_attempt, "%Y-%m-%d %H:%M") {
                    Ok(res) => Ok(res),
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub fn get_key_value_from_line(&mut self, line: &str, site: &mut Site) {
        if let Some((key, val)) = line.split_once(':') {
            let lhs = key.trim();
            let rhs = val.trim();

            match lhs {
                "title" => self.title = rhs.trim().to_string(),
                "date_created" => match Self::match_possible_dates(rhs) {
                    Ok(res) => {
                        self.date_created = res;
                        self.date_created_timestamp = res.timestamp();
                    }
                    Err(_) => {
                        site.errors
                            .add_invalid_date_created(self.get_filepath_as_str());
                    }
                },

                "date_updated" => match Self::match_possible_dates(rhs) {
                    Ok(res) => {
                        self.date_updated = res;
                        self.date_updated_timestamp = res.timestamp()
                    }
                    Err(_) => {
                        site.errors
                            .add_invalid_date_updated(self.get_filepath_as_str());
                    }
                },
                "summary" => {
                    self.summary = Some(rhs.to_string());
                }
                "template" => {
                    self.template = rhs.to_string();
                }
                "publish" => {
                    self.publish = rhs != "false"
                }

                "tag" | "tags" => {
                    let tags = rhs.split(',');
                    let vec: Vec<_> = tags
                        .collect::<Vec<&str>>()
                        .iter()
                        .map(|tag| tag.trim().to_string())
                        .filter(|tag| *tag != "")
                        .collect();
                    self.tags = vec;
                }
                _ => (),
            }
        }
    }

    pub fn date_modified_str(&self) -> String {
        return self.date_updated.format("%Y-%m-%d %H:%M").to_string();
    }

    pub fn get_filepath_as_str(&self) -> String {
        self.filepath
            .clone()
            .into_os_string()
            .into_string()
            .unwrap()
    }
}
