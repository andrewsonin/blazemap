#![cfg(feature = "loom")]

use blazemap::{
    define_key_wrapper, define_key_wrapper_bounded, define_plain_id,
    loom::TestableId,
    prelude::BlazeMapIdWrapper,
    sync::RwLock,
    traits::{CapacityInfoProvider, TypeInfoContainer},
};
use loom::{sync::Arc, thread};
use std::string::ToString;

fn run_model<F>(f: F)
where
    F: Fn() + Sync + Send + 'static,
{
    use std::{
        io::Write,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
    };

    let iters = Arc::new(AtomicU32::new(0));
    let iters1 = iters.clone();

    loom::model(move || {
        iters.fetch_add(1, Ordering::Relaxed);
        f();
    });

    let iters = iters1.load(Ordering::Relaxed);
    #[allow(clippy::explicit_write)] // print even when stdout is captured
    write!(std::io::stdout(), "[{iters} iters] ").unwrap();
}

const STRING_0: &str = "0 zero";
const STRING_1: &str = "1 first";
const STRING_2: &str = "2 second";

static LAZY_STRING_0: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| STRING_0.to_string());
static LAZY_STRING_1: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| STRING_1.to_string());
static LAZY_STRING_2: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| STRING_2.to_string());

#[test]
fn key_wrapper_cmp() {
    define_key_wrapper! {
        struct Id(String)
    }
    run_model(|| {
        use blazemap::type_info_containers::key_wrapper::StaticContainer;

        let type_info_container = Arc::new(RwLock::new(StaticContainer::new()));
        let key_0 = Arc::new(unsafe { Id::new(&type_info_container, LAZY_STRING_0.clone()) });

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t1 = thread::spawn(move || {
            let key_1 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_1.clone()) };
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_1 > key_0)
        });

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t2 = thread::spawn(move || {
            let key_2 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_2.clone()) };
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_2 > key_0)
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}

#[test]
fn key_wrapper_all_instances_iter() {
    define_key_wrapper! {
        struct Id(String)
    }
    run_model(|| {
        use blazemap::type_info_containers::key_wrapper::StaticContainer;

        let type_info_container = Arc::new(RwLock::new(StaticContainer::new()));
        let _key_0 = unsafe { Id::new(&type_info_container, LAZY_STRING_0.clone()) };

        let type_info_container_clone = type_info_container.clone();
        let t1 = thread::spawn(move || {
            let key_1 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_1.clone()) };
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_1.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                num_iters += 1;
                let _ = instance > key_1;
                let _ = instance == key_1;
            }
            assert!(num_iters >= 1);
        });

        let type_info_container_clone = type_info_container.clone();
        let t2 = thread::spawn(move || {
            let key_2 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_2.clone()) };
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_2.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                num_iters += 1;
                let _ = instance > key_2;
                let _ = instance == key_2;
            }
            assert!(num_iters >= 1);
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}

#[test]
fn key_wrapper_bounded_cmp() {
    define_key_wrapper_bounded! {
        struct Id(String);
        MAX_CAP = 3
    }
    run_model(|| {
        use blazemap::type_info_containers::key_wrapper_bounded::StaticContainer;

        let type_info_container = Arc::new(StaticContainer::new());
        let key_0 = Arc::new(unsafe { Id::new(&type_info_container, LAZY_STRING_0.clone()) });

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t1 = thread::spawn(move || {
            let key_1 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_1.clone()) };
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_1 > key_0)
        });

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t2 = thread::spawn(move || {
            let key_2 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_2.clone()) };
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_2 > key_0)
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}

#[test]
fn key_wrapper_bounded_all_instances_iter() {
    define_key_wrapper_bounded! {
        struct Id(String);
        MAX_CAP = 3
    }
    run_model(|| {
        use blazemap::type_info_containers::key_wrapper_bounded::StaticContainer;

        let type_info_container = Arc::new(StaticContainer::new());
        let _key_0 = unsafe { Id::new(&type_info_container, LAZY_STRING_0.clone()) };

        let type_info_container_clone = type_info_container.clone();
        let t1 = thread::spawn(move || {
            let key_1 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_1.clone()) };
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_1.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                if instance == key_1 {
                    // Skip this case as it may cause an RwLock deadlock due to multiple reads
                    // from the current thread, which cannot happen in the prod stage.
                    continue;
                }
                num_iters += 1;
                let _ = instance > key_1;
                let _ = instance == key_1;
            }
            assert!(num_iters >= 1);
        });

        let type_info_container_clone = type_info_container.clone();
        let t2 = thread::spawn(move || {
            let key_2 = unsafe { Id::new(&type_info_container_clone, LAZY_STRING_2.clone()) };
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_2.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                if instance == key_2 {
                    // Skip this case as it may cause an RwLock deadlock due to multiple reads
                    // from the current thread, which cannot happen in the prod stage.
                    continue;
                }
                num_iters += 1;
                let _ = instance > key_2;
                let _ = instance == key_2;
            }
            assert!(num_iters >= 1);
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}

#[test]
fn plain_id_cmp() {
    define_plain_id! {
        struct Id
    }
    run_model(|| {
        use blazemap::type_info_containers::plain_id::StaticContainer;

        let type_info_container = Arc::new(StaticContainer::new());
        let key_0 = Arc::new(Id::new(&type_info_container));

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t1 = thread::spawn(move || {
            let key_1 = Id::new(&type_info_container_clone);
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_1 > key_0)
        });

        let type_info_container_clone = type_info_container.clone();
        let key_0_clone = key_0.clone();
        let t2 = thread::spawn(move || {
            let key_2 = Id::new(&type_info_container_clone);
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let key_0 = TestableId::new(*key_0_clone, &type_info_container_clone);
            assert!(key_2 > key_0)
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}

#[test]
fn plain_id_all_instances_iter() {
    define_plain_id! {
        struct Id
    }
    run_model(|| {
        use blazemap::type_info_containers::plain_id::StaticContainer;

        let type_info_container = Arc::new(StaticContainer::new());
        let _key_0 = Id::new(&type_info_container);

        let type_info_container_clone = type_info_container.clone();
        let t1 = thread::spawn(move || {
            let key_1 = Id::new(&type_info_container_clone);
            let key_1 = TestableId::new(key_1, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_1.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                num_iters += 1;
                let _ = instance > key_1;
                let _ = instance == key_1;
            }
            assert!(num_iters >= 1);
        });

        let type_info_container_clone = type_info_container.clone();
        let t2 = thread::spawn(move || {
            let key_2 = Id::new(&type_info_container_clone);
            let key_2 = TestableId::new(key_2, &type_info_container_clone);
            let mut num_iters = 0;
            for instance in key_2.all_instances_iter() {
                let instance = TestableId::new(instance, &type_info_container_clone);
                num_iters += 1;
                let _ = instance > key_2;
                let _ = instance == key_2;
            }
            assert!(num_iters >= 1);
        });

        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(
            type_info_container
                .capacity_info_provider()
                .offset_capacity(),
            3
        );
    });
}
