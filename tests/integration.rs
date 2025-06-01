use derive_double::double;

#[double(MyEmptyTraitDummy)]
trait MyEmptyTrait {}

#[allow(dead_code)]
struct MyStruct;

impl MyEmptyTraitDummy for MyStruct {}
