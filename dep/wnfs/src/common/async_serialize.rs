use std::rc::Rc;

use async_trait::async_trait;
use libipld::{error::SerdeError, serde as ipld_serde, Ipld};
use serde::{Serialize, Serializer};

use crate::BlockStore;

//--------------------------------------------------------------------------------------------------
// Macros
//--------------------------------------------------------------------------------------------------

macro_rules! impl_async_serialize {
    ( $( $ty:ty $( : < $( $generics:ident ),+ > )? ),+ ) => {
        $(
            #[async_trait(?Send)]
            impl $( < $( $generics ),+ > )? AsyncSerialize for $ty $( where $( $generics: Serialize ),+  )? {
                async fn async_serialize<S: Serializer, BS: BlockStore + ?Sized>(
                    &self,
                    serializer: S,
                    _: &mut BS,
                ) -> Result<S::Ok, S::Error> {
                    self.serialize(serializer)
                }
            }
        )+
    };
}

//--------------------------------------------------------------------------------------------------
// Type Definitions
//--------------------------------------------------------------------------------------------------

/// A **data structure** that can be serialized into any data format supported
/// by Serde.
///
/// This trait is slightly different from Serde's Serialize trait because it allows for asynchronous
/// serialisation and it is designed for the IPLD ecosystem where a `Store` is sometimes needed to
/// properly resolve the internal state of certain data structures to Cids.
///
/// An example of this is the PublicDirectory which can contain links to other IPLD nodes.
/// These links need to be resolved to Cids during serialization if they aren't already.
#[async_trait(?Send)]
pub trait AsyncSerialize {
    /// Serializes the type.
    async fn async_serialize<S, B>(&self, serializer: S, store: &mut B) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        B: BlockStore + ?Sized;

    /// Serialize with an IPLD serializer.
    async fn async_serialize_ipld<B>(&self, store: &mut B) -> Result<Ipld, SerdeError>
    where
        B: BlockStore + ?Sized,
    {
        self.async_serialize(ipld_serde::Serializer, store).await
    }
}

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

#[async_trait(?Send)]
impl<T: AsyncSerialize> AsyncSerialize for Rc<T> {
    async fn async_serialize<S, B>(&self, serializer: S, store: &mut B) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        B: BlockStore + ?Sized,
    {
        self.as_ref().async_serialize(serializer, store).await
    }
}

impl_async_serialize! { usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8 }
impl_async_serialize! { String, &str }
impl_async_serialize! {
    (A,): <A>,
    (A, B): <A, B>,
    (A, B, C): <A, B, C>,
    (A, B, C, D): <A, B, C, D>,
    (A, B, C, D, E): <A, B, C, D, E>,
    (A, B, C, D, E, F): <A, B, C, D, E, F>,
    (A, B, C, D, E, F, G): <A, B, C, D, E, F, G>,
    (A, B, C, D, E, F, G, H): <A, B, C, D, E, F, G, H>,
    (A, B, C, D, E, F, G, H, I): <A, B, C, D, E, F, G, H, I>,
    (A, B, C, D, E, F, G, H, I, J): <A, B, C, D, E, F, G, H, I, J>,
    (A, B, C, D, E, F, G, H, I, J, K): <A, B, C, D, E, F, G, H, I, J, K>
}
