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
    Ignored,
    OpenBad,
    OpenBadRow,
    OpenBadCol,
    OpenBadBoth,
    ShortBad,
    
    // Short_Level1,
    // Short_Level2,
    Unknown
}

pub struct Pixel {
pub    value: f32,
pub    valid: BadType
}

//pub struct _BondingStats {
//    pub bad_opens : u64, 
//    pub number_of_bad_columns: u64,
//    pub number_of_bad_rows: u64,
//    pub number_of_bad_shorts: u64,
//    pub threshold_for_shorts: u64,
//    pub number_of_pixels_measured: u64,
//    pub number_of_open_bads_in_bad_cols: u64,
//    pub number_of_open_bads_in_bad_rows: u64
//}

pub struct ShortDiagonalStats {
    pub number_of_pixels_measured: u64,
    pub number_of_bad_shorts: u64
}