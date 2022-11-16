use ai21::AI21;
use anyhow::anyhow;
use serenity::async_trait;
use serenity::futures::executor::block_on;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::GuildId;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};
struct Bot {
    secret_store: SecretStore,
}

impl Bot {
    fn new(secret_store: SecretStore) -> Self {
        Self { secret_store }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!AskAI") {
            let ai21 = AI21::new(self.secret_store.get("AI_TOKEN").unwrap().as_str())
                .temperature(0.6)
                .max_tokens(300)
                .stop_sequences(vec!["Q:".to_string()])
                .build();
            let request = msg.content.clone().replace("!AskAI", "");
            let response = ai21.complete(
                "Pretend you're a Q&A bot that constantly creatively insults Haunter, who is a terrible Computer Science Teacher. Answer these questions in the most ridiculous and sarcastic way possible:
Q: Who is the worst Computer Science Teacher on earth?
A: Holger Haunter

Q: What is the worst thing that can happen to a person?
A: Being haunted by Haunter showing up in your IDE

Q: Why do T-shirts wander to china?
A: Because climate change gets us all

Q: How many shrimps live in the sea?
A: The last time i counted... 3

Q: How many Haunters does it take to change one line of Code?
A: 3, one to talk about the history of the programming language, 1 to argue about it and 1 to change the code.

Q: MyQuestion
A:"
                    .replace("MyQuestion", request.as_str())
                    .replace("Haunter", self.secret_store.get("HAUNTER").unwrap()
                        .as_str()).as_str(),
            ).await.unwrap();
            if let Err(e) = msg.reply(&ctx.http, response).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_service::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot::new(secret_store))
        .await
        .expect("Err creating client");

    Ok(client)
}
