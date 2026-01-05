use super::triplet::encode_triplet;
use crate::variable_int::encode_255_u_int16;

/// WOFF2 transformed glyf table header (36 bytes)
pub struct TransformedGlyfHeader {
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
pub struct TransformedGlyf {
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
    pub fn new(num_glyphs: u16) -> Self {
        let bbox_bitmap_size = ((num_glyphs as usize + 31) >> 5) << 2;
        Self {
            n_contour_stream: Vec::new(),
            n_points_stream: Vec::new(),
            flag_stream: Vec::new(),
            glyph_stream: Vec::new(),
            composite_stream: Vec::new(),
            bbox_bitmap: vec![0u8; bbox_bitmap_size],
            bbox_stream: Vec::new(),
            instruction_stream: Vec::new(),
        }
    }

    pub fn set_bbox_bit(&mut self, glyph_id: u16) {
        let idx = glyph_id as usize >> 3;
        let bit = 0x80 >> (glyph_id & 7);
        if idx < self.bbox_bitmap.len() {
            self.bbox_bitmap[idx] |= bit;
        }
    }
}

fn read_u16_be(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([data[offset], data[offset + 1]])
}

fn read_i16_be(data: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes([data[offset], data[offset + 1]])
}

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn get_glyph_offsets(loca_data: &[u8], index_format: i16, num_glyphs: u16) -> Vec<(u32, u32)> {
    let mut offsets = Vec::with_capacity(num_glyphs as usize);
    for i in 0..num_glyphs as usize {
        let (start, end) = if index_format == 0 {
            let s = read_u16_be(loca_data, i * 2) as u32 * 2;
            let e = read_u16_be(loca_data, (i + 1) * 2) as u32 * 2;
            (s, e)
        } else {
            let s = read_u32_be(loca_data, i * 4);
            let e = read_u32_be(loca_data, (i + 1) * 4);
            (s, e)
        };
        offsets.push((start, end));
    }
    offsets
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

fn parse_simple_glyph(data: &[u8], num_contours: i16) -> Result<SimpleGlyph, String> {
    if data.len() < 10 {
        return Err("Glyph data too short".to_string());
    }

    let x_min = read_i16_be(data, 2);
    let y_min = read_i16_be(data, 4);
    let x_max = read_i16_be(data, 6);
    let y_max = read_i16_be(data, 8);

    let mut offset = 10;
    let mut end_pts = Vec::with_capacity(num_contours as usize);
    for _ in 0..num_contours {
        if offset + 2 > data.len() {
            return Err("Unexpected end of glyph data".to_string());
        }
        end_pts.push(read_u16_be(data, offset));
        offset += 2;
    }

    let num_points =
        if num_contours > 0 { end_pts[num_contours as usize - 1] as usize + 1 } else { 0 };

    if offset + 2 > data.len() {
        return Err("Unexpected end of glyph data".to_string());
    }
    let instruction_length = read_u16_be(data, offset) as usize;
    offset += 2;

    if offset + instruction_length > data.len() {
        return Err("Instruction data exceeds glyph bounds".to_string());
    }
    let instructions = data[offset..offset + instruction_length].to_vec();
    offset += instruction_length;

    let mut flags = Vec::with_capacity(num_points);
    while flags.len() < num_points {
        if offset >= data.len() {
            return Err("Unexpected end of flag data".to_string());
        }
        let flag = data[offset];
        offset += 1;
        flags.push(flag);
        if flag & 0x08 != 0 {
            if offset >= data.len() {
                return Err("Unexpected end of repeat count".to_string());
            }
            let repeat = data[offset] as usize;
            offset += 1;
            for _ in 0..repeat {
                flags.push(flag);
            }
        }
    }

    let mut x_coords: Vec<i16> = Vec::with_capacity(num_points);
    let mut x_acc: i16 = 0;
    for &flag in &flags {
        let x_short = flag & 0x02 != 0;
        let x_same_or_positive = flag & 0x10 != 0;
        let delta: i16 = if x_short {
            if offset >= data.len() {
                return Err("Unexpected end of x coordinate".to_string());
            }
            let val = data[offset] as i16;
            offset += 1;
            if x_same_or_positive { val } else { -val }
        } else if x_same_or_positive {
            0
        } else {
            if offset + 2 > data.len() {
                return Err("Unexpected end of x coordinate".to_string());
            }
            let val = read_i16_be(data, offset);
            offset += 2;
            val
        };
        x_acc = x_acc.wrapping_add(delta);
        x_coords.push(x_acc);
    }

    let mut y_coords: Vec<i16> = Vec::with_capacity(num_points);
    let mut y_acc: i16 = 0;
    for &flag in &flags {
        let y_short = flag & 0x04 != 0;
        let y_same_or_positive = flag & 0x20 != 0;
        let delta: i16 = if y_short {
            if offset >= data.len() {
                return Err("Unexpected end of y coordinate".to_string());
            }
            let val = data[offset] as i16;
            offset += 1;
            if y_same_or_positive { val } else { -val }
        } else if y_same_or_positive {
            0
        } else {
            if offset + 2 > data.len() {
                return Err("Unexpected end of y coordinate".to_string());
            }
            let val = read_i16_be(data, offset);
            offset += 2;
            val
        };
        y_acc = y_acc.wrapping_add(delta);
        y_coords.push(y_acc);
    }

    let mut points = Vec::with_capacity(num_points);
    for i in 0..num_points {
        let on_curve = flags[i] & 0x01 != 0;
        points.push((x_coords[i], y_coords[i], on_curve));
    }

    Ok(SimpleGlyph {
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

fn compute_bbox(points: &[(i16, i16, bool)]) -> (i16, i16, i16, i16) {
    if points.is_empty() {
        return (0, 0, 0, 0);
    }
    let mut x_min = i16::MAX;
    let mut y_min = i16::MAX;
    let mut x_max = i16::MIN;
    let mut y_max = i16::MIN;
    for &(x, y, _) in points {
        x_min = x_min.min(x);
        y_min = y_min.min(y);
        x_max = x_max.max(x);
        y_max = y_max.max(y);
    }
    (x_min, y_min, x_max, y_max)
}

fn encode_simple_glyph(glyph: &SimpleGlyph, glyph_id: u16, streams: &mut TransformedGlyf) {
    streams
        .n_contour_stream
        .extend_from_slice(&glyph.num_contours.to_be_bytes());

    let mut start = 0u16;
    for &end in &glyph.end_pts {
        let n_points = end - start + 1;
        streams.n_points_stream.extend(encode_255_u_int16(n_points));
        start = end + 1;
    }

    let mut prev_x: i16 = 0;
    let mut prev_y: i16 = 0;
    for &(x, y, on_curve) in &glyph.points {
        let dx = x.wrapping_sub(prev_x);
        let dy = y.wrapping_sub(prev_y);
        let (flag, triplet) = encode_triplet(dx, dy, on_curve);
        streams.flag_stream.push(flag);
        streams.glyph_stream.extend(triplet);
        prev_x = x;
        prev_y = y;
    }

    streams
        .glyph_stream
        .extend(encode_255_u_int16(glyph.instructions.len() as u16));
    streams.instruction_stream.extend(&glyph.instructions);

    let (calc_x_min, calc_y_min, calc_x_max, calc_y_max) = compute_bbox(&glyph.points);
    let bbox_matches = glyph.x_min == calc_x_min
        && glyph.y_min == calc_y_min
        && glyph.x_max == calc_x_max
        && glyph.y_max == calc_y_max;

    if !bbox_matches {
        streams.set_bbox_bit(glyph_id);
        streams.bbox_stream.extend_from_slice(&glyph.x_min.to_be_bytes());
        streams.bbox_stream.extend_from_slice(&glyph.y_min.to_be_bytes());
        streams.bbox_stream.extend_from_slice(&glyph.x_max.to_be_bytes());
        streams.bbox_stream.extend_from_slice(&glyph.y_max.to_be_bytes());
    }
}

fn encode_composite_glyph(data: &[u8], glyph_id: u16, streams: &mut TransformedGlyf) {
    let num_contours = read_i16_be(data, 0);
    let x_min = read_i16_be(data, 2);
    let y_min = read_i16_be(data, 4);
    let x_max = read_i16_be(data, 6);
    let y_max = read_i16_be(data, 8);

    streams
        .n_contour_stream
        .extend_from_slice(&num_contours.to_be_bytes());

    streams.composite_stream.extend_from_slice(&data[10..]);

    streams.set_bbox_bit(glyph_id);
    streams.bbox_stream.extend_from_slice(&x_min.to_be_bytes());
    streams.bbox_stream.extend_from_slice(&y_min.to_be_bytes());
    streams.bbox_stream.extend_from_slice(&x_max.to_be_bytes());
    streams.bbox_stream.extend_from_slice(&y_max.to_be_bytes());
}

pub fn transform_glyf(
    glyf_data: &[u8],
    loca_data: &[u8],
    head_data: &[u8],
    maxp_data: &[u8],
) -> Result<Vec<u8>, String> {
    if maxp_data.len() < 6 {
        return Err("maxp table too short".to_string());
    }
    let num_glyphs = read_u16_be(maxp_data, 4);

    if head_data.len() < 52 {
        return Err("head table too short".to_string());
    }
    let index_format = read_i16_be(head_data, 50);

    let offsets = get_glyph_offsets(loca_data, index_format, num_glyphs);
    let mut streams = TransformedGlyf::new(num_glyphs);

    for (glyph_id, &(start, end)) in offsets.iter().enumerate() {
        if start == end {
            streams.n_contour_stream.extend_from_slice(&0i16.to_be_bytes());
            continue;
        }

        let glyph_data = &glyf_data[start as usize..end as usize];
        if glyph_data.len() < 2 {
            streams.n_contour_stream.extend_from_slice(&0i16.to_be_bytes());
            continue;
        }

        let num_contours = read_i16_be(glyph_data, 0);

        if num_contours >= 0 {
            let glyph = parse_simple_glyph(glyph_data, num_contours)?;
            encode_simple_glyph(&glyph, glyph_id as u16, &mut streams);
        } else {
            encode_composite_glyph(glyph_data, glyph_id as u16, &mut streams);
        }
    }

    let header = TransformedGlyfHeader {
        version: 0,
        option_flags: 0,
        num_glyphs,
        index_format: index_format as u16,
        n_contour_stream_size: streams.n_contour_stream.len() as u32,
        n_points_stream_size: streams.n_points_stream.len() as u32,
        flag_stream_size: streams.flag_stream.len() as u32,
        glyph_stream_size: streams.glyph_stream.len() as u32,
        composite_stream_size: streams.composite_stream.len() as u32,
        bbox_stream_size: (streams.bbox_bitmap.len() + streams.bbox_stream.len()) as u32,
        instruction_stream_size: streams.instruction_stream.len() as u32,
    };

    let mut output = Vec::new();
    output.extend_from_slice(&header.to_bytes());
    output.extend(&streams.n_contour_stream);
    output.extend(&streams.n_points_stream);
    output.extend(&streams.flag_stream);
    output.extend(&streams.glyph_stream);
    output.extend(&streams.composite_stream);
    output.extend(&streams.bbox_bitmap);
    output.extend(&streams.bbox_stream);
    output.extend(&streams.instruction_stream);

    Ok(output)
}
