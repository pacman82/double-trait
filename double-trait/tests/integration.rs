// We are more interested that the code compiles and  not so much in the actual functionality.
#![allow(dead_code)]

use async_trait::async_trait;
use double_trait::{Dummy, double};

#[test]
fn implement_double_instead_of_original_trait() {
    // Given an original trait with a derived `DummyTrait` test double
    #[double(MyEmptyTraitDummy)]
    trait MyEmptyTrait {}

    // When implementing `DummyTrait` for a struct `MyStruct`
    struct MyStruct;
    impl MyEmptyTraitDummy for MyStruct {}

    // Then `MyStruct` also implements `OrgTrait`.
    fn use_trait(_: impl MyEmptyTrait) {
        // This function is just a placeholder to ensure the trait is used
    }
    use_trait(MyStruct);
}

#[test]
fn invoke_implemented_method_through_original_trait() {
    // Given an original trait with a method `answer`
    #[double(DummyTrait)]
    trait OrgTrait {
        fn answer(&self) -> i32;

        fn some_other_method(&self);
    }

    // When overriding default implementation of `answer` in `DummyTrait`
    struct MyStruct;
    impl DummyTrait for MyStruct {
        fn answer(&self) -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, OrgTrait::answer(&MyStruct));
}

#[tokio::test]
async fn async_method_invocation() {
    // Given an original trait with a method `answer`
    #[double(DummyTrait)]
    trait OrgTrait {
        async fn answer(&self) -> i32;

        async fn foobar(&self);
    }

    // When ovverriding default implementation of `answer` in `DummyTrait`
    struct MyStruct;
    impl DummyTrait for MyStruct {
        async fn answer(&self) -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, OrgTrait::answer(&MyStruct).await);
}

#[tokio::test]
async fn associated_method_invocation() {
    // Given an original trait with a method `answer`
    #[double(DummyTrait)]
    trait OrgTrait {
        async fn answer() -> i32;
    }

    // When ovverriding default implementation of `answer` in `DummyTrait`
    struct MyStruct;
    impl DummyTrait for MyStruct {
        async fn answer() -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, <MyStruct as OrgTrait>::answer().await);
}

#[tokio::test]
async fn impl_future_method_invocation() {
    use std::future::Future;
    // Given an original trait with a method `answer`
    #[double(DummyTrait)]
    trait OrgTrait {
        fn answer(&self) -> impl Future<Output = i32>;
    }

    // When overriding default implementation of `answer` in `DummyTrait`
    struct MyStruct;
    impl DummyTrait for MyStruct {
        fn answer(&self) -> impl Future<Output = i32> {
            async { 42 }
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, OrgTrait::answer(&MyStruct).await);
}

#[test]
fn dummy_implements_double_trait() {
    // When deriving a double `DummyTrait`
    #[double(DummyTrait)]
    trait OrgTrait {}

    // Then `Dummy` implements `OrgTrait`
    fn use_trait(_: impl OrgTrait) {}
    use_trait(Dummy);
}

#[tokio::test]
async fn combine_with_async_trait() {
    // Given an original trait with a method `answer`
    #[async_trait]
    #[double(DummyTrait)]
    trait OrgTrait {
        async fn answer(&self) -> i32;
    }

    // When overriding default implementation of `answer` in `DummyTrait`
    struct MyStruct;
    #[async_trait]
    impl DummyTrait for MyStruct {
        async fn answer(&self) -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, OrgTrait::answer(&MyStruct).await);
}

#[test]
fn trait_with_existing_default_method_impl() {
    // Compliation test. Test assertion is, that this does not fail to compile.
    #[double(DoubleTrait)]
    trait OrgTrait {
        fn answer(&self) -> i32 {
            42
        }
    }
}

#[test]
fn trait_with_associated_types() {
    // Compliation test. Test assertion is, that this does not fail to compile.
    #[double(DoubleTrait)]
    trait OrgTrait {
        type AssociatedType;
    }
}

#[test]
#[should_panic(expected = "not implemented: DummyTrait::answer")]
fn calling_unimplemented_double_method_mentions_method_name() {
    // Given an original trait with a method `answer`
    #[double(DummyTrait)]
    trait OrgTrait {
        fn answer(&self) -> i32;
    }

    // When invoking the default implementation of `answer`
    OrgTrait::answer(&Dummy);

    // Then the error message mentions the method name
}


// Replace me with a compilation test using `trybuild`.
// #[double(MyTraitWithImplReturnDouble)]
// trait MyTraitWithImplReturn {
//     fn method(&self) -> impl Iterator<Item = ()>;
// }
