use log::warn;
use macros::{widget, widget_style};

use crate::{
    context::{LoadExtent, ManageIntrinsic, ManageWidgetData},
    decorator::{
        content::Content, DecoratorExt, DrawDecorator, EventHitTestDecorator, MeasureDecorator,
    },
    events::{EventContext, EventHandling, EventHitTest, EventRouter, HitTestResult, PendingEvent},
    stage::{
        deinit::{Deinit, DeinitContext},
        draw::{draw_debug_bounds, Draw, DrawContext, Drawer},
        init::{Init, InitContext},
        invalidate::{Invalidate, InvalidateContext, RebuildStatus},
        layout::{Layout, LayoutContext},
        measure::{self, Constraints, ManageMeasures, Measure, MeasureContext, SizingMode},
    },
    types::{
        alignment::Alignment,
        border::Border,
        extent::Extent,
        identifiers::{WidgetClass, WidgetId, WidgetKey},
        offset::Offset,
        spacing::Spacing,
        style::{Configure, WidgetStyle},
        Color, Point,
    },
    widget::{
        flex_container::FlexContainer, WidgetGetType, WidgetInformation, WidgetInformationContext,
        WidgetSizingMode,
    },
};

/// A simple box that stays the same size.
///
/// Unlike the [`FlexContainer`], which stretches and moves to fit many things,
/// the `Container` is a rigid frame for just **one** child widget.
///
/// The Container does three main things:
/// * It sets a fixed width and height that never change.
/// * It holds exactly one child widget inside itself.
/// * It acts as a wall, so the child inside cannot push the box to make it bigger.
///
/// Use this when you need a UI element to stay exactly the same,
/// like a fixed icon or a status light that should never grow or shrink.
#[widget(kind = container)]
#[derive(bon::Builder, Default)]
pub struct Container {
    /// The internal spacing between the widget's boundary box and its actual content.
    ///
    /// This field defines a buffer zone (Top, Right, Bottom, Left) that
    /// effectively shrinks the available area for the widget's content
    /// without changing the widget's outer dimensions. It ensures
    /// content does not touch the edges of its container.
    #[style]
    spacing: Spacing,

    /// A hard-coded, fixed dimension for this axis.
    ///
    /// When set, the container will occupy exactly this many units regardless
    /// of its content's size or the parent's constraints. This effectively
    /// "locks" the widget's size, preventing it from expanding or
    /// shrinking during the layout pass.
    #[style]
    width: usize,

    /// A hard-coded, fixed dimension for this axis.
    ///
    /// When set, the container will occupy exactly this many units regardless
    /// of its content's size or the parent's constraints. This effectively
    /// "locks" the widget's size, preventing it from expanding or
    /// shrinking during the layout pass.
    #[style]
    height: usize,
}

/// A targeted configuration set used to override or provide specific
/// parameters for a Container widget based on its unique identifier.
///
/// Instead of traversing the widget tree to modify an existing Container,
/// this struct allows external systems to inject layout and styling
/// data—such as alignment and borders—directly into the widget's
/// compilation phase. If no configuration is associated with a
/// widget's ID, it continues to use its own internal state.
#[widget_style(kind = minimal, targets(Container, FlexContainer))]
#[derive(bon::Builder, Debug, Clone)]
pub struct ContainerStyle {
    pub background_color: Color,
    pub border: Border,
    pub spacing: Spacing,
    pub alignment: Alignment,
}

impl WidgetGetType for Container {
    fn get_type(&self) -> &'static str {
        "container"
    }
}

impl<C> WidgetSizingMode<C> for Container
where
    C: WidgetInformationContext,
{
    fn sizing_mode(&self, _context: &C) -> SizingMode {
        SizingMode::Fixed
    }
}

impl<C> Init<C> for Container
where
    C: InitContext,
{
    fn on_init(&mut self, context: &mut C) {
        if let Some(WidgetStyle::Container(container_style)) = context.get_style(&self.class) {
            self.configure(container_style.clone());
        }

        if let Some(child) = &mut self.child {
            child.init(context);
        }
    }
}

impl<C> Deinit<C> for Container where C: DeinitContext {}

impl<C> Invalidate<C> for Container
where
    C: InvalidateContext,
{
    fn on_style_update(&mut self, _context: &mut C, style: WidgetStyle) {
        if let WidgetStyle::Container(container_style) = style {
            self.configure(container_style);
        }
    }

    fn on_rebuild(&mut self, _context: &mut C) -> RebuildStatus {
        RebuildStatus::NothingChanged
    }
}

