    use assert_proc::assert_proc;
    use mocktoffel::mock;
        #[assert_proc]
        #[assert_duplicated = "FooMock"]
        #[mock]
        #[derive(Default)]
        struct Foo {
            foo: i32
        }


