use std::collections::BTreeSet;

use futures_util::TryStreamExt;
use macro_rules_attribute::apply;
use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological,
};
use object_rainbow_history::{
    Compat, MappedDiff, Parallel, Sequential,
    remap::{MapToSet, MappedToSet},
};
use object_rainbow_trie::{TrieMap, TrieSet};
use smol_macros::main;
use ulid::Ulid;

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Size,
    MaybeHasNiche,
)]
struct ChannelId(Ulid);

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Size,
    MaybeHasNiche,
)]
struct MessageId(Ulid);

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Size,
    MaybeHasNiche,
)]
struct UserId(Ulid);

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Size,
    MaybeHasNiche,
)]
struct Message {
    channel: ChannelId,
    user: UserId,
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
struct MessageByChannel {
    channel: ChannelId,
    message: MessageId,
    user: UserId,
}

#[derive(
    Debug,
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
struct MessageByUser {
    user: UserId,
    channel: ChannelId,
    message: MessageId,
}

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
)]
struct MessageToChannel;

#[derive(
    ToOutput,
    InlineOutput,
    Tagged,
    ListHashes,
    Topological,
    Parse,
    ParseInline,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
)]
struct MessageToUser;

impl MapToSet<MessageId, Message> for MessageToChannel {
    type T = MessageByChannel;

    fn map(
        &self,
        message: MessageId,
        Message { channel, user }: Message,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        async move {
            Ok(MessageByChannel {
                channel,
                message,
                user,
            })
        }
    }
}

impl MapToSet<MessageId, Message> for MessageToUser {
    type T = MessageByUser;

    fn map(
        &self,
        message: MessageId,
        Message { channel, user }: Message,
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::T>> {
        async move {
            Ok(MessageByUser {
                user,
                channel,
                message,
            })
        }
    }
}

type MessagesByChannels =
    MappedDiff<Compat<TrieSet<MessageByChannel>>, MappedToSet<MessageToChannel>>;
type MessagesByUsers = MappedDiff<Compat<TrieSet<MessageByUser>>, MappedToSet<MessageToUser>>;
type Tree = Sequential<TrieMap<MessageId, Message>, Parallel<MessagesByChannels, MessagesByUsers>>;
type Diff = (MessageId, Option<Message>);
type History = object_rainbow_history::History<Tree, Diff>;

trait Table {
    fn messages_by_channels(
        &self,
    ) -> impl Send + Future<Output = object_rainbow::Result<TrieSet<MessageByChannel>>>;
}

impl Table for History {
    fn messages_by_channels(
        &self,
    ) -> impl Send + Future<Output = object_rainbow::Result<TrieSet<MessageByChannel>>> {
        async move { Ok(self.tree().await?.second().a().tree().0.clone()) }
    }
}

#[apply(main!)]
async fn main() -> object_rainbow::Result<()> {
    let mut history = History::new();
    let channel = ChannelId(Ulid::new());
    let user = UserId(Ulid::new());
    let message = MessageId(Ulid::new());
    history
        .commit((message, Some(Message { channel, user })))
        .await?;
    assert!(
        history
            .messages_by_channels()
            .await?
            .contains(&MessageByChannel {
                channel,
                message,
                user,
            })
            .await?,
    );
    assert!(
        history
            .tree()
            .await?
            .second()
            .b()
            .tree()
            .0
            .contains(&MessageByUser {
                user,
                channel,
                message,
            })
            .await?,
    );
    let messages_by_channel = history
        .messages_by_channels()
        .await?
        .range_stream(
            &MessageByChannel {
                channel,
                message: MessageId(Ulid::from_parts(u64::MIN, u128::MIN)),
                user: UserId(Ulid::from_parts(u64::MIN, u128::MIN)),
            }..&MessageByChannel {
                channel,
                message: MessageId(Ulid::from_parts(u64::MAX, u128::MAX)),
                user: UserId(Ulid::from_parts(u64::MAX, u128::MAX)),
            },
        )
        .try_collect::<BTreeSet<_>>()
        .await?;
    assert_eq!(
        messages_by_channel,
        BTreeSet::from([MessageByChannel {
            channel,
            message,
            user,
        }]),
    );
    history.commit((message, None)).await?;
    assert!(
        !history
            .messages_by_channels()
            .await?
            .contains(&MessageByChannel {
                channel,
                message,
                user,
            })
            .await?,
    );
    assert!(
        !history
            .tree()
            .await?
            .second()
            .b()
            .tree()
            .0
            .contains(&MessageByUser {
                user,
                channel,
                message,
            })
            .await?,
    );
    Ok(())
}
