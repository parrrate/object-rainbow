use std::sync::Arc;

use object_rainbow::{
    Address, Error, Hash, InlineOutput, Parse, ParseInline, ParseInput, ParseSlice,
    ParseSliceExtra, ParseSliceRefless, PointInput, PointVisitor, Resolve, SingularFetch, Tagged,
    ToOutput, Traversible, length_prefixed::LpVec,
};
use object_rainbow_point::ExtractResolve;

use crate::ExternalStore;

#[derive(ToOutput, InlineOutput, Parse, ParseInline)]
struct Header<Id> {
    tags: Hash,
    topology: Arc<LpVec<Id>>,
}

#[derive(ToOutput)]
struct ExternallyStored<T, Id> {
    header: Header<Id>,
    object: T,
}

impl<
    S: ExternalStore,
    E: 'static + Send + Sync + Clone,
    I: PointInput<Extra = (S, E)>,
    T: Parse<I::WithExtra<E>> + Tagged,
> Parse<I> for ExternallyStored<T, S::Id>
{
    fn parse(mut input: I) -> object_rainbow::Result<Self> {
        let header: Header<S::Id> = input.parse_refless_inline()?;
        if header.tags != T::HASH {
            return Err(object_rainbow::error_consistency!("tags mismatch"));
        }
        let resolve = ExternalResolve {
            store: input.extra().0.clone(),
            topology: header.topology.clone(),
        };
        let extra = input.extra().1.clone();
        let object = input
            .with_extra(extra)
            .with_resolve(Arc::new(resolve))
            .parse()?;
        Ok(Self { header, object })
    }
}

struct ExternalResolve<S, Id = <S as ExternalStore>::Id> {
    store: S,
    topology: Arc<LpVec<Id>>,
}

#[derive(Parse, ParseInline)]
struct Raw<Id> {
    header: Header<Id>,
    data: Vec<u8>,
}

impl<S: ExternalStore> ExternalResolve<S> {
    async fn resolve_bytes(&self, address: Address) -> object_rainbow::Result<Vec<u8>> {
        let Raw { data, .. } = Raw::<S::Id>::parse_slice_refless(
            self.store.fetch(&self.translate(address)?).await?.as_ref(),
        )?;
        Ok(data)
    }

    async fn resolve_full(
        &self,
        address: Address,
    ) -> object_rainbow::Result<(Vec<u8>, Arc<dyn Resolve>)> {
        let Raw {
            header: Header { topology, .. },
            data,
        } = Raw::<S::Id>::parse_slice_refless(
            self.store.fetch(&self.translate(address)?).await?.as_ref(),
        )?;
        Ok((
            data,
            Arc::new(Self {
                store: self.store.clone(),
                topology,
            }),
        ))
    }
}

impl<S: ExternalStore> Resolve for ExternalResolve<S> {
    fn resolve<'a>(
        &'a self,
        address: Address,
        _: &'a Arc<dyn Resolve>,
    ) -> object_rainbow::FailFuture<'a, object_rainbow::ByteNode> {
        Box::pin(self.resolve_full(address))
    }

    fn resolve_data(&'_ self, address: Address) -> object_rainbow::FailFuture<'_, Vec<u8>> {
        Box::pin(self.resolve_bytes(address))
    }
}

impl<S: ExternalStore> ExternalResolve<S> {
    fn translate(&self, address: Address) -> object_rainbow::Result<S::Id> {
        self.topology
            .get(address.index)
            .cloned()
            .ok_or(Error::AddressOutOfBounds)
    }
}

type Extracted<'s, Id> =
    Vec<std::pin::Pin<Box<dyn 's + Send + Future<Output = object_rainbow::Result<Id>>>>>;

struct ExtractResolution<'a, 's, S, Id = <S as ExternalStore>::Id> {
    extracted: &'a mut Extracted<'s, Id>,
    store: &'s S,
}

impl<S: ExternalStore> PointVisitor for ExtractResolution<'_, '_, S> {
    fn visit(&mut self, point: &(impl 'static + SingularFetch<T: Traversible> + Clone)) {
        let fetch = point.clone();
        self.extracted
            .push(Box::pin(store_point(self.store, fetch)));
    }
}

pub(crate) async fn store_point<S: ExternalStore, T: Traversible>(
    store: &S,
    fetch: impl 'static + SingularFetch<T = T>,
) -> object_rainbow::Result<S::Id> {
    if let Some((address, resolve)) = fetch.extract_resolve::<ExternalResolve<S>>()
        && resolve.store == *store
    {
        let id = resolve.translate(*address)?;
        return Ok(id);
    }
    store_object(store, fetch.fetch().await?).await
}

pub(crate) async fn store_object<S: ExternalStore, T: Traversible>(
    store: &S,
    object: T,
) -> object_rainbow::Result<S::Id> {
    let mut futures = Vec::with_capacity(object.point_count());
    object.traverse(&mut ExtractResolution {
        extracted: &mut futures,
        store,
    });
    let topology = futures_util::future::try_join_all(futures).await?;
    let topology = Arc::new(LpVec(topology));
    let header = Header {
        tags: T::HASH,
        topology,
    };
    let stored = ExternallyStored { header, object };
    let data = &*stored.vec();
    store
        .save_data(data, &stored.header.topology, stored.object.with_hash())
        .await
}

pub(crate) async fn load_extra<
    S: ExternalStore,
    T: ParseSliceExtra<E> + Tagged,
    E: 'static + Send + Sync + Clone,
>(
    store: &S,
    id: &S::Id,
    extra: E,
) -> object_rainbow::Result<T> {
    let ExternallyStored { object, .. } = ExternallyStored::parse_slice_extra(
        store.fetch(id).await?.as_ref(),
        &(Arc::new(Vec::new()) as _),
        &(store.clone(), extra),
    )?;
    Ok(object)
}

pub(crate) async fn load<S: ExternalStore, T: ParseSlice + Tagged>(
    store: &S,
    id: &S::Id,
) -> object_rainbow::Result<T> {
    load_extra(store, id, ()).await
}
