use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use super::{triplet::encode_triplet, varint::encode_255_u_int16};
use crate::Error;

/// WOFF2 transformed glyf table header (36 bytes)
struct TransformedGlyfHeader {
    pub version: u16,      // 0x0000
    pub option_flags: u16, // bit 0: has overlap simple bitmap
    pub num_glyphs: u16,
    pub index_format: u16, // from head.indexToLocFormat
    pub n_contour_stream_size: u32,
    pub n_points_stream_size: u32,
    pub flag_stream_size: u32,
    pub glyph_stream_size: u32,
    pub composite_stream_size: u32,
    pub bbox_stream_size: u32,
    pub instruction_stream_size: u32,
}

impl TransformedGlyfHeader {
    pub fn to_bytes(&self) -> [u8; 36] {
        let mut out = [0u8; 36];
        out[0..2].copy_from_slice(&self.version.to_be_bytes());
        out[2..4].copy_from_slice(&self.option_flags.to_be_bytes());
        out[4..6].copy_from_slice(&self.num_glyphs.to_be_bytes());
        out[6..8].copy_from_slice(&self.index_format.to_be_bytes());
        out[8..12].copy_from_slice(&self.n_contour_stream_size.to_be_bytes());
        out[12..16].copy_from_slice(&self.n_points_stream_size.to_be_bytes());
        out[16..20].copy_from_slice(&self.flag_stream_size.to_be_bytes());
        out[20..24].copy_from_slice(&self.glyph_stream_size.to_be_bytes());
        out[24..28].copy_from_slice(&self.composite_stream_size.to_be_bytes());
        out[28..32].copy_from_slice(&self.bbox_stream_size.to_be_bytes());
        out[32..36].copy_from_slice(&self.instruction_stream_size.to_be_bytes());
        out
    }
}

/// Builder for transformed glyf data
struct TransformedGlyf {
    pub n_contour_stream: Vec<u8>,
    pub n_points_stream: Vec<u8>,
    pub flag_stream: Vec<u8>,
    pub glyph_stream: Vec<u8>,
    pub composite_stream: Vec<u8>,
    pub bbox_bitmap: Vec<u8>,
    pub bbox_stream: Vec<u8>,
    pub instruction_stream: Vec<u8>,
}

impl TransformedGlyf {
    pub fn new(num_glyphs: u16, glyf_size: usize) -> Self {
        let bbox_bitmap_size = ((num_glyphs as usize + 31) >> 5) << 2;
        Self {
            n_contour_stream: Vec::with_capacity(num_glyphs as usize * 2),
            n_points_stream: Vec::with_capacity(glyf_size / 4),
            flag_stream: Vec::with_capacity(glyf_size / 2),
            glyph_stream: Vec::with_capacity(glyf_size),
            composite_stream: Vec::with_capacity(glyf_size / 8),
            bbox_bitmap: vec![0u8; bbox_bitmap_size],
            bbox_stream: Vec::with_capacity(num_glyphs as usize),
            instruction_stream: Vec::with_capacity(glyf_size / 4),
        }
    }

    fn set_bbox_bit(&mut self, glyph_id: u16) {
        let idx = glyph_id as usize >> 3;
        let bit = 0x80 >> (glyph_id & 7);
        if idx < self.bbox_bitmap.len() {
            self.bbox_bitmap[idx] |= bit;
        }
    }

    fn push_bbox(&mut self, glyph_id: u16, x_min: i16, y_min: i16, x_max: i16, y_max: i16) {
        self.set_bbox_bit(glyph_id);
        self.bbox_stream.extend_from_slice(&x_min.to_be_bytes());
        self.bbox_stream.extend_from_slice(&y_min.to_be_bytes());
        self.bbox_stream.extend_from_slice(&x_max.to_be_bytes());
        self.bbox_stream.extend_from_slice(&y_max.to_be_bytes());
    }

    fn push_empty(&mut self) {
        self.n_contour_stream.extend_from_slice(&0i16.to_be_bytes());
    }

