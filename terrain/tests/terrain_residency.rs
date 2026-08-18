use glam::Vec3;
use terrain::{ChunkCoordinate, ChunkGeometry, ResidencyLimits, TerrainResidency};

fn observer() -> Vec3 {
    Vec3::new(16.0, 0.0, 16.0)
}

fn limits() -> ResidencyLimits {
    ResidencyLimits {
        max_level: 4,
        split_factor: 2.0,

        budget: usize::MAX,
        capacity: usize::MAX,
    }
}

fn settle(residency: &mut TerrainResidency, limits: ResidencyLimits) -> usize {
    let mut rounds = 0;

    loop {
        let update = residency.update(observer(), limits);

        residency.publish_visible(&update.visible);

        let settled = update.requested.is_empty();

        for coordinate in update.requested {
            residency.mark_resident(coordinate);
        }

        for coordinate in residency.retired(observer(), limits) {
            residency.mark_released(coordinate);
        }

        if settled {
            return rounds;
        }

        rounds += 1;

        assert!(rounds < 1000, "residency never settled");
    }
}

#[test]
fn the_first_round_asks_for_the_roots_not_the_leaves() {
    let residency = TerrainResidency::create();

    let update = residency.update(observer(), limits());

    assert!(!update.requested.is_empty());

    for coordinate in update.requested.iter() {
        assert_eq!(
            coordinate.level,
            limits().max_level,
            "nothing finer than a root can be asked for before the roots exist"
        );
    }

    assert_eq!(
        update.requested.len(),
        ((2 * TerrainResidency::root_span(limits().split_factor) + 1).pow(2)) as usize
    );
}

#[test]
fn a_node_is_not_split_until_all_four_children_are_resident() {
    let mut residency = TerrainResidency::create();

    let root = ChunkGeometry::chunk_of(observer(), limits().max_level);

    residency.mark_resident(root);

    let children = root.children();

    for child in children.iter().take(3) {
        residency.mark_resident(*child);
    }

    let update = residency.update(observer(), limits());

    assert!(
        !residency.retired(observer(), limits()).contains(&root),
        "the parent must stay while a child is missing"
    );
    assert!(update.requested.contains(&children[3]));
}

#[test]
fn a_split_parent_stays_loaded_but_stops_being_drawn() {
    let mut residency = TerrainResidency::create();

    let root = ChunkGeometry::chunk_of(observer(), limits().max_level);

    residency.mark_resident(root);

    for child in root.children() {
        residency.mark_resident(child);
    }

    let update = residency.update(observer(), limits());

    assert!(
        !residency.retired(observer(), limits()).contains(&root),
        "an ancestor must stay loaded so a missing leaf only costs one level"
    );
    assert!(
        !update.visible.contains(&root),
        "an ancestor must not be drawn on top of its children"
    );
    assert!(
        root.children().iter().all(|child| update.visible.contains(child)),
        "the children take over the drawing"
    );
}

#[test]
fn losing_one_leaf_falls_back_by_a_single_level() {
    let mut residency = TerrainResidency::create();

    settle(&mut residency, limits());

    let leaf = ChunkGeometry::chunk_of(observer(), 0);

    assert!(residency.is_resident(leaf));

    residency.mark_released(leaf);

    let visible = residency.visible(observer(), limits());

    let fallback = visible
        .iter()
        .find(|coordinate| {
            TerrainResidency::distance_to(observer(), **coordinate) == 0.0
        })
        .copied();

    assert_eq!(
        fallback.map(|coordinate| coordinate.level),
        Some(1),
        "a missing leaf must fall back to its parent, not further"
    );
}

#[test]
fn the_settled_tree_covers_its_roots_without_holes_or_overlap() {
    let mut residency = TerrainResidency::create();

    settle(&mut residency, limits());

    let update = residency.update(observer(), limits());

    assert!(update.requested.is_empty());
    assert!(residency.retired(observer(), limits()).is_empty());

    let covered: f64 = residency
        .visible(observer(), limits())
        .into_iter()
        .map(|coordinate| {
            let size = ChunkGeometry::chunk_size(coordinate.level) as f64;

            size * size
        })
        .sum();

    let span = TerrainResidency::root_span(limits().split_factor);
    let roots = (2 * span + 1).pow(2) as f64;
    let root_size = ChunkGeometry::chunk_size(limits().max_level) as f64;

    assert!(
        (covered - roots * root_size * root_size).abs() < 1.0,
        "settled coverage must equal the root area exactly"
    );
}

