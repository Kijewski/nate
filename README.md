## **N**ot **a** **T**emplate **E**ngine

This is not a template engine, but sugar to implicitly call `write!(…)` like in PHP.
The only difference is that the output gets XML escaped automatically unless opted-out explicitly.

E.g.

*   templates/greeting.html:

    ```xml
    <h1>Hello, {{user}}!</h1>
    ```

    The path is relative to the cargo manifest dir (where you find Cargo.toml) of the calling.

*   src/main.rs:

    ```rust
    use nate::Nate;
    
    #[derive(Nate)]
    #[template(path = "templates/greeting.html")]
    struct Greetings<'a> {
        user: &'a str,
    }
    
    fn main() {
        let mut output = String::new();
        let tmpl = Greetings { user: "<World>" };
        write!(output, "{}", tmpl).unwrap();
        println!("{}", output);
    }
    ```

*   Output:

    ```html
    <h1>Hello, &#60;World&#62;!</h1>
    ```

No new traits are introduced, instead using `#[derive(Nate)]` works by implementing fmt::Display.
This also makes nesting of NaTE templates possible.

A more complex example would be:  

*   src/main.rs:

    ```rust
    use nate::Nate;

    #[derive(Nate)]
    #[template(path = "templates/99-bottles.html")]
    struct Template {
        limit: usize,
    }

    #[test]
    fn ninetynine_bottles_of_beer() {
        print!("{}", Template { limit: 99 });
    }
    ```

*   templates/99-bottles.txt:

    ```html
    {%-
        for i in (1..=self.limit).rev() {
            if i == 1 {
    -%}
    1 bottle of beer on the wall.
    1 bottle of beer.
    Take one down, pass it around.
    {%-
            } else {
    -%}
    {{i}} bottles of beer on the wall.
    {{i}} bottles of beer.
    Take one down, pass it around.

    {%
            }
        }
    -%}
    ```

Inside of a `{% code block %}` you can write any and all rust code.

Values in `{{ value blocks }}` are printed XML escaped.

Values in `{{{ raw blocks }}}` are printed verbatim.

For values in `{{{{ debug blocks }}}}` their debug message is printed as in `"{:?}"`.

For values in `{{{{{ verbose blocks }}}}}` their debug message is printed verbose as in `"{:#?}"`.

With `{< include >}` block you can include a template file.
It then behaves like it was copy-pasted into the current file.
If the path starts with "." or "..", the file is searched relative to the current file.
Otherwise it is search in the project root.

Using hyphens `-` at the start/end of a block, whitespaces before/after the block are trimmed.

Data blocks `{{…}}` to `{{{{{…}}}}}` and includes `{<…>}` must not be empty.
Code `{%…%}` and comment `{#…#}` blocks may be empty.

Blocks don't need to be closed at the end of the file.

To debug any errors you can add an argument as in `#[template(output = "some/path/generated.rs")]`.
The generated code is stored in there even if there were parsing errors in the Rust code.
The path is relative to the project root (where your Cargo.toml lives).
