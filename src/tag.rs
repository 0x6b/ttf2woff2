#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Tag(pub [u8; 4]);

const KNOWN_TAGS: [[u8; 4]; 63] = [
    *b"cmap", *b"head", *b"hhea", *b"hmtx", *b"maxp", *b"name", *b"OS/2", *b"post", *b"cvt ",
    *b"fpgm", *b"glyf", *b"loca", *b"prep", *b"CFF ", *b"VORG", *b"EBDT", *b"EBLC", *b"gasp",
    *b"hdmx", *b"kern", *b"LTSH", *b"PCLT", *b"VDMX", *b"vhea", *b"vmtx", *b"BASE", *b"GDEF",
    *b"GPOS", *b"GSUB", *b"EBSC", *b"JSTF", *b"MATH", *b"CBDT", *b"CBLC", *b"COLR", *b"CPAL",
    *b"SVG ", *b"sbix", *b"acnt", *b"avar", *b"bdat", *b"bloc", *b"bsln", *b"cvar", *b"fdsc",
    *b"feat", *b"fmtx", *b"fvar", *b"gvar", *b"hsty", *b"just", *b"lcar", *b"mort", *b"morx",
    *b"opbd", *b"prop", *b"trak", *b"Zapf", *b"Silf", *b"Glat", *b"Gloc", *b"Feat", *b"Sill",
];

impl Tag {
    pub fn known_index(&self) -> Option<u8> {
        KNOWN_TAGS.iter().position(|t| t == &self.0).map(|i| i as u8)
    }

    pub fn is_glyf(&self) -> bool {
        self.0 == *b"glyf"
    }

    pub fn is_loca(&self) -> bool {
        self.0 == *b"loca"
    }

    pub fn is_head(&self) -> bool {
        self.0 == *b"head"
    }

    pub fn is_maxp(&self) -> bool {
        self.0 == *b"maxp"
    }

    pub fn to_flags(self, transform_version: u8) -> u8 {
        match self.known_index() {
            Some(idx) => idx | (transform_version << 6),
            None => 63 | (transform_version << 6),
        }
    }
}

impl From<[u8; 4]> for Tag {
    fn from(value: [u8; 4]) -> Self {
        Self(value)
    }
}

impl AsRef<[u8; 4]> for Tag {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_index() {
        assert_eq!(Tag(*b"cmap").known_index(), Some(0));
        assert_eq!(Tag(*b"head").known_index(), Some(1));
        assert_eq!(Tag(*b"glyf").known_index(), Some(10));
        assert_eq!(Tag(*b"loca").known_index(), Some(11));
        assert_eq!(Tag(*b"XXXX").known_index(), None);
    }

    #[test]
    fn test_is_methods() {
        assert!(Tag(*b"glyf").is_glyf());
        assert!(Tag(*b"loca").is_loca());
        assert!(!Tag(*b"head").is_glyf());
    }
}
