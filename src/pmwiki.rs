use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(not(feature = "html"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IPmwiki<'a> {
    Text(&'a str),
    Line(Vec<IPmwiki<'a>>),
    Bold(Vec<IPmwiki<'a>>),
    Italic(Vec<IPmwiki<'a>>),
    #[cfg(feature = "font-color")]
    Color(&'a str, &'a str),
    BulletList(Vec<IPmwiki<'a>>),
    NumberedList(Vec<IPmwiki<'a>>),
    ListItem(Vec<IPmwiki<'a>>),
    Link(&'a str, &'a str),
    Heading(u8, Vec<IPmwiki<'a>>),
    Silentbreak,
    ForceLinebreak,
    HorizontalLine,
    #[cfg(feature = "fold")]
    Fold(&'a str),
    Image(&'a str, &'a str),
    #[cfg(feature = "link-button")]
    LinkButton(&'a str, &'a str, &'a str),
    DontFormat(&'a str),
    Table(Vec<IPmwiki<'a>>),
    TableHeaderRow(Vec<IPmwiki<'a>>),
    TableHeaderCell(Vec<IPmwiki<'a>>),
    TableRow(Vec<IPmwiki<'a>>),
    TableCell(Vec<IPmwiki<'a>>),
}

impl Eq for IPmwiki<'_> {}
