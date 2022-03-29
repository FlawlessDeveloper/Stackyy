use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::sync::Arc;

pub use descriptors::Descriptor;
pub use descriptors::DescriptorAction;
use descriptors::execute_fn as descriptors_runtime;
use descriptors::type_check_fn as descriptors_typecheck;
use functions::runtime as calling_runtime;
use functions::typecheck as calling_typecheck;
use internals::runtime as internals_runtime;
use internals::typecheck as internals_typecheck;
use simple::runtime as simple_runtime;
use simple::typecheck as simple_typecheck;

use crate::parser::Function;
use crate::util::operation::OperationData;
use crate::util::type_check::{TypeCheckError, Types};
use crate::VM;

mod simple;
mod internals;
mod functions;
mod descriptors;


pub const SIMPLE_TYPECHECK: SyncLazy<Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>> = SyncLazy::new(|| {
    Arc::new(simple_typecheck::create_push_type_check())
});

pub const SIMPLE_RUNTIME: SyncLazy<Arc<Box<dyn Fn(&OperationData, &mut VM)>>> = SyncLazy::new(|| {
    Arc::new(simple_runtime::create_push())
});

pub const CALLING_TYPECHECK: SyncLazy<Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>> = SyncLazy::new(|| {
    Arc::new(calling_typecheck::create_calling_type_check())
});

pub const CALLING_RUNTIME: SyncLazy<Arc<Box<dyn Fn(&OperationData, &mut VM)>>> = SyncLazy::new(|| {
    Arc::new(calling_runtime::create_fn())
});

pub const INTERNAL_TYPECHECK: SyncLazy<Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>> = SyncLazy::new(|| {
    Arc::new(internals_typecheck::get_internal_typecheck())
});

pub const INTERNAL_RUNTIME: SyncLazy<Arc<Box<dyn Fn(&OperationData, &mut VM)>>> = SyncLazy::new(|| {
    Arc::new(internals_runtime::get_internal_executor())
});

pub const DESCRIPTOR_TYPECHECK: SyncLazy<Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>> = SyncLazy::new(|| {
    Arc::new(descriptors::type_check_fn())
});

pub const DESCRIPTOR_RUNTIME: SyncLazy<Arc<Box<dyn Fn(&OperationData, &mut VM)>>> = SyncLazy::new(|| {
    Arc::new(descriptors::execute_fn())
});