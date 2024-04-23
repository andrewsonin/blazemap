#![allow(clippy::module_name_repetitions)]

mod key_wrapper;
mod key_wrapper_bounded;
mod plain_id;

#[cfg(all(test, not(loom)))]
mod tests {
    use crate::{
        define_key_wrapper, define_key_wrapper_bounded, define_plain_id, prelude::BlazeMapId,
    };

    #[cfg(feature = "serde")]
    mod serde_compatible {
        use crate::{
            define_key_wrapper, define_key_wrapper_bounded, define_plain_id, traits::BlazeMapId,
        };

        #[test]
        fn key_wrapper() {
            define_key_wrapper! {
                struct BlazeMapKeyExample(String);
                Derive(as for Original Type): {
                    Default,
                    Debug,
                    Display,
                    Ord,
                    Serialize,
                    Deserialize
                }
            }

            let first = BlazeMapKeyExample::new("first".to_string());
            let second = BlazeMapKeyExample::new("second".to_string());
            assert_eq!(first.get_offset(), 0);
            assert_eq!(second.get_offset(), 1);
            assert_eq!(serde_json::ser::to_string(&first).unwrap(), r#""first""#);
            assert_eq!(serde_json::ser::to_string(&second).unwrap(), r#""second""#);
        }

        #[test]
        fn plain_id() {
            define_plain_id! {
                struct BlazeMapIdExample;
                Derive: {
                    Ord,
                    Serialize
                }
            }

            let first = BlazeMapIdExample::new();
            let second = BlazeMapIdExample::new();
            assert_eq!(first.get_offset(), 0);
            assert_eq!(second.get_offset(), 1);
            assert_eq!(serde_json::ser::to_string(&first).unwrap(), "0");
            assert_eq!(serde_json::ser::to_string(&second).unwrap(), "1");
        }

        #[test]
        fn key_wrapper_bounded() {
            define_key_wrapper_bounded! {
                struct BlazeMapKeyExample(String);
                MAX_CAP = 2;
                Derive(as for Original Type): {
                    Default,
                    Debug,
                    Display,
                    Ord,
                    Serialize,
                    Deserialize
                }
            }

            let first = BlazeMapKeyExample::new("first".to_string());
            let second = BlazeMapKeyExample::new("second".to_string());
            assert_eq!(first.get_offset(), 0);
            assert_eq!(second.get_offset(), 1);
            assert_eq!(serde_json::ser::to_string(&first).unwrap(), r#""first""#);
            assert_eq!(serde_json::ser::to_string(&second).unwrap(), r#""second""#);
        }

        #[test]
        #[should_panic(expected = "capacity 2 overflow")]
        fn key_wrapper_bounded_overflow() {
            define_key_wrapper_bounded! {
                struct BlazeMapKeyExample(String);
                MAX_CAP = 2;
                Derive(as for Original Type): {
                    Default,
                    Debug,
                    Display,
                    Ord,
                    Serialize,
                    Deserialize
                }
            }

            let _first = BlazeMapKeyExample::new("first".to_string());
            let _second = BlazeMapKeyExample::new("second".to_string());
            let _third = BlazeMapKeyExample::new("third".to_string());
        }
    }

    #[test]
    fn key_wrapper() {
        define_key_wrapper! {
            struct BlazeMapKeyExample1(usize);
            Derive(as for Original Type): {
                Default,
                Debug,
                Display,
                Ord
            }
        }

        define_key_wrapper! {
            struct BlazeMapKeyExample2(usize);
            Derive(as for Original Type): {
                Default,
                Debug,
                Display,
                PartialOrd
            }
        }

        define_key_wrapper! {
            struct BlazeMapKeyExample3(usize);
            Derive(as for Original Type): {
                Default,
                Debug,
                Display
            };
            Derive(as for usize): {
                Ord
            }
        }

        define_key_wrapper! {
            struct BlazeMapKeyExample4(usize);
            Derive(as for Original Type): {
                Default,
                Debug,
                Display
            };
            Derive(as for usize): {
                PartialOrd
            }
        }
    }

    #[test]
    fn plain_id() {
        define_plain_id! {
            struct BlazeMapIdExample1;
            Derive: {
                Ord
            }
        }

        define_plain_id! {
            struct BlazeMapIdExample2;
            Derive: {
                PartialOrd
            }
        }

        let first = BlazeMapIdExample1::new();
        let second = BlazeMapIdExample1::new();
        assert_eq!(first.get_offset(), 0);
        assert_eq!(second.get_offset(), 1);

        let first = BlazeMapIdExample2::new();
        let second = BlazeMapIdExample2::new();
        assert_eq!(first.get_offset(), 0);
        assert_eq!(second.get_offset(), 1);
    }

    #[test]
    fn key_wrapper_bounded() {
        define_key_wrapper_bounded! {
            struct BlazeMapKeyExample1(usize);
            MAX_CAP = 2;
            Derive(as for Original Type): {
                Default,
                Debug,
                Display,
                Ord
            }
        }

        define_key_wrapper_bounded! {
            struct BlazeMapKeyExample2(usize);
            MAX_CAP = 2;
            Derive(as for Original Type): {
                Default,
                Debug,
                Display,
                PartialOrd
            }
        }

        define_key_wrapper_bounded! {
            struct BlazeMapKeyExample3(usize);
            MAX_CAP = 2;
            Derive(as for Original Type): {
                Default,
                Debug,
                Display
            };
            Derive(as for usize): {
                Ord
            }
        }

        define_key_wrapper_bounded! {
            struct BlazeMapKeyExample4(usize);
            MAX_CAP = 2;
            Derive(as for Original Type): {
                Default,
                Debug,
                Display
            };
            Derive(as for usize): {
                PartialOrd
            }
        }
    }
}
