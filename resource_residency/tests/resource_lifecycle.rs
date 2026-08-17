mod common;

use common::Harness;

#[test]
fn acquire_sync_publishes_the_resource_immediately() {
    let harness = Harness::create();

    let handle = harness.provider.acquire_sync(1);

    let resource = harness
        .provider
        .get_resource(handle.id)
        .expect("acquire_sync must publish the resource before returning");

    assert_eq!(resource.config, 1);
    assert_eq!(harness.created(), 1);
}

#[test]
fn identical_configs_share_one_handle() {
    let harness = Harness::create();

    let first = harness.provider.acquire_sync(1);
    let second = harness.provider.acquire_sync(1);

    assert_eq!(first.id.inner, second.id.inner);
    assert_eq!(harness.created(), 1);
}

#[test]
fn different_configs_get_their_own_resources() {
    let harness = Harness::create();

    let first = harness.provider.acquire_sync(1);
    let second = harness.provider.acquire_sync(2);

    assert_ne!(first.id.inner, second.id.inner);
    assert_eq!(harness.created(), 2);
    assert_eq!(
        harness
            .provider
            .get_resource(second.id)
            .expect("second resource must be available")
            .config,
        2
    );
}

#[test]
fn scheduled_creation_is_published_only_after_it_runs() {
    let harness = Harness::create();

    let handle = harness.provider.get_or_load(1);

    harness.advance(Harness::FRAMES_IN_FLIGHT);

    assert_eq!(harness.scheduled_creations(), 1);
    assert!(harness.provider.get_resource(handle.id).is_none());

    assert_eq!(harness.run_scheduled_creations(), 1);

    harness.advance(1);

    assert_eq!(
        harness
            .provider
            .get_resource(handle.id)
            .expect("resource must be published once its creation ran")
            .config,
        1
    );
}

#[test]
fn resource_stays_alive_while_a_handle_is_held() {
    let harness = Harness::create();

    let handle = harness.provider.acquire_sync(1);

    harness.advance(Harness::FRAMES_IN_FLIGHT * 4);

    assert_eq!(harness.destroyed(), 0);
    assert!(harness.provider.get_resource(handle.id).is_some());
}

#[test]
fn dropped_resource_survives_the_frames_in_flight_delay() {
    let harness = Harness::create();

    drop(harness.provider.acquire_sync(1));

    harness.advance(1);

    assert_eq!(
        harness.destroyed(),
        0,
        "a resource must outlive the frames still in flight"
    );

    harness.advance_until("resource destroyed", || harness.destroyed() == 1);

    assert_eq!(harness.erased(), 1);
}

#[test]
fn resource_dropped_before_creation_finished_is_destroyed() {
    let harness = Harness::create();

    drop(harness.provider.get_or_load(1));

    harness.advance(Harness::FRAMES_IN_FLIGHT * 2);

    assert_eq!(harness.destroyed(), 0);

    harness.run_scheduled_creations();

    harness.advance_until("resource destroyed", || harness.destroyed() == 1);

    assert_eq!(harness.created(), 1);
}

#[test]
fn index_is_not_reused_while_creation_is_in_flight() {
    let harness = Harness::create();

    let first = harness.provider.get_or_load(1);
    let first_id = first.id;

    drop(first);

    harness.advance(Harness::FRAMES_IN_FLIGHT * 2);

    let second = harness.provider.get_or_load(2);

    assert_ne!(
        second.id.inner, first_id.inner,
        "an index whose creation has not finished must not be handed out again"
    );
}

#[test]
fn later_resource_is_not_served_by_a_previous_occupant() {
    let harness = Harness::create();

    drop(harness.provider.acquire_sync(1));

    harness.advance_until("first resource destroyed", || harness.destroyed() == 1);

    let second = harness.provider.acquire_sync(2);

    assert_eq!(
        harness
            .provider
            .get_resource(second.id)
            .expect("second resource must be available")
            .config,
        2
    );
}

#[test]
fn failed_creation_frees_the_cache_entry() {
    let harness = Harness::with_failures(1);

    let first = harness.provider.get_or_load(1);
    let first_id = first.id;

    harness.run_scheduled_creations();

    harness.advance(1);

    let second = harness.provider.get_or_load(1);

    assert_ne!(
        second.id.inner, first_id.inner,
        "a failed load must not keep serving its handle from the cache"
    );

    harness.run_scheduled_creations();

    harness.advance(1);

    assert!(harness.provider.get_resource(second.id).is_some());
    assert_eq!(harness.created(), 1);
}

#[test]
fn released_indices_are_reused_up_to_capacity() {
    let harness = Harness::create();

    for config in 0..Harness::CAPACITY * 3 {
        drop(harness.provider.acquire_sync(config));

        harness.advance_until("resource destroyed", || harness.destroyed() == config + 1);
    }

    assert_eq!(harness.created(), Harness::CAPACITY * 3);
    assert_eq!(harness.destroyed(), Harness::CAPACITY * 3);
}

#[test]
#[should_panic(expected = "Out of resource indices!")]
fn exhausting_the_index_space_panics() {
    let harness = Harness::create();

    let _handles: Vec<_> = (0..Harness::CAPACITY + 1)
        .map(|config| harness.provider.acquire_sync(config))
        .collect();
}

#[test]
#[should_panic(expected = "Failed to unwrap resource")]
fn holding_a_resource_handle_panics_the_provider() {
    let harness = Harness::create();

    let handle = harness.provider.acquire_sync(1);
    let _resource = harness
        .provider
        .get_resource(handle.id)
        .expect("resource must be available after acquire_sync");

    drop(handle);

    harness.advance(Harness::FRAMES_IN_FLIGHT * 3);
}
