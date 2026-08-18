use glam::Vec3;
use terrain::{ChunkCoordinate, ChunkGeometry, TerrainResidency};

const MAX_LEVEL: u32 = 4;
const SPLIT_FACTOR: f32 = 2.0;

fn observer() -> Vec3 {
    Vec3::new(16.0, 0.0, 16.0)
}

fn covered_area(desired: &std::collections::HashSet<ChunkCoordinate>) -> f64 {
    desired
        .iter()
        .map(|coordinate| {
            let size = ChunkGeometry::chunk_size(coordinate.level) as f64;

            size * size
        })
        .sum()
}

#[test]
fn the_tree_covers_its_roots_without_holes_or_overlap() {
    let desired = TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR);

    let roots = (2 * TerrainResidency::root_span(SPLIT_FACTOR) + 1).pow(2) as f64;
    let root_size = ChunkGeometry::chunk_size(MAX_LEVEL) as f64;

    assert!(
        (covered_area(&desired) - roots * root_size * root_size).abs() < 1.0,
        "the covered area must equal the area of the root nodes exactly"
    );
}

#[test]
fn the_chunk_under_the_observer_is_split_to_the_finest_level() {
    let desired = TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR);

    let under_observer = ChunkGeometry::chunk_of(observer(), 0);

    assert!(
        desired.contains(&under_observer),
        "the observer must stand on a level 0 chunk"
    );
}

#[test]
fn distant_nodes_stay_coarse() {
    let desired = TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR);

    let coarsest = desired.iter().map(|coordinate| coordinate.level).max();

    assert_eq!(
        coarsest,
        Some(MAX_LEVEL),
        "far corners must survive as whole root nodes"
    );
}

#[test]
fn the_level_of_a_node_tracks_its_distance() {
    let desired = TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR);

    for coordinate in desired.iter() {
        let distance = TerrainResidency::distance_to(observer(), *coordinate);
        let size = ChunkGeometry::chunk_size(coordinate.level);

        if coordinate.level > 0 {
            assert!(
                distance >= size * SPLIT_FACTOR,
                "{coordinate:?} was kept coarse while sitting {distance} away"
            );
        }

        if coordinate.level < MAX_LEVEL {
            let parent = coordinate.parent();
            let parent_distance = TerrainResidency::distance_to(observer(), parent);
            let parent_size = ChunkGeometry::chunk_size(parent.level);

            assert!(
                parent_distance < parent_size * SPLIT_FACTOR,
                "{coordinate:?} exists although its parent at {parent_distance} should have been kept whole"
            );
        }
    }
}

#[test]
fn a_larger_split_factor_gives_more_detail() {
    let coarse = TerrainResidency::desired(observer(), MAX_LEVEL, 1.0);
    let fine = TerrainResidency::desired(observer(), MAX_LEVEL, 4.0);

    assert!(fine.len() > coarse.len());
}

#[test]
fn a_deeper_tree_reaches_further() {
    let shallow = TerrainResidency::desired(observer(), 2, SPLIT_FACTOR);
    let deep = TerrainResidency::desired(observer(), 5, SPLIT_FACTOR);

    assert!(covered_area(&deep) > covered_area(&shallow));
}

#[test]
fn nodes_of_different_levels_never_collide_as_keys() {
    let mut residency = TerrainResidency::create();

    let fine = ChunkCoordinate::create(0, 0, 0);
    let coarse = ChunkCoordinate::create(0, 0, 3);

    residency.mark_resident(fine);

    assert!(residency.is_resident(fine));
    assert!(!residency.is_resident(coarse));
    assert_eq!(residency.resident_count(), 1);
}

#[test]
fn moving_far_away_retires_the_old_set() {
    let mut residency = TerrainResidency::create();

    for coordinate in TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR) {
        residency.mark_resident(coordinate);
    }

    let far = Vec3::new(100_000.0, 0.0, 100_000.0);

    let update = residency.update(far, MAX_LEVEL, SPLIT_FACTOR);

    assert_eq!(update.retired.len(), residency.resident_count());
    assert!(!update.requested.is_empty());
}

#[test]
fn a_settled_observer_asks_for_nothing() {
    let mut residency = TerrainResidency::create();

    for coordinate in TerrainResidency::desired(observer(), MAX_LEVEL, SPLIT_FACTOR) {
        residency.mark_resident(coordinate);
    }

    let update = residency.update(observer(), MAX_LEVEL, SPLIT_FACTOR);

    assert!(update.requested.is_empty());
    assert!(update.retired.is_empty());
}

#[test]
fn the_update_does_not_change_residency_on_its_own() {
    let residency = TerrainResidency::create();

    residency.update(observer(), MAX_LEVEL, SPLIT_FACTOR);

    assert_eq!(residency.resident_count(), 0);
}

#[test]
fn releasing_a_chunk_makes_it_requestable_again() {
    let mut residency = TerrainResidency::create();

    let coordinate = ChunkGeometry::chunk_of(observer(), 0);

    residency.mark_resident(coordinate);
    residency.mark_released(coordinate);

    assert_eq!(residency.resident_count(), 0);
    assert!(
        residency
            .update(observer(), MAX_LEVEL, SPLIT_FACTOR)
            .requested
            .contains(&coordinate)
    );
}

#[test]
fn the_default_settings_fit_the_mesh_budget() {
    let desired = TerrainResidency::desired(
        observer(),
        TerrainResidency::DEFAULT_MAX_LEVEL,
        TerrainResidency::DEFAULT_SPLIT_FACTOR,
    );

    let reach = ChunkGeometry::chunk_size(TerrainResidency::DEFAULT_MAX_LEVEL)
        * TerrainResidency::root_span(TerrainResidency::DEFAULT_SPLIT_FACTOR) as f32;

    println!("default tree: {} nodes, reach {reach} m", desired.len());

    assert!(
        desired.len() < 400,
        "the default tree must fit the mesh budget, got {} nodes",
        desired.len()
    );
}
