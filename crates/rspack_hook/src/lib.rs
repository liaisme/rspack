use std::{borrow::Borrow, hash::Hash};

use async_trait::async_trait;
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

#[async_trait]
pub trait Interceptor<H: Hook> {
  async fn call(&self, _hook: &H) -> Result<Vec<<H as Hook>::Tap>> {
    unreachable!("Interceptor::call should only used in async hook")
  }

  fn call_blocking(&self, _hook: &H) -> Result<Vec<<H as Hook>::Tap>> {
    unreachable!("Interceptor::call_blocking should only used in sync hook")
  }
}

pub trait Hook {
  type Tap;

  fn used_stages(&self) -> FxHashSet<i32>;

  fn intercept(&mut self, interceptor: impl Interceptor<Self> + Send + Sync + 'static)
  where
    Self: Sized;
}

// pub trait Plugin<HookContainer> {
//   fn apply(&self, hook_container: &mut HookContainer);
// }

#[derive(Debug)]
pub struct HookMap<K, H> {
  map: FxHashMap<K, H>,
}

impl<K, H> Default for HookMap<K, H> {
  fn default() -> Self {
    Self {
      map: Default::default(),
    }
  }
}

impl<K, H> HookMap<K, H>
where
  K: Eq + Hash,
  H: Default,
{
  pub fn r#for(&mut self, key: impl Into<K>) -> &mut H {
    self.map.entry(key.into()).or_default()
  }
}

impl<K, H> HookMap<K, H> {
  pub fn get<Q>(&self, key: &Q) -> Option<&H>
  where
    K: Borrow<Q> + Eq + Hash,
    Q: Eq + Hash + ?Sized,
  {
    self.map.get(key)
  }

  pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut H>
  where
    K: Borrow<Q> + Eq + Hash,
    Q: Eq + Hash + ?Sized,
  {
    self.map.get_mut(key)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&K, &H)> {
    self.map.iter()
  }
}

#[doc(hidden)]
pub mod __macro_helper {
  pub use async_trait::async_trait;
  pub use rspack_error::Result;
  pub use rustc_hash::FxHashSet;
  pub use tracing;
}

pub use rspack_macros::{define_hook, plugin, plugin_hook};
