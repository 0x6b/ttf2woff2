pub const TTF_FLAVOR: u32 = 0x00010000;
pub const CFF_FLAVOR: u32 = 0x4F54544F; // 'OTTO'

pub struct SfntTable {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

pub struct Sfnt {
    pub flavor: u32,
    pub tables: Vec<SfntTable>,
}

impl Sfnt {
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 12 {
            return Err("Data too short for SFNT header");
        }

        let flavor = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;

        if flavor != TTF_FLAVOR && flavor != CFF_FLAVOR {
            return Err("Invalid SFNT flavor");
        }

        let required_len = 12 + num_tables * 16;
        if data.len() < required_len {
            return Err("Data too short for table directory");
        }

        let mut tables = Vec::with_capacity(num_tables);
        for i in 0..num_tables {
            let offset = 12 + i * 16;
            let tag = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            let checksum = u32::from_be_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            let table_offset = u32::from_be_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]);
            let length = u32::from_be_bytes([
                data[offset + 12],
                data[offset + 13],
                data[offset + 14],
                data[offset + 15],
            ]);

            let end = table_offset as usize + length as usize;
            if end > data.len() {
                return Err("Table extends beyond data");
            }

            tables.push(SfntTable { tag, checksum, offset: table_offset, length });
        }

        Ok(Self { flavor, tables })
    }

    pub fn get_table_data<'a>(&self, tag: &[u8; 4], data: &'a [u8]) -> Option<&'a [u8]> {
        self.tables.iter().find(|t| &t.tag == tag).map(|t| {
            let start = t.offset as usize;
            let end = start + t.length as usize;
            &data[start..end]
        })
    }
}
