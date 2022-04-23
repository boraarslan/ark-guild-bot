use enum_iterator::IntoEnumIterator;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::io::Read;

use super::*;

#[derive(Deserialize, Clone)]
pub struct ContentInfo {
    pub name: String,
    #[serde(rename(deserialize = "type"))]
    pub content_type: String,
    pub tier: u8,
    pub ilvl_req: i32,
    pub content_size: usize,
    pub image: String,
    pub banner: String,
    pub guide: String,
    pub gameplay_video: String,
    pub introduction: String,
}

// I love iterators
pub static GUARDIAN_RAIDS: Lazy<Vec<Content>> = Lazy::new(|| {
    Content::into_enum_iter()
        .take_while(|x| *x != Content::DemonBeastCanyon)
        .collect()
});

pub static ABYSS_DUNGEONS: Lazy<Vec<Content>> = Lazy::new(|| {
    Content::into_enum_iter()
        .skip_while(|x| *x != Content::DemonBeastCanyon)
        .take_while(|x| *x != Content::Argos1)
        .collect()
});

pub static ABYSS_RAIDS: Lazy<Vec<Content>> = Lazy::new(|| {
    Content::into_enum_iter()
        .skip_while(|x| *x != Content::Argos1)
        .collect()
});

pub static CONTENT_TOML: Lazy<Vec<String>> = Lazy::new(|| {
    let mut contents = vec![];
    for content in Content::into_enum_iter() {
        let mut file = std::fs::File::open(format!("./contents/{}.toml", content)).unwrap();
        let mut content_toml = String::new();
        file.read_to_string(&mut content_toml).unwrap();
        contents.push(content_toml);
    }
    contents
});

pub static CONTENT_DATA: Lazy<HashMap<String, ContentInfo>> = Lazy::new(|| {
    let mut content_map = HashMap::new();
    for (index, content) in Content::into_enum_iter().enumerate() {
        let content_info: ContentInfo = toml::from_str(CONTENT_TOML[index].as_str()).unwrap();
        content_map.insert(content.to_string(), content_info);
    }
    content_map
});

impl From<entity::sea_orm_active_enums::Content> for &ContentInfo {
    fn from(content: entity::sea_orm_active_enums::Content) -> Self {
        CONTENT_DATA.get(&content.to_string()).unwrap()
    }
}
