use ash::vk::{
    AttachmentLoadOp, AttachmentStoreOp, ClearValue, ImageLayout, ImageView,
    RenderingAttachmentInfo,
};

pub struct ResolvedAttachment {
    image_view: ImageView,
    layout: ImageLayout,
    load_op: AttachmentLoadOp,
    store_op: AttachmentStoreOp,
    clear_value: ClearValue,
}

impl ResolvedAttachment {
    pub fn new(
        image_view: ImageView,
        layout: ImageLayout,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: ClearValue,
    ) -> Self {
        Self {
            image_view,
            layout,
            load_op,
            store_op,
            clear_value,
        }
    }
    pub fn to_info<'a>(&self) -> RenderingAttachmentInfo<'a> {
        RenderingAttachmentInfo::default()
            .image_view(self.image_view)
            .image_layout(self.layout)
            .load_op(self.load_op)
            .store_op(self.store_op)
            .clear_value(self.clear_value)
    }
}
