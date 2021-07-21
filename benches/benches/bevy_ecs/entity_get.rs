#![feature(get_mut_unchecked)]

//! This benchmark evaluates component-by-entity acquisition performance.
//! There is entities in the world (`Point`), which is pointed by `PointEntities`.
//! In each `PointEntities` all pointed `Point`s updated.
//!
//! For performance comparison, there is also:
//! * `PointBoxes` which contains `Box<Point>`s
//! * `PointArcMutexes` which contains `Arc<Mutex<Point>>`s
//! * `PointArcs` which contains `Arc<Point>`s. Retrieval through unsafe `get_mut_unchecked`. (it is safe to do so - because only one system at a time traverse mutable data in bevy)

use std::boxed::Box;
use std::sync::Arc;
use std::sync::Mutex;
use std::mem::MaybeUninit;
use bevy::ecs::{
    world::World,
    entity::Entity,
    system::{Query, IntoSystem},
    schedule::{SystemStage, Stage}
};
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, entity_bench, box_bench, arc_mutex_bench, arc_unsafe_bench);
criterion_main!(benches);

const ENTITIES_COUNT : usize = 1000;
const POINTS_COUNT   : usize = 10;

struct Point{
    x : u32,
    y : u32,
}



struct PointEntities {
    point_entities: [Entity; POINTS_COUNT]
}

fn point_entity_system(
    mut points: Query<&mut Point>,
    point_refs: Query<&PointEntities>,
){
    for point_ref in point_refs.iter(){
    for point_entity in point_ref.point_entities{
        let mut point = points.get_mut(point_entity).unwrap();
        point.x += 1;
        point.y += 1;
    }
    }
}

fn entity_bench(criterion: &mut Criterion){
    // Setup world
    let mut world = World::default();

    // Setup stage with a system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(point_entity_system.system());

    // Setup test entities
    for i in 0..ENTITIES_COUNT {
        let mut point_refs = PointEntities { point_entities: [Entity::new(0); POINTS_COUNT]};
        let point_entity = world.spawn().insert(Point{x: i as u32, y: i as u32 }).id();
        for j in 0..POINTS_COUNT {
            point_refs.point_entities[j] = point_entity;
        }
        world.spawn().insert(point_refs);
    }

    // Run systems
    criterion.bench_function("entity", |b| b.iter(|| update_stage.run(&mut world)));
}



struct PointBoxes {
    points : [Box<Point>; POINTS_COUNT]
}

fn point_rc_system(
    mut point_rcs: Query<&mut PointBoxes>,
){
    for mut point_rc in &mut point_rcs.iter_mut(){
    for mut point_ in &mut point_rc.points{
        let mut point = &mut *point_;
        point.x += 1;
        point.y += 1;
    }
    }
}

fn box_bench(criterion: &mut Criterion){
    // Setup world
    let mut world = World::default();

    // Setup stage with a system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(point_rc_system.system());

    // Setup test entities
    for i in 0..ENTITIES_COUNT {
        let point_rcs = {
            // Create an array of uninitialized values.
            let mut array: [MaybeUninit<Box<Point>>; POINTS_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };

            for (_, element) in array.iter_mut().enumerate() {
                let foo = Box::new(Point{x: i as u32, y: i as u32 });
                *element = MaybeUninit::new(foo);
            }

            unsafe { std::mem::transmute::<_, [Box<Point>; POINTS_COUNT]>(array) }
        };
        world.spawn().insert(PointBoxes {points:point_rcs});
    }

    // Run systems
    criterion.bench_function("box unique", |b| b.iter(|| update_stage.run(&mut world)));
}



struct PointArcMutexes {
    points : [Arc<Mutex<Point>>; POINTS_COUNT]
}

fn point_arc_mutex_system(
    mut point_arcs: Query<&mut PointArcMutexes>,
){
    for mut point_arc in &mut point_arcs.iter_mut(){
    for mut point_ in &mut point_arc.points{
        let mut point = point_.lock().unwrap();
        point.x += 1;
        point.y += 1;
    }
    }
}

fn arc_mutex_bench(criterion: &mut Criterion){
    // Setup world
    let mut world = World::default();

    // Setup stage with a system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(point_arc_mutex_system.system());

    // Setup test entities
    for i in 0..ENTITIES_COUNT {
        let foo = Arc::new(Mutex::new(Point{x: i as u32, y: i as u32 }));
        let point_arcs = {
            // Create an array of uninitialized values.
            let mut array: [MaybeUninit<Arc<Mutex<Point>>>; POINTS_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
            for (_, element) in array.iter_mut().enumerate() {
                *element = MaybeUninit::new(foo.clone());
            }

            unsafe { std::mem::transmute::<_, [Arc<Mutex<Point>>; POINTS_COUNT]>(array) }
        };
        world.spawn().insert(PointArcMutexes {points: point_arcs});
    }

    // Run systems
    criterion.bench_function("arc mutex", |b| b.iter(|| update_stage.run(&mut world)));
}


struct PointARCs {
    points : [Arc<Point>; POINTS_COUNT]
}

fn point_arc_system(
    mut point_arcs: Query<&mut PointARCs>,
){
    for mut point_arc in &mut point_arcs.iter_mut(){
    for mut point_ in &mut point_arc.points{
        let mut point = unsafe{ &mut *Arc::get_mut_unchecked(&mut point_) };
        point.x += 1;
        point.y += 1;
    }
    }
}

fn arc_unsafe_bench(criterion: &mut Criterion){
    // Setup world
    let mut world = World::default();

    // Setup stage with a system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(point_arc_system.system());

    // Setup test entities
    for i in 0..ENTITIES_COUNT {
        let foo = Arc::new(Point{x: i as u32, y: i as u32 });
        let point_arcs = {
            // Create an array of uninitialized values.
            let mut array: [MaybeUninit<Arc<Point>>; POINTS_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
            for (_, element) in array.iter_mut().enumerate() {
                *element = MaybeUninit::new(foo.clone());
            }

            unsafe { std::mem::transmute::<_, [Arc<Point>; POINTS_COUNT]>(array) }
        };
        world.spawn().insert(PointARCs {points: point_arcs});
    }

    // Run systems
    criterion.bench_function("arc unsafe", |b| b.iter(|| update_stage.run(&mut world)));
}