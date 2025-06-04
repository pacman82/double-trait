# Double Trait

A procedural macro to derive a mirror of a trait designed to make it easier to implement test doubles.

## Usage

```rust
#[cfg(test)]
use double_trait::double;

// Given an original trait with a derived `DummyTrait` test double
#[cfg_attr(test, double(DummyTrait))]
trait OrgTrait {
    fn answer(&self) -> i32;

    fn some_other_method(&self);
}

// ...

#[cfg(test)]
mod tests {
    #[test]
    fn test_function_using_org_trait() {
        // When implementing Dummy trait for a Stub.
        struct Stub;
        impl DummyTrait for Stub {
            fn answer(&self) -> i32 {
                42
            }
        }

        // Then `Stub` also implements `OrgTrait`.
        assert_eq!(42, OrgTrait::answer(&Stub));
    }
}
```
