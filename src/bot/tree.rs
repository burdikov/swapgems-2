use teloxide::dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt};
use teloxide::{dptree, RequestError};
use teloxide::dptree::Handler;
use teloxide::prelude::{DependencyMap, Update};

use super::handlers::*;
use super::commands::*;
use super::filters::*;


pub fn build_handler() -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(dptree::entry()
                    .filter_command::<SimpleCommand>()
                    .endpoint(handle_simple_command))
                .branch(dptree::filter(msg_from_maintainer)
                    .filter_command::<MaintainerCommand>()
                    .endpoint(handle_maintainer_command))
                .branch(dptree::filter(me_added_to_group)
                    .endpoint(handle_added_to_group))
                .branch(dptree::filter(has_shared_users).endpoint(handle_shared_users))

        )
        .branch(
            Update::filter_callback_query()
                .endpoint(handle_callback_query)
        )
}