    fn encode_simple(&mut self, glyph_id: u16, glyph: &SimpleGlyph) {
        self.n_contour_stream
            .extend_from_slice(&glyph.num_contours.to_be_bytes());

        let mut start = 0u16;
        for &end in &glyph.end_pts {
            let n_points = end - start + 1;
            self.n_points_stream
                .extend_from_slice(encode_255_u_int16(n_points).as_slice());
            start = end + 1;
        }

        let mut prev_x: i16 = 0;
        let mut prev_y: i16 = 0;
        for &(x, y, on_curve) in &glyph.points {
            let dx = x.wrapping_sub(prev_x);
            let dy = y.wrapping_sub(prev_y);
            let (flag, triplet) = encode_triplet(dx, dy, on_curve);
            self.flag_stream.push(flag);
            self.glyph_stream.extend_from_slice(triplet.as_slice());
            prev_x = x;
            prev_y = y;
        }

        self.glyph_stream
            .extend_from_slice(encode_255_u_int16(glyph.instructions.len() as u16).as_slice());
        self.instruction_stream.extend(&glyph.instructions);

        let (calc_x_min, calc_y_min, calc_x_max, calc_y_max) = glyph.compute_bbox();
        let bbox_matches = glyph.x_min == calc_x_min
            && glyph.y_min == calc_y_min
            && glyph.x_max == calc_x_max
            && glyph.y_max == calc_y_max;

        if !bbox_matches {
            self.push_bbox(glyph_id, glyph.x_min, glyph.y_min, glyph.x_max, glyph.y_max);
        }
    }

    fn encode_composite(&mut self, glyph_id: u16, data: &[u8]) {
        let num_contours = i16::from_be_bytes([data[0], data[1]]);
        let x_min = i16::from_be_bytes([data[2], data[3]]);
        let y_min = i16::from_be_bytes([data[4], data[5]]);
        let x_max = i16::from_be_bytes([data[6], data[7]]);
        let y_max = i16::from_be_bytes([data[8], data[9]]);

        self.n_contour_stream.extend_from_slice(&num_contours.to_be_bytes());
        self.composite_stream.extend_from_slice(&data[10..]);

        self.push_bbox(glyph_id, x_min, y_min, x_max, y_max);
    }

    fn finish(self, index_format: u16) -> Vec<u8> {
        let header = TransformedGlyfHeader {
            version: 0,
            option_flags: 0,
            num_glyphs: (self.n_contour_stream.len() / 2) as u16,
            index_format,
            n_contour_stream_size: self.n_contour_stream.len() as u32,
            n_points_stream_size: self.n_points_stream.len() as u32,
            flag_stream_size: self.flag_stream.len() as u32,
            glyph_stream_size: self.glyph_stream.len() as u32,
            composite_stream_size: self.composite_stream.len() as u32,
            bbox_stream_size: (self.bbox_bitmap.len() + self.bbox_stream.len()) as u32,
            instruction_stream_size: self.instruction_stream.len() as u32,
        };

        let total_size = 36
            + self.n_contour_stream.len()
            + self.n_points_stream.len()
            + self.flag_stream.len()
            + self.glyph_stream.len()
            + self.composite_stream.len()
            + self.bbox_bitmap.len()
            + self.bbox_stream.len()
            + self.instruction_stream.len();

        let mut output = Vec::with_capacity(total_size);
        output.extend_from_slice(&header.to_bytes());
        output.extend_from_slice(&self.n_contour_stream);
        output.extend_from_slice(&self.n_points_stream);
        output.extend_from_slice(&self.flag_stream);
        output.extend_from_slice(&self.glyph_stream);
        output.extend_from_slice(&self.composite_stream);
        output.extend_from_slice(&self.bbox_bitmap);
        output.extend_from_slice(&self.bbox_stream);
        output.extend_from_slice(&self.instruction_stream);

        output
    }
}

struct SimpleGlyph {
    num_contours: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    end_pts: Vec<u16>,
    instructions: Vec<u8>,
    points: Vec<(i16, i16, bool)>,
}

