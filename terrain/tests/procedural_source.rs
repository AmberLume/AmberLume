use terrain::{ChunkCoordinate, ChunkGeometry, ProceduralTerrainSource, TerrainSource};

fn source() -> ProceduralTerrainSource {
    ProceduralTerrainSource::create()
}

#[test]
fn payload_carries_a_full_bordered_window() {
    let payload = source()
        .load(ChunkCoordinate::ORIGIN)
        .expect("procedural source must always produce a chunk");

    assert_eq!(payload.heights().len(), ChunkGeometry::WINDOW_LENGTH);
    assert!(payload.minimum() <= payload.maximum());
}

#[test]
fn bounds_cover_the_owned_nodes_only() {
    let payload = source()
        .load(ChunkCoordinate::ORIGIN)
        .expect("procedural source must always produce a chunk");

    let bounds = payload.bounds();

    assert_eq!(bounds[0], -16.0);
    assert_eq!(bounds[2], -16.0);
    assert_eq!(bounds[3], 16.0);
    assert_eq!(bounds[5], 16.0);
    assert_eq!(bounds[1], payload.minimum());
    assert_eq!(bounds[4], payload.maximum());
}

#[test]
fn generation_is_deterministic() {
    let first = source().load(ChunkCoordinate::create(3, -7)).expect("chunk");
    let second = source().load(ChunkCoordinate::create(3, -7)).expect("chunk");

    assert_eq!(first.heights(), second.heights());
}

#[test]
fn neighbour_chunks_agree_on_shared_heights() {
    let source = source();
    let nodes = ChunkGeometry::NODES as i32;

    let left = source.load(ChunkCoordinate::ORIGIN).expect("chunk");
    let right = source.load(ChunkCoordinate::create(1, 0)).expect("chunk");
    let far = source.load(ChunkCoordinate::create(0, 1)).expect("chunk");

    for index in 0..nodes {
        assert_eq!(left.height(nodes - 1, index), right.height(0, index));
        assert_eq!(left.height(index, nodes - 1), far.height(index, 0));
    }
}

#[test]
fn neighbour_chunks_agree_on_shared_normals() {
    let source = source();
    let nodes = ChunkGeometry::NODES as i32;

    let left = source.load(ChunkCoordinate::ORIGIN).expect("chunk");
    let right = source.load(ChunkCoordinate::create(1, 0)).expect("chunk");

    for index in 1..nodes - 1 {
        assert_eq!(left.normal(nodes - 1, index), right.normal(0, index));
    }
}

#[test]
fn collision_heights_match_the_rendered_nodes() {
    let payload = source().load(ChunkCoordinate::create(-2, 5)).expect("chunk");

    let heights = payload.collision_heights();

    assert_eq!(heights.len(), ChunkGeometry::LAYER_LENGTH);

    for row in 0..ChunkGeometry::NODES as i32 {
        for column in 0..ChunkGeometry::NODES as i32 {
            let index = (row * ChunkGeometry::NODES as i32 + column) as usize;

            assert_eq!(heights[index], payload.height(column, row));
        }
    }
}

#[test]
fn terrain_has_visible_relief() {
    let payload = source().load(ChunkCoordinate::ORIGIN).expect("chunk");

    assert!(
        payload.maximum() - payload.minimum() > 1.0,
        "a chunk must show relief, got range {}",
        payload.maximum() - payload.minimum()
    );
}
