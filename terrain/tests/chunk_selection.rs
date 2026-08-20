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
        rebuild_margin: ResidencyLimits::DEFAULT_REBUILD_MARGIN,

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

#[test]
fn drifting_inside_the_margin_leaves_the_anchor_where_it_was() {
    let anchor = observer();
    let drift = Vec3::new(anchor.x + limits().rebuild_margin * 0.9, anchor.y, anchor.z);

    assert_eq!(ChunkSelection::anchor(anchor, drift, limits()), anchor);
}

#[test]
fn leaving_the_margin_takes_the_anchor_along() {
    let anchor = observer();
    let far = Vec3::new(anchor.x + limits().rebuild_margin * 1.1, anchor.y, anchor.z);

    assert_eq!(ChunkSelection::anchor(anchor, far, limits()), far);
}

#[test]
fn falling_never_moves_the_anchor() {
    let anchor = observer();
    let below = Vec3::new(anchor.x, anchor.y - 10_000.0, anchor.z);

    assert_eq!(
        ChunkSelection::anchor(anchor, below, limits()),
        anchor,
        "detail follows the ground plane, so height alone must not trigger a rebuild"
    );
}

#[test]
fn a_camera_orbiting_inside_the_margin_never_touches_the_terrain() {
    let target = observer();
    let radius = 4.0_f32;

    let mut anchor = target;
    let mut previous = selected_set();

    let mut changed = 0;

    for step in 0..720 {
        let angle = (step as f32).to_radians();
        let position = target + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

        anchor = ChunkSelection::anchor(anchor, position, limits());

        let selected = ChunkSelection::select(anchor, limits())
            .into_iter()
            .collect::<HashSet<_>>();

        changed += usize::from(selected != previous);

        previous = selected;
    }

    assert_eq!(
        changed, 0,
        "an orbit smaller than the margin must not re-select, re-stitch or rebuild anything"
    );
}
