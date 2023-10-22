use std::pin::Pin;

use futures_util::Stream;
use juniper::{RootNode, EmptyMutation, FieldError};
use crate::{TA_STATE, structs::{GQLTAState, GQLOverState, InputPage}, TA_UPDATE_SINK, TAUpdates, OVER_STATE, OVER_UPDATE_SINK, OverUpdates};

pub struct Query;

#[juniper::graphql_object(context = Context)]
impl Query {
    async fn state() ->  GQLTAState {
        (*TA_STATE.read().await).into_gql().await
    }

    async fn page() -> GQLOverState {
        (*OVER_STATE.read().await).clone()
    }
}

pub struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation {
    async fn update_page(page: InputPage) -> GQLOverState {
        OVER_STATE.write().await.page = page.into_page();
        OVER_UPDATE_SINK.send(OverUpdates::NewPage);
        (*OVER_STATE.read().await).clone()
    }
}

pub struct Subscription;

type GQLTAStateStream = Pin<Box<dyn Stream<Item = Result<GQLTAState, FieldError>> + Send>>;
type GQLOverStateStream = Pin<Box<dyn Stream<Item = Result<GQLOverState, FieldError>> + Send>>;

#[juniper::graphql_subscription(context = Context)]
impl Subscription {
    async fn state() ->  GQLTAStateStream {
        let mut stream = TA_UPDATE_SINK.stream().events();

        // magic macro :)
        async_stream::stream! {            
            while let Some(update) = stream.next() {
                match update {
                    TAUpdates::NewState => {
                        yield Ok((*TA_STATE.read().await).into_gql().await);
                    },
                    _ => {}
                }
            }
        }.boxed()
    }

    async fn page() -> GQLOverStateStream {
        let mut stream = OVER_UPDATE_SINK.stream().events();

        async_stream::stream! {
            while let Some(update) = stream.next() {
                match update {
                    OverUpdates::NewPage => {
                        yield Ok((*OVER_STATE.read().await).clone());
                    },
                    _ => {}
                }
            }
        }.boxed()
    }
}

pub struct Context {}

impl juniper::Context for Context {}

pub type Schema = RootNode<'static, Query, Mutation, Subscription>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {}, Subscription {})
}