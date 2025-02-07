use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rusty_audio::Audio;
use std::{
    error::Error, io, sync::mpsc::{self, Receiver}, thread::{self}, time::{Duration, Instant}
};
use std::io::Write;
use std::fs::OpenOptions;
use std::fs;
use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    level::Level,
    menu::Menu,
    player::Player,
    render,
    score::Score,
    rusty_bot::Agent,
};

const RUSTY_BOT_MAX_GAMES_TO_LEARN: i32 = 500;
const MAX_STEPS_ALLOWED: u16 = 45_000; // 45_000 // 60(fps) -> 750

fn render_screen(render_rx: Receiver<Frame>) {
    let mut last_frame = frame::new_frame();
    let mut stdout = io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    while let Ok(curr_frame) = render_rx.recv() {
        render::render(&mut stdout, &last_frame, &curr_frame, false);
        last_frame = curr_frame;
    }
}

fn reset_game(in_menu: &mut bool, player: &mut Player, invaders: &mut Invaders, rusty_bot: bool, game_number: i32) {
    if rusty_bot && game_number < RUSTY_BOT_MAX_GAMES_TO_LEARN {
        *in_menu = false;
    } 
    else {
        *in_menu = true;
    }
    *player = Player::new();
    *invaders = Invaders::new();
}

fn write_agent_brain(q_table_brain: &Vec<Vec<f32>>) {
    // FIXME: try catch thing ? not working ToT
    fs::remove_file("q_table.csv").expect("could not remove file");
    let mut q_table = OpenOptions::new().create_new(true)
    .append(true)
    .open("q_table.csv")
    .expect("cannot open file");
    q_table.write("left,right,shoot\n".as_bytes()).expect("write failed");
    for row in q_table_brain.iter() {
        for val in row.iter() {
            q_table.write(format!("{},", val.to_string()).as_bytes()).expect("failed to write");
        }
        q_table.write("\n".as_bytes()).expect("failed to write");
        // row.iter().map(|value| {format!("{}", value)}).into_iter().;
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    for item in &["explode", "lose", "move", "pew", "startup", "win"] {
        audio.add(item, &format!("audio/original/{}.wav", item));
    }
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        render_screen(render_rx);
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    let mut score = Score::new();
    let mut menu = Menu::new();
    let mut in_menu = true;
    let mut level = Level::new();
    let mut rusty_bot: bool = false;
    let mut agent: Agent = Agent::new(0.09, 0.98);
    let mut game_number = 1;
    let mut current_state;
    let mut action;
    let mut steps: u16 = 0;
    fs::remove_file("data.csv").expect("could not remove file");
    let mut score_data_file = OpenOptions::new().create_new(true)
    .append(true)
    .open("data.csv")
    .expect("cannot open file");
    score_data_file.write("score,best_score\n0,0".as_bytes()).expect("write failed");
    write_agent_brain(&agent.q_table);

    'gameloop: loop {
        // Per-frame inittxt
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();
        let mut reward = 0.0;

        if in_menu {
            // Input handlers for the menu
            while event::poll(Duration::default())? {
                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Up => menu.change_option(true),
                        KeyCode::Down => menu.change_option(false),
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if menu.selection == 0 {
                                in_menu = false;
                            } else if menu.selection == 1 {
                                in_menu = false;
                                rusty_bot = true;
                            }
                            else {
                                break 'gameloop;
                            }
                        }
                        _ => {}
                    }
                }
            }
            menu.draw(&mut curr_frame);

            let _ = render_tx.send(curr_frame);
            thread::sleep(Duration::from_millis(1));
            continue;
        }

        // Input handlers for the game
        current_state = agent.get_state(&mut invaders, &mut player);
        action = agent.act(current_state, game_number);
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        reset_game(&mut in_menu, &mut player, &mut invaders, false, 0);
                    }
                    _ => {}
                }
            }
        }

        if rusty_bot {
            // If allowed bot to play let bot decide what to play.
            match action {
                0 => player.move_left(),
                1 => player.move_right(),
                2 => {player.shoot();},
                _ => {}
            }
            steps += 1;
        }
        // Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        let hits: u16 = player.detect_hits(&mut invaders);
        if hits > 0 {
            audio.play("explode");
            score.add_points(hits);
            // % of hits of remaining army
            // reward += (hits as f32) / ((1.0 + invaders.army.len() as f32) - hits as f32);
            reward += 2.0;
        }
        // Draw & render

        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders, &score, &level];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        if rusty_bot {
            score.write_best_score(&mut curr_frame);
            let formatted = format!("GEN: {}", game_number);

            // iterate over all characters
            for (i, c) in formatted.chars().enumerate() {
                // put them in the first row
                curr_frame[i+20][1] = c;
            }
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win or lose?
        if invaders.all_killed() {
            if level.increment_level() {
                audio.play("win");
                score.update_best_points();
                // reward += 1.0;
                let new_state = agent.get_state(&mut invaders, &mut player);
                agent.learn(current_state, action, reward, new_state);
                score_data_file
                    .write(format!("\n{},{}", score.get_count(), score.get_best_score()).as_bytes())
                    .expect("write failed");
                write_agent_brain(&agent.q_table);
                break 'gameloop;
            }
            invaders = Invaders::new();
        } else if (invaders.reached_bottom()) || (rusty_bot == true && steps > MAX_STEPS_ALLOWED) {
            // reward -= 1.0;
            audio.play("lose");
            reset_game(&mut in_menu, &mut player, &mut invaders, rusty_bot, game_number);
            game_number += 1;
            score.update_best_points();
            // Write to a file
            score_data_file
                .write(format!("\n{},{}", score.get_count(), score.get_best_score()).as_bytes())
                .expect("write failed");
            score.reset_count();
            steps = 0;
            write_agent_brain(&agent.q_table);
        }
        let new_state = agent.get_state(&mut invaders, &mut player);

        agent.learn(current_state, action, reward, new_state);  
        write_agent_brain(&agent.q_table); 
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
