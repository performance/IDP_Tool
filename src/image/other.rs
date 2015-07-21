#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub enum PixelType {
    Short16,
    Float32
}


/// Result of a decoding process
pub enum DecodingResult {
    /// A vector of unsigned bytes
    U16(Vec<u16>),
    /// A vector of f32s
    F32(Vec<f32>)
}

// A buffer for image decoding
pub enum DecodingBuffer<'a> {
    /// A slice of unsigned words
    U16(&'a mut [u16]),
    /// A slice of f32
    F32(&'a mut [f32]),
}

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub enum BadType {
    DeadBand,
    OpenBad,
    OpenBadRow,
    OpenBadCol,
    OpenBadBoth,
    // Short_Level1,
    // Short_Level2,
    Unknown
}

pub struct Pixel {
pub    value: f32,
pub    valid: BadType
}

