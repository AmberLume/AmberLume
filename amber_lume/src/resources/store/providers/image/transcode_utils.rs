use anyhow::Result;
use anyhow::anyhow;
use basis_universal::{
    DecodeFlags, LowLevelUastcTranscoder, SliceParametersUastc, TranscoderBlockFormat,
};
use ktx2::Reader;
use zstd::bulk::decompress;

pub fn transcode(
    reader: &Reader<&[u8]>,
    target_format: TranscoderBlockFormat,
) -> Result<Vec<Vec<u8>>> {
    let transcoder = LowLevelUastcTranscoder::new();

    reader
        .levels()
        .enumerate()
        .map(|(index, level)| {
            let level_width = (reader.header().pixel_width >> index).max(1);
            let level_height = (reader.header().pixel_height >> index).max(1);

            let uncompressed_size = calculate_uastc_uncompressed_size(reader, index as u32);
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
                    target_format,
                )
                .map_err(|e| anyhow!("Low-level transcode failed: {:?}", e))
        })
        .collect()
}

fn calculate_uastc_uncompressed_size(reader: &Reader<&[u8]>, level_index: u32) -> usize {
    let header = reader.header();
    let width = (header.pixel_width >> level_index).max(1);
    let height = (header.pixel_height >> level_index).max(1);

    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;

    (blocks_x * blocks_y * 16) as usize
}
