#[derive(Debug, Clone)]
pub struct MetadataBlock {
    pub is_last: bool,
    pub content: MetadataBlockData,
}

#[derive(Debug, Clone)]
pub enum MetadataBlockData {
    StreamInfo(MetadataBlockStreamInfo),
    Padding,                // No content
    Application(Box<[u8]>), // Application defined
    SeekTable(MetadataBlockSeekTable),
    VorbisComment(Box<[u8]>), // Parsing is Not Our Job™
    CueSheet(MetadataBlockCueSheet),
    Picture(MetadataBlockPicture),
    Reserved(u8),
    Invalid,
    TodoUnimplemented(u8),
}

#[derive(Debug, Clone)]
pub struct MetadataBlockStreamInfo {
    pub min_block_size: u16,
    pub max_block_size: u16,
    pub min_frame_size: u32,    // 24 bits used
    pub max_frame_size: u32,    // 24 bits used
    pub sample_rate: u32,       // 20 bits used
    pub num_channels: u8,       // 3 bits used
    pub sample_depth: u8,       // 5 bits used
    pub num_samples: u64,       // 36 bits used
    pub decoded_checksum: u128, // MD5
}

#[derive(Debug, Clone)]
pub struct MetadataBlockSeekTable {
    pub seek_points: Box<[SeekPoint]>,
}

#[derive(Debug, Clone)]
pub struct SeekPoint {
    pub sample_number: u64,
    pub frame_offset: u64,
    pub num_samples: u16,
}

#[derive(Debug, Clone)]
pub struct MetadataBlockCueSheet {
    pub catalog_number: Box<[u8]>,
    pub num_lead_in_samples: u64,
    pub is_cd: bool,
    pub tracks: Box<[CueSheetTrack]>,
}

#[derive(Debug, Clone)]
pub struct CueSheetTrack {
    pub track_offset: u64,
    pub track_num: u8,
    pub track_isrc: [u8; 12],
    pub track_type: bool,
    pub pre_emphasis: bool,
    pub indices: Box<[CueSheetTrackIndex]>,
}

#[derive(Debug, Clone)]
pub struct CueSheetTrackIndex {
    pub offset: u64,
    pub index_point: u8,
}

#[derive(Debug, Clone)]
pub struct MetadataBlockPicture {
    pub picture_type: PictureType,
    pub mime_type: Box<[u8]>,
    pub description: Box<[u8]>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub num_colors_used: u32,
    pub picture: Box<[u8]>,
}

#[derive(Debug, Clone)]
pub enum PictureType {
    Other = 0,
    FileIcon32 = 1,
    FileIcon = 2,
    FrontCover = 3,
    BackCover = 4,
    Leaflet = 5,
    Media = 6,
    LeadArtist = 7,
    Artist = 8,
    Conductor = 9,
    Band = 10,
    Composer = 11,
    Lyricist = 12,
    RecordingLocation = 13,
    DuringRecording = 14,
    DuringPerformance = 15,
    Movie = 16,
    BrightlyColouredFish = 17,
    Illustration = 18,
    BandLogo = 19,
    PublisherLogo = 20,
    Reserved,
}
