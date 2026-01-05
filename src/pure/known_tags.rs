pub const KNOWN_TAGS: [[u8; 4]; 63] = [
    *b"cmap", *b"head", *b"hhea", *b"hmtx", *b"maxp", *b"name", *b"OS/2", *b"post", *b"cvt ",
    *b"fpgm", *b"glyf", *b"loca", *b"prep", *b"CFF ", *b"VORG", *b"EBDT", *b"EBLC", *b"gasp",
    *b"hdmx", *b"kern", *b"LTSH", *b"PCLT", *b"VDMX", *b"vhea", *b"vmtx", *b"BASE", *b"GDEF",
    *b"GPOS", *b"GSUB", *b"EBSC", *b"JSTF", *b"MATH", *b"CBDT", *b"CBLC", *b"COLR", *b"CPAL",
    *b"SVG ", *b"sbix", *b"acnt", *b"avar", *b"bdat", *b"bloc", *b"bsln", *b"cvar", *b"fdsc",
    *b"feat", *b"fmtx", *b"fvar", *b"gvar", *b"hsty", *b"just", *b"lcar", *b"mort", *b"morx",
    *b"opbd", *b"prop", *b"trak", *b"Zapf", *b"Silf", *b"Glat", *b"Gloc", *b"Feat", *b"Sill",
];

pub fn find_tag_index(tag: &[u8; 4]) -> Option<u8> {
    KNOWN_TAGS.iter().position(|t| t == tag).map(|i| i as u8)
}

pub fn get_tag(index: u8) -> Option<[u8; 4]> {
    KNOWN_TAGS.get(index as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tag_index_known_tags() {
        assert_eq!(find_tag_index(b"cmap"), Some(0));
        assert_eq!(find_tag_index(b"head"), Some(1));
        assert_eq!(find_tag_index(b"OS/2"), Some(6));
        assert_eq!(find_tag_index(b"CFF "), Some(13));
        assert_eq!(find_tag_index(b"Sill"), Some(62));
    }

    #[test]
    fn test_find_tag_index_unknown_tag() {
        assert_eq!(find_tag_index(b"XXXX"), None);
        assert_eq!(find_tag_index(b"    "), None);
    }

    #[test]
    fn test_get_tag_valid_indices() {
        assert_eq!(get_tag(0), Some(*b"cmap"));
        assert_eq!(get_tag(1), Some(*b"head"));
        assert_eq!(get_tag(6), Some(*b"OS/2"));
        assert_eq!(get_tag(13), Some(*b"CFF "));
        assert_eq!(get_tag(62), Some(*b"Sill"));
    }

    #[test]
    fn test_get_tag_invalid_indices() {
        assert_eq!(get_tag(63), None);
        assert_eq!(get_tag(255), None);
    }

    #[test]
    fn test_roundtrip() {
        for i in 0..63u8 {
            let tag = get_tag(i).unwrap();
            assert_eq!(find_tag_index(&tag), Some(i));
        }
    }
}
