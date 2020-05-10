1. [ ] Add a way to add activities to state
2. [ ] Don't know if states have a `name` property. But all states can have an
   id. We can change the `name` property to `id` for all states.
3. Is it better to colocate the rust parser with the chrome extension and then
   move it out if it makes sense? Let's keep it separate until we get the parser
   to work.
4. [ ] NEXT Copy other tokenizer tests from the JS version
5. [ ] Figure out how to share the input string between different test functions
6. Rust tip: If you want  to run your tests in watch mode using `cargo watch`
   and also be able to print to console in tests - 
   `cargo watch “test -- --nocapture”`
7.