#[test]
fn the_budget_caps_how_much_arrives_per_round() {
    let residency = TerrainResidency::create();

    let mut limits = limits();
    limits.budget = 3;

    assert_eq!(residency.update(observer(), limits).requested.len(), 3);
}

#[test]
fn the_capacity_stops_the_tree_from_growing() {
    let mut residency = TerrainResidency::create();

    let mut limits = limits();
    limits.capacity = 9;

    settle(&mut residency, limits);

    assert!(
        residency.resident_count() <= 9,
        "the tree must not exceed its capacity, got {}",
        residency.resident_count()
    );
}

#[test]
fn coarse_nodes_are_requested_before_fine_ones() {
    let mut residency = TerrainResidency::create();

    let mut previous = u32::MAX;

    for _ in 0..4 {
        let update = residency.update(observer(), limits());

        for coordinate in update.requested.iter() {
            assert!(
                coordinate.level <= previous,
                "detail must not overtake coverage"
            );

            previous = coordinate.level;
        }

        for coordinate in update.requested {
            residency.mark_resident(coordinate);
        }
    }
}

#[test]
fn the_settled_tree_reaches_level_zero_under_the_observer() {
    let mut residency = TerrainResidency::create();

    settle(&mut residency, limits());

    assert!(residency.is_resident(ChunkGeometry::chunk_of(observer(), 0)));
}

#[test]
fn a_node_sees_the_delta_of_a_coarser_neighbour() {
    let mut residency = TerrainResidency::create();

    let fine = ChunkCoordinate::create(3, 0, 0);

    let east = ChunkGeometry::chunk_center(fine.offset(1, 0));
    let coarse = ChunkGeometry::chunk_of(east, 2);

    residency.publish_visible(&[fine, coarse]);

    assert_eq!(coarse, ChunkCoordinate::create(1, 0, 2));
    assert_eq!(residency.level_deltas(fine, 4), [0, 2, 0, 0]);
}

#[test]
fn a_settled_tree_only_steps_one_level_between_neighbours() {
    let mut residency = TerrainResidency::create();

    settle(&mut residency, limits());

    let update = residency.update(observer(), limits());

    assert!(update.requested.is_empty());

    for coordinate in residency.visible(observer(), limits()) {
        for delta in residency.level_deltas(coordinate, limits().max_level) {
            assert!(delta <= 1, "{coordinate:?} borders a much coarser neighbour");
        }
    }
}

#[test]
fn nodes_of_different_levels_never_collide_as_keys() {
    let mut residency = TerrainResidency::create();

    let fine = ChunkCoordinate::create(0, 0, 0);
    let coarse = ChunkCoordinate::create(0, 0, 3);

    residency.mark_resident(fine);

    assert!(residency.is_resident(fine));
    assert!(!residency.is_resident(coarse));
}

#[test]
fn moving_far_away_retires_the_old_set() {
    let mut residency = TerrainResidency::create();

    settle(&mut residency, limits());

    let far = Vec3::new(100_000.0, 0.0, 100_000.0);

    let update = residency.update(far, limits());

    assert!(!residency.retired(far, limits()).is_empty());
    assert!(!update.requested.is_empty());
}

#[test]
fn children_arrive_as_whole_groups() {
    let mut residency = TerrainResidency::create();

    let mut limits = limits();
    limits.budget = usize::MAX;

    for coordinate in residency.update(observer(), limits).requested {
        residency.mark_resident(coordinate);
    }

    limits.budget = 3;

    assert!(
        residency.update(observer(), limits).requested.is_empty(),
        "a group of four must not be split across rounds"
    );

    limits.budget = 4;

    assert_eq!(residency.update(observer(), limits).requested.len(), 4);
}

#[test]
fn a_bounded_budget_still_settles() {
    let mut residency = TerrainResidency::create();

    let rounds = settle(&mut residency, ResidencyLimits::create());

    assert!(rounds > 1, "a bounded budget must take several rounds");
    assert!(residency.resident_count() > 100);
}

#[test]
fn an_ancestor_is_never_drawn_together_with_its_children() {
    let mut residency = TerrainResidency::create();

    let mut limits = limits();
    limits.budget = usize::MAX;

    for coordinate in residency.update(observer(), limits).requested {
        residency.mark_resident(coordinate);
    }

    let root = ChunkGeometry::chunk_of(observer(), limits.max_level);

    let children = residency.update(observer(), limits).requested;

    assert!(children.iter().all(|child| child.level == root.level - 1));

    for coordinate in children {
        residency.mark_resident(coordinate);
    }

    let visible = residency.visible(observer(), limits);

    assert!(
        !visible.contains(&root),
        "the parent must stop drawing as soon as its children are bound, not a frame later"
    );
}
