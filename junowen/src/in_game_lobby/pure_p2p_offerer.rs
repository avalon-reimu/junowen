use std::ffi::c_void;

use clipboard_win::{get_clipboard_string, set_clipboard_string};
use junowen_lib::{
    connection::{
        signaling::{
            parse_signaling_code, socket::async_read_write_socket::SignalingServerMessage,
            SignalingCodeType,
        },
        DataChannel, PeerConnection,
    },
    InputValue, Th19,
};
use tokio::sync::mpsc;
use tracing::trace;

use crate::session::{battle::BattleSession, spectator::SpectatorSessionGuest};

use super::{
    common_menu::{
        CommonMenu, LobbyScene, MenuAction, MenuContent, MenuDefine, MenuItem, OnMenuInputResult,
    },
    helper::{render_small_text_line, render_text_line},
    signaling::Signaling,
};

pub struct PureP2pOfferer<T> {
    offer_type: SignalingCodeType,
    answer_type: SignalingCodeType,
    create_session: fn(PeerConnection, DataChannel) -> T,
    messages: [&'static str; 3],
    common_menu: CommonMenu,
    signaling: Signaling,
    session_tx: mpsc::Sender<T>,
    answer: Option<String>,
    /// 0: require generate, 1: copied, 2: already copied, 3: copied again
    copy_state: u8,
}

impl<T> PureP2pOfferer<T>
where
    T: Send + 'static,
{
    pub fn new(
        offer_type: SignalingCodeType,
        answer_type: SignalingCodeType,
        create_session: fn(PeerConnection, DataChannel) -> T,
        messages: [&'static str; 3],
        session_tx: mpsc::Sender<T>,
    ) -> Self {
        Self {
            offer_type,
            answer_type,
            create_session,
            messages,
            common_menu: CommonMenu::new(
                "Ju.N.Owen",
                false,
                720,
                MenuDefine::new(
                    2,
                    vec![
                        MenuItem::new(
                            "Regenerate",
                            MenuContent::Action(MenuAction::Action(0, true)),
                        ),
                        MenuItem::new(
                            "Copy your code",
                            MenuContent::Action(MenuAction::Action(1, true)),
                        ),
                        MenuItem::new(
                            "Paste guest's code",
                            MenuContent::Action(MenuAction::Action(2, false)),
                        ),
                    ],
                ),
            ),
            signaling: Signaling::new(session_tx.clone(), create_session),
            session_tx,
            answer: None,
            copy_state: 0,
        }
    }

    pub fn on_input_menu(
        &mut self,
        current_input: InputValue,
        prev_input: InputValue,
        th19: &Th19,
    ) -> Option<LobbyScene> {
        self.signaling.recv();
        if self.signaling.connected() {
            self.reset();
        }
        if self.copy_state == 0 {
            if let Some(offer) = self.signaling.offer() {
                trace!("copied");
                set_clipboard_string(&self.offer_type.to_string(offer)).unwrap();
                self.copy_state = 1;
            }
        }
        match self
            .common_menu
            .on_input_menu(current_input, prev_input, th19)
        {
            OnMenuInputResult::None => None,
            OnMenuInputResult::Cancel => {
                self.copy_state = 2;
                if self.answer.is_some() || self.signaling.error().is_some() {
                    self.reset();
                }
                Some(LobbyScene::Root)
            }
            OnMenuInputResult::Action(MenuAction::SubScene(scene)) => Some(scene),
            OnMenuInputResult::Action(MenuAction::Action(action, _)) => {
                if action == 0 {
                    self.reset();
                }
                if action == 1 {
                    set_clipboard_string(
                        &self
                            .offer_type
                            .to_string(self.signaling.offer().as_ref().unwrap()),
                    )
                    .unwrap();
                    self.copy_state = if self.copy_state <= 1 { 1 } else { 3 };
                }
                if action == 2 {
                    let Ok(ok) = get_clipboard_string() else {
                        th19.play_sound(th19.sound_manager(), 0x10, 0);
                        return None;
                    };
                    let Ok((answer_type, answer)) = parse_signaling_code(&ok) else {
                        th19.play_sound(th19.sound_manager(), 0x10, 0);
                        return None;
                    };
                    if answer_type != self.answer_type {
                        th19.play_sound(th19.sound_manager(), 0x10, 0);
                        return None;
                    }
                    th19.play_sound(th19.sound_manager(), 0x07, 0);
                    self.answer = Some(self.answer_type.to_string(&answer));
                    self.signaling
                        .msg_tx_mut()
                        .take()
                        .unwrap()
                        .send(SignalingServerMessage::SetAnswerDesc(answer))
                        .unwrap();
                    self.common_menu =
                        CommonMenu::new("Ju.N.Owen", false, 720, MenuDefine::new(0, vec![]))
                }
                None
            }
        }
    }

    pub fn on_render_texts(&self, th19: &Th19, text_renderer: *const c_void) {
        self.common_menu.on_render_texts(th19, text_renderer);

        let mut line = 0;
        'a: {
            let Some(offer) = &self.signaling.offer() else {
                render_text_line(th19, text_renderer, 0, b"Preparing...");
                break 'a;
            };
            let text = if [2, 3].contains(&self.copy_state) {
                "Your signaling code is already created:"
            } else {
                "Your signaling code:"
            };
            render_text_line(th19, text_renderer, line, text.as_bytes());
            line += 2;
            let offer = self.offer_type.to_string(offer);
            let chunks = offer.as_bytes().chunks(100);
            let offer_len = (chunks.len() as f64 / 2.0).ceil() as u32;
            chunks.enumerate().for_each(|(i, chunk)| {
                render_small_text_line(th19, text_renderer, line * 2 + i as u32, chunk);
            });
            line += offer_len + 1;
            if [1, 3].contains(&self.copy_state) {
                render_text_line(th19, text_renderer, line, b"It was copied to Clipboard.");
                let text = self.messages[0].as_bytes();
                render_text_line(th19, text_renderer, line + 1, text);
            }
            line += 3;
            render_text_line(th19, text_renderer, line, self.messages[1].as_bytes());
            let Some(answer) = &self.answer else {
                break 'a;
            };
            let chunks = answer.as_bytes().chunks(100);
            let answer_len = (chunks.len() as f64 / 2.0).ceil() as u32;
            line += 2;
            chunks.enumerate().for_each(|(i, chunk)| {
                render_small_text_line(th19, text_renderer, line * 2 + i as u32, chunk);
            });
            line += answer_len + 1;
            let text = self.messages[2].as_bytes();
            render_text_line(th19, text_renderer, line, text);
        }
        if let Some(err) = self.signaling.error() {
            line += 1;
            render_text_line(th19, text_renderer, line, err.to_string().as_bytes());
        }
    }

    fn reset(&mut self) {
        *self = Self::new(
            self.offer_type,
            self.answer_type,
            self.create_session,
            self.messages,
            self.session_tx.clone(),
        );
    }
}

pub fn pure_p2p_host(session_tx: mpsc::Sender<BattleSession>) -> PureP2pOfferer<BattleSession> {
    PureP2pOfferer::new(
        SignalingCodeType::BattleOffer,
        SignalingCodeType::BattleAnswer,
        |pc, dc| BattleSession::new(pc, dc, true),
        [
            "Share your signaling code with guest.",
            "Guest's signaling code:",
            "Waiting for guest to connect...",
        ],
        session_tx,
    )
}

pub fn pure_p2p_spectator(
    session_tx: mpsc::Sender<SpectatorSessionGuest>,
) -> PureP2pOfferer<SpectatorSessionGuest> {
    PureP2pOfferer::new(
        SignalingCodeType::SpectatorOffer,
        SignalingCodeType::SpectatorAnswer,
        SpectatorSessionGuest::new,
        [
            "Share your signaling code with player.",
            "Player's signaling code:",
            "Waiting for player to connect...",
        ],
        session_tx,
    )
}