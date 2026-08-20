use glam::Vec3;
use std::collections::HashSet;
use terrain::{ChunkCoordinate, ChunkGeometry, ChunkSelection, ResidencyLimits};

const SIDES: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

fn observer() -> Vec3 {
    Vec3::new(16.0, 0.0, 16.0)
}

fn limits() -> ResidencyLimits {
    ResidencyLimits {
        max_level: 4,
        split_factor: 2.0,

        capacity: usize::MAX,

        ray_tracing_distance: ResidencyLimits::DEFAULT_RAY_TRACING_DISTANCE,
    }
}

fn selected_set() -> HashSet<ChunkCoordinate> {
    ChunkSelection::select(observer(), limits()).into_iter().collect()
}

fn selected_level_at(selected: &HashSet<ChunkCoordinate>, point: Vec3) -> Option<u32> {
    (0..=limits().max_level).find(|level| selected.contains(&ChunkGeometry::chunk_of(point, *level)))
}

#[test]
fn the_same_observer_always_selects_the_same_chunks() {
    assert_eq!(
        ChunkSelection::select(observer(), limits()),
        ChunkSelection::select(observer(), limits()),
        "selection carries no state, so it cannot drift between calls"
    );
}

#[test]
fn the_observer_stands_on_a_level_zero_chunk() {
    assert!(selected_set().contains(&ChunkGeometry::chunk_of(observer(), 0)));
}

#[test]
fn detail_drops_off_with_distance() {
    let selected = ChunkSelection::select(observer(), limits());

    let finest = selected.iter().map(|coordinate| coordinate.level).min();
    let coarsest = selected.iter().map(|coordinate| coordinate.level).max();

    assert_eq!(finest, Some(0));
    assert_eq!(coarsest, Some(limits().max_level));
}

#[test]
fn the_selection_tiles_its_roots_without_holes_or_overlap() {
    let selected = ChunkSelection::select(observer(), limits());

    assert_eq!(
        selected.len(),
        selected.iter().copied().collect::<HashSet<_>>().len(),
        "a chunk must not be selected twice"
    );

    let covered: f64 = selected
        .iter()
        .map(|coordinate| {
            let size = ChunkGeometry::chunk_size(coordinate.level) as f64;

            size * size
        })
        .sum();

    let span = ChunkSelection::root_span(limits().split_factor);
    let roots = (2 * span + 1).pow(2) as f64;
    let root_size = ChunkGeometry::chunk_size(limits().max_level) as f64;

    assert!(
        (covered - roots * root_size * root_size).abs() < 1.0,
        "covered area must equal the root area exactly, got {covered}"
    );
}

#[test]
fn an_ancestor_is_never_selected_together_with_its_children() {
    let selected = selected_set();

    for coordinate in selected.iter() {
        let mut ancestor = *coordinate;

        while ancestor.level < limits().max_level {
            ancestor = ancestor.parent();

            assert!(
                !selected.contains(&ancestor),
                "{ancestor:?} is drawn on top of its own child {coordinate:?}"
            );
        }
    }
}

#[test]
fn neighbours_never_differ_by_more_than_one_level() {
    for coordinate in ChunkSelection::select(observer(), limits()) {
        for delta in ChunkSelection::level_deltas(coordinate, observer(), limits()) {
            assert!(delta <= 1, "{coordinate:?} borders a much coarser neighbour");
        }
    }
}

#[test]
fn the_deltas_report_the_level_of_the_actually_selected_neighbour() {
    let selected = selected_set();

    let mut transitions = 0;

    for coordinate in selected.iter() {
        let deltas = ChunkSelection::level_deltas(*coordinate, observer(), limits());

        for (side, (x, z)) in SIDES.iter().enumerate() {
            let probe = ChunkGeometry::chunk_center(coordinate.offset(*x, *z));

            let Some(level) = selected_level_at(&selected, probe) else {
                continue;
            };

            assert_eq!(
                deltas[side],
                level.saturating_sub(coordinate.level),
                "{coordinate:?} side {side} disagrees with the neighbour that is really drawn"
            );

            transitions += usize::from(deltas[side] > 0);
        }
    }

    assert!(
        transitions > 0,
        "a tree with several levels must have level transitions to stitch"
    );
}
