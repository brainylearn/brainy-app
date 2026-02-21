use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::injector::Injector;

#[async_trait]
pub trait ScopeInjectable {
    async fn from_injector_scope(scope: &InjectorScope<'_>) -> Self;
}

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
        if let Some(singleton) = find_by_type::<T>(self.injector.singleton_registry()) {
            return singleton;
        }

        if let Some(scoped) = find_by_type::<T>(&*self.resolved_scopes.lock().await) {
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

fn find_by_type<T: Any + Send + Sync + ?Sized + 'static>(
    hash_map: &HashMap<TypeId, Box<dyn Any + Send + Sync>>,
) -> Option<Arc<T>> {
    hash_map
        .get(&TypeId::of::<T>())
        .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
        .cloned()
}
