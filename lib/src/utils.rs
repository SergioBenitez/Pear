#[macro_export]
macro_rules! whitelist {
    ($input:expr, $($inner:tt)*) => ($crate::ParseResult::Done($($inner)*));
}

#[macro_export]
macro_rules! unlift {
    ($input:expr, $($inner:tt)*) => {
        $($inner)*
    }
}

#[macro_export]
macro_rules! from {
    ($input:expr, $result:expr) => ({
        match parse!($input, $result) {
            $crate::ParseResult::Done(result) => $crate::ParseResult::from(result),
            $crate::ParseResult::Error(e) => $crate::ParseResult::Error(e)
        }
    });
}

// Idea: Have this know about the parser's name when it can.
#[macro_export]
macro_rules! parse_error {
    ($input:expr, $name:expr, $error:expr) => (
        $crate::ParseError::custom($name, $error)
    );
}

