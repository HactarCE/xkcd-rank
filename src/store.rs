use std::path::PathBuf;

use eyre::{ensure, Result};
use serde::{Deserialize, Serialize};

/// Store of downloaded comics.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Store {
    pub comics: Vec<Option<Comic>>,
    pub tier_assignments: Vec<u8>,
    #[serde(skip)]
    pub unsaved: bool,
}
impl Store {
    pub fn load() -> Self {
        std::fs::read_to_string(crate::util::comics_json_path())
            .ok()
            .and_then(|json_string| serde_json::from_str(&json_string).ok())
            .unwrap_or_default()
    }

    pub fn save(&mut self) {
        let path = crate::util::comics_json_path();
        let contents = serde_json::to_string(self).expect("error serializing data store");
        match std::fs::write(path, contents) {
            Ok(_) => self.unsaved = false,
            Err(e) => eprintln!("error saving data store: {e}"),
        }
    }

    pub fn has_comic(&self, i: usize) -> bool {
        self.comics.get(i).is_some_and(|entry| entry.is_some())
    }

    pub fn fetch_comic(&mut self, i: usize) -> Result<&Comic> {
        ensure!(i > 0, "comic #0 doesn't exist");

        while self.comics.len() <= i {
            self.comics.push(None);
        }

        match &self.comics[i] {
            Some(_) => Ok(self.comics[i].as_ref().unwrap()),
            None => Ok(self.comics[i].insert(Comic::get_nth(i)?)),
        }
    }

    pub fn ensure_tiers_exist(&mut self) {
        if self.tier_assignments.len() <= self.comics.len() {
            self.tier_assignments.resize(self.comics.len() + 1, 0);
        }
    }

    pub fn get_tier_of_comic(&self, i: usize) -> u8 {
        *self.tier_assignments.get(i).unwrap_or(&0)
    }
    pub fn set_tier_of_comic(&mut self, i: usize, tier: u8) {
        self.unsaved = true;
        self.ensure_tiers_exist();
        if i < self.comics.len() {
            self.tier_assignments[i] = tier;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comic {
    pub num: usize,

    pub year: String,
    pub month: String,
    pub day: String,

    pub title: String,
    pub img: String,
    pub alt: String,
    pub link: String,

    pub news: String,

    pub safe_title: String,
    pub transcript: String,
}
impl Comic {
    pub fn img_path(&self) -> PathBuf {
        let image_ext = self.img.rsplit_once('.').unwrap_or((&self.img, "")).1;
        crate::util::img_path(self.num, image_ext)
    }

    pub fn has_image_downloaded(&self) -> bool {
        std::fs::exists(self.img_path()).unwrap_or(false)
    }

    pub fn download_image(&self) -> Result<()> {
        std::fs::create_dir_all("cache/img")?;

        let mut buffer = vec![];
        ureq::get(&self.img_2x().unwrap_or_default())
            .call()
            .or_else(|_| ureq::get(&self.img).call())?
            .into_reader()
            .read_to_end(&mut buffer)?;

        std::fs::write(self.img_path(), buffer)?;

        Ok(())
    }

    pub fn get_from_url(url: &str) -> Result<Comic> {
        Ok(ureq::get(url).call()?.into_json()?)
    }
    fn get_nth(n: usize) -> Result<Comic> {
        if n == 404 {
            return Ok(Comic {
                num: 404,
                year: "2008".to_owned(),
                month: "4".to_owned(),
                day: "1".to_owned(),
                title: "404 Not Found".to_owned(),
                img: String::new(),
                alt: "I have always been of the opinion that http://xkcd.com/404/ is an actual comic, if a slightly avant-garde one. I actually went out of my way to modify the 'random' button to include it, but that annoyed too many people—most of whom reasonably assumed it was a bug—and I eventually undid it.".to_owned(),
                link: String::new(),
                news: String::new(),
                safe_title: "404 Not Found".to_owned(),
                transcript: "nginx".to_owned(),
            });
        }

        Self::get_from_url(&format!("https://xkcd.com/{n}/info.0.json"))
    }

    fn img_2x(&self) -> Option<String> {
        Some(self.img.strip_suffix(".png")?.to_owned() + "_2x.png")
    }
}
