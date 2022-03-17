pub use functions::runtime as calling_runtime;
pub use functions::typecheck as calling_typecheck;
pub use internals::runtime as internals_runtime;
pub use internals::typecheck as internals_typecheck;
pub use simple::runtime as simple_runtime;
pub use simple::typecheck as simple_typecheck;

mod simple;
mod internals;
mod functions;

