use anyhow::Result;
use junowen_lib::{Input, Menu, ScreenId, Th19};

use crate::session::{MatchInitial, RandomNumberInitial, Session};

pub fn on_input_players(
    first_time: bool,
    session: &mut Session,
    menu: &Menu,
    th19: &mut Th19,
    match_initial: &mut Option<MatchInitial>,
) -> Result<()> {
    if first_time {
        th19.set_difficulty_cursor(1).unwrap();
        th19.p1_mut().character = 0;
        th19.p2_mut().character = 1;
        for player_select in th19.app().main_loop_tasks.player_selects_mut() {
            player_select.player.card = 0;
        }

        if session.host() {
            if match_initial.is_none() {
                let init = MatchInitial {
                    game_settings: th19.game_settings_in_menu().unwrap(),
                };
                session.send_init_match(init.clone());
                *match_initial = Some(init);
            }
            session.send_init_random_number(RandomNumberInitial {
                seed1: th19.rand_seed1().unwrap(),
                seed2: th19.rand_seed2().unwrap(),
                seed3: th19.rand_seed3().unwrap(),
                seed4: th19.rand_seed4().unwrap(),
                seed5: th19.rand_seed5().unwrap(),
                seed6: th19.rand_seed6().unwrap(),
                seed7: th19.rand_seed7().unwrap(),
                seed8: th19.rand_seed8().unwrap(),
            });
        } else {
            if match_initial.is_none() {
                *match_initial = Some(session.recv_init_match().unwrap());
            }
            let (init, delay_remainings) = session.recv_init_round().unwrap();
            println!("delay_remainings: {}", delay_remainings);
            th19.set_rand_seed1(init.seed1).unwrap();
            th19.set_rand_seed2(init.seed2).unwrap();
            th19.set_rand_seed3(init.seed3).unwrap();
            th19.set_rand_seed4(init.seed4).unwrap();
            th19.set_rand_seed5(init.seed5).unwrap();
            th19.set_rand_seed6(init.seed6).unwrap();
            th19.set_rand_seed7(init.seed7).unwrap();
            th19.set_rand_seed8(init.seed8).unwrap();
        }
    }

    if menu.screen_id == ScreenId::DifficultySelect {
        return Ok(());
    }

    session.enqueue_input(th19.input().p1_input().0 as u8);
    let (p1, p2) = session.dequeue_inputs()?;
    let input = th19.input_mut();
    input.set_p1_input(Input(p1 as u32));
    input.set_p2_input(Input(p2 as u32));

    Ok(())
}

pub fn on_input_menu(session: &mut Session, th19: &mut Th19) -> Result<()> {
    let menu = th19.app().main_loop_tasks.find_menu_mut().unwrap();
    if menu.screen_id != ScreenId::DifficultySelect {
        return Ok(());
    }

    session.enqueue_input(if session.host() {
        th19.menu_input().0 as u8
    } else {
        Input::NULL as u8
    });
    let (p1, _p2) = session.dequeue_inputs()?;
    *th19.menu_input_mut() = Input(p1 as u32);
    Ok(())
}

pub fn on_loaded_game_settings(match_initial: &MatchInitial, th19: &mut Th19) {
    th19.put_game_settings_in_game(&match_initial.game_settings)
        .unwrap();
}
