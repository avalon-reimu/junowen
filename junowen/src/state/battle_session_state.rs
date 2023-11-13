use std::{ffi::c_void, mem, sync::mpsc::RecvError};

use anyhow::Result;
use junowen_lib::{GameSettings, Menu, ScreenId, Th19};

use crate::session::battle::BattleSession;

use super::{
    battle_game::BattleGame, battle_select::BattleSelect, in_session, prepare::Prepare,
    spectator_host::SpectatorHostState,
};

pub enum BattleSessionState {
    Null,
    Prepare(Prepare<BattleSession>),
    Select(BattleSelect),
    GameLoading {
        session: BattleSession,
        spectator_host_state: SpectatorHostState,
    },
    Game(BattleGame),
    BackToSelect {
        session: BattleSession,
        spectator_host_state: SpectatorHostState,
    },
}

impl BattleSessionState {
    pub fn game_settings(&self) -> Option<&GameSettings> {
        match self {
            Self::Null => unreachable!(),
            Self::GameLoading { session, .. } | Self::BackToSelect { session, .. } => {
                session.match_initial().map(|x| &x.game_settings)
            }
            Self::Prepare(i) => i.session().match_initial().map(|x| &x.game_settings),
            Self::Select(i) => i.session().match_initial().map(|x| &x.game_settings),
            Self::Game(i) => i.session().match_initial().map(|x| &x.game_settings),
        }
    }

    pub fn change_to_select(&mut self) {
        let old = mem::replace(self, Self::Null);
        let (session, spectator_host_state) = match old {
            Self::Null => unreachable!(),
            Self::Prepare(prepare) => (prepare.inner_session(), SpectatorHostState::standby()),
            Self::Select { .. } => unreachable!(),
            Self::GameLoading { .. } => unreachable!(),
            Self::Game { .. } => unreachable!(),
            Self::BackToSelect {
                session,
                spectator_host_state,
            } => (session, spectator_host_state),
        };
        *self = Self::Select(BattleSelect::new(session, spectator_host_state));
    }
    pub fn change_to_game_loading(&mut self) {
        let old = mem::replace(self, Self::Null);
        let Self::Select(old) = old else {
            unreachable!()
        };
        let (session, spectator_host_state) = old.inner_state();
        *self = Self::GameLoading {
            session,
            spectator_host_state,
        }
    }
    pub fn change_to_game(&mut self) {
        let old = mem::replace(self, Self::Null);
        let Self::GameLoading {
            session,
            spectator_host_state,
        } = old
        else {
            unreachable!()
        };
        *self = Self::Game(BattleGame::new(session, spectator_host_state));
    }
    pub fn change_to_back_to_select(&mut self) {
        let old = mem::replace(self, Self::Null);
        let Self::Game(game) = old else {
            unreachable!()
        };
        let (session, spectator_host_state) = game.inner_state();
        *self = Self::BackToSelect {
            session,
            spectator_host_state,
        }
    }

    pub fn update_state(&mut self, th19: &Th19) -> Option<Option<&'static Menu>> {
        match self {
            Self::Null => unreachable!(),
            Self::Prepare(prepare) => {
                let Some(menu) = th19.app().main_loop_tasks().find_menu() else {
                    return Some(None);
                };
                if prepare.update_state(menu, th19) {
                    self.change_to_select();
                }
                Some(Some(menu))
            }
            Self::Select { .. } => {
                let menu = th19.app().main_loop_tasks().find_menu().unwrap();
                match menu.screen_id {
                    ScreenId::GameLoading => {
                        self.change_to_game_loading();
                        Some(Some(menu))
                    }
                    ScreenId::PlayerMatchupSelect => None,
                    _ => Some(Some(menu)),
                }
            }
            Self::GameLoading { .. } => {
                let Some(game) = th19.round() else {
                    return Some(None);
                };
                if !game.is_first_frame() {
                    return Some(None);
                }
                self.change_to_game();
                Some(None)
            }
            Self::Game { .. } => {
                if th19.round().is_some() {
                    return Some(None);
                }
                self.change_to_back_to_select();
                Some(None)
            }
            Self::BackToSelect { .. } => {
                let Some(menu) = th19.app().main_loop_tasks().find_menu() else {
                    return Some(None);
                };
                if menu.screen_id != ScreenId::CharacterSelect {
                    return Some(Some(menu));
                }
                self.change_to_select();
                Some(Some(menu))
            }
        }
    }

    pub fn update_th19_on_input_players(
        &mut self,
        menu: Option<&Menu>,
        th19: &mut Th19,
    ) -> Result<(), RecvError> {
        match self {
            Self::Null => unreachable!(),
            Self::Prepare(prepare) => prepare.update_th19_on_input_players(th19),
            Self::Select(select) => select.update_th19_on_input_players(menu.unwrap(), th19)?,
            Self::GameLoading { .. } => {}
            Self::Game(game) => game.update_th19(th19)?,
            Self::BackToSelect { .. } => {}
        }
        Ok(())
    }

    pub fn on_input_menu(&mut self, th19: &mut Th19) -> Result<(), RecvError> {
        match self {
            Self::Null => unreachable!(),
            Self::Prepare(prepare) => prepare.update_th19_on_input_menu(th19),
            Self::Select(select) => select.update_th19_on_input_menu(th19)?,
            Self::GameLoading { .. } => {}
            Self::Game { .. } => {}
            Self::BackToSelect { .. } => {}
        }
        Ok(())
    }

    pub fn on_render_texts(&self, th19: &Th19, text_renderer: *const c_void) {
        let (session, spectator_host_state) = {
            match self {
                Self::Null => unreachable!(),
                Self::Prepare(inner) => (inner.session(), None),
                Self::Select(inner) => (inner.session(), Some(inner.spectator_host_state())),
                Self::GameLoading {
                    session,
                    spectator_host_state,
                } => (session, Some(spectator_host_state)),
                Self::Game(inner) => (inner.session(), Some(inner.spectator_host_state())),
                Self::BackToSelect {
                    session,
                    spectator_host_state,
                } => (session, Some(spectator_host_state)),
            }
        };
        let (p1, p2) = if session.host() {
            (
                th19.online_vs_mode().player_name(),
                session.remote_player_name().into(),
            )
        } else {
            (
                session.remote_player_name().into(),
                th19.online_vs_mode().player_name(),
            )
        };
        in_session::on_render_texts(
            th19,
            session.host(),
            session.delay(),
            &p1,
            &p2,
            spectator_host_state,
            text_renderer,
        );
    }

    pub fn on_round_over(&mut self, th19: &mut Th19) -> Result<(), RecvError> {
        let Self::Game(game) = self else {
            return Ok(());
        };
        game.on_round_over(th19)
    }
}