impl SimpleGlyph {
    fn parse(data: &[u8], num_contours: i16) -> Result<Self, Error> {
        if data.len() < 10 {
            return Err(Error::InvalidGlyph("data too short"));
        }

        let mut cursor = Cursor::new(data);
        cursor.set_position(2); // Skip num_contours

        let x_min = cursor
            .read_i16::<BigEndian>()
            .map_err(|_| Error::InvalidGlyph("failed to read bbox"))?;
        let y_min = cursor
            .read_i16::<BigEndian>()
            .map_err(|_| Error::InvalidGlyph("failed to read bbox"))?;
        let x_max = cursor
            .read_i16::<BigEndian>()
            .map_err(|_| Error::InvalidGlyph("failed to read bbox"))?;
        let y_max = cursor
            .read_i16::<BigEndian>()
            .map_err(|_| Error::InvalidGlyph("failed to read bbox"))?;

        let mut end_pts = Vec::with_capacity(num_contours as usize);
        for _ in 0..num_contours {
            let ep = cursor
                .read_u16::<BigEndian>()
                .map_err(|_| Error::InvalidGlyph("unexpected end of data"))?;
            end_pts.push(ep);
        }

        let num_points =
            if num_contours > 0 { end_pts[num_contours as usize - 1] as usize + 1 } else { 0 };

        let instruction_length = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| Error::InvalidGlyph("unexpected end of data"))?
            as usize;

        let offset = cursor.position() as usize;
        if offset + instruction_length > data.len() {
            return Err(Error::InvalidGlyph("instruction data exceeds bounds"));
        }
        let instructions = data[offset..offset + instruction_length].to_vec();
        cursor.set_position((offset + instruction_length) as u64);

        let flags = Self::parse_flags(&mut cursor, data, num_points)?;
        let x_coords = Self::parse_x_coords(&mut cursor, data, &flags)?;
        let y_coords = Self::parse_y_coords(&mut cursor, data, &flags)?;

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let on_curve = flags[i] & 0x01 != 0;
            points.push((x_coords[i], y_coords[i], on_curve));
        }

        Ok(Self {
            num_contours,
            x_min,
            y_min,
            x_max,
            y_max,
            end_pts,
            instructions,
            points,
        })
    }

    fn parse_flags(
        cursor: &mut Cursor<&[u8]>,
        data: &[u8],
        num_points: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut flags = Vec::with_capacity(num_points);
        while flags.len() < num_points {
            let pos = cursor.position() as usize;
            if pos >= data.len() {
                return Err(Error::InvalidGlyph("unexpected end of flag data"));
            }
            let flag = data[pos];
            cursor.set_position((pos + 1) as u64);
            flags.push(flag);
            if flag & 0x08 != 0 {
                let pos = cursor.position() as usize;
                if pos >= data.len() {
                    return Err(Error::InvalidGlyph("unexpected end of repeat count"));
                }
                let repeat = data[pos] as usize;
                cursor.set_position((pos + 1) as u64);
                let new_len = flags.len() + repeat;
                flags.resize(new_len, flag);
            }
        }
        Ok(flags)
    }

    fn parse_coords(
        cursor: &mut Cursor<&[u8]>,
        data: &[u8],
        flags: &[u8],
        short_bit: u8,
        same_or_positive_bit: u8,
        err: &'static str,
    ) -> Result<Vec<i16>, Error> {
        let mut coords = Vec::with_capacity(flags.len());
        let mut acc: i16 = 0;
        for &flag in flags {
            let is_short = flag & short_bit != 0;
            let same_or_positive = flag & same_or_positive_bit != 0;
            let delta: i16 = if is_short {
                let pos = cursor.position() as usize;
                if pos >= data.len() {
                    return Err(Error::InvalidGlyph(err));
                }
                let val = data[pos] as i16;
                cursor.set_position((pos + 1) as u64);
                if same_or_positive { val } else { -val }
            } else if same_or_positive {
                0
            } else {
                cursor.read_i16::<BigEndian>().map_err(|_| Error::InvalidGlyph(err))?
            };
            acc = acc.wrapping_add(delta);
            coords.push(acc);
        }
        Ok(coords)
    }

    fn parse_x_coords(
        cursor: &mut Cursor<&[u8]>,
        data: &[u8],
        flags: &[u8],
    ) -> Result<Vec<i16>, Error> {
        Self::parse_coords(cursor, data, flags, 0x02, 0x10, "unexpected end of x coordinate")
    }

    fn parse_y_coords(
        cursor: &mut Cursor<&[u8]>,
        data: &[u8],
        flags: &[u8],
    ) -> Result<Vec<i16>, Error> {
        Self::parse_coords(cursor, data, flags, 0x04, 0x20, "unexpected end of y coordinate")
    }

    fn compute_bbox(&self) -> (i16, i16, i16, i16) {
        if self.points.is_empty() {
            return (0, 0, 0, 0);
        }
        let mut x_min = i16::MAX;
        let mut y_min = i16::MAX;
        let mut x_max = i16::MIN;
        let mut y_max = i16::MIN;
        for &(x, y, _) in &self.points {
            x_min = x_min.min(x);
            y_min = y_min.min(y);
            x_max = x_max.max(x);
            y_max = y_max.max(y);
        }
        (x_min, y_min, x_max, y_max)
    }
}

