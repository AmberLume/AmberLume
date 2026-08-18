use glam::Vec3;
use terrain::{ChunkCoordinate, ChunkGeometry, TerrainResidency};

fn observer_at(coordinate: ChunkCoordinate) -> Vec3 {
    ChunkGeometry::chunk_center(coordinate)
}

#[test]
fn the_chunk_under_the_observer_is_always_requested() {
    let residency = TerrainResidency::create();

    let coordinate = ChunkCoordinate::create(2, -3);

    assert!(
        residency
            .update(observer_at(coordinate), 1)
            .requested
            .contains(&coordinate)
    );
}

#[test]
fn one_chunk_of_reach_covers_every_neighbour_including_diagonals() {
    let residency = TerrainResidency::create();

    let update = residency.update(observer_at(ChunkCoordinate::ORIGIN), 1);

    for z in -1..=1 {
        for x in -1..=1 {
            let coordinate = ChunkCoordinate::create(x, z);

            assert!(
                update.requested.contains(&coordinate),
                "{coordinate:?} must be loaded, diagonals included"
            );
        }
    }

    assert_eq!(update.requested.len(), 9);
}

#[test]
fn chunks_further_than_the_load_distance_stay_out() {
    let residency = TerrainResidency::create();

    let update = residency.update(observer_at(ChunkCoordinate::ORIGIN), 1);

    assert!(!update.requested.contains(&ChunkCoordinate::create(2, 0)));
    assert!(!update.requested.contains(&ChunkCoordinate::create(0, -2)));
}

#[test]
fn the_reach_is_kept_ahead_from_anywhere_inside_the_chunk() {
    let residency = TerrainResidency::create();

    let load_distance = 2;

    let center = observer_at(ChunkCoordinate::ORIGIN);
    let ahead = ChunkCoordinate::create(load_distance as i32, 0);

    for step in [-15.0, -8.0, 0.0, 8.0, 15.0] {
        let observer = Vec3::new(center.x + step, center.y, center.z);

        assert!(
            residency.update(observer, load_distance).requested.contains(&ahead),
            "at offset {step} the chunk {load_distance} ahead must already be loaded"
        );
    }
}

#[test]
fn a_wider_reach_loads_strictly_more() {
    let residency = TerrainResidency::create();

    let observer = observer_at(ChunkCoordinate::ORIGIN);

    let near = residency.update(observer, 1).requested;
    let far = residency.update(observer, 3).requested;

    assert!(far.len() > near.len());

    for coordinate in near {
        assert!(far.contains(&coordinate), "{coordinate:?} must survive a wider reach");
    }
}

#[test]
fn the_default_load_distance_keeps_a_full_chunk_of_slack() {
    let residency = TerrainResidency::create();

    let update = residency.update(
        observer_at(ChunkCoordinate::ORIGIN),
        TerrainResidency::DEFAULT_LOAD_DISTANCE,
    );

    for z in -1..=1 {
        for x in -1..=1 {
            assert!(
                update.requested.contains(&ChunkCoordinate::create(x, z)),
                "the default reach must never be tighter than the immediate ring"
            );
        }
    }
}

#[test]
fn distance_is_measured_to_the_chunk_not_to_its_centre() {
    let observer = observer_at(ChunkCoordinate::ORIGIN);

    assert_eq!(
        TerrainResidency::distance_to(observer, ChunkCoordinate::ORIGIN),
        0.0
    );
    assert_eq!(
        TerrainResidency::distance_to(observer, ChunkCoordinate::create(1, 0)),
        ChunkGeometry::HALF_SIZE
    );
}

#[test]
fn resident_chunks_are_not_requested_again() {
    let mut residency = TerrainResidency::create();

    let coordinate = ChunkCoordinate::create(4, 4);

    residency.mark_resident(coordinate);

    let update = residency.update(observer_at(coordinate), 1);

    assert!(!update.requested.contains(&coordinate));
    assert!(residency.is_resident(coordinate));
}

#[test]
fn chunks_left_behind_are_retired() {
    let mut residency = TerrainResidency::create();

    let left_behind = ChunkCoordinate::create(-4, 0);

    residency.mark_resident(left_behind);

    let update = residency.update(observer_at(ChunkCoordinate::ORIGIN), 1);

    assert_eq!(update.retired, vec![left_behind]);
}

#[test]
fn a_wider_reach_keeps_a_chunk_that_a_tighter_one_drops() {
    let mut residency = TerrainResidency::create();

    let far = ChunkCoordinate::create(2, 0);

    residency.mark_resident(far);

    let observer = observer_at(ChunkCoordinate::ORIGIN);

    assert_eq!(residency.update(observer, 1).retired, vec![far]);
    assert!(residency.update(observer, 2).retired.is_empty());
}

#[test]
fn releasing_a_chunk_makes_it_requestable_again() {
    let mut residency = TerrainResidency::create();

    let coordinate = ChunkCoordinate::ORIGIN;

    residency.mark_resident(coordinate);
    residency.mark_released(coordinate);

    assert_eq!(residency.resident_count(), 0);
    assert!(
        residency
            .update(observer_at(coordinate), 1)
            .requested
            .contains(&coordinate)
    );
}

#[test]
fn the_update_does_not_change_residency_on_its_own() {
    let residency = TerrainResidency::create();

    let coordinate = ChunkCoordinate::create(-7, 9);

    residency.update(observer_at(coordinate), 1);

    assert_eq!(residency.resident_count(), 0);
    assert!(!residency.is_resident(coordinate));
}
