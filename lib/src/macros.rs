#[macro_export]
macro_rules! is_debug {
    ($($e:tt)*) => ({
        #[cfg(debug_assertions)]
        let result = ::std::env::var("PEAR_DEBUG").is_ok();
        #[cfg(not(debug_assertions))]
        let result = false;
        result
    })
}

#[macro_export]
macro_rules! declare {
    ($input:ident $(<$($gen:tt),+>)* (Token = $t:ty, Slice = $s:ty, Many = $m:ty)) => {
        declare!($input $(<$($gen),+>)*($t, $s, $s, $m));
    };

    ($input:ident $(<$($gen:tt),+>)* (Token = $t:ty, Slice = $s:ty, InSlice = $is:ty, Many = $m:ty)) => {
        declare!($input $(<$($gen),+>)*($t, $s, $is, $m));
    };

    ($input:ident $(<$($gen:tt),+>)* ($t:ty, $s:ty, $is:ty, $m:ty)) => {
        trait $input $(<$($gen),+>)*: $crate::Input<Token=$t, Slice=$s, InSlice=$is, Many=$m> {  }

        impl<$($($gen,)+)* T> $input $(<$($gen)+>)* for T
            where T: $crate::Input<Token=$t, Slice=$s, InSlice=$is, Many=$m> + $($($gen),+)* {  }
    }
}

#[macro_export]
macro_rules! parse {
    ($parser:ident : $e:expr) => ({
        let input = $e;
        (move || {
            let result = $parser(input)?;
            $crate::parsers::eof(input)?;
            $crate::AsResult::as_result(result)
        })()
    })
}

// Idea: Have this know about the parser's name when it can.
#[macro_export]
macro_rules! parse_error {
    ($input:expr, $name:expr, $error:expr) => (
        $crate::ParseErr::new($name, $error)
    );
}

// Idea: Have this know about the parser's name when it can.
#[macro_export]
macro_rules! pear_error {
    // ($name:expr, $error:expr) => ($crate::ParseErr::new($name, $error));
    // ($error:expr) => ($crate::ParseErr::new("<unknown>", $error));
    // println!("String: {:?}", string);
    // ($fmt:expr) => { ... };
    ($name:expr, $err:expr) => (pear_error!($name, $err,));
    ($name:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::ParseErr::new($name, format!($fmt, $($arg)*))
    };
}
