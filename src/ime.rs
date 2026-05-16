use ctru::services::{apt::Apt, gfx::Gfx};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImeStage {
    Nothing,
    SelectAllDown,
    SelectAllUp,
    BackSpaceDown,
    BackSpaceUp,
    PutText,
    EscapeDown,
    EscapeUp,
}

impl ImeStage {
    pub(crate) const START: ImeStage = ImeStage::SelectAllDown;
    pub(crate) const CANCEL: ImeStage = ImeStage::EscapeDown;
    pub(crate) fn next(self) -> Self {
        use ImeStage::*;
        match self {
            Nothing => Nothing,
            SelectAllDown => SelectAllUp,
            SelectAllUp => BackSpaceDown,
            BackSpaceDown => BackSpaceUp,
            BackSpaceUp => PutText,
            PutText => EscapeDown,
            EscapeDown => EscapeUp,
            EscapeUp => Nothing,
        }
    }
    pub(crate) fn add_event(self, events: &mut Vec<egui::Event>) -> bool {
        use ImeStage::*;
        match self {
            Nothing => false,
            SelectAllDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: true,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            SelectAllUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: false,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            BackSpaceDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            BackSpaceUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            PutText => true,
            EscapeDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            EscapeUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
        }
    }
}

/// For running after the bottom screen's `ctx.run`
pub(crate) fn ime_part_b(
    ime: &mut Option<egui::output::IMEOutput>,
    ime_stage: &ImeStage,
    current_text_value: &mut Option<String>,
    current_float_value: &mut Option<f64>,
    out: &egui::FullOutput,
) {
}

pub struct ImeState {
    stage: ImeStage,
    output: Option<egui::output::IMEOutput>,
    current_text: Option<String>,
    current_float: Option<f64>,
}

impl ImeState {
    pub fn new() -> ImeState {
        ImeState {
            stage: ImeStage::Nothing,
            output: None,
            current_text: None,
            current_float: None,
        }
    }

    pub fn part_a(&mut self, gfx: &Gfx, apt: &Apt, events: &mut Vec<egui::Event>) {
        if self.output.is_some() && self.stage == ImeStage::Nothing {
            use ctru::applets::swkbd;
            let mut kbd =
                swkbd::SoftwareKeyboard::new(swkbd::Kind::Normal, swkbd::ButtonConfig::LeftRight);
            kbd.set_initial_text(
                self.current_text
                    .take()
                    .filter(|x| !x.is_empty())
                    .map(|x| std::borrow::Cow::Owned(x))
                    .or(self
                        .current_float
                        .take()
                        .map(|x| std::borrow::Cow::Owned(x.to_string()))),
            );
            let (text, button) = kbd.launch(apt, gfx).unwrap();
            if button == swkbd::Button::Right {
                self.current_text = Some(text);
                self.stage = ImeStage::START;
            } else {
                self.stage = ImeStage::CANCEL;
            }
        }

        if self.stage.add_event(events) {
            events.push(egui::Event::Text(
                self.current_text.take().unwrap_or_default(),
            ));
        }

        self.stage = self.stage.next();
    }

    pub fn part_b(&mut self, out: &egui::output::FullOutput) {
        for e in &out.platform_output.events {
            match e {
                egui::output::OutputEvent::Clicked(widget_info) => {
                    if self.stage == ImeStage::Nothing {
                        self.current_text = widget_info.current_text_value.clone();
                        self.current_float = widget_info.value.clone();
                    }
                }
                _ => (),
            }
        }
        self.output = out.platform_output.ime;
    }
}
