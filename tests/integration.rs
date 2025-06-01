#![allow(dead_code)]

use derive_double::double;

#[double(MyEmptyTraitDummy)]
trait MyEmptyTrait {}

struct MyStruct;

impl MyEmptyTraitDummy for MyStruct {}
