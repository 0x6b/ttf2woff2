use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt};

use super::tag::Tag;
use crate::{Error, Error::DataTooShort};

const TTF_FLAVOR: u32 = 0x00010000;
const OTTO_FLAVOR: u32 = 0x4F54544F; // "OTTO" — CFF-flavored OpenType

pub(crate) struct SfntTable {
    pub tag: Tag,
    pub offset: u32,
    pub length: u32,
}

pub(crate) struct Sfnt {
    pub flavor: u32,
    pub tables: Vec<SfntTable>,
}

impl TryFrom<&[u8]> for Sfnt {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(data);

        let flavor = cursor
            .read_u32::<BigEndian>()
            .map_err(|_| DataTooShort { context: "SFNT header" })?;
        let num_tables = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| DataTooShort { context: "SFNT header" })?
            as usize;

        cursor.set_position(12);

        if flavor != TTF_FLAVOR && flavor != OTTO_FLAVOR {
            return Err(Error::UnsupportedFormat);
        }

        let mut tables = Vec::with_capacity(num_tables);
        for _ in 0..num_tables {
            let mut tag_bytes = [0u8; 4];
            cursor
                .read_exact(&mut tag_bytes)
                .map_err(|_| DataTooShort { context: "table directory" })?;
            let _checksum = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| DataTooShort { context: "table directory" })?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| DataTooShort { context: "table directory" })?;
            let length = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| DataTooShort { context: "table directory" })?;

            let end = offset as usize + length as usize;
            if end > data.len() {
                return Err(Error::TableOutOfBounds);
            }

            tables.push(SfntTable { tag: Tag(tag_bytes), offset, length });
        }

        Ok(Self { flavor, tables })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_sfnt(flavor: u32) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&flavor.to_be_bytes());
        data.extend_from_slice(&1u16.to_be_bytes()); // numTables
        data.extend_from_slice(&[0u8; 6]); // searchRange etc.
        data.extend_from_slice(b"cmap"); // tag
        data.extend_from_slice(&0u32.to_be_bytes()); // checksum
        data.extend_from_slice(&12u32.to_be_bytes()); // offset
        data.extend_from_slice(&0u32.to_be_bytes()); // length
        data
    }

    #[test]
    fn accepts_ttf_flavor() {
        let data = minimal_sfnt(0x00010000);
        assert!(Sfnt::try_from(&data[..]).is_ok());
    }

    #[test]
    fn accepts_otto_flavor() {
        let data = minimal_sfnt(0x4F54544F);
        let sfnt = Sfnt::try_from(&data[..]).unwrap();
        assert_eq!(sfnt.flavor, 0x4F54544F);
        assert_eq!(sfnt.tables.len(), 1);
        assert_eq!(sfnt.tables[0].tag, Tag(*b"cmap"));
    }

    #[test]
    fn rejects_true_flavor() {
        let data = minimal_sfnt(0x74727565); // "true"
        assert!(matches!(Sfnt::try_from(&data[..]), Err(Error::UnsupportedFormat)));
    }

    #[test]
    fn rejects_garbage_flavor() {
        let data = minimal_sfnt(0xDEADBEEF);
        assert!(matches!(Sfnt::try_from(&data[..]), Err(Error::UnsupportedFormat)));
    }

    #[test]
    fn rejects_short_data() {
        let data = [0u8; 4];
        assert!(matches!(Sfnt::try_from(&data[..]), Err(Error::DataTooShort { .. })));
    }
}
