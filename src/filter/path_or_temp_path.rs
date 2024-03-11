use std::hash::Hash;
use std::sync::Arc;

use super::PathOrIdentifier;

#[derive(Debug, Clone)]
pub enum PathOrTempPath {
    Path(std::path::PathBuf),
    TempPath(Arc<tempfile::TempDir>),
}

impl PartialEq for PathOrTempPath {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for PathOrTempPath {}

impl Hash for PathOrTempPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state)
    }
}

impl PathOrTempPath {
    pub fn from_path_or_identifier(
        tmp_path: &std::path::Path,
        path_or_identifier: &Option<PathOrIdentifier>,
    ) -> std::io::Result<Self> {
        if let Some(path_or_identifier) = path_or_identifier {
            match path_or_identifier {
                PathOrIdentifier::Path(path) => Ok(PathOrTempPath::Path(path.clone())),
                PathOrIdentifier::Identifier(identifier) => Ok(Self::TempPath(
                    tempfile::TempDir::with_prefix_in(identifier, tmp_path)?.into(),
                )),
            }
        } else {
            Ok(Self::TempPath(tempfile::TempDir::new_in(tmp_path)?.into()))
        }
    }

    pub fn path(&self) -> &std::path::Path {
        match self {
            Self::Path(path) => path,
            Self::TempPath(path) => path.path(),
        }
    }
}
