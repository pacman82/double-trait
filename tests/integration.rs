// We are more interested that the code compiles and  not so much in the actual functionality.
#![allow(dead_code)]

use derive_double::double;

#[test]
fn implement_double_instead_of_original_trait() {
    // Given an empty trait
    #[double(MyEmptyTraitDummy)]
    trait MyEmptyTrait {}

    // When implementing the double for a struct
    struct MyStruct;
    impl MyEmptyTraitDummy for MyStruct {}

    // Then the struct implements the original trait
    fn use_trait(_: impl MyEmptyTrait) {
        // This function is just a placeholder to ensure the trait is used
    }
    use_trait(MyStruct);
}

#[test]
fn invoke_implemented_method_through_original_trait() {
    // Given an empty trait
    #[double(DummyTrait)]
    trait OrgTrait {
        fn answer(&self) -> i32;
    }

    // When implementing the double for a struct
    struct MyStruct;
    impl DummyTrait for MyStruct {
        fn answer(&self) -> i32 {
            42
        }
    }

    assert_eq!(42, OrgTrait::answer(&MyStruct));
}