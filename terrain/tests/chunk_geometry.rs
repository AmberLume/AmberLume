use glam::Vec3;
use terrain::{ChunkCoordinate, ChunkGeometry, ChunkTopology, RegionCoordinate};

fn node_position(index: u32) -> Vec3 {
    let nodes = ChunkGeometry::NODES;
    let column = index % nodes;
    let row = index / nodes;

    Vec3::new(
        column as f32 * ChunkGeometry::CELL_SIZE,
        0.0,
        row as f32 * ChunkGeometry::CELL_SIZE,
    )
}

#[test]
fn default_geometry_matches_the_agreed_numbers() {
    assert_eq!(ChunkGeometry::CELLS, 32);
    assert_eq!(ChunkGeometry::NODES, 33);
    assert_eq!(ChunkGeometry::NODE_COUNT, 1089);
    assert_eq!(ChunkGeometry::INDEX_COUNT, 6144);
    assert_eq!(ChunkGeometry::OWNED_HEIGHT_COUNT, 1024);
    assert_eq!(ChunkGeometry::WINDOW_STRIDE, 35);
    assert_eq!(ChunkGeometry::WINDOW_LENGTH, 1225);
    assert_eq!(ChunkGeometry::CHUNK_SIZE, 32.0);
}

#[test]
fn chunk_center_sits_half_a_chunk_from_its_origin() {
    let center = ChunkGeometry::chunk_center(ChunkCoordinate::create(1, -1));

    assert_eq!(center.x, 48.0);
    assert_eq!(center.z, -16.0);
    assert_eq!(ChunkGeometry::chunk_of(center), ChunkCoordinate::create(1, -1));
}

#[test]
fn neighbour_chunks_share_their_border_nodes() {
    let left = ChunkCoordinate::ORIGIN;
    let right = left.offset(1, 0);
    let nodes = ChunkGeometry::NODES as i32;

    for row in 0..nodes {
        let from_left = ChunkGeometry::node_world_position(left, nodes - 1, row);
        let from_right = ChunkGeometry::node_world_position(right, 0, row);

        assert_eq!(from_left.x, from_right.x);
        assert_eq!(from_left.z, from_right.z);
    }
}

#[test]
fn chunks_map_onto_regions_including_negative_coordinates() {
    let inside = ChunkCoordinate::create(5, 63);
    let negative = ChunkCoordinate::create(-1, -64);

    assert_eq!(inside.region(), RegionCoordinate::create(0, 0));
    assert_eq!(inside.local_x(), 5);
    assert_eq!(inside.local_z(), 63);

    assert_eq!(negative.region(), RegionCoordinate::create(-1, -1));
    assert_eq!(negative.local_x(), 63);
    assert_eq!(negative.local_z(), 0);
}

#[test]
fn topology_covers_every_node_once_per_grid() {
    let topology = ChunkTopology::build();

    assert_eq!(topology.index_count(), ChunkGeometry::INDEX_COUNT);

    let mut referenced = vec![false; ChunkGeometry::NODE_COUNT as usize];

    for index in topology.indices() {
        referenced[*index as usize] = true;
    }

    assert!(referenced.into_iter().all(|value| value));
}

#[test]
fn flat_topology_faces_up() {
    let topology = ChunkTopology::build();

    for triangle in topology.indices().chunks_exact(3) {
        let first = node_position(triangle[0]);
        let second = node_position(triangle[1]);
        let third = node_position(triangle[2]);

        assert!((second - first).cross(third - first).y > 0.0);
    }
}

#[test]
fn cell_is_split_along_the_anti_diagonal() {
    let topology = ChunkTopology::build();
    let nodes = ChunkGeometry::NODES;

    assert_eq!(
        topology.indices()[..6],
        [0, nodes, 1, nodes, nodes + 1, 1]
    );
}
