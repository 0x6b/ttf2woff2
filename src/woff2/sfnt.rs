use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt};

use super::tag::Tag;
use crate::Error;

const TTF_FLAVOR: u32 = 0x00010000;

pub(crate) struct SfntTable {
    pub tag: Tag,
    pub offset: u32,
    pub length: u32,
}

pub(crate) struct Sfnt {
    pub flavor: u32,
    pub tables: Vec<SfntTable>,
}

impl Sfnt {
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        let mut cursor = Cursor::new(data);

        let flavor = cursor
            .read_u32::<BigEndian>()
            .map_err(|_| Error::DataTooShort { context: "SFNT header" })?;
        let num_tables = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| Error::DataTooShort { context: "SFNT header" })?
            as usize;

        // Skip search_range, entry_selector, range_shift (6 bytes)
        cursor.set_position(12);

        if flavor != TTF_FLAVOR {
            return Err(Error::UnsupportedFormat);
        }

        let mut tables = Vec::with_capacity(num_tables);
        for _ in 0..num_tables {
            let mut tag_bytes = [0u8; 4];
            cursor
                .read_exact(&mut tag_bytes)
                .map_err(|_| Error::DataTooShort { context: "table directory" })?;
            let _checksum = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| Error::DataTooShort { context: "table directory" })?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| Error::DataTooShort { context: "table directory" })?;
            let length = cursor
                .read_u32::<BigEndian>()
                .map_err(|_| Error::DataTooShort { context: "table directory" })?;

            let end = offset as usize + length as usize;
            if end > data.len() {
                return Err(Error::TableOutOfBounds);
            }

            tables.push(SfntTable { tag: Tag(tag_bytes), offset, length });
        }

        Ok(Self { flavor, tables })
    }
}
