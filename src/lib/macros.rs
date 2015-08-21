macro_rules! deref {
    ($name:ident::$field:tt => $target:ty) => (itemize! {
        impl ::std::ops::Deref for $name {
            type Target = $target;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }
    });
    (mut $name:ident::$field:tt => $target:ty) => (itemize! {
        impl ::std::ops::DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    });
}

macro_rules! itemize(($($blob:item)*) => ($($blob)*));

macro_rules! getter {
    (ref $name:ident: $kind:ty) => (
        #[inline(always)]
        pub fn $name(&self) -> &$kind {
            &self.$name
        }
    );
    ($name:ident: $kind:ty) => (
        #[inline(always)]
        pub fn $name(&self) -> $kind {
            self.$name
        }
    );
}

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => return Err(Box::new(error)),
    });
);

macro_rules! order {
    ($name:ident($field:tt) ascending) => (order! { $name($field) Less < Greater });
    ($name:ident($field:tt) descending) => (order! { $name($field) Greater < Less });
    ($name:ident($field:tt) $less:ident < $greater:ident) => (itemize! {
        impl ::std::cmp::Eq for $name {
        }

        impl ::std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                if self.$field < other.$field {
                    ::std::cmp::Ordering::$less
                } else if self.$field > other.$field {
                    ::std::cmp::Ordering::$greater
                } else {
                    ::std::cmp::Ordering::Equal
                }
            }
        }

        impl ::std::cmp::PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl ::std::cmp::PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    });
}

macro_rules! path(
    ($config:ident, $destination:expr) => ({
        let path = some!($config.get::<String>("path"), "the path to {} is missing", $destination);
        let mut path = ::std::path::PathBuf::from(path);
        if path.is_relative() {
            if let Some(ref root) = $config.get::<::std::path::PathBuf>("root") {
                path = root.join(path);
            }
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        path
    });
);

macro_rules! raise(
    ($message:expr) => (return Err(Box::new(::ErrorString($message.to_string()))));
    ($($arg:tt)*) => (return Err(Box::new(::ErrorString(format!($($arg)*)))));
);

macro_rules! rc {
    ($(#[$attr:meta])* pub struct $outer:ident($inner:ident) $body:tt) => (
        $(#[$attr])*
        pub struct $outer(::std::rc::Rc<$inner>);

        rewrite! { [$(#[$attr])* pub struct $inner] [] $body }
        deref! { $outer::0 => $inner }
    );
    ($outer:ident($inner:ident { $($field:ident: $value:expr),* })) => (
        rc!($outer($inner { $($field: $value,)* }))
    );
    ($outer:ident($inner:ident { $($field:ident: $value:expr,)* })) => (
        $outer(::std::rc::Rc::new($inner { $($field: $value,)* }))
    );
}

macro_rules! rewrite(
    ($header:tt [$($member:tt)*] {}) => (
        rewrite!(output $header [$($member)*]);
    );
    ($header:tt [$($member:tt)*] { pub $name:ident: $kind:ty, $($t:tt)* }) => (
        rewrite!($header [$($member)* pub $name: $kind,] { $($t)* });
    );
    ($header:tt [$($member:tt)*] { $name:ident: $kind:ty, $($t:tt)* }) => (
        rewrite!($header [$($member)* $name: $kind,] { $($t)* });
    );
    (output [$($chunk:tt)*] [$($member:tt)*]) => (
        itemize!(
            $($chunk)* {
                $($member)*
            }
        );
    );
);

macro_rules! some(
    ($option:expr) => (match $option {
        Some(value) => value,
        _ => raise!("encountered a logic error"),
    });
    ($option:expr, $($arg:tt)+) => (match $option {
        Some(value) => value,
        _ => raise!($($arg)*),
    });
);
