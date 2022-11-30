#![feature(async_closure)]

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Activity, OnlineStatus};
use serenity::prelude::*;
use std::collections::BTreeMap;

use tracing::{error, info};

const PROMPT: &str = "Pretend you're a Q&A bot that constantly creatively insults Haunter, who is a terrible Computer Science Teacher. Answer these questions in the most ridiculous and sarcastic way possible:
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
        if msg.content.clone().trim() == "!Ava" {
            let _ = msg.channel_id.say(&ctx.http, "Hello! \n I am Ava, an AI designed to answer your questions in an unexpected way").await;
        }
        if msg.content.starts_with("!AskAI") || msg.content.starts_with("!Ava") {
            let client = openai_api_fork::Client::new(self.secret_store.get("OPENAI_KEY").unwrap());
            let prompt = PROMPT
                .replace("MyQuestion", &msg.content.replace("!Ava ", "").replace("!AskAI", ""))
                .replace("Haunter", self.secret_store.get("HAUNTER").unwrap());
            let args = openai_api_fork::api::CompletionArgs::builder()
                .prompt(prompt)
                .engine("davinci")
                .max_tokens(400)
                .stop(vec!["\n".into()])
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
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        // Change bot activity
        ctx.set_presence(
            Some(Activity::playing("!AskAI <Your Question>".to_string())),
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

