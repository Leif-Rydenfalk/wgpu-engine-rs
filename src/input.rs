use input_actions::{
	binding, event,
	source::{Key, MouseButton},
};
use winit::keyboard::KeyCode;
use std::convert::{TryFrom, TryInto};

// TODO: Winit gamepad support is still in progress https://github.com/rust-windowing/winit/issues/944
pub fn parse_winit_event<T>(
	event: &winit::event::Event<T>,
) -> Result<(event::Source, event::Event), ()> {
	use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta};
	match event {
		winit::event::Event::DeviceEvent {
			event: DeviceEvent::MouseMotion { delta },
			..
		} => Ok((
			event::Source::Mouse,
			event::Event::new(binding::Source::Mouse(binding::Mouse::Move),  event::State::MouseMove(delta.0, delta.1))
		)),
		winit::event::Event::DeviceEvent {
			event:
				DeviceEvent::MouseWheel {
					delta: MouseScrollDelta::LineDelta(horizontal, vertical),
				},
			..
		} => Ok((
			event::Source::Mouse,
			event::Event::new(
				binding::Source::Mouse(binding::Mouse::Scroll),
				event::State::MouseScroll(*horizontal, *vertical),
            )
		)),
		winit::event::Event::DeviceEvent {
			event: DeviceEvent::Button { button, state },
			..
		} => MouseButton::try_from(*button)
			.map(|button_enum| {
				(
					event::Source::Mouse,
					event::Event::new(
						binding::Source::Mouse(binding::Mouse::Button(button_enum)),
						event::State::ButtonState(match state {
							ElementState::Pressed => event::ButtonState::Pressed,
							ElementState::Released => event::ButtonState::Released,
						}),
                    )
				)
			})
			.map_err(|id| {
				println!("ERROR failed to parse button id {:?}", id);
				()
			}),
		winit::event::Event::DeviceEvent {
			event:
				DeviceEvent::Key(KeyboardInput {
					state,
					virtual_keycode: Some(keycode),
					..
				}),
			..
		} => (*keycode)
			.try_into()
			.map(|keycode| {
				(
					event::Source::Keyboard,
					event::Event::new(
						binding::Source::Keyboard(keycode),
						event::State::ButtonState(match state {
							ElementState::Pressed => event::ButtonState::Pressed,
							ElementState::Released => event::ButtonState::Released,
						}),
                    )
				)
			})
			.map_err(|_| ()),
		_ => Err(()),
	}
}

impl TryFrom<winit::event::ButtonId> for MouseButton {
	type Error = winit::event::ButtonId;
	fn try_from(id: winit::event::ButtonId) -> Result<Self, Self::Error> {
		match id {
			1 => Ok(MouseButton::Left),
			2 => Ok(MouseButton::Center),
			3 => Ok(MouseButton::Right),
			_ => Err(id),
		}
	}
}

