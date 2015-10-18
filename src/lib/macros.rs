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

macro_rules! ok(
    ($result:expr) => (match $result {
        Ok(result) => result,
        Err(error) => raise!(error),
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
    ($config:ident, $($argument:tt)+) => ({
        let path = some!($config.get::<String>("path"), $($argument)+);
        let mut path = ::std::path::PathBuf::from(path);
        if path.is_relative() {
            if let Some(root) = $config.get::<String>("root") {
                path = ::std::path::Path::new(root).join(path);
            }
        }
        if ::std::fs::metadata(&path).is_err() {
            raise!("the file {:?} does not exist", &path);
        }
        path
    });
);

macro_rules! raise(
    ($message:expr) => (return Err(::Error::new($message)));
    ($($argument:tt)*) => (return Err(::Error::new(format!($($argument)*))));
);

macro_rules! some(
    ($option:expr) => (match $option {
        Some(value) => value,
        _ => raise!("encountered a logic error"),
    });
    ($option:expr, $($argument:tt)+) => (match $option {
        Some(value) => value,
        _ => raise!($($argument)*),
    });
);
