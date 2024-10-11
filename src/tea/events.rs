// widget event

pub enum WidgetEvent {
    MouseLeftClick { x: f32, y: f32 },
}

pub struct WidgetEventResult<UserEvent> {
    // matcha-ui system event

    // pub system_event: Option<{SystemMessage}>,

    // user event
    pub user_event: Option<UserEvent>,
}

impl<UserEvent> WidgetEventResult<UserEvent> {
    pub fn swap_user_event<UserEvent2>(self, event: UserEvent2) -> WidgetEventResult<UserEvent2> {
        WidgetEventResult {
            user_event: Some(event),
        }
    }
}

impl<R> Default for WidgetEventResult<R> {
    fn default() -> Self {
        WidgetEventResult { user_event: None }
    }
}
