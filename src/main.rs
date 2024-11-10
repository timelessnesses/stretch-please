use std::error::Error as std_error;
use rand::Rng;

use dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std_error>> {
    dotenv::dotenv().ok();
    let token = std::env::var("STRETCH_PLEASE_TOKEN").expect("No token present");
    let intents = poise::serenity_prelude::GatewayIntents::all();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![stretch()],
            ..Default::default()
        })
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        }).build();
    let mut client = poise::serenity_prelude::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    println!("Started");
    client.start_autosharded().await?;
    Ok(())
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (), Error>;

#[poise::command(slash_command, prefix_command)]
async fn stretch(
    ctx: Context<'_>,
    #[description = "Picture"]
    picture: poise::serenity_prelude::Attachment,
    #[description = "Stretch type"]
    stretch_type: Option<StretchType>
) -> Result<(), Error> {
    let opened = image::ImageReader::new(std::io::Cursor::new(picture.download().await?)).with_guessed_format()?.decode()?;
    let stretch_type = stretch_type.unwrap_or(StretchType::get_random());
    let (width, height) = stretch_type.to_stretch_function()(opened.width() as usize, opened.height() as usize);
    println!("Stretching {} ({}x{}) to {}x{}", picture.url, opened.width(), opened.height(), width, height);
    let done = opened.resize_exact(width as u32, height as u32, image::imageops::FilterType::Nearest);
    let mut buffer = std::io::Cursor::new(Vec::new());
    done.write_to(&mut buffer, image::ImageFormat::Png)?;
    ctx.send(poise::CreateReply::default().attachment(poise::serenity_prelude::CreateAttachment::bytes(buffer.into_inner(), "stretch.png"))).await?;
    return Ok(())
}

#[derive(poise::ChoiceParameter)]
enum StretchType {
    Horizontal,
    Vertical,
    Both,
}

impl StretchType {
    fn to_stretch_function(&self) -> fn(usize, usize) -> (usize, usize) {
        STRETCH_FUNCTIONS[self.to_index()]
    }

    fn to_index(&self) -> usize {
        match self {
            StretchType::Horizontal => 0,
            StretchType::Vertical => 1,
            StretchType::Both => 2,
        }
    }

    fn get_random() -> Self {
        let index = rand::thread_rng().gen_range(0..3);
        match index {
            0 => StretchType::Horizontal,
            1 => StretchType::Vertical,
            2 => StretchType::Both,
            _ => panic!("Invalid index")
        }
    }
}

static STRETCH_FUNCTIONS: [fn(usize, usize) -> (usize, usize);3] = [stretch_horizontal, stretch_vertical, stretch_both];

fn stretch_horizontal(w: usize, h: usize) -> (usize, usize) {
    let modifier = rand::thread_rng().gen_range(0.5..2.0);
    return ((w as f32 * modifier).round() as usize, h);
}

fn stretch_vertical(w: usize, h: usize) -> (usize, usize) {
    let modifier = rand::thread_rng().gen_range(0.5..2.0);
    return (w, (h as f32 * modifier).round() as usize);
}

fn stretch_both(w: usize, h: usize) -> (usize, usize) {
    return ((w as f32 * rand::thread_rng().gen_range(0.5..2.0)).round() as usize, (h as f32 * rand::thread_rng().gen_range(0.2..2.0)).round() as usize);
}