// We are more interested that the code compiles and  not so much in the actual functionality.
#![allow(dead_code)]

use double_trait::dummies;

#[test]
fn forward_original_trait() {
    // Given an original trait with a derived `dummies`
    #[dummies]
    trait MyEmptyTrait {}

    // When implementing `MyEmptyTrait` for a struct `MyStruct`
    struct MyStruct;
    impl MyEmptyTrait for MyStruct {}

    // Then `MyStruct` also implements `OrgTrait`.
    fn use_trait(_: impl MyEmptyTrait) {
        // This function is just a placeholder to ensure the trait is used
    }
    use_trait(MyStruct);
}
