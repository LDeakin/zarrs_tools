use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PathOrIdentifier {
    Path(std::path::PathBuf),
    Identifier(String),
}

pub fn parse_path_or_identifier(path_or_id: &str) -> std::io::Result<PathOrIdentifier> {
    if path_or_id.starts_with('$') {
        Ok(PathOrIdentifier::Identifier(path_or_id.to_string()))
    } else {
        Ok(PathOrIdentifier::Path(PathBuf::from(&path_or_id)))
    }
}

impl<'de> serde::Deserialize<'de> for PathOrIdentifier {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let path_or_id = String::deserialize(d)?;
        parse_path_or_identifier(&path_or_id)
            .map_err(|_| serde::de::Error::custom("could not create temporary directory"))
    }
}
