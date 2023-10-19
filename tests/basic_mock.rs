use assert_proc::assert_proc;
use mocktoffel::mock;

#[assert_proc]
#[assert_duplicated = FooMock]
#[mock]
#[derive(Debug, Default)]
struct Foo {
    foo: String,
}

fn main() {}

#[assert_proc]
#[assert_duplicated = BarMock]
#[mock]
#[derive(Default)]
enum Bar {
    #[default]
    Never,
    Gonna,
    Give,
    You,
    Up,
}
