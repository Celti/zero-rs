use bytes::buf::Reader;
use chrono::{DateTime, Duration, Utc};
use crate::schema::*;
use crate::{SALT, URL};
use data_encoding::BASE64URL_NOPAD as BASE64;
//use diesel::{AsChangeset, Identifiable, Insertable, Queryable}; // FIXME Rust 2018 / Diesel 1.4+
use magic::Cookie;
use multipart::server::Multipart;
use ring::digest::{digest, SHA256};
use std::io::Read;
use warp::{body::FullBody, Rejection};

#[derive(Clone, Debug, Default, AsChangeset, Identifiable, Insertable, Queryable)]
pub struct Item {
    pub id:        i64,
    pub content:   Vec<u8>,
    pub filename:  String,
    pub mimetype:  String,
    pub digest:    String,
    pub label:     String,
    pub destruct:  bool,
    pub private:   bool,
    pub is_url:    bool,
    pub sunset:    Option<DateTime<Utc>>,
    pub timestamp: Option<DateTime<Utc>>,
}

impl Item {
    pub fn new_with_id(id: i64) -> Item {
        Item {
            id,
            ..Default::default()
        }
    }

    pub fn read_multipart_body(
        mut self,
        mut data: Multipart<Reader<FullBody>>,
    ) -> Result<Item, Rejection> {
        while let Ok(Some(mut entry)) = data.read_entry() {
            match entry.headers.name.as_ref().as_ref() {
                "c" => {
                    if self.is_url {
                        continue;
                    }

                    let cookie = Cookie::open(magic::flags::MIME)
                        .map_err(|_| warp::reject::server_error())?;
                    cookie
                        .load(&["/usr/share/file/misc/magic.mgc"])
                        .map_err(|_| warp::reject::server_error())?;

                    let count = entry
                        .data
                        .read_to_end(&mut self.content)
                        .map_err(|_| warp::reject::server_error())?;

                    if count == 0 {
                        continue;
                    }

                    let salted = [self.content.as_slice(), SALT.as_slice()].concat();

                    self.digest = BASE64.encode(digest(&SHA256, &salted).as_ref());
                    self.filename = entry.headers.filename.unwrap_or_default();
                    self.mimetype = cookie
                        .buffer(&self.content)
                        .unwrap_or_else(|_| String::from("application/octet-stream"));
                }
                "u" => {
                    if self.content.is_empty() || self.is_url {
                        let mut buf = String::new();
                        let count = entry
                            .data
                            .read_to_string(&mut buf)
                            .map_err(|_| warp::reject::server_error())?;

                        if count == 0 {
                            continue;
                        }

                        self.is_url = true;
                        self.content = Vec::from(buf.trim().as_bytes());

                        let salted = [self.content.as_slice(), SALT.as_slice()].concat();

                        self.digest = BASE64.encode(digest(&SHA256, &salted).as_ref());
                        self.mimetype = String::from("text/uri-list");
                    } else {
                        return Err(warp::reject::bad_request());
                    }

                }
                "destruct" => {
                    let mut buf = String::new();
                    let count = entry
                        .data
                        .read_to_string(&mut buf)
                        .map_err(|_| warp::reject::bad_request())?;

                    if count == 0 {
                        continue;
                    }

                    self.destruct = buf.parse().map_err(|_| warp::reject::bad_request())?;
                }
                "private" => {
                    let mut buf = String::new();
                    let count = entry
                        .data
                        .read_to_string(&mut buf)
                        .map_err(|_| warp::reject::bad_request())?;

                    if count == 0 {
                        continue;
                    }

                    self.private = buf.parse().map_err(|_| warp::reject::bad_request())?;
                }
                "sunset" => {
                    let mut buf = String::new();
                    let count = entry
                        .data
                        .read_to_string(&mut buf)
                        .map_err(|_| warp::reject::bad_request())?;

                    if count == 0 {
                        continue;
                    }

                    let duration =
                        Duration::minutes(buf.parse().map_err(|_| warp::reject::bad_request())?);
                    self.sunset = Some(Utc::now() + duration);
                }
                "label" => {
                    let mut buf = String::new();
                    let count = entry
                        .data
                        .read_to_string(&mut buf)
                        .map_err(|_| warp::reject::bad_request())?;

                    if count == 0 {
                        continue;
                    }

                    if buf.starts_with('~') {
                        self.label = buf;
                    } else {
                        self.label = format!("~{}", buf);
                    }
                }
                _ => return Err(warp::reject::bad_request()),
            }
        }

        if self.label.is_empty() {
            let lz = self.id.leading_zeros();
            let idx = ((71 - lz) / 8) as usize;
            self.label = BASE64.encode(&self.id.to_ne_bytes()[0..idx]);
        }

        if self.filename.is_empty() {
            if let Some(ext) = mime_guess::get_mime_extensions_str(&self.mimetype) {
                self.filename = format!("{}.{}", self.label, ext[0]);
            } else {
                self.filename = String::from(self.label.trim_left_matches('~'));
            }
        }

        Ok(self)
    }

    pub fn url(&self) -> String {
        if self.private {
            format!("https://{}/+{}", *URL, self.digest)
        } else {
            format!("https://{}/{}", *URL, self.label)
        }
    }
}
