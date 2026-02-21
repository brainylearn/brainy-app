use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::common::injector::injector::Injector;

pub struct InjectorScope<'a> {
    injector: &'a Injector,
    resolved_scopes: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl<'a> InjectorScope<'a> {
    pub fn new(injector: &'a Injector) -> Self {
        Self {
            injector,
            resolved_scopes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn resolve<T: Any + Send + Sync + ?Sized + 'static>(&'a self) -> Arc<T> {
        if let Some(singleton) = self
            .injector
            .singleton_registry()
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
        {
            return singleton;
        }

        // TODO: this and singleton are duplicate, DRY
        if let Some(scoped) = self
            .resolved_scopes
            .lock()
            .await
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
        {
            return scoped;
        }

        if let Some(factory) = self
            .injector
            .scoped_factory_registry()
            .get(&TypeId::of::<T>())
        {
            let boxed = factory(self).await;
            if let Some(scoped) = boxed.downcast_ref::<Arc<T>>().cloned() {
                self.resolved_scopes
                    .lock()
                    .await
                    .insert(TypeId::of::<T>(), Box::new(scoped.clone()));
                return scoped;
            }
        }

        panic!("Could not resolve {}", type_name::<T>())
    }
}
