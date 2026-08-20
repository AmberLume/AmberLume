use glam::Vec3;
use std::collections::HashSet;
use terrain::{ChunkCoordinate, ChunkEviction, ChunkGeometry, ChunkSelection, ResidencyLimits};

fn observer() -> Vec3 {
    Vec3::new(16.0, 0.0, 16.0)
}

fn limits(capacity: usize) -> ResidencyLimits {
    ResidencyLimits {
        max_level: 4,
        split_factor: 2.0,

        capacity,
        rebuild_margin: ResidencyLimits::DEFAULT_REBUILD_MARGIN,

        ray_tracing_distance: ResidencyLimits::DEFAULT_RAY_TRACING_DISTANCE,
    }
}

fn row(count: i32) -> Vec<ChunkCoordinate> {
    (0..count)
        .map(|x| ChunkCoordinate::create(x, 0, 0))
        .collect()
}

#[test]
fn nothing_is_evicted_below_capacity() {
    let loaded = row(10);

    assert!(ChunkEviction::excess(&loaded, &[], observer(), 100).is_empty());
}

#[test]
fn exactly_the_excess_leaves() {
    let loaded = row(10);

    assert_eq!(ChunkEviction::excess(&loaded, &[], observer(), 7).len(), 3);
}

#[test]
fn the_farthest_chunks_go_first() {
    let loaded = row(10);
    let near = ChunkGeometry::chunk_center(loaded[0]);

    let evicted = ChunkEviction::excess(&loaded, &[], near, 7);

    assert_eq!(
        evicted.iter().copied().collect::<HashSet<_>>(),
        loaded[7..].iter().copied().collect::<HashSet<_>>(),
        "pressure must fall on the chunks the observer is least likely to come back to"
    );
}

#[test]
fn a_chunk_that_is_on_screen_is_never_evicted() {
    let loaded = row(10);
    let near = ChunkGeometry::chunk_center(loaded[0]);

    let evicted = ChunkEviction::excess(&loaded, &loaded, near, 1);

    assert!(
        evicted.is_empty(),
        "everything is selected, so pressure has nothing it is allowed to drop"
    );
}

#[test]
fn an_observer_moving_back_and_forth_never_evicts_below_capacity() {
    let limits = limits(ResidencyLimits::DEFAULT_CAPACITY);

    let here = observer();
    let there = Vec3::new(here.x + 400.0, here.y, here.z);

    let mut loaded: HashSet<ChunkCoordinate> = HashSet::new();

    for step in 0..120 {
        let position = if step % 2 == 0 { here } else { there };

        let selected = ChunkSelection::select(position, limits);

        loaded.extend(selected.iter().copied());

        let evicted = ChunkEviction::excess(
            &loaded.iter().copied().collect::<Vec<_>>(),
            &selected,
            position,
            limits.capacity,
        );

        assert!(
            evicted.is_empty(),
            "below capacity nothing is dropped, so an observer going back and forth never reloads"
        );
    }

    assert!(
        loaded.len() < limits.capacity,
        "the test only proves anything while the union really fits, got {}",
        loaded.len()
    );
}

#[test]
fn moving_far_away_drops_the_old_area_under_pressure() {
    let limits = limits(0);

    let here = observer();
    let far = Vec3::new(here.x + 100_000.0, here.y, here.z);

    let near_selection = ChunkSelection::select(here, limits);
    let far_selection = ChunkSelection::select(far, limits);

    let loaded = near_selection
        .iter()
        .chain(far_selection.iter())
        .copied()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let evicted = ChunkEviction::excess(&loaded, &far_selection, far, far_selection.len());

    let near_selection = near_selection.into_iter().collect::<HashSet<_>>();
    let far_selection = far_selection.into_iter().collect::<HashSet<_>>();

    assert!(!evicted.is_empty(), "the old area must be released once it no longer fits");
    assert!(evicted.iter().all(|coordinate| !far_selection.contains(coordinate)));
    assert!(evicted.iter().all(|coordinate| near_selection.contains(coordinate)));
}
