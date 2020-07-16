use pear::macros::parse;

use json::*;

fn main() {
    let test = r#"
    {
        "Image": {
            "Width":  800,
            "Height": 600,
            "Title":  "View from 15th Floor",
            "Thumbnail": {
                "Url":    "http://www.example.com/image/481989943",
                "Height": 125,
                "Width":  100e10
            },
            "Animated" : false,
            "IDs": [116, 943, 234, 38793)
        },
        "escaped characters": "\u2192\uD83D\uDE00\"\t\uD834\uDD1E"
    }"#;

    let result = parse!(value: &mut pear::input::Text::from(test));
    match result {
        Ok(v) => println!("Value: {:#?}", v),
        Err(e) => println!("Error: {}", e)
    }
}
