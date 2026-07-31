//! Recipe `explorerMenu` — VS Code Explorer context QuickPick opt-in.

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::dsl;

/// Explorer resource kind for a menu entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExplorerMenuKind {
    File,
    Folder,
}

impl ExplorerMenuKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => dsl::recipe::explorer_menu::entry::field::kind::value::FILE,
            Self::Folder => dsl::recipe::explorer_menu::entry::field::kind::value::FOLDER,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            dsl::recipe::explorer_menu::entry::field::kind::value::FILE => Some(Self::File),
            dsl::recipe::explorer_menu::entry::field::kind::value::FOLDER => Some(Self::Folder),
            _ => None,
        }
    }
}

/// One Explorer menu rule: match `kind`, optionally gate on MiniJinja `if` over `path`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplorerMenuEntry {
    pub kind: ExplorerMenuKind,
    #[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_expr: Option<String>,
    /// Recipe arg name → MiniJinja expression over click `path` only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, String>,
}

/// Recipe-level Explorer menu opt-in (list of entries; single object is sugar).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ExplorerMenu {
    pub entries: Vec<ExplorerMenuEntry>,
}

impl ExplorerMenu {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries whose `kind` matches the Explorer click kind.
    pub fn entries_for_kind(&self, kind: ExplorerMenuKind) -> impl Iterator<Item = &ExplorerMenuEntry> {
        self.entries.iter().filter(move |e| e.kind == kind)
    }
}

impl<'de> Deserialize<'de> for ExplorerMenu {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExplorerMenuVisitor;

        impl<'de> Visitor<'de> for ExplorerMenuVisitor {
            type Value = ExplorerMenu;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "explorerMenu as a mapping { kind, if?, args? } or a sequence of such mappings",
                )
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let entry = ExplorerMenuEntry::deserialize(de::value::MapAccessDeserializer::new(
                    map,
                ))?;
                Ok(ExplorerMenu {
                    entries: vec![entry],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<ExplorerMenuEntry>()? {
                    entries.push(entry);
                }
                Ok(ExplorerMenu { entries })
            }
        }

        deserializer.deserialize_any(ExplorerMenuVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_object_sugar() {
        let menu: ExplorerMenu = serde_yaml::from_str(
            r#"
kind: folder
if: path is startingwith("lib/")
"#,
        )
        .unwrap();
        assert_eq!(menu.entries.len(), 1);
        assert_eq!(menu.entries[0].kind, ExplorerMenuKind::Folder);
        assert_eq!(
            menu.entries[0].if_expr.as_deref(),
            Some("path is startingwith(\"lib/\")")
        );
    }

    #[test]
    fn sequence_with_file_and_folder() {
        let menu: ExplorerMenu = serde_yaml::from_str(
            r#"
- kind: folder
- kind: file
  if: path is startingwith("lib/")
"#,
        )
        .unwrap();
        assert_eq!(menu.entries.len(), 2);
        assert_eq!(menu.entries[0].kind, ExplorerMenuKind::Folder);
        assert!(menu.entries[0].if_expr.is_none());
        assert_eq!(menu.entries[1].kind, ExplorerMenuKind::File);
    }

    #[test]
    fn args_map_bindings() {
        let menu: ExplorerMenu = serde_yaml::from_str(
            r#"
kind: file
args:
  file: path
  featureDir: path | parent
  folderName: path | parent | basename
"#,
        )
        .unwrap();
        assert_eq!(menu.entries.len(), 1);
        let args = &menu.entries[0].args;
        assert_eq!(args.get("file").map(String::as_str), Some("path"));
        assert_eq!(
            args.get("featureDir").map(String::as_str),
            Some("path | parent")
        );
        assert_eq!(
            args.get("folderName").map(String::as_str),
            Some("path | parent | basename")
        );
    }
}
