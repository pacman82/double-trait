// We are more interested that the code compiles and  not so much in the actual functionality.
#![allow(dead_code)]

use double_trait::{Dummy, dummies};

#[test]
fn invoke_method_on_partial_implementation_of_trait() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> i32;

        fn some_other_method(&self);
    }

    // When overriding default implementation of `answer` in `MyTrait`
    struct MyStruct;
    impl MyTrait for MyStruct {
        fn answer(&self) -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `MyTrait::answer` via `MyStruct`
    assert_eq!(42, MyTrait::answer(&MyStruct));
    // Also, implicitly tested by virtue of this compling, is that `MyStruct` is an implementation
    // of `MyTrait`, even though `some_other_method` is **not** implemented.
}

#[tokio::test]
async fn partially_implement_async_trait() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        async fn answer(&self) -> i32;

        async fn foobar(&self);
    }

    // When ovverriding default implementation of `answer` in `MyTrait`
    struct MyStruct;
    impl MyTrait for MyStruct {
        async fn answer(&self) -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, MyTrait::answer(&MyStruct).await);
}

#[tokio::test]
async fn associated_async_method_invocation() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        async fn answer() -> i32;
    }

    // When overriding default implementation of `answer` in `MyTrait`
    struct MyStruct;
    impl MyTrait for MyStruct {
        async fn answer() -> i32 {
            42
        }
    }

    // The new implementation is used than invoking `OrgTrait::answer` via `MyStruct`
    assert_eq!(42, <MyStruct as MyTrait>::answer().await);
}

#[tokio::test]
async fn impl_future_method_invocation() {
    use std::future::Future;
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> impl Future<Output = i32>;
    }

    // When overriding default implementation of `answer` in `MyTrait`
    struct MyStruct;
    impl MyTrait for MyStruct {
        fn answer(&self) -> impl Future<Output = i32> {
            async { 42 }
        }
    }

    // The new implementation is used than invoking `MyTrait::answer` via `MyStruct`
    assert_eq!(42, MyTrait::answer(&MyStruct).await);
}

#[tokio::test]
async fn annotate_traits_with_impl_iterator() {
    // Given an original trait with two methods returning `impl Iterator`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> impl Iterator<Item = String>;

        fn question(&self) -> impl Iterator<Item = String>;
    }

    // When overriding only one of them
    struct MyStruct;
    impl MyTrait for MyStruct {
        fn answer(&self) -> impl Iterator<Item = String> {
            (0..1).map(|i| format!("Item {}", i))
        }
    }

    // Then MyStruct is an implementation of MyTrait due to default methods
    assert_eq!("Item 0", MyTrait::answer(&MyStruct).next().unwrap());
}

#[test]
fn dummy_implements_my_trait() {
    // When annotating dummies for `MyTrait`
    #[dummies]
    trait MyTrait {}

    // Then `Dummy` implements `MyTrait`
    fn use_trait(_: impl MyTrait) {}
    use_trait(Dummy);
}

#[test]
fn trait_with_existing_default_method_impl() {
    // Compliation test. Test assertion is, that this does not fail to compile.
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> i32 {
            42
        }
    }
}

#[test]
fn trait_with_associated_types() {
    // Compliation test. Test assertion is, that this does not fail to compile.
    #[dummies]
    trait OrgTrait {
        type AssociatedType;
    }
}

#[test]
#[should_panic(expected = "not implemented: MyTrait::answer")]
fn calling_unimplemented_double_method_mentions_method_name() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> i32;
    }

    // When invoking the default implementation of `answer`
    MyTrait::answer(&Dummy);

    // Then the error message mentions the method name
}

#[test]
fn don_t_panic_on_invoking_method_with_no_return_type() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self);
    }

    // When invoking the default implementation of `answer`
    Dummy.answer();

    // Then no panic occurs
}

#[tokio::test]
async fn don_t_panic_on_invoking_method_returning_future_with_unit() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> impl Future<Output = ()>;
    }

    // When invoking the default implementation of `answer`
    Dummy.answer().await;

    // Then no panic occurs
}

#[tokio::test]
async fn future_of_iterator() {
    // Given an original trait with a method `answer`
    #[dummies]
    trait MyTrait {
        fn answer(&self) -> impl Future<Output = impl Iterator<Item = i32>>;
    }

    // When invoking the default implementation of `answer`
    let values: Vec<_> = Dummy.answer().await.collect();

    // Then
    assert!(values.is_empty())
}