struct GlyphOffsetIter<'a> {
    cursor: Cursor<&'a [u8]>,
    index_format: i16,
    remaining: u16,
}

impl<'a> GlyphOffsetIter<'a> {
    fn new(loca_data: &'a [u8], index_format: i16, num_glyphs: u16) -> Self {
        Self {
            cursor: Cursor::new(loca_data),
            index_format,
            remaining: num_glyphs,
        }
    }
}

impl Iterator for GlyphOffsetIter<'_> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;

        let (start, end) = if self.index_format == 0 {
            let s = self.cursor.read_u16::<BigEndian>().unwrap_or(0) as u32 * 2;
            let e = self.cursor.read_u16::<BigEndian>().unwrap_or(0) as u32 * 2;
            self.cursor.set_position(self.cursor.position() - 2);
            (s, e)
        } else {
            let s = self.cursor.read_u32::<BigEndian>().unwrap_or(0);
            let e = self.cursor.read_u32::<BigEndian>().unwrap_or(0);
            self.cursor.set_position(self.cursor.position() - 4);
            (s, e)
        };
        Some((start, end))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for GlyphOffsetIter<'_> {}

pub(crate) fn transform_glyf(
    glyf_data: &[u8],
    loca_data: &[u8],
    head_data: &[u8],
    maxp_data: &[u8],
) -> Result<Vec<u8>, Error> {
    if maxp_data.len() < 6 {
        return Err(Error::DataTooShort { context: "maxp table" });
    }
    let mut cursor = Cursor::new(maxp_data);
    cursor.set_position(4);
    let num_glyphs = cursor
        .read_u16::<BigEndian>()
        .map_err(|_| Error::DataTooShort { context: "maxp table" })?;

    if head_data.len() < 52 {
        return Err(Error::DataTooShort { context: "head table" });
    }
    let mut cursor = Cursor::new(head_data);
    cursor.set_position(50);
    let index_format = cursor
        .read_i16::<BigEndian>()
        .map_err(|_| Error::DataTooShort { context: "head table" })?;

    let mut streams = TransformedGlyf::new(num_glyphs, glyf_data.len());

    for (glyph_id, (start, end)) in
        GlyphOffsetIter::new(loca_data, index_format, num_glyphs).enumerate()
    {
        if start == end {
            streams.push_empty();
            continue;
        }

        let glyph_data = &glyf_data[start as usize..end as usize];
        if glyph_data.len() < 2 {
            streams.push_empty();
            continue;
        }

        let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);

        if num_contours >= 0 {
            let glyph = SimpleGlyph::parse(glyph_data, num_contours)?;
            streams.encode_simple(glyph_id as u16, &glyph);
        } else {
            streams.encode_composite(glyph_id as u16, glyph_data);
        }
    }

    Ok(streams.finish(index_format as u16))
}
