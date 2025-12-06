// use crate::resources::common::resource_backend::ResourceKey;
//
// pub struct Hasher {
//     handle: blake3::Hasher,
// }
//
// impl Hasher {
//     pub fn new() -> Self {
//         Self {
//             handle: blake3::Hasher::new(),
//         }
//     }
//
//     pub fn finalize(&mut self) -> ResourceKey {
//         let digest = self.handle.finalize();
//         let mut out = [0u8; 16];
//         out.copy_from_slice(&digest.as_bytes()[..16]);
//         out
//     }
//
//     // pub fn hash_pipeline_config(value: &PipelineConfig) -> ResourceKey {
//     //     let mut hasher = get_hasher();
//     //
//     //     hash_string(&mut hasher, &value.shader_name);
//     //     hash_u32(&mut hasher, value.layout as u32);
//     //     hash_optional_string(&mut hasher, &value.vs_entry);
//     //     hash_optional_string(&mut hasher, &value.fs_entry);
//     //     hash_bool(&mut hasher, value.vs_options.zero_initialize_workgroup_memory);
//     //     hash_bool(&mut hasher, value.fs_options.zero_initialize_workgroup_memory);
//     //
//     //     hash_optional_string(&mut hasher, &value.label);
//     //
//     //     hash_u32(&mut hasher, value.vertex_buffers.len() as u32);
//     //     for vb in &value.vertex_buffers {
//     //         hash_vertex_buffer_layout(&mut hasher, vb);
//     //     }
//     //
//     //     hash_u32(&mut hasher, value.targets.len() as u32);
//     //     for t in &value.targets {
//     //         hash_bool(&mut hasher, t.is_some());
//     //         if let Some(ct) = t {
//     //             hash_color_target_state(&mut hasher, ct);
//     //         }
//     //     }
//     //
//     //     hash_primitive_state(&mut hasher, &value.primitive);
//     //
//     //     hash_bool(&mut hasher, value.depth.is_some());
//     //     if let Some(ds) = &value.depth {
//     //         hash_depth_stencil_state(&mut hasher, ds);
//     //     }
//     //
//     //     hash_u32(&mut hasher, value.push_constants.len() as u32);
//     //     for r in &value.push_constants {
//     //         hash_push_constant_range(&mut hasher, r);
//     //     }
//     //
//     //     hash_u32(&mut hasher, value.sample_count);
//     //     hash_u64(&mut hasher, value.sample_mask);
//     //     hash_bool(&mut hasher, value.alpha_to_coverage);
//     //     hash_optional_u32(&mut hasher, value.multiview.map(NonZeroU32::get));
//     //
//     //     finalize_key(hasher)
//     // }
//
//     // #[inline]
//     // fn hash_debug<T: Debug>(hasher: &mut Hasher, value: T) {
//     //     let value = format!("{:?}", value);
//     //
//     //     hasher.update(&value.as_bytes());
//     // }
//
//     #[inline]
//     pub fn hash_u32(&mut self, value: u32) {
//         self.handle.update(&value.to_le_bytes());
//     }
//
//     // #[inline]
//     // fn hash_u64(hasher: &mut Hasher, value: u64) {
//     //     hasher.update(&value.to_le_bytes());
//     // }
//     //
//     // #[inline]
//     // fn hash_i32(hasher: &mut Hasher, value: i32) {
//     //     hasher.update(&value.to_le_bytes());
//     // }
//     //
//     // #[inline]
//     // fn hash_i64(hasher: &mut Hasher, value: i64) {
//     //     hasher.update(&value.to_le_bytes());
//     // }
//     //
//     // #[inline]
//     // fn hash_f32(hasher: &mut Hasher, value: f32) {
//     //     hasher.update(&value.to_le_bytes());
//     // }
//     //
//     // #[inline]
//     // fn hash_f64(hasher: &mut Hasher, value: f64) {
//     //     hasher.update(&value.to_le_bytes());
//     // }
//     //
//     // #[inline]
//     // fn hash_bool(hasher: &mut Hasher, value: bool) {
//     //     hasher.update(&[value as u8]);
//     // }
//     //
//     // #[inline]
//     // fn hash_optional_u32(hasher: &mut Hasher, value: Option<u32>) {
//     //     hash_bool(hasher, value.is_some());
//     //
//     //     if let Some(value) = value {
//     //         hash_u32(hasher, value)
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn hash_str(hasher: &mut Hasher, value: &str) {
//     //     hash_u32(hasher, value.len() as u32);
//     //
//     //     hasher.update(value.as_bytes());
//     // }
//
//     #[inline]
//     pub fn hash_string(&mut self, value: &String) {
//         self.hash_u32(value.len() as u32);
//
//         self.handle.update(value.as_bytes());
//     }
//
//     // #[inline]
//     // fn hash_optional_str(hasher: &mut Hasher, value: Option<&str>) {
//     //     hash_bool(hasher, value.is_some());
//     //
//     //     if let Some(value) = value {
//     //         hash_str(hasher, value)
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn hash_optional_string(hasher: &mut Hasher, value: &Option<String>) {
//     //     hash_bool(hasher, value.is_some());
//     //
//     //     if let Some(value) = value {
//     //         hash_string(hasher, value)
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn hash_blend_component(hasher: &mut Hasher, value: BlendComponent) {
//     //     hash_u32(hasher, value.src_factor as u32);
//     //     hash_u32(hasher, value.dst_factor as u32);
//     //     hash_u32(hasher, value.operation as u32);
//     // }
//     //
//     // #[inline]
//     // fn hash_blend_state(hasher: &mut Hasher, value: &BlendState) {
//     //     hash_blend_component(hasher, value.color);
//     //     hash_blend_component(hasher, value.alpha);
//     // }
//     //
//     // #[inline]
//     // fn hash_color_target_state(hasher: &mut Hasher, value: &ColorTargetState) {
//     //     hash_debug(hasher, value.format);
//     //     hash_u32(hasher, value.write_mask.bits());
//     //     hash_bool(hasher, value.blend.is_some());
//     //
//     //     if let Some(value) = value.blend {
//     //         hash_blend_state(hasher, &value);
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn hash_vertex_attributes(hasher: &mut Hasher, value: &[VertexAttribute]) {
//     //     hash_u32(hasher, value.len() as u32);
//     //
//     //     for value in value {
//     //         hash_u64(hasher, value.offset);
//     //         hash_u32(hasher, value.shader_location);
//     //         hash_u32(hasher, value.format as u32);
//     //     }
//     // }
//     //
//     // #[inline]
//     // fn hash_vertex_buffer_layout(hasher: &mut Hasher, value: &VertexBufferLayout) {
//     //     hash_u64(hasher, value.array_stride);
//     //     hash_u32(hasher, value.step_mode as u32);
//     //     hash_vertex_attributes(hasher, value.attributes);
//     // }
//     //
//     // #[inline]
//     // fn hash_primitive_state(hasher: &mut Hasher, value: &PrimitiveState) {
//     //     hash_u32(hasher, value.topology as u32);
//     //
//     //     hash_bool(hasher, value.strip_index_format.is_some());
//     //     if let Some(value) = value.strip_index_format {
//     //         hash_u32(hasher, value as u32);
//     //     }
//     //
//     //     hash_u32(hasher, value.front_face as u32);
//     //
//     //     hash_bool(hasher, value.cull_mode.is_some());
//     //     if let Some(value) = value.cull_mode {
//     //         hash_u32(hasher, value as u32);
//     //     }
//     //
//     //     hash_bool(hasher, value.unclipped_depth);
//     //     hash_u32(hasher, value.polygon_mode as u32);
//     //     hash_bool(hasher, value.conservative);
//     // }
//     //
//     // #[inline]
//     // fn hash_stencil_face_state(hasher: &mut Hasher, value: &StencilFaceState) {
//     //     hash_u32(hasher, value.compare as u32);
//     //     hash_u32(hasher, value.fail_op as u32);
//     //     hash_u32(hasher, value.depth_fail_op as u32);
//     //     hash_u32(hasher, value.pass_op as u32);
//     // }
//
//     // #[inline]
//     // fn hash_depth_stencil_state(hasher: &mut Hasher, value: &DepthStencilState) {
//     //     hash_debug(hasher, value.format);
//     //     hash_bool(hasher, value.depth_write_enabled);
//     //     hash_u32(hasher, value.depth_compare as u32);
//     //     hash_stencil_face_state(hasher, &value.stencil.front);
//     //     hash_stencil_face_state(hasher, &value.stencil.back);
//     //     hash_u32(hasher, value.stencil.read_mask);
//     //     hash_u32(hasher, value.stencil.write_mask);
//     //     hash_i32(hasher, value.bias.constant);
//     //     hash_f32(hasher, value.bias.slope_scale);
//     //     hash_f32(hasher, value.bias.clamp);
//     // }
//
//     // #[inline]
//     // fn hash_push_constant_range(hasher: &mut Hasher, value: &PushConstantRange) {
//     //     hash_u32(hasher, value.stages.bits());
//     //     hash_u32(hasher, value.range.start);
//     //     hash_u32(hasher, value.range.end);
//     // }
// }
