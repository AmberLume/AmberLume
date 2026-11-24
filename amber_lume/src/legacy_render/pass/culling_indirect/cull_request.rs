use render_graph::DrawBucket;

#[derive(Copy, Clone)]
pub struct CullRequest {
    pub accept_mask: u32,
    pub bucket: DrawBucket,
}
