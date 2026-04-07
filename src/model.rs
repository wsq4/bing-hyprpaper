use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Query<'a> {
    pub format: &'a str,
    pub idx: u8,
    pub n: u8,
    pub mkt: &'a str,
    pub uhd: u8,
    pub uhdwidth: u32,
    pub uhdheight: u32,
}

    // {
    //   "startdate": "20251011",
    //   "fullstartdate": "202510110700",
    //   "enddate": "20251012",
    //   "url": "/th?id=OHR.WoodDuckHen_EN-US0382439406_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp",
    //   "urlbase": "/th?id=OHR.WoodDuckHen_EN-US0382439406",
    //   "copyright": "Wood duck hen (© ps50ace/iStock/Getty Images)",
    //   "copyrightlink": "https://www.bing.com/search?q=World+Migratory+Bird+Day&form=hpcapt&filters=HpDate%3a%2220251011_0700%22",
    //   "title": "Nest stop, Mexico!",
    //   "quiz": "/search?q=Bing+homepage+quiz&filters=WQOskey:%22HPQuiz_20251011_WoodDuckHen%22&FORM=HPQUIZ",
    //   "wp": true,
    //   "hsh": "383a612d3d17f82b406b9bd4c16b01ef",
    //   "drk": 1,
    //   "top": 1,
    //   "bot": 1,
    //   "hs": []
    // },

#[derive(Debug, Deserialize)]
pub struct Image {
    pub startdate: String,
    // pub fullstartdate: String,
    // pub enddate: String,
    pub url: String,
    // pub urlbase: String,
    pub copyright: String,
    pub copyrightlink: String,
    pub title: String,
    // pub quiz: String,
    // pub wp: bool,
    pub hsh: String,
    // pub drk: u8,
    // pub top: u8,
    // pub bot: u8,
    // pub hs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageMetaData {
    pub copyright: String,
    pub copyrightlink: String,
    pub title: String,
}

impl From<& Image> for  ImageMetaData{
    fn from(value: & Image) -> Self {
        ImageMetaData {
            copyright: value.copyright.clone(),
            copyrightlink: value.copyrightlink.clone(),
            title: value.title.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub images: Vec<Image>,
    // pub tooltips: serde_json::Value,
}

pub struct ImageStoreItem {
    pub path: PathBuf,
    pub file_created: std::time::SystemTime,
}