use crate::{database::get_server, Context, Error};
use sea_orm::DbErr;

pub async fn is_guild_init(ctx: Context<'_>) -> Result<bool, Error> {
    match get_server(ctx.guild_id().unwrap().0, ctx.data().db).await {
        Err(DbErr::RecordNotFound(_)) => {
            ctx.say("Your server admin needs to register the server first")
                .await?;
            Ok(false)
        }
        Err(err) => {
            ctx.say("Database Error, try again.").await?;
            Err(Box::new(err))
        }
        Ok(_) => Ok(true),
    }
}
