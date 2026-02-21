use std::{
    any::{Any, TypeId},
    collections::HashMap,
    pin::Pin,
    sync::Arc,
};

use crate::common::injector::injector_scope::InjectorScope;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type FactoryFunction = dyn Send
    + Sync
    + for<'a> Fn(&'a InjectorScope<'a>) -> BoxFuture<'a, Box<dyn Any + Send + Sync>>;

pub struct Injector {
    singleton_registry: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    scoped_factory_registry: HashMap<TypeId, Box<FactoryFunction>>,
}

impl Injector {
    pub fn new() -> Self {
        Self {
            singleton_registry: HashMap::new(),
            scoped_factory_registry: HashMap::new(),
        }
    }

    pub fn singleton_registry(&self) -> &HashMap<TypeId, Box<dyn Any + Send + Sync>> {
        &self.singleton_registry
    }

    pub fn scoped_factory_registry(&self) -> &HashMap<TypeId, Box<FactoryFunction>> {
        &self.scoped_factory_registry
    }

    pub fn register_singleton<T: Any + Send + Sync + ?Sized>(&mut self, implementation: Arc<T>) {
        self.singleton_registry
            .insert(TypeId::of::<T>(), Box::new(implementation));
    }

    pub fn register_factory<T: Any + Send + Sync + ?Sized + 'static>(
        &mut self,
        factory: impl for<'a> Fn(&'a InjectorScope) -> BoxFuture<'a, Arc<T>> + Send + Sync + 'static,
    ) {
        self.scoped_factory_registry.insert(
            TypeId::of::<T>(),
            Box::new(move |scope: &InjectorScope| {
                let fut = factory(scope);
                Box::pin(async move {
                    let result: Arc<T> = fut.await;
                    Box::new(result) as Box<dyn Any + Send + Sync>
                }) as BoxFuture<Box<dyn Any + Send + Sync>>
            }),
        );
    }

    pub fn start_scope(&self) -> InjectorScope<'_> {
        InjectorScope::new(self)
    }
}
