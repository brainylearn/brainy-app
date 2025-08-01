// TODO: refactor
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use async_trait::async_trait;

pub trait Event: Any + Send + Sync {}

#[async_trait]
pub trait EventHandler<T: Event>: Send + Sync {
    async fn handle(&self, e: &T);
}

#[async_trait]
trait ErasedHandler: Send + Sync {
    async fn handle_erased(&self, event: &(dyn Any + Send + Sync));
}

pub struct EventBus {
    handlers: HashMap<TypeId, Vec<Box<dyn ErasedHandler>>>,
}

struct HandlerWrapper<T: Event, H: EventHandler<T>> {
    handler: H,
    _phantom: std::marker::PhantomData<T>,
}

#[async_trait]
impl<T: Event, H: EventHandler<T>> ErasedHandler for HandlerWrapper<T, H> {
    async fn handle_erased(&self, event: &(dyn Any + Send + Sync)) {
        if let Some(concrete_event) = event.downcast_ref::<T>() {
            self.handler.handle(concrete_event).await;
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn subscribe<T: Event, H: EventHandler<T> + 'static>(&mut self, handler: H) {
        let type_id = TypeId::of::<T>();
        self.handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Box::new(HandlerWrapper {
                handler,
                _phantom: std::marker::PhantomData,
            }));
    }

    pub async fn publish<T: Event>(&self, event: T) {
        let type_id = TypeId::of::<T>();
        if let Some(handlers) = self.handlers.get(&type_id) {
            for handler in handlers {
                handler.handle_erased(&event).await;
            }
        }
    }
}
