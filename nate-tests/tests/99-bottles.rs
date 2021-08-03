use std::fmt::Result;

use nate::Nate;

const EXPECTED: &str = r#"5 bottles of beer on the wall.
5 bottles of beer.
Take one down, pass it around.

4 bottles of beer on the wall.
4 bottles of beer.
Take one down, pass it around.

3 bottles of beer on the wall.
3 bottles of beer.
Take one down, pass it around.

2 bottles of beer on the wall.
2 bottles of beer.
Take one down, pass it around.

1 bottle of beer on the wall.
1 bottle of beer.
Take one down, pass it around."#;

#[test]
fn five_bottles_of_beer() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/99-bottles.html")]
    struct Template {
        limit: usize,
    }

    let buf = format!("{}", Template { limit: 5 });
    assert_eq!(buf, EXPECTED);
    Ok(())
}

#[test]
fn five_bottles_of_beer_implicit_braces() -> Result {
    #[derive(Nate)]
    #[template(path = "templates/99-bottles-implicit-braces.html")]
    struct Template {
        limit: usize,
    }

    let buf = format!("{}", Template { limit: 5 });
    assert_eq!(buf, EXPECTED);
    Ok(())
}
