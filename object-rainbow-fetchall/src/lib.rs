use std::{
    collections::{BTreeMap, BTreeSet, btree_map},
    pin::Pin,
};

use async_executor::Executor;
use flume::Sender;
use futures_channel::oneshot;
use object_rainbow::{Fetch, Hash, Point, PointVisitor, Singular, Traversible};
use object_rainbow_local_map::LocalMap;

type Dependency = Box<
    dyn 'static
        + Send
        + FnOnce(Context<'_>) -> Pin<Box<dyn '_ + Send + Future<Output = object_rainbow::Result<()>>>>,
>;

enum Request {
    Depencencies {
        dependencies: BTreeMap<Hash, Dependency>,
        callback: oneshot::Sender<object_rainbow::Result<()>>,
    },
    End {
        hash: Hash,
        tags_hash: Hash,
        topology: Vec<Hash>,
        data: Vec<u8>,
        callback: oneshot::Sender<object_rainbow::Result<()>>,
    },
}

struct Context<'r> {
    request: &'r Sender<Request>,
}

struct DependencyVisitor<'v> {
    dependencies: &'v mut BTreeMap<Hash, Dependency>,
    topology: &'v mut Vec<Hash>,
}

impl<'v> PointVisitor for DependencyVisitor<'v> {
    fn visit<T: Traversible>(&mut self, point: &Point<T>) {
        if let btree_map::Entry::Vacant(e) = self.dependencies.entry(point.hash()) {
            let point = point.clone();
            e.insert(Box::new(move |context| {
                Box::pin(async move { context.save_point(&point).await })
            }));
        }
        self.topology.push(point.hash());
    }
}

impl<'r> Context<'r> {
    async fn save_point(&self, point: &Point<impl Traversible>) -> object_rainbow::Result<()> {
        let mut dependencies = BTreeMap::new();
        let mut topology = Vec::new();
        let object = point.fetch().await?;
        object.accept_points(&mut DependencyVisitor {
            dependencies: &mut dependencies,
            topology: &mut topology,
        });
        {
            let (callback, wait) = oneshot::channel();
            self.request
                .send_async(Request::Depencencies {
                    dependencies,
                    callback,
                })
                .await
                .ok();
            let Ok(r) = wait.await else {
                return Err(object_rainbow::error_fetch!("dependency cancelled"));
            };
            r?;
        }
        {
            let (callback, wait) = oneshot::channel();
            self.request
                .send_async(Request::End {
                    hash: point.hash(),
                    tags_hash: object.hashes().tags,
                    topology,
                    data: object.output(),
                    callback,
                })
                .await
                .ok();
            let Ok(r) = wait.await else {
                return Err(object_rainbow::error_fetch!("save cancelled"));
            };
            r?;
        }
        Ok(())
    }
}

pub async fn fetchall(point: &Point<impl Traversible>) -> object_rainbow::Result<LocalMap> {
    let mut map = LocalMap::new();
    {
        let mut started = BTreeSet::new();
        let (send, recv) = flume::bounded(0);
        let outer = Executor::new();
        let inner = Executor::new();
        let task = inner.spawn(async {
            while let Ok(request) = recv.recv_async().await {
                match request {
                    Request::Depencencies {
                        dependencies,
                        callback,
                    } => {
                        let mut tasks = Vec::new();
                        for (hash, save) in dependencies {
                            if started.insert(hash) {
                                tasks.push(outer.spawn(save(Context { request: &send })));
                            }
                        }
                        outer
                            .spawn(async move {
                                for task in tasks {
                                    if let Err(e) = task.await {
                                        callback.send(Err(e)).ok();
                                        return;
                                    }
                                }
                                callback.send(Ok(())).ok();
                            })
                            .detach();
                    }
                    Request::End {
                        hash,
                        tags_hash,
                        topology,
                        data,
                        callback,
                    } => {
                        assert!(!map.contains(hash));
                        callback
                            .send(map.insert(hash, tags_hash, topology, data))
                            .ok();
                    }
                }
            }
        });
        let _task = inner.spawn(outer.run(task));
        inner
            .run(Context { request: &send }.save_point(point))
            .await?;
    }
    assert!(map.contains(point.hash()));
    Ok(map)
}
