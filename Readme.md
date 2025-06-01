# Double Derive

A procedural macro to derive a mirror of a trait designed to make it easier to implement test doubles.

Might just work for you, but early in development.

```rust
// Given an original trait with a derived `DummyTrait` test double
#[double(DummyTrait)]
trait OrgTrait {
    fn answer(&self) -> i32;

    fn some_other_method(&self);
}

// When implementing `DummyTrait` for a struct `MyStruct`
struct MyStruct;
impl DummyTrait for MyStruct {
    fn answer(&self) -> i32 {
        42
    }
}

// Then `MyStruct` also implements `OrgTrait`.
assert_eq!(42, OrgTrait::answer(&MyStruct));
```