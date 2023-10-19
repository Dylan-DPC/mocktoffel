use assert_proc::assert_proc;
use mocktoffel::{mock, toffel};

#[mock]
#[derive(Default)]
struct Bar {
    foo: i32,
}

#[assert_proc]
#[toffel]
#[derive(Default)]
struct Foo {
    #[assert_field_type = BarMock]
    #[mocked]
    foo: Bar,
}

// #[assert_proc]
#[toffel]
enum Baz {
    Never(#[mocked] Bar),
    Gonna,
    Let,
    You,
    Down,
}
