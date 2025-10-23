# Double Trait

A procedural macro to derive a mirror of a trait designed to make it easier to implement test doubles.

## Usage

```rust
// Given a trait `MyTrait` with derived dummy implementations in tests
#[cfg_attr(test, double_trait::dummies)]
trait MyTrait {
    fn answer(&self) -> i32;

    fn some_other_method(&self);
}

// ...

#[cfg(test)]
mod tests {
    #[test]
    fn test_function_using_org_trait() {
        // When implementing a Stub for `MyTrait
        struct Stub;
        impl MyTrait for Stub {
            fn answer(&self) -> i32 {
                42
            }
        }

        // Then `Stub` also implements `MyTrait`. Despite only implementing one of the methods
        // explictily.
        assert_eq!(42, OrgTrait::answer(&Stub));
    }
}
```
