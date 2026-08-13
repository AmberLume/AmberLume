use crate::block_format::BlockFormat;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use basis_universal::{DecodeFlags, LowLevelUastcTranscoder, SliceParametersUastc};
use ktx2::{DfdBlockBasic, Reader, SupercompressionScheme, TransferFunction};
use zstd::bulk::decompress;

pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub is_srgb: bool,

    pub levels: Vec<Vec<u8>>,
}

impl TextureData {
    pub fn decode(bytes: &[u8], target_format: BlockFormat) -> Result<Self> {
        let reader = Reader::new(bytes)?;
        let header = reader.header();

        if header.supercompression_scheme != Some(SupercompressionScheme::Zstandard) {
            bail!("Unsupported supercompression scheme: {:?}", header.supercompression_scheme);
        }

        let is_srgb = reader
            .dfd_blocks()
            .next()
            .and_then(|block| DfdBlockBasic::parse(block.data).ok())
            .and_then(|block| block.header.transfer_function) == Some(TransferFunction::SRGB);

        let levels = Self::transcode_levels(&reader, target_format)?;

        Ok(Self {
            width: header.pixel_width,
            height: header.pixel_height,
            mip_levels: header.level_count,
            is_srgb,

            levels,
        })
    }

    pub fn level_extent(&self, level_index: u32) -> (u32, u32) {
        ((self.width >> level_index).max(1), (self.height >> level_index).max(1))
    }

    fn transcode_levels(reader: &Reader<&[u8]>, target_format: BlockFormat) -> Result<Vec<Vec<u8>>> {
        let transcoder = LowLevelUastcTranscoder::new();
        let block_format = target_format.transcoder_block_format();

        reader
            .levels()
            .enumerate()
            .map(|(index, level)| {
                let level_width = (reader.header().pixel_width >> index).max(1);
                let level_height = (reader.header().pixel_height >> index).max(1);

                let uncompressed_size = uastc_uncompressed_size(level_width, level_height);
                let uastc_data = decompress(level.data, uncompressed_size)?;

                let slice_params = SliceParametersUastc {
                    num_blocks_x: (level_width + 3) / 4,
                    num_blocks_y: (level_height + 3) / 4,
                    has_alpha: true,
                    original_width: level_width,
                    original_height: level_height,
                };

                transcoder
                    .transcode_slice(
                        &uastc_data,
                        slice_params,
                        DecodeFlags::empty(),
                        block_format,
                    )
                    .map_err(|e| anyhow!("Low-level transcode failed: {:?}", e))
            })
            .collect()
    }
}

fn uastc_uncompressed_size(width: u32, height: u32) -> usize {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;

    (blocks_x * blocks_y * 16) as usize
}
