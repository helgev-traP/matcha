use super::{
    context::SharedContext,
    events::UiEventResult,
    types::range::Range2D,
    ui::{Dom, DomComPareResult, Object, UiBackground, UiContext, UpdateWidgetError, Widget},
};

/// ! rewrite of component_old.rs
// MARK: - Component
pub struct Component<Model: Send + 'static, T: 'static> {
    label: Option<String>,
    // id: uuid::Uuid,

    // model
    model: Model,
    // model_updated: bool,

    // elm view function
    view_fn: fn(&Model) -> Box<dyn Dom<T>>,
}

// constructor
impl<Model: Send + 'static, T: 'static> Component<Model, T> {
    pub fn new(
        label: Option<&str>,
        model: Model,
        view: fn(&Model) -> Box<dyn Dom<T>>,
    ) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            model,
            view_fn: view,
        }
    }
}

// access to the model
impl<Model: Send + 'static, T: 'static> Component<Model, T> {
    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        // self.model_updated = true;
        &mut self.model
    }
}

// methods
impl<Model: Send + 'static, T: 'static> Component<Model, T> {
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // pub fn view(&mut self) -> Box<dyn Dom<T>> {
    //     if self.model_updated {
    //         self.model_updated = false;
    //         Box::new(ComponentDom::Dom {
    //             label: self.label.clone(),
    //             id: self.id,
    //             dom: (self.view_fn)(&self.model),
    //         })
    //     } else {
    //         Box::new(ComponentDom::NoChange {
    //             label: self.label.clone(),
    //             id: self.id,
    //             dom: (self.view_fn)(&self.model),
    //         })
    //     }
    // }
}

// MARK: - ComponentDom

pub enum ComponentDom<T: 'static> {
    Dom {
        label: Option<String>,
        id: uuid::Uuid,
        dom: Box<dyn Dom<T>>,
    },
    NoChange {
        label: Option<String>,
        id: uuid::Uuid,
        dom: Box<dyn Dom<T>>,
    },
}

impl<T: 'static> Dom<T> for ComponentDom<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        match self {
            ComponentDom::Dom { label, id, dom } => Box::new(ComponentWidget {
                label: label.clone(),
                id: *id,
                node: dom.build_widget_tree(),
            }),
            ComponentDom::NoChange { label, id, dom } => {
                let dom = dom.as_ref();

                Box::new(ComponentWidget {
                    label: label.clone(),
                    id: *id,
                    node: dom.build_widget_tree(),
                })
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: - ComponentWidget

pub struct ComponentWidget<T: 'static> {
    label: Option<String>,
    id: uuid::Uuid,
    node: Box<dyn Widget<T>>,
}

impl<T> Widget<T> for ComponentWidget<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), UpdateWidgetError> {
        let Some(dom) = dom.as_any().downcast_ref::<ComponentDom<T>>() else {
            return Err(UpdateWidgetError::TypeMismatch);
        };

        match dom {
            ComponentDom::Dom { label, id, dom } => {
                // update the label
                self.label = label.clone();

                // update the id
                self.id = *id;

                // Update the widget tree
                self.node.update_widget_tree(dom.as_ref())
            }
            ComponentDom::NoChange { label, id, dom } => {
                if self.label == *label && self.id == *id {
                    Ok(())
                } else {
                    // update the label
                    self.label = label.clone();

                    // update the id
                    self.id = *id;

                    // Update the widget tree
                    self.node.update_widget_tree(dom.as_ref())
                }
            }
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        let Some(dom) = dom.as_any().downcast_ref::<ComponentDom<T>>() else {
            return DomComPareResult::Different;
        };

        match dom {
            ComponentDom::Dom { dom, .. } => match self.node.compare(dom.as_ref()) {
                DomComPareResult::Same => DomComPareResult::Same,
                DomComPareResult::Different => DomComPareResult::Different,
                DomComPareResult::Changed(x) => DomComPareResult::Changed(x),
            },
            ComponentDom::NoChange { .. } => DomComPareResult::Same,
        }
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        self.node
            .widget_event(event, parent_size, context, tag, frame)
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        self.node.px_size(parent_size, context, tag, frame)
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        self.node.draw_range(parent_size, context, tag, frame)
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        self.node.cover_area(parent_size, context, tag, frame)
    }

    fn redraw(&self) -> bool {
        self.node.redraw()
    }

    fn render(&mut self, ui_background: UiBackground, ui_context: UiContext) -> Vec<Object> {
        self.node.render(ui_background, ui_context)
    }
}
