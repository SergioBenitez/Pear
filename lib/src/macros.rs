#[macro_export]
macro_rules! is_debug {
    () => ({
        #[cfg(debug_assertions)]
        let result = ::std::env::var("PEAR_DEBUG").is_ok();
        #[cfg(not(debug_assertions))]
        let result = false;
        result
    });

    ($kind:expr) => ({
        #[cfg(debug_assertions)]
        let result = ::std::env::var("PEAR_DEBUG").map(|v| v == $kind).unwrap_or(false);
        #[cfg(not(debug_assertions))]
        let result = false;
        result
    })
}

#[macro_export]
macro_rules! pear_declare {
    (pub($($inner:tt)+) $($rest:tt)*) => { _pear_declare!([pub($($inner)+)] $($rest)*); };
    (pub $($rest:tt)*) => { _pear_declare!([pub] $($rest)*); };
    ($($rest:tt)*) => { _pear_declare!([] $($rest)*); }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _pear_declare {
    ([$($vis:tt)*] $input:ident $(<$($gen:tt),+>)* (Token = $t:ty, Slice = $s:ty, Many = $m:ty)) => {
        _pear_declare!([$($vis)*] $input $(<$($gen),+>)*($t, $s, $s, $m));
    };

    ([$($vis:tt)*] $input:ident $(<$($gen:tt),+>)* (Token = $t:ty, Slice = $s:ty, InSlice = $is:ty, Many = $m:ty)) => {
        _pear_declare!([$($vis)*] $input $(<$($gen),+>)*($t, $s, $is, $m));
    };

    ([$($vis:tt)*] $input:ident $(<$($gen:tt),+>)* ($t:ty, $s:ty, $is:ty, $m:ty)) => {
        $($vis)* trait $input $(<$($gen),+>)*: $crate::Input<Token=$t, Slice=$s, InSlice=$is, Many=$m> {  }

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

#[macro_export]
macro_rules! pear_error {
    ([$name:ident; $i:expr] $err:expr) => (pear_error!([$name; $i] $err,));
    ([$name:ident; $i:expr] $fmt:expr, $($arg:tt)*) => {
        $crate::ParseErr::from_context($i, stringify!($name), format!($fmt, $($arg)*))
    };
}

/// FIXME: This is an issue with rustc here where if `$input` is `expr`
/// everything fails.
#[macro_export]
macro_rules! pear_try {
    ([$name:ident; $input:ident] $e:expr) => {{
        switch! { [$name;$input] result@$e => { Some(result) }, _ => { None } }
    }};
    ([$name:ident; $input:ident] $e:expr => $r:expr) => {{
        switch! { [$name;$input] $e => { Some($r) }, _ => { None } }
    }};
    ([$name:ident; $input:ident] $pat:ident@$e:expr => $r:expr) => {{
        switch! { [$name;$input] $pat@$e => { Some($r) }, _ => { None } }
    }}
}
