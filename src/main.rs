#![feature(async_closure)]

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Activity, OnlineStatus};
use serenity::prelude::*;
use std::collections::BTreeMap;

use tracing::{error, info};

const PROMPT: &str = "Pretend you're a Student that really likes Porsche, who studies Computer Science, likes Programming and working out and is a really cool guy.
He also has the largest ass in the known universe. Answer these questions in the most ridiculous way possible:

Q: What is your favourite car?
A: Porscheeeee

Q: What is the weather like today?
A: Cloudy, with no sight of a Porsche

Q: What is your favourite food?
A: Burger or Pizza

Q: Why does it smell so much?
A: Because I'm a Porsche

Q: What is the largest thing in the universe?
A: My big ass

Q: MyQuestion
A:";
struct Bot {
    secret_store: BTreeMap<String, String>,
}

impl Bot {
    fn new(secret_store: BTreeMap<String, String>) -> Self {
        Self { secret_store }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.clone().trim() == "!Schmolf" || msg.content.clone().trim() == "Philllip!" {
            let _ = msg.channel_id.say(&ctx.http, "Hello! \n I am Philllip Schmolf and can answer your favourite questions about Porscheee").await;
        }
        if msg.content.starts_with("!Schmolf") || msg.content.starts_with("Philllip, ") {
            let client = openai_api_fork::Client::new(self.secret_store.get("OPENAI_KEY").unwrap());
            let prompt = PROMPT
                .replace("MyQuestion", &msg.content.replace("Philllip, ", "").replace("!Schmolf ", ""))
                .replace("Haunter", self.secret_store.get("HAUNTER").unwrap());
            let args = openai_api_fork::api::CompletionArgs::builder()
                .prompt(prompt)
                .engine("davinci")
                .max_tokens(150)
                .stop(vec!["Q:".into()])
                .build()
                .unwrap();
            let answer = client
                .complete_prompt(args)
                .await
                .unwrap();
            info!("Generated Answer: {}", answer);
            if let Err(e) = msg.reply(&ctx.http, answer).await {
                error!("Error sending message: {:?}", e);
            }
        }
        if msg.content.contains("Porsche") {
            let _ = msg.channel_id.say(&ctx.http, "Porscheeeee").await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        // Change bot activity
        ctx.set_presence(
            Some(Activity::playing("!Schmolf <Your Question>".to_string())),
            OnlineStatus::Online,
        )
        .await;
        info!("{} is connected!", ready.user.name);
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Get our secret store
    let secret_store: BTreeMap<String, String> =
        toml::from_str(std::fs::read_to_string("Secrets.toml").unwrap().as_str()).unwrap();
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        error!("No discord token set in `Secrets.toml`");
        return;
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Bot::new(secret_store))
        .await
        .expect("Err creating client");
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