impl<C> Measure<C, f32> for Container
where
    C: MeasureContext<f32>,
{
    fn intrinsic_content(&self, context: &mut C) -> measure::Intrinsic<f32>
    where
        C: ManageIntrinsic<f32, WidgetId>,
    {
        Content::intrinsic_fn(|| {
            if let Some(child) = &self.child {
                child.intrinsic(context)
            } else {
                measure::Intrinsic::default()
            }
        })
        .spacing(self.spacing.unwrap_or_default())
        .box_size(
            self.width.as_option().map(|&width| width as f32),
            self.height.as_option().map(|&height| height as f32),
        )
        .intrinsic()
    }

    fn measure_content(&self, context: &mut C, constraints: Constraints<Extent<f32>>) -> Extent<f32>
    where
        C: ManageMeasures<f32, WidgetId>,
    {
        Content::measure_fn(|container_constraints| {
            if let Some(child) = &self.child {
                child.measure(context, container_constraints)
            } else {
                Extent::default()
            }
        })
        .spacing(self.spacing.unwrap_or_default())
        .box_size(
            self.width.as_option().map(|&width| width as f32),
            self.height.as_option().map(|&height| height as f32),
        )
        .measure(constraints)
    }
}

impl<C> Layout<C, f32> for Container
where
    C: LayoutContext<f32>,
{
    fn layout(&mut self, context: &mut C) {
        if context.load(self.id).is_none() {
            warn!("Container widget with id {} didn't measured!", *self.id);
        }

        if let Some(child) = &mut self.child {
            child.layout(context);
        }
    }
}

impl<C> Draw<C, f32> for Container
where
    C: DrawContext<f32>,
{
    fn draw_content(
        &self,
        context: &C,
        offset: &Offset<f32>,
        provided_extent: Extent<f32>,
        drawer: &mut Drawer,
    ) {
        Content::draw_fn(
            |offset: &Offset<f32>, provided_extent: Extent<f32>, drawer: &mut Drawer| {
                if let Some(child) = &self.child {
                    let alignment = self.alignment.clone().unwrap_or_default();
                    let child_extent =
                        <C as LoadExtent<f32, WidgetId>>::load(context, child.get_id())
                            .unwrap_or_default();

                    let horizontal_start = alignment
                        .horizontal
                        .get_start(provided_extent.width, child_extent.width);
                    let vertical_start = alignment
                        .vertical
                        .get_start(provided_extent.height, child_extent.height);

                    let offset_for_child = *offset + Offset::new(horizontal_start, vertical_start);
                    child.draw(context, &offset_for_child, drawer);

                    if context.get_debug_options().show_layout_bounds {
                        draw_debug_bounds(drawer.surface.canvas(), *offset, provided_extent);
                    }
                }
            },
        )
        .spacing(self.spacing.unwrap_or_default())
        .background(self.background_color.clone().unwrap_or_default())
        .border(self.border.clone().unwrap_or_default())
        .box_size(
            self.width.as_option().map(|&width| width as f32),
            self.height.as_option().map(|&height| height as f32),
        )
        .draw(offset, provided_extent, drawer);
    }
}

impl<C> EventHitTest<C, f32> for Container
where
    C: EventContext<f32>,
{
    fn on_hit_test(
        &self,
        context: &C,
        local_coords: Point<f32>,
        provided_extent: Extent<f32>,
        router: &mut EventRouter,
    ) -> HitTestResult {
        Content::hit_test_fn(
            |local_coords: Point<f32>, _provided_extent: Extent<f32>, router: &mut EventRouter| {
                if let Some(child) = &self.child {
                    let result = child.hit_test(context, local_coords, router);

                    match result {
                        HitTestResult::Hit => router.set_next_index(self.id, 0),
                        HitTestResult::Missed | HitTestResult::Failed => (),
                    }

                    result
                } else {
                    HitTestResult::Missed
                }
            },
        )
        .spacing(self.spacing.unwrap_or_default())
        .border(self.border.clone().unwrap_or_default())
        .box_size(
            self.width.as_option().map(|&width| width as f32),
            self.height.as_option().map(|&height| height as f32),
        )
        .hit_test(self.id, local_coords, provided_extent, router)
    }
}

impl<C> EventHandling<C, f32> for Container
where
    C: EventContext<f32>,
{
    fn handle_events(
        &mut self,
        context: &mut C,
        _pending_events: Vec<PendingEvent>,
        _next_child: usize,
        router: &EventRouter,
    ) {
        if let Some(child) = &mut self.child {
            child.route_events(context, router);
        }
    }
}
