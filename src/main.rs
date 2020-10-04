use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use failure::{bail, format_err, Error};
use mpris::{PlaybackStatus, Player, PlayerFinder};
use notify_rust::{Notification, Urgency};

fn get_last_player_file_path() -> Result<PathBuf, Error> {
    let p = env::var("XDG_RUNTIME_DIR")?;
    Ok(Path::new(&p).join("media_control_last_player_file"))
}

fn load_data_from_file() -> Result<(String, u32), Error> {
    let path = match get_last_player_file_path() {
        Err(_) => {
            println!("Unknown path to load from!");
            return Ok((String::new(), 0));
        }
        Ok(p) => p,
    };
    let mut s = String::new();
    match File::open(&path) {
        Err(_) => println!("Could not open file at {:?}!", path),
        Ok(mut file) => match file.read_to_string(&mut s) {
            Err(_) => println!("Could not read file at {:?}!", path),
            Ok(_) => {}
        },
    };

    let mut s = s.split(';');
    let last_player = s
        .next()
        .ok_or_else(|| format_err!("No data could be loaded."))?;
    let notification_id = s
        .next()
        .ok_or_else(|| format_err!("No data could be loaded."))?
        .parse::<u32>()?;
    Ok((String::from(last_player), notification_id))
}

fn write_data_to_file(last_player: &str, id: u32) -> Result<(), Error> {
    let p = get_last_player_file_path()?;
    File::create(p)?.write_all(format!("{};{}", last_player, id).as_bytes())?;
    Ok(())
}

fn switch_player(players: &Vec<Player>, current_index: usize, step: i32) -> usize {
    (((current_index as i32) + step) % (players.len() as i32)) as usize
}

fn notify(
    players: &Vec<Player>,
    current_index: usize,
    command: &CommandType,
    notification_id: u32,
) -> Result<u32, Error> {
    let selected_player: &Player = &players[current_index];
    let playback_status = selected_player.get_playback_status()?;
    let summary = format!(
        "{} ({})",
        match command {
            CommandType::Play => "Play",
            CommandType::Pause => "Pause",
            CommandType::PlayPause => {
                match playback_status {
                    PlaybackStatus::Playing => "Pause",
                    _ => "Play",
                }
            }
            CommandType::Next => "Next track",
            CommandType::Previous => "Previous track",
            CommandType::NextPlayer | CommandType::PreviousPlayer => "Selected player",
        },
        selected_player.identity()
    );
    let body: Result<String, Error> = match command {
        CommandType::NextPlayer | CommandType::PreviousPlayer => Ok(players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = p.identity();
                if i == current_index {
                    format!("→ {}", name)
                } else {
                    format!("<span color=\"grey\">{}</span>", name)
                }
            })
            .collect::<Vec<String>>()
            .join("\n")),
        _ => {
            let metadata = selected_player.get_metadata()?;
            let artists = match metadata.artists() {
                Some(artists) => artists.join(", "),
                None => String::new(),
            };
            let title = match metadata.title() {
                Some(title) => title,
                None => "",
            };
            Ok(format!("{} - {}", artists, title))
        }
    };
    let body = body?;
    let icon = format!(
        "/usr/share/icons/gnome/48x48/actions/{}.png",
        match command {
            CommandType::Play => "gtk-media-play-ltr",
            CommandType::Pause => "gtk-media-pause",
            CommandType::PlayPause => {
                match playback_status {
                    PlaybackStatus::Playing => "gtk-media-pause",
                    _ => "gtk-media-play-ltr",
                }
            }
            CommandType::Next => "gtk-media-next-ltr",
            CommandType::Previous => "gtk-media-previous-ltr",
            CommandType::NextPlayer => "forward",
            CommandType::PreviousPlayer => "back",
        }
    );
    let urgency = match command {
        CommandType::PreviousPlayer | CommandType::NextPlayer => Urgency::Normal,
        _ => Urgency::Low,
    };
    let timeout = match command {
        CommandType::PreviousPlayer | CommandType::NextPlayer => 6000,
        _ => 1500,
    };
    let mut notification = Notification::new()
        .summary(&summary)
        .body(&body)
        .icon(&icon)
        .urgency(urgency)
        .timeout(timeout)
        .finalize();
    let notification = if notification_id == 0u32 {
        notification
    } else {
        notification.id(notification_id).finalize()
    };
    match notification.show() {
        Err(e) => bail!("{:?}", e),
        Ok(handle) => Ok(handle.id()),
    }
}

#[derive(PartialEq)]
enum CommandType {
    Play,
    Pause,
    PlayPause,
    Next,
    NextPlayer,
    Previous,
    PreviousPlayer,
}

fn parse_args() -> Result<CommandType, Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Not enough arguments!");
    }

    let ret = match &args[1].as_str() {
        &"play" => CommandType::Play,
        &"pause" => CommandType::Pause,
        &"playpause" => CommandType::PlayPause,
        &"play-pause" => CommandType::PlayPause,
        &"next" => CommandType::Next,
        &"previous" => CommandType::Previous,
        &"next_player" => CommandType::NextPlayer,
        &"previous_player" => CommandType::PreviousPlayer,
        command => {
            bail!("Unknown command \"{}\"!", command);
        }
    };
    Ok(ret)
}

fn main() -> Result<(), Error> {
    let (last_player_id, notification_id) = load_data_from_file().unwrap_or((String::new(), 0));

    let finder = PlayerFinder::new()?;
    let players: Vec<Player> = finder.find_all()?;
    let mut player_index = 0;

    for (index, player) in players.iter().enumerate() {
        if player.unique_name() == last_player_id {
            player_index = index;
            break;
        }
    }

    let selected_player: &Player = &players[player_index];

    let command = parse_args()?;
    match command {
        CommandType::Play => selected_player.play(),
        CommandType::Pause => selected_player.pause(),
        CommandType::PlayPause => selected_player.play_pause(),
        CommandType::Next => selected_player.next(),
        CommandType::Previous => selected_player.previous(),
        CommandType::NextPlayer => {
            player_index = switch_player(&players, player_index, 1);
            Ok(())
        }
        CommandType::PreviousPlayer => {
            player_index = switch_player(&players, player_index, -1);
            Ok(())
        }
    }?;
    let notification_id = notify(&players, player_index, &command, notification_id)?;

    write_data_to_file(players[player_index].unique_name(), notification_id)?;

    Ok(())
}
