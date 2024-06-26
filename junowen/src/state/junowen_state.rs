mod on_rewrite_controller_assignments;
mod standby;

use std::{ffi::c_void, sync::mpsc::RecvError};

use anyhow::Result;
use junowen_lib::{
    structs::app::{MainMenu, ScreenId},
    structs::{others::RenderingText, selection::Selection, settings::GameSettings},
    Fn011560, Fn0b7d40, Fn0d5ae0, Fn10f720, Th19,
};
use tracing::trace;

use crate::{
    file::Features,
    in_game_lobby::{Lobby, TitleMenuModifier},
    session::{battle::BattleSession, spectator::SpectatorSession},
    signaling::waiting_for_match::{WaitingForMatch, WaitingForSpectator},
};

use super::{
    battle_session_state::BattleSessionState, spectator_session_state::SpectatorSessionState,
};

use self::on_rewrite_controller_assignments::on_rewrite_controller_assignments;

pub enum JunowenState {
    Standby,
    BattleSession(BattleSessionState),
    SpectatorSession(SpectatorSessionState),
}

impl JunowenState {
    pub fn game_settings(&self) -> Option<&GameSettings> {
        match self {
            Self::Standby => None,
            Self::BattleSession(session_state) => session_state.game_settings(),
            Self::SpectatorSession(session_state) => session_state.game_settings(),
        }
    }

    pub fn has_session(&self) -> bool {
        !matches!(self, Self::Standby)
    }

    fn start_battle_session(
        &mut self,
        battle_session: BattleSession,
        waiting: WaitingForSpectator,
    ) {
        *self = Self::BattleSession(BattleSessionState::prepare(battle_session, waiting));
    }

    fn end_session(&mut self) {
        *self = Self::Standby;
    }

    pub fn abort_session(&mut self, th19: &mut Th19) {
        self.end_session();
        th19.set_no_wait(false);
    }

    pub fn start_spectator_session(&mut self, session: SpectatorSession) {
        *self = Self::SpectatorSession(SpectatorSessionState::prepare(session));
    }

    fn update_state(
        &mut self,
        th19: &Th19,
        waiting_for_match: &mut Option<WaitingForMatch>,
    ) -> (bool, Option<&'static MainMenu>) {
        match self {
            Self::Standby => {
                let Some(old_waiting) = waiting_for_match.take() else {
                    return (false, None);
                };
                if let Some(main_menu) = th19.app().main_loop_tasks().find_main_menu() {
                    if main_menu.screen_id() == ScreenId::OnlineVSMode {
                        return (false, None);
                    }
                }
                match old_waiting {
                    WaitingForMatch::Opponent(waiting) => {
                        match waiting.try_into_session_and_waiting_for_spectator() {
                            Ok((session, waiting)) => {
                                trace!("session received");
                                self.start_battle_session(session, waiting);
                                (true, None)
                            }
                            Err(waiting) => {
                                *waiting_for_match = Some(WaitingForMatch::Opponent(waiting));
                                (false, None)
                            }
                        }
                    }
                    WaitingForMatch::SpectatorHost(waiting) => match waiting.try_into_session() {
                        Ok(session) => {
                            trace!("session received");
                            self.start_spectator_session(session);
                            (true, None)
                        }
                        Err(waiting) => {
                            *waiting_for_match = Some(WaitingForMatch::SpectatorHost(waiting));
                            (false, None)
                        }
                    },
                }
            }
            Self::BattleSession(session_state) => {
                let Some(menu_opt) = session_state.update_state(th19) else {
                    self.end_session();
                    return (true, None);
                };
                (false, menu_opt)
            }
            Self::SpectatorSession(session_state) => {
                let Some(menu_opt) = session_state.update_state(th19) else {
                    self.end_session();
                    return (true, None);
                };
                (false, menu_opt)
            }
        }
    }

    fn update_th19_on_input_players(
        &mut self,
        changed: bool,
        menu: Option<&MainMenu>,
        th19: &mut Th19,
    ) -> Result<(), RecvError> {
        match self {
            Self::Standby => {
                if changed {
                    th19.set_no_wait(false);
                }
                Ok(())
            }
            Self::BattleSession(session_state) => {
                session_state.update_th19_on_input_players(menu, th19)
            }
            Self::SpectatorSession(session_state) => {
                session_state.update_th19_on_input_players(menu, th19)
            }
        }
    }

    pub fn on_input_players(
        &mut self,
        th19: &mut Th19,
        waiting_for_match: &mut Option<WaitingForMatch>,
    ) -> Result<(), RecvError> {
        let (changed, menu_opt) = self.update_state(th19, waiting_for_match);
        self.update_th19_on_input_players(changed, menu_opt, th19)
    }

    pub fn on_input_menu(
        &mut self,
        th19: &mut Th19,
        title_menu_modifier: &mut TitleMenuModifier,
        lobby: &mut Lobby,
    ) -> Result<(), RecvError> {
        match self {
            Self::Standby => {
                standby::update_th19_on_input_menu(th19, title_menu_modifier, lobby);
            }
            Self::BattleSession(session_state) => session_state.on_input_menu(th19)?,
            Self::SpectatorSession(session_state) => {
                if !session_state.on_input_menu(th19)? {
                    self.abort_session(th19);
                }
            }
        }
        Ok(())
    }

    pub fn render_object(
        &self,
        title_menu_modifier: &TitleMenuModifier,
        old: Fn0b7d40,
        obj_renderer: *const c_void,
        obj: *const c_void,
    ) {
        if !self.has_session() {
            standby::render_object(title_menu_modifier, old, obj_renderer, obj);
            return;
        }
        old(obj_renderer, obj);
    }

    pub fn render_text(
        &self,
        th19: &Th19,
        title_menu_modifier: &TitleMenuModifier,
        old: Fn0d5ae0,
        text_renderer: *const c_void,
        text: &mut RenderingText,
    ) -> u32 {
        if !self.has_session() {
            return standby::render_text(th19, title_menu_modifier, old, text_renderer, text);
        }
        old(text_renderer, text)
    }

    pub fn on_render_texts(
        &self,
        features: &[Features],
        th19: &Th19,
        title_menu_modifier: &TitleMenuModifier,
        lobby: &Lobby,
        text_renderer: *const c_void,
    ) {
        match self {
            Self::Standby => {
                standby::on_render_texts(th19, title_menu_modifier, lobby, text_renderer);
            }
            Self::BattleSession(session_state) => {
                session_state.on_render_texts(features, th19, text_renderer)
            }
            Self::SpectatorSession(session_state) => {
                session_state.on_render_texts(th19, text_renderer)
            }
        }
    }

    pub fn on_round_over(&mut self, th19: &mut Th19) -> Result<(), RecvError> {
        match self {
            Self::Standby => Ok(()),
            Self::BattleSession(session_state) => session_state.on_round_over(th19),
            Self::SpectatorSession(session_state) => session_state.on_round_over(th19),
        }
    }

    pub fn is_online_vs(&self, this: *const Selection, old: Fn011560) -> u8 {
        let ret = old(this);
        if !self.has_session() {
            return ret;
        }
        1
    }

    pub fn on_rewrite_controller_assignments(
        &self,
        th19: &mut Th19,
        old_fn: fn(&mut Th19) -> Fn10f720,
    ) {
        if !self.has_session() {
            old_fn(th19)();
            return;
        }
        on_rewrite_controller_assignments(th19, old_fn);
    }

    pub fn on_loaded_game_settings(&self, th19: &mut Th19) {
        if let Some(game_settings) = self.game_settings() {
            th19.put_game_settings_in_game(game_settings).unwrap();
        }
    }
}
