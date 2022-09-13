use std::env;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

use select::document::Document;
use select::predicate::Attr;

const WEEK_DAYS: [&str; 5] = [
    "Segunda-Feira",
    "Terça-Feira",
    "Quarta-Feira",
    "Quinta-Feira",
    "Sexta-Feira",
];

async fn ementa(day: usize, all: bool) -> Vec<String> {
    println!("dia {}", day);
    println!("all {}", all);
    let url = "https://eatdreamsmile.pt/";
    let mut info = Vec::<String>::new();
    let mut current_day = url.to_string();
    current_day.push_str(WEEK_DAYS[day].to_lowercase().replace("ç", "c").as_str());

    let response = reqwest::get(current_day)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let document = Document::from(response.as_str());

    let mut i = 0;
    for node in document.find(Attr("class", "wpb_wrapper")) {
        if i >= 3 {
            for child in node.children() {
                if child.name() == Some("h4") && child.text().len() > 4 {
                    info.push(child.text().replace("\n", "").to_string());
                }
            }
        }
        i += 1;
    }
    info[0] = info[0].replace("Sopa: ", "");
    info[1] = info[1].replace("Prato Mediterrânico: ", "");
    info[2] = info[2].replace("Prato Vegetariano: ", "");
    info[3] = info[3].replace("Sopa: ", "");
    info[4] = info[4].replace("Prato Mediterrânico: ", "");
    info[5] = info[5].replace("Prato Vegetariano: ", "");
    println!("{:?}", info);

    return info;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if command.data.name.as_str() == "ementa" {
                let options = command
                    .data
                    .options
                    .get(0)
                    .unwrap()
                    .resolved
                    .as_ref()
                    .unwrap();

                let message = command
                    .data
                    .options
                    .get(1)
                    .unwrap()
                    .resolved
                    .as_ref()
                    .unwrap();

                let mut dia = 0;
                if let ApplicationCommandInteractionDataOptionValue::String(option) = options {
                    dia = option.parse::<usize>().unwrap()
                }
                if let ApplicationCommandInteractionDataOptionValue::String(almoco) = message {
                    let pratos = ementa(dia, almoco != "false").await;
                    let almoco_str: &str;
                    if almoco == "false" {
                        almoco_str = "Jantar";
                    } else {
                        almoco_str = "Almoço";
                    }

                    if let Err(why) = command
                        .create_interaction_response(&ctx.http, |response| {
                            if pratos.len() == 1 {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(format!(
                                            "Nenhuma ementa disponível para o dia {}",
                                            dia
                                        ))
                                    });
                            } else {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.create_embed(|e| {
                                            e.title("Ementa do Social")
                                                .description(format!(
                                                    "{} - {}",
                                                    WEEK_DAYS[dia], almoco_str
                                                ))
                                                .colour(0x00a0e4);
                                            let i;
                                            if almoco == "false" {
                                                i = 3;
                                            } else {
                                                i = 0;
                                            }
                                            e.field("Sopa 🍜", &pratos[i], true);
                                            e.field("Prato Mediterrânico 🥩", &pratos[i + 1], true);
                                            e.field("Prato Vegetariano 🥦", &pratos[i + 2], true);
                                            e
                                        })
                                    });
                            }
                            response
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    };
                };
            }
        }
    }
    async fn ready(&self, ctx: Context, _: Ready) {
        let guild_command = GuildId(env::var("SERVER_ID").unwrap().parse().unwrap())
            .set_application_commands(&ctx.http, |commands| {
                commands.create_application_command(|command| {
                    command
                        .name("ementa")
                        .description("Manda a ementa do Social")
                        .create_option(|option| {
                            option
                                .name("dia")
                                .description("Escolhe o dia da refeição")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                                .add_string_choice("segunda-feira", "0")
                                .add_string_choice("terça-feira", "1")
                                .add_string_choice("quarta-feira", "2")
                                .add_string_choice("quinta-feira", "3")
                                .add_string_choice("sexta-feira", "4")
                        })
                        .create_option(|option| {
                            option
                                .name("refeição")
                                .description("Almoço ou jantar")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                                .add_string_choice("almoço", "true")
                                .add_string_choice("jantar", "false")
                        })
                })
            })
            .await;
        println!(
            "I created the following guild command: {:#?}",
            guild_command
        );
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("BOT_TOKEN").unwrap();
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(env::var("BOT_ID").unwrap().parse().unwrap())
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
