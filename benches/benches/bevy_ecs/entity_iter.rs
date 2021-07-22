use bevy::ecs::{
    world::World,
    system::{Query, IntoSystem},
    schedule::{SystemStage, Stage}
};
use std::vec::Vec;
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(
    benches,
    entity_iter_bench,
    vec_iter_bench,
    vec_box_iter_bench
);
criterion_main!(benches);

const ENTITIES_COUNT : usize = 100000;

struct Point{
    x : u32,
    y : u32,
}

fn point_entity_system(
    mut points: Query<&mut Point>,
){
    /*for mut point in points.iter_mut(){
        point.x += 1;
        point.y += 1;
    }*/

    // Virtually no difference
    points.for_each_mut(|mut point|{
        point.x += 1;
        point.y += 1;
    });
}

fn entity_iter_bench(criterion: &mut Criterion){
    // Setup world
    let mut world = World::default();

    // Setup stage with a system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(point_entity_system.system());

    // Setup test entities
    for i in 0..ENTITIES_COUNT {
        world.spawn().insert(Point{x: i as u32, y: i as u32 });
    }

    // Run systems
    criterion.bench_function("entity iter", |b| b.iter(|| update_stage.run(&mut world)));
}


fn vec_iter_bench(criterion: &mut Criterion){
    let mut vec = Vec::with_capacity(ENTITIES_COUNT);
    for i in 0..ENTITIES_COUNT {
        vec.push(Point{x: i as u32, y: i as u32 });
    }

    // Run systems
    criterion.bench_function("vec iter", |b| b.iter(||
        for mut point in &mut vec{
            point.x += 1;
            point.y += 1;
        }
    ));
}

fn vec_box_iter_bench(criterion: &mut Criterion){
    let mut vec = Vec::with_capacity(ENTITIES_COUNT);
    for i in 0..ENTITIES_COUNT {
        vec.push(Box::new(Point{x: i as u32, y: i as u32 }));
    }

    // Run systems
    criterion.bench_function("vec box iter", |b| b.iter(||
        for mut point in &mut vec{
            point.x += 1;
            point.y += 1;
        }
    ));
}