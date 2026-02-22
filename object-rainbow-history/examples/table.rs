use macro_rules_attribute::apply;
use object_rainbow::{
    InlineOutput, ListHashes, MaybeHasNiche, Parse, ParseInline, Size, Tagged, ToOutput,
    Topological,
};
use object_rainbow_history::{Compat, History, MapDiff, MappedDiff, Parallel, Sequential};
use object_rainbow_trie::{TrieMap, TrieSet};
use smol_macros::main;
use ulid::Ulid;

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
struct ChannelId(Ulid);

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
struct MessageId(Ulid);

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
)]
struct MessageByChannel {
    channel: ChannelId,
    message: MessageId,
    user: UserId,
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

impl MapDiff<(Option<Message>, (MessageId, Option<Message>))> for MessageToChannel {
    type Inner = Vec<(MessageByChannel, bool)>;

    fn map(
        &self,
        (old, (message, new)): (Option<Message>, (MessageId, Option<Message>)),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Inner>> {
        async move {
            let mut diff = Vec::new();
            if let Some(Message { channel, user }) = old {
                diff.push((
                    MessageByChannel {
                        channel,
                        message,
                        user,
                    },
                    true,
                ));
            }
            if let Some(Message { channel, user }) = new {
                diff.push((
                    MessageByChannel {
                        channel,
                        message,
                        user,
                    },
                    false,
                ));
            }
            Ok(diff)
        }
    }
}

impl MapDiff<(Option<Message>, (MessageId, Option<Message>))> for MessageToUser {
    type Inner = Vec<(MessageByUser, bool)>;

    fn map(
        &self,
        (old, (message, new)): (Option<Message>, (MessageId, Option<Message>)),
    ) -> impl Send + Future<Output = object_rainbow::Result<Self::Inner>> {
        async move {
            let mut diff = Vec::new();
            if let Some(Message { channel, user }) = old {
                diff.push((
                    MessageByUser {
                        user,
                        channel,
                        message,
                    },
                    true,
                ));
            }
            if let Some(Message { channel, user }) = new {
                diff.push((
                    MessageByUser {
                        user,
                        channel,
                        message,
                    },
                    false,
                ));
            }
            Ok(diff)
        }
    }
}

type MessagesByChannels = MappedDiff<Compat<TrieSet<MessageByChannel>>, MessageToChannel>;
type MessagesByUsers = MappedDiff<Compat<TrieSet<MessageByUser>>, MessageToUser>;
type Tree = Sequential<TrieMap<MessageId, Message>, Parallel<MessagesByChannels, MessagesByUsers>>;
type Diff = (MessageId, Option<Message>);

#[apply(main!)]
async fn main() -> object_rainbow::Result<()> {
    let mut history = History::<Tree, Diff>::new();
    let channel = ChannelId(Ulid::new());
    let user = UserId(Ulid::new());
    let message = MessageId(Ulid::new());
    history
        .commit((message, Some(Message { channel, user })))
        .await?;
    assert!(
        history
            .tree()
            .await?
            .second()
            .a()
            .tree()
            .0
            .contains(&MessageByChannel {
                channel,
                message,
                user
            })
            .await?
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
                message
            })
            .await?
    );
    history.commit((message, None)).await?;
    assert!(
        !history
            .tree()
            .await?
            .second()
            .a()
            .tree()
            .0
            .contains(&MessageByChannel {
                channel,
                message,
                user
            })
            .await?
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
                message
            })
            .await?
    );
    Ok(())
}
