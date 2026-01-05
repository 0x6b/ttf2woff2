pub const WOFF2_SIGNATURE: u32 = 0x774F4632; // 'wOF2'

pub struct Woff2Header {
    pub signature: u32,
    pub flavor: u32,
    pub length: u32,
    pub num_tables: u16,
    pub reserved: u16,
    pub total_sfnt_size: u32,
    pub total_compressed_size: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub meta_offset: u32,
    pub meta_length: u32,
    pub meta_orig_length: u32,
    pub priv_offset: u32,
    pub priv_length: u32,
}

impl Woff2Header {
    pub fn to_bytes(&self) -> [u8; 48] {
        let mut bytes = [0u8; 48];
        bytes[0..4].copy_from_slice(&self.signature.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.flavor.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.length.to_be_bytes());
        bytes[12..14].copy_from_slice(&self.num_tables.to_be_bytes());
        bytes[14..16].copy_from_slice(&self.reserved.to_be_bytes());
        bytes[16..20].copy_from_slice(&self.total_sfnt_size.to_be_bytes());
        bytes[20..24].copy_from_slice(&self.total_compressed_size.to_be_bytes());
        bytes[24..26].copy_from_slice(&self.major_version.to_be_bytes());
        bytes[26..28].copy_from_slice(&self.minor_version.to_be_bytes());
        bytes[28..32].copy_from_slice(&self.meta_offset.to_be_bytes());
        bytes[32..36].copy_from_slice(&self.meta_length.to_be_bytes());
        bytes[36..40].copy_from_slice(&self.meta_orig_length.to_be_bytes());
        bytes[40..44].copy_from_slice(&self.priv_offset.to_be_bytes());
        bytes[44..48].copy_from_slice(&self.priv_length.to_be_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_48_bytes() {
        let header = Woff2Header {
            signature: WOFF2_SIGNATURE,
            flavor: 0x00010000,
            length: 0,
            num_tables: 0,
            reserved: 0,
            total_sfnt_size: 0,
            total_compressed_size: 0,
            major_version: 1,
            minor_version: 0,
            meta_offset: 0,
            meta_length: 0,
            meta_orig_length: 0,
            priv_offset: 0,
            priv_length: 0,
        };
        assert_eq!(header.to_bytes().len(), 48);
    }

    #[test]
    fn header_signature_is_big_endian() {
        let header = Woff2Header {
            signature: WOFF2_SIGNATURE,
            flavor: 0x00010000,
            length: 0,
            num_tables: 0,
            reserved: 0,
            total_sfnt_size: 0,
            total_compressed_size: 0,
            major_version: 1,
            minor_version: 0,
            meta_offset: 0,
            meta_length: 0,
            meta_orig_length: 0,
            priv_offset: 0,
            priv_length: 0,
        };
        let bytes = header.to_bytes();
        assert_eq!(&bytes[0..4], b"wOF2");
    }
}