impl TryFrom<KeyCode> for Key {
	type Error = ();
	fn try_from(winit: KeyCode) -> Result<Self, Self::Error> {
		match winit {
			KeyCode::Numpad1 => Ok(Key::Key1),
			KeyCode::Numpad2 => Ok(Key::Key2),
			KeyCode::Numpad3 => Ok(Key::Key3),
			KeyCode::Numpad4 => Ok(Key::Key4),
			KeyCode::Numpad5 => Ok(Key::Key5),
			KeyCode::Numpad6 => Ok(Key::Key6),
			KeyCode::Numpad7 => Ok(Key::Key7),
			KeyCode::Numpad8 => Ok(Key::Key8),
			KeyCode::Numpad9 => Ok(Key::Key9),
			KeyCode::Numpad0 => Ok(Key::Key0),
            KeyCode::KeyA => Ok(Key::A),
            KeyCode::KeyB => Ok(Key::B),
            KeyCode::KeyC => Ok(Key::C),
            KeyCode::KeyD => Ok(Key::D),
            KeyCode::KeyE => Ok(Key::E),
            KeyCode::KeyF => Ok(Key::F),
            KeyCode::KeyG => Ok(Key::G),
            KeyCode::KeyH => Ok(Key::H),
            KeyCode::KeyI => Ok(Key::I),
            KeyCode::KeyJ => Ok(Key::J),
            KeyCode::KeyK => Ok(Key::K),
            KeyCode::KeyL => Ok(Key::L),
            KeyCode::KeyM => Ok(Key::M),
            KeyCode::KeyN => Ok(Key::N),
            KeyCode::KeyO => Ok(Key::O),
            KeyCode::KeyP => Ok(Key::P),
            KeyCode::KeyQ => Ok(Key::Q),
            KeyCode::KeyR => Ok(Key::R),
            KeyCode::KeyS => Ok(Key::S),
            KeyCode::KeyT => Ok(Key::T),
            KeyCode::KeyU => Ok(Key::U),
            KeyCode::KeyV => Ok(Key::V),
            KeyCode::KeyW => Ok(Key::W),
            KeyCode::KeyX => Ok(Key::X),
            KeyCode::KeyY => Ok(Key::Y),
            KeyCode::KeyZ => Ok(Key::Z),            
			KeyCode::Escape => Ok(Key::Escape),
			KeyCode::F1 => Ok(Key::F1),
			KeyCode::F2 => Ok(Key::F2),
			KeyCode::F3 => Ok(Key::F3),
			KeyCode::F4 => Ok(Key::F4),
			KeyCode::F5 => Ok(Key::F5),
			KeyCode::F6 => Ok(Key::F6),
			KeyCode::F7 => Ok(Key::F7),
			KeyCode::F8 => Ok(Key::F8),
			KeyCode::F9 => Ok(Key::F9),
			KeyCode::F10 => Ok(Key::F10),
			KeyCode::F11 => Ok(Key::F11),
			KeyCode::F12 => Ok(Key::F12),
			KeyCode::F13 => Ok(Key::F13),
			KeyCode::F14 => Ok(Key::F14),
			KeyCode::F15 => Ok(Key::F15),
			KeyCode::F16 => Ok(Key::F16),
			KeyCode::F17 => Ok(Key::F17),
			KeyCode::F18 => Ok(Key::F18),
			KeyCode::F19 => Ok(Key::F19),
			KeyCode::F20 => Ok(Key::F20),
			KeyCode::F21 => Ok(Key::F21),
			KeyCode::F22 => Ok(Key::F22),
			KeyCode::F23 => Ok(Key::F23),
			KeyCode::F24 => Ok(Key::F24),
			// KeyCode::Snapshot => Ok(Key::Snapshot),
			// KeyCode::Scroll => Ok(Key::ScrollLock),
			KeyCode::Pause => Ok(Key::Pause),
			KeyCode::Insert => Ok(Key::Insert),
			KeyCode::Home => Ok(Key::Home),
			KeyCode::Delete => Ok(Key::Delete),
			KeyCode::End => Ok(Key::End),
			KeyCode::PageDown => Ok(Key::PageDown),
			KeyCode::PageUp => Ok(Key::PageUp),
			KeyCode::ArrowLeft => Ok(Key::Left),
			KeyCode::ArrowUp => Ok(Key::Up),
			KeyCode::ArrowRight => Ok(Key::Right),
			KeyCode::ArrowDown => Ok(Key::Down),
			KeyCode::Backspace => Ok(Key::Back),
			KeyCode::Enter => Ok(Key::Return),
			KeyCode::Space => Ok(Key::Space),
			// KeyCode::Compose => Err(()),
			// KeyCode::Caret => Err(()),
			KeyCode::NumLock => Ok(Key::Numlock),
			KeyCode::Numpad0 => Ok(Key::Numpad0),
			KeyCode::Numpad1 => Ok(Key::Numpad1),
			KeyCode::Numpad2 => Ok(Key::Numpad2),
			KeyCode::Numpad3 => Ok(Key::Numpad3),
			KeyCode::Numpad4 => Ok(Key::Numpad4),
			KeyCode::Numpad5 => Ok(Key::Numpad5),
			KeyCode::Numpad6 => Ok(Key::Numpad6),
			KeyCode::Numpad7 => Ok(Key::Numpad7),
			KeyCode::Numpad8 => Ok(Key::Numpad8),
			KeyCode::Numpad9 => Ok(Key::Numpad9),
			KeyCode::NumpadAdd => Ok(Key::NumpadPlus),
			KeyCode::NumpadDivide => Ok(Key::NumpadSlash),
			KeyCode::NumpadDecimal => Err(()),
			KeyCode::NumpadComma => Err(()),
			KeyCode::NumpadEnter => Ok(Key::NumpadEnter),
			// KeyCode::NumpadEquals => Err(()),
			KeyCode::NumpadMultiply => Ok(Key::NumpadAsterisk),
			KeyCode::NumpadSubtract => Ok(Key::NumpadMinus),
			// KeyCode::AbntC1 => Err(()),
			// KeyCode::AbntC2 => Err(()),
			// KeyCode::Apostrophe => Ok(Key::Apostrophe),
			// KeyCode::Apps => Err(()),
			// KeyCode::Asterisk => Err(()),
			// KeyCode::At => Err(()),
			// KeyCode::Ax => Err(()),
			KeyCode::Backslash => Ok(Key::Backslash),
			// KeyCode::Calculator => Err(()),
			// KeyCode::Capital => Ok(Key::CapitalLock),
			// KeyCode::Colon => Err(()),
			KeyCode::Comma => Ok(Key::Comma),
			KeyCode::Convert => Err(()),
			// KeyCode::Equals => Ok(Key::Equals),
			// KeyCode::Grave => Ok(Key::Grave),
			// KeyCode::Kana => Err(()),
			// KeyCode::Kanji => Err(()),
			// KeyCode::LAlt => Ok(Key::LAlt),
			// KeyCode::LBracket => Ok(Key::LBracket),
			// KeyCode::LControl => Ok(Key::LControl),
			// KeyCode::LShift => Ok(Key::LShift),
			// KeyCode::LWin => Ok(Key::LWin),
			// KeyCode::Mail => Err(()),
			KeyCode::MediaSelect => Err(()),
			KeyCode::MediaStop => Err(()),
			KeyCode::Minus => Ok(Key::Minus),
			// KeyCode::Mute => Err(()),
			// KeyCode::MyComputer => Err(()),
			// KeyCode::NavigateForward => Err(()),
			// KeyCode::NavigateBackward => Err(()),
			// KeyCode::NextTrack => Err(()),
			// KeyCode::NoConvert => Err(()),
			// KeyCode::OEM102 => Err(()),
			KeyCode::Period => Ok(Key::Period),
			// KeyCode::PlayPause => Err(()),
			// KeyCode::Plus => Err(()),
			KeyCode::Power => Err(()),
			// KeyCode::PrevTrack => Err(()),
			// KeyCode::RAlt => Ok(Key::RAlt),
			// KeyCode::RBracket => Ok(Key::RBracket),
			// KeyCode::RControl => Ok(Key::RControl),
			// KeyCode::RShift => Ok(Key::RShift),
			// KeyCode::RWin => Ok(Key::RWin),
			KeyCode::Semicolon => Ok(Key::Semicolon),
			KeyCode::Slash => Ok(Key::Slash),
			KeyCode::Sleep => Err(()),
			// KeyCode::Stop => Err(()),
			// KeyCode::Sysrq => Err(()),
			KeyCode::Tab => Ok(Key::Tab),
			// KeyCode::Underline => Err(()),
			// KeyCode::Unlabeled => Err(()),
			// KeyCode::VolumeDown => Err(()),
			// KeyCode::VolumeUp => Err(()),
			// KeyCode::Wake => Err(()),
			// KeyCode::WebBack => Err(()),
			// KeyCode::WebFavorites => Err(()),
			// KeyCode::WebForward => Err(()),
			// KeyCode::WebHome => Err(()),
			// KeyCode::WebRefresh => Err(()),
			// KeyCode::WebSearch => Err(()),
			// KeyCode::WebStop => Err(()),
			// KeyCode::Yen => Err(()),
			KeyCode::Copy => Err(()),
			KeyCode::Paste => Err(()),
			KeyCode::Cut => Err(()),
            _ => unimplemented!("Key")
		}
	}
}
