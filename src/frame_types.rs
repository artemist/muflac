#[derive(Debug, Clone)]
pub struct Frame {
    pub is_variable: bool,
    pub block_size: u32,
    pub sample_rate: u32,
    pub num_channels: u8,
    pub channel_assignment: ChannelAssignment,
    pub sample_depth: u8,
    pub frame_or_sample_number: Option<u64>,
    pub header_crc: u8,
    pub subframes: Box<[Subframe]>,
    pub overall_crc: u16,
}

#[derive(Debug, Copy, Clone)]
pub enum ChannelAssignment {
    Direct,
    LeftSide,
    RightSide,
    MidSide,
}

#[derive(Debug, Clone)]
pub struct Subframe {
    pub wasted_bits: u8,
    pub data: SubframeData,
}
#[derive(Debug, Clone)]
pub enum SubframeData {
    Constant(ConstantSubframe),
    Verbatim(VerbatimSubframe),
    Fixed(FixedSubframe),
    LPC(LPCSubframe),
    Reserved,
}

#[derive(Debug, Clone)]
pub struct ConstantSubframe {
    pub content: i32,
}
#[derive(Debug, Clone)]
pub struct VerbatimSubframe {
    pub content: Box<[i32]>,
}
#[derive(Debug, Clone)]
pub struct FixedSubframe {
    pub order: u8,
    pub warmup: Box<[i32]>,
    pub residual: Residual,
}

#[derive(Debug, Clone)]
pub struct LPCSubframe {
    pub order: u8,
    pub warmup: Box<[i32]>,
    pub coefficient_precision: u8,
    pub shift: i8, // Should be sign extended to i8
    pub coefficients: Box<[i16]>,
    pub residual: Residual,
}

#[derive(Debug, Clone)]
pub struct Residual {
    pub parameter_size: u8, // RICE is 4, RICE2 = 5
    pub order: u8,
    pub partitions: Box<[RICEPartition]>,
}

#[derive(Debug, Clone)]
pub struct RICEPartition {
    pub encoding_parameter: u8,
    pub residual: Box<[i32]>,
}
