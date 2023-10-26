use std::{ffi::c_void, sync::mpsc::RecvError};

use junowen_lib::{
    Fn011560, Fn0b7d40, Fn0d5ae0, Fn10f720, GameSettings, Menu, RenderingText, Selection, Th19,
};
use tokio::sync::mpsc::Receiver;
use tracing::trace;

use crate::{
    in_game_lobby::{Lobby, TitleMenuModifier},
    session::battle::BattleSession,
};

use super::{battle_session_state::BattleSessionState, in_session, prepare::Prepare, standby};

pub enum JunowenState {
    Standby,
    BattleSession(BattleSessionState),
}

impl JunowenState {
    pub fn game_settings(&self) -> Option<&GameSettings> {
        match self {
            Self::Standby => None,
            Self::BattleSession(session_state) => session_state.game_settings(),
        }
    }

    pub fn has_session(&self) -> bool {
        !matches!(self, Self::Standby)
    }

    pub fn start_battle_session(&mut self, battle_session: BattleSession) {
        *self = Self::BattleSession(BattleSessionState::Prepare(Prepare::new(battle_session)));
    }

    pub fn end_session(&mut self) {
        *self = Self::Standby;
    }

    fn update_state(
        &mut self,
        th19: &Th19,
        battle_session_rx: &mut Receiver<BattleSession>,
    ) -> Option<Option<&'static Menu>> {
        match self {
            Self::Standby => {
                if let Ok(session) = battle_session_rx.try_recv() {
                    trace!("session received");
                    self.start_battle_session(session);
                    return Some(None);
                };
                None
            }
            Self::BattleSession(session_state) => {
                let Some(ret) = session_state.update_state(th19) else {
                    self.end_session();
                    return None;
                };
                Some(ret)
            }
        }
    }

    fn update_th19_on_input_players(
        &mut self,
        menu: Option<&Menu>,
        th19: &mut Th19,
    ) -> Result<(), RecvError> {
        match self {
            Self::Standby => unreachable!(),
            Self::BattleSession(session_state) => {
                session_state.update_th19_on_input_players(menu, th19)
            }
        }
    }

    pub fn on_input_players(
        &mut self,
        th19: &mut Th19,
        battle_session_rx: &mut Receiver<BattleSession>,
    ) -> Result<(), RecvError> {
        let Some(menu) = self.update_state(th19, battle_session_rx) else {
            return Ok(());
        };
        self.update_th19_on_input_players(menu, th19)
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
                session_state.on_render_texts(th19, text_renderer)
            }
        }
    }

    pub fn on_round_over(&mut self, th19: &mut Th19) -> Result<(), RecvError> {
        match self {
            Self::Standby => Ok(()),
            Self::BattleSession(session_state) => session_state.on_round_over(th19),
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
        in_session::on_rewrite_controller_assignments(th19, old_fn);
    }

    pub fn on_loaded_game_settings(&self, th19: &mut Th19) {
        if let Some(game_settings) = self.game_settings() {
            th19.put_game_settings_in_game(game_settings).unwrap();
        }
    }
}