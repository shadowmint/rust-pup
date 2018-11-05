#[macro_export]
macro_rules! format_log {
    ({$($arg2:tt)+} , { $($key:expr => $value:expr),+ }) => ({
        $crate::Record::new(&format!($($arg2)+))
            .property("module", module_path!())
            .property("file", file!())
            .property("line", &format!("{}", line!()))
            $(.property($key, $value))+
    });
    ($($arg:tt)+) => ({
       $crate::Record::new(&format!($($arg)+))
                .property("module", module_path!())
                .property("file", file!())
                .property("line", &format!("{}", line!()))
    });
}