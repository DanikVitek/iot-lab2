use tracing_appender::rolling::RollingFileAppender;

pub mod config;
pub mod control;
pub mod data;
pub mod error;
pub mod service;

#[macro_export]
macro_rules! reclone {
    ($name:ident) => {
        let $name = $name.clone();
    };
    (mut $name:ident) => {
        let mut $name = $name.clone();
    };
}

pub trait KtConvenience: Sized {
    #[inline(always)]
    fn also(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }

    #[inline(always)]
    fn apply(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }

    #[inline(always)]
    fn take_if(self, predicate: impl FnOnce(&Self) -> bool) -> Option<Self> {
        if predicate(&self) {
            Some(self)
        } else {
            None
        }
    }
}

impl<T> KtConvenience for T {}

/// Struct, used for logging to both file and stdout.
pub struct FileStdoutWriter {
    file: RollingFileAppender,
}

impl FileStdoutWriter {
    pub fn new(file: RollingFileAppender) -> Self {
        Self { file }
    }
}

impl std::io::Write for FileStdoutWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)?;
        let written = self.file.write(buf)?;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        self.file.flush()
    }
}